// src/server/mod.rs

use embassy_net::tcp::TcpSocket;
use embedded_io_async::{Read, Write};

use crate::hal::*;

pub mod http;
pub mod sse;

/// Default Bearer token used when `RobotConfig::token` is empty.
/// Operators MUST change this via `PATCH /v1/config` before deployment.
const DEFAULT_TOKEN: &str = "changeme";

/// Compare two token strings in constant time to prevent timing-based
/// enumeration. Returns `true` only if both strings are identical in length
/// and content. The XOR accumulator ensures no early-exit on the first
/// differing byte.
fn tokens_equal(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Returns true if the given path is an admin endpoint that should remain
/// accessible during the `Provisioning` state.
fn is_admin_route(method: &str, path: &str) -> bool {
    matches!(
        (method, path),
        ("GET", "/v1/config")
            | ("PATCH", "/v1/config")
            | ("GET", "/v1/config/backup")
            | ("POST", "/v1/config/restore")
            | ("GET", "/v1/status")
    )
}

/// Composes all HAL trait implementations into a single struct.
///
/// Each field is a concrete owned type bounded by the corresponding HAL
/// trait. Using generic type parameters (instead of `dyn Trait`) keeps the
/// code compatible with native `async fn` in traits, which are not
/// object-safe in Rust's current edition.
pub struct RobotHal<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean> {
    pub control: Ctrl,
    pub status: Stat,
    pub config: Cfg,
    pub storage: Stor,
    pub sensors: Sens,
    pub dispense: Disp,
    pub cleaning: Clean,
}

pub struct ApiServer<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean> {
    pub hal: RobotHal<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean>,
}

impl<
        Ctrl: ControlHal,
        Stat: StatusHal,
        Cfg: ConfigHal,
        Stor: StorageHal,
        Sens: SensorHal,
        Disp: DispenseHal,
        Clean: CleaningHal,
    > ApiServer<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean>
{
    /// Main server loop — accepts connections on port 80.
    pub async fn run(&mut self, net_stack: embassy_net::Stack<'_>) {
        loop {
            let mut rx_buf = [0u8; 4096];
            let mut tx_buf = [0u8; 4096];
            let mut socket = TcpSocket::new(net_stack, &mut rx_buf, &mut tx_buf);
            socket.accept(80).await.unwrap();

            self.handle_connection(&mut socket).await;
        }
    }

    async fn handle_connection<S: Read + Write + Unpin>(&mut self, socket: &mut S) {
        let request = match http::read_http_request(socket).await {
            Ok(r) => r,
            Err(_) => return,
        };

        let method = request.method.as_str();
        let path = request.path.as_str();

        // Authenticate before dispatching to any handler.
        let cfg = self.hal.config.get_active_config().await;
        let effective_token = if cfg.token.is_empty() {
            DEFAULT_TOKEN
        } else {
            cfg.token.as_str()
        };
        let authorized = request
            .header("Authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|tok| tokens_equal(tok, effective_token))
            .unwrap_or(false);
        if !authorized {
            http::write_json(socket, 401, &serde_json::json!({"error": "Unauthorized"}))
                .await
                .ok();
            return;
        }

        // Provisioning gate: reject non-admin routes when state is Provisioning.
        let state = self.hal.status.state().await;
        if state == RobotState::Provisioning && !is_admin_route(method, path) {
            http::write_json(
                socket,
                503,
                &serde_json::json!({
                    "error": "Robot is in provisioning state. Restore a configuration first."
                }),
            )
            .await
            .ok();
            return;
        }

        match (method, path) {
            // ----- status -----
            ("GET", "/v1/status") => {
                handlers::status::handle_status(&self.hal.status, socket).await;
            }

            // ----- control -----
            ("POST", "/v1/control/power") => {
                handlers::control::handle_power(&mut self.hal.control, &request, socket).await;
            }
            ("POST", "/v1/control/power-save") => {
                handlers::control::handle_power_save(&mut self.hal.control, &request, socket).await;
            }
            ("POST", "/v1/control/reset") => {
                handlers::control::handle_reset(&mut self.hal.control, socket).await;
            }

            // ----- config -----
            ("GET", "/v1/config") => {
                handlers::config::handle_config_get(&self.hal.config, socket).await;
            }
            ("PATCH", "/v1/config") => {
                handlers::config::handle_config_patch(
                    &mut self.hal.config,
                    &mut self.hal.storage,
                    &mut self.hal.dispense,
                    &request,
                    socket,
                )
                .await;
            }

            // ----- backup / restore -----
            ("GET", "/v1/config/backup") => {
                handlers::config::handle_backup(&self.hal.storage, socket).await;
            }
            ("POST", "/v1/config/restore") => {
                handlers::config::handle_restore(
                    &mut self.hal.config,
                    &mut self.hal.storage,
                    &mut self.hal.dispense,
                    &request,
                    socket,
                )
                .await;
            }

            // ----- sensors -----
            ("GET", "/v1/sensors/glass") => {
                handlers::sensors::handle_glass(&self.hal.sensors, socket).await;
            }
            ("GET", "/v1/sensors/levels") => {
                handlers::sensors::handle_levels(&self.hal.sensors, socket).await;
            }

            // ----- dispense (collection) -----
            ("POST", "/v1/dispense/jobs") => {
                handlers::dispense::handle_create_job(&mut self.hal.dispense, &request, socket)
                    .await;
            }
            ("GET", "/v1/dispense/jobs") => {
                handlers::dispense::handle_list_jobs(&self.hal.dispense, socket).await;
            }

            // ----- cleaning -----
            ("POST", "/v1/cleaning/start") => {
                handlers::cleaning::handle_start(&mut self.hal.cleaning, socket).await;
            }
            ("POST", "/v1/cleaning/stop") => {
                handlers::cleaning::handle_stop(&mut self.hal.cleaning, socket).await;
            }

            // ----- dynamic paths & fallback -----
            _ => {
                // /v1/dispense/jobs/{job_id}
                if let Some(job_id) = path.strip_prefix("/v1/dispense/jobs/") {
                    match method {
                        "GET" => {
                            handlers::dispense::handle_job_status(
                                &self.hal.dispense,
                                job_id,
                                socket,
                            )
                            .await;
                        }
                        "POST" => {
                            handlers::dispense::handle_cancel_job(
                                &mut self.hal.dispense,
                                job_id,
                                socket,
                            )
                            .await;
                        }
                        _ => {
                            http::write_json(
                                socket,
                                405,
                                &serde_json::json!({"error": "method not allowed"}),
                            )
                            .await
                            .ok();
                        }
                    }
                } else {
                    http::write_json(socket, 404, &serde_json::json!({"error": "not found"}))
                        .await
                        .ok();
                }
            }
        }
    }
}

pub mod handlers {
    pub mod cleaning;
    pub mod config;
    pub mod control;
    pub mod dispense;
    pub mod sensors;
    pub mod status;
}
