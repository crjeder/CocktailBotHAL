// src/server/handlers/config.rs
//
// Handlers for GET/PATCH /v1/config, GET /v1/config/backup, POST /v1/config/restore

use alloc::string::String;
use alloc::vec::Vec;
use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::{AdminConfig, ConfigHal, DispenseHal, JobState, StorageHal};
use crate::server::http::{self, HttpRequest};

/// GET /v1/config — return active (RAM) configuration.
pub async fn handle_config_get<Cfg: ConfigHal, W: Write + Unpin>(config: &Cfg, socket: &mut W) {
    let cfg = config.get_active_config().await;
    http::write_json(socket, 200, &serde_json::json!(cfg))
        .await
        .ok();
}

/// PATCH /v1/config — update active configuration.
/// Runs pre-flight (flush queue + wait) before applying, then auto-persists.
pub async fn handle_config_patch<
    Cfg: ConfigHal,
    Disp: DispenseHal,
    Stor: StorageHal,
    W: Write + Unpin,
>(
    config: &mut Cfg,
    storage: &mut Stor,
    dispense: &mut Disp,
    request: &HttpRequest,
    socket: &mut W,
) {
    let admin_cfg: AdminConfig = match http::parse_body(request) {
        Ok(c) => c,
        Err(_) => {
            http::write_json(
                socket,
                400,
                &serde_json::json!({"error": "invalid request body"}),
            )
            .await
            .ok();
            return;
        }
    };

    let cancelled_job_ids = flush_queue_and_wait(dispense).await;

    if let Err(e) = config.update_active_config(admin_cfg.clone()).await {
        http::write_hal_error(socket, &e).await.ok();
        return;
    }

    if let Err(e) = storage.restore(admin_cfg).await {
        http::write_hal_error(socket, &e).await.ok();
        return;
    }

    http::write_json(
        socket,
        200,
        &serde_json::json!({
            "status": "updated",
            "cancelled_job_ids": cancelled_job_ids,
        }),
    )
    .await
    .ok();
}

/// GET /v1/config/backup — return a backup of the persisted AdminConfig.
pub async fn handle_backup<Stor: StorageHal, W: Write + Unpin>(storage: &Stor, socket: &mut W) {
    match storage.backup().await {
        Ok(payload) => {
            http::write_json(socket, 200, &serde_json::json!(payload))
                .await
                .ok();
        }
        Err(e) => {
            http::write_hal_error(socket, &e).await.ok();
        }
    }
}

/// POST /v1/config/restore — restore AdminConfig from a backup.
/// Verifies CRC32 checksum, runs pre-flight, writes flash + RAM.
pub async fn handle_restore<
    Cfg: ConfigHal,
    Stor: StorageHal,
    Disp: DispenseHal,
    W: Write + Unpin,
>(
    config: &mut Cfg,
    storage: &mut Stor,
    dispense: &mut Disp,
    request: &HttpRequest,
    socket: &mut W,
) {
    #[derive(Deserialize)]
    struct RestoreBody {
        data: AdminConfig,
        checksum: String,
    }

    let body: RestoreBody = match http::parse_body(request) {
        Ok(b) => b,
        Err(_) => {
            http::write_json(
                socket,
                400,
                &serde_json::json!({"error": "invalid request body"}),
            )
            .await
            .ok();
            return;
        }
    };

    // Verify CRC32 checksum of the serialised AdminConfig.
    let serialised = match serde_json::to_vec(&body.data) {
        Ok(v) => v,
        Err(_) => {
            http::write_json(
                socket,
                500,
                &serde_json::json!({"error": "failed to serialise config for checksum verification"}),
            )
            .await
            .ok();
            return;
        }
    };
    let computed_crc = crc32(&serialised);
    let expected_hex = alloc::format!("{:08x}", computed_crc);
    if body.checksum != expected_hex {
        http::write_json(
            socket,
            422,
            &serde_json::json!({
                "error": "checksum mismatch",
                "expected": expected_hex,
                "received": body.checksum,
            }),
        )
        .await
        .ok();
        return;
    }

    let cancelled_job_ids = flush_queue_and_wait(dispense).await;

    if let Err(e) = storage.restore(body.data.clone()).await {
        http::write_hal_error(socket, &e).await.ok();
        return;
    }

    if let Err(e) = config.update_active_config(body.data).await {
        http::write_hal_error(socket, &e).await.ok();
        return;
    }

    http::write_json(
        socket,
        200,
        &serde_json::json!({
            "status": "restored",
            "cancelled_job_ids": cancelled_job_ids,
        }),
    )
    .await
    .ok();
}

/// Pre-flight helper: cancel all queued jobs and wait for running job to finish.
/// Returns the list of cancelled job IDs.
async fn flush_queue_and_wait<Disp: DispenseHal>(dispense: &mut Disp) -> Vec<String> {
    let mut cancelled_ids = Vec::new();
    let jobs = dispense.list_jobs().await;
    for job in &jobs {
        if matches!(job.state, JobState::Queued) {
            if dispense.cancel_job(&job.job_id).await.is_ok() {
                cancelled_ids.push(job.job_id.clone());
            }
        }
    }

    // Wait for any running job to finish.
    loop {
        let jobs = dispense.list_jobs().await;
        let has_running = jobs.iter().any(|j| matches!(j.state, JobState::Running));
        if !has_running {
            break;
        }
    }

    cancelled_ids
}

/// CRC32 (IEEE 802.3) — simple table-free bit-by-bit implementation.
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
