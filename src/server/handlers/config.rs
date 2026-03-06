// src/server/handlers/config.rs
//
// Handlers for GET/PATCH /v1/config, GET /v1/config/backup,
// and POST /v1/config/restore.

use embedded_io_async::Write;
use serde::Deserialize;

use crate::hal::{
    crc32_hex, AdminConfig, ConfigHal, DispenseHal, JobState, PasswordHasher, StorageHal,
};
use crate::server::http::{self, HttpRequest};

// ============================================================================
// Pre-flight helper
// ============================================================================

/// Cancel all queued jobs and wait for any running job to finish.
///
/// Returns the IDs of jobs that were cancelled.  Called before applying any
/// config mutation to ensure the robot is idle before the new config takes
/// effect.
pub async fn flush_queue_and_wait<Disp: DispenseHal>(
    dispense: &mut Disp,
) -> alloc::vec::Vec<alloc::string::String> {
    use embassy_time::{Duration, Timer};

    let jobs = dispense.list_jobs().await;
    let mut cancelled = alloc::vec::Vec::new();
    for job in &jobs {
        if matches!(job.state, JobState::Queued) {
            if dispense.cancel_job(&job.job_id).await.is_ok() {
                cancelled.push(job.job_id.clone());
            }
        }
    }
    // Wait for any running job to reach a terminal state.
    loop {
        let jobs = dispense.list_jobs().await;
        if !jobs.iter().any(|j| matches!(j.state, JobState::Running)) {
            break;
        }
        Timer::after(Duration::from_millis(50)).await;
    }
    cancelled
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /v1/config — return active (RAM) configuration.
///
/// `admin_password` is always redacted (returned as `""`) so the stored hash
/// is not exposed to non-admin clients.
pub async fn handle_config_get<Cfg: ConfigHal, W: Write + Unpin>(config: &Cfg, socket: &mut W) {
    let mut cfg = config.get_active_config().await;
    cfg.admin_password = alloc::string::String::new();
    http::write_json(socket, 200, &serde_json::json!(cfg))
        .await
        .ok();
}

/// PATCH /v1/config — update admin configuration.
///
/// The body is an `AdminConfig` JSON object.  If `admin_password` is non-empty
/// it is treated as a new plaintext password and re-hashed before storing.
/// If empty, the current hash is preserved.
///
/// Pre-flight: queued jobs are cancelled and any running job is awaited before
/// the config is applied.  Auto-persists to flash via `StorageHal::restore`.
pub async fn handle_config_patch<
    Cfg: ConfigHal,
    Stor: StorageHal,
    Disp: DispenseHal,
    H: PasswordHasher,
    W: Write + Unpin,
>(
    config: &mut Cfg,
    storage: &mut Stor,
    dispense: &mut Disp,
    hasher: &H,
    request: &HttpRequest,
    socket: &mut W,
) {
    let mut cfg: AdminConfig = match http::parse_body(request) {
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

    if cfg.admin_password.is_empty() {
        // Preserve the current stored hash — client sent "" meaning "keep current".
        cfg.admin_password = config.get_active_config().await.admin_password;
    } else {
        match hasher.hash(&cfg.admin_password) {
            Ok(hash) => {
                cfg.admin_password = hash;
            }
            Err(e) => {
                http::write_hal_error(socket, &e).await.ok();
                return;
            }
        }
    }

    let cancelled = flush_queue_and_wait(dispense).await;

    if let Err(e) = config.update_active_config(cfg.clone()).await {
        http::write_hal_error(socket, &e).await.ok();
        return;
    }
    if let Err(e) = storage.restore(cfg).await {
        http::write_hal_error(socket, &e).await.ok();
        return;
    }

    http::write_json(
        socket,
        200,
        &serde_json::json!({"status": "updated", "cancelled_job_ids": cancelled}),
    )
    .await
    .ok();
}

/// GET /v1/config/backup — read persistent configuration as a signed payload.
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

/// POST /v1/config/restore — restore configuration from a backup payload.
///
/// Body: `{ "data": <AdminConfig>, "checksum": "<hex>" }`
///
/// Verifies the CRC32 checksum before applying.  Returns 422 on mismatch.
/// Pre-flight: same as PATCH /config.  Auto-activates config in RAM.
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
    struct Body {
        data: AdminConfig,
        checksum: alloc::string::String,
    }

    let body: Body = match http::parse_body(request) {
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

    // Verify checksum.
    let json = match serde_json::to_string(&body.data) {
        Ok(s) => s,
        Err(_) => {
            http::write_json(
                socket,
                500,
                &serde_json::json!({"error": "failed to serialise config"}),
            )
            .await
            .ok();
            return;
        }
    };
    let expected = crc32_hex(json.as_bytes());
    if expected != body.checksum {
        http::write_json(
            socket,
            422,
            &serde_json::json!({
                "error": "checksum mismatch",
                "expected": expected,
                "received": body.checksum,
            }),
        )
        .await
        .ok();
        return;
    }

    let cancelled = flush_queue_and_wait(dispense).await;

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
        &serde_json::json!({"status": "restored", "cancelled_job_ids": cancelled}),
    )
    .await
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::mock::{
        test_admin_config, test_robot_config, MockConfigHal, MockDispenseHal, MockPasswordHasher,
        MockStorageHal, MockWrite,
    };
    use crate::server::http::HttpRequest;
    use futures::executor::block_on;

    #[test]
    fn config_get_returns_200_with_config() {
        block_on(async {
            let cfg = MockConfigHal::new(test_robot_config());
            let mut buf = MockWrite::new();
            handle_config_get(&cfg, &mut buf).await;
            let resp = buf.as_str();
            assert!(resp.contains("HTTP/1.1 200"));
            assert!(resp.contains("vodka"));
        });
    }

    #[test]
    fn config_get_redacts_admin_password() {
        block_on(async {
            let mut robot_cfg = test_robot_config();
            robot_cfg.admin_password = "hashed$secret".to_string();
            let cfg = MockConfigHal::new(robot_cfg);
            let mut buf = MockWrite::new();
            handle_config_get(&cfg, &mut buf).await;
            assert!(!buf.as_str().contains("hashed$secret"));
        });
    }

    #[test]
    fn config_patch_returns_200() {
        block_on(async {
            let admin = test_admin_config();
            let body = serde_json::to_string(&admin).unwrap();
            let req = HttpRequest {
                method: "PATCH".to_string(),
                path: "/v1/config".to_string(),
                headers: alloc::vec![],
                body: body.into_bytes(),
            };
            let mut cfg = MockConfigHal::new(test_robot_config());
            let mut storage = MockStorageHal::new();
            let mut dispense = MockDispenseHal::new();
            let hasher = MockPasswordHasher;
            let mut buf = MockWrite::new();
            handle_config_patch(
                &mut cfg,
                &mut storage,
                &mut dispense,
                &hasher,
                &req,
                &mut buf,
            )
            .await;
            assert!(buf.as_str().contains("HTTP/1.1 200"));
        });
    }

    #[test]
    fn config_backup_no_config_returns_500() {
        block_on(async {
            let storage = MockStorageHal::new();
            let mut buf = MockWrite::new();
            handle_backup(&storage, &mut buf).await;
            assert!(buf.as_str().contains("HTTP/1.1 500"));
        });
    }
}
