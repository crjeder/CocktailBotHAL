// src/server/mod.rs

use embassy_net::tcp::TcpSocket;
use embedded_io_async::{Read, Write};

use crate::hal::*;

pub mod http;
pub mod sse;

/// Default Bearer token used when `AdminConfig::token` is empty.
/// Operators MUST change this via `PATCH /v1/config` before deployment.
const DEFAULT_TOKEN: &str = "changeme";

/// Default admin password used when `AdminConfig::admin_password` is empty.
/// Operators MUST change this via `PATCH /v1/config` before deployment.
const DEFAULT_ADMIN_PASSWORD: &str = "changeme";

/// Routes that require HTTP Basic Auth with the admin password.
/// All other routes require a Bearer token.
const ADMIN_ROUTES: &[(&str, &str)] = &[
    ("PATCH", "/v1/config"),
    ("GET", "/v1/config/backup"),
    ("POST", "/v1/config/restore"),
    ("POST", "/v1/control/power"),
    ("POST", "/v1/control/power-save"),
    ("POST", "/v1/control/reset"),
    ("POST", "/v1/cleaning/start"),
    ("POST", "/v1/cleaning/stop"),
];

/// Non-admin routes that remain active even when the robot is in
/// `Provisioning` state.  All other non-admin routes return 503 while
/// provisioning.
const PROVISIONING_ALLOWED: &[(&str, &str)] = &[("GET", "/v1/status"), ("GET", "/v1/config")];

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

/// Composes all HAL trait implementations into a single struct.
///
/// Each field is a concrete owned type bounded by the corresponding HAL
/// trait. Using generic type parameters (instead of `dyn Trait`) keeps the
/// code compatible with native `async fn` in traits, which are not
/// object-safe in Rust's current edition.
pub struct RobotHal<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean, Hasher> {
    pub control: Ctrl,
    pub status: Stat,
    pub config: Cfg,
    pub storage: Stor,
    pub sensors: Sens,
    pub dispense: Disp,
    pub cleaning: Clean,
    pub hasher: Hasher,
}

pub struct ApiServer<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean, Hasher> {
    pub hal: RobotHal<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean, Hasher>,
}

impl<
        Ctrl: ControlHal,
        Stat: StatusHal,
        Cfg: ConfigHal,
        Stor: StorageHal,
        Sens: SensorHal,
        Disp: DispenseHal,
        Clean: CleaningHal,
        Hasher: PasswordHasher,
    > ApiServer<Ctrl, Stat, Cfg, Stor, Sens, Disp, Clean, Hasher>
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
        let is_admin_route = ADMIN_ROUTES.iter().any(|(m, p)| *m == method && *p == path);
        let authorized = if is_admin_route {
            // Admin routes require HTTP Basic Auth with the admin password.
            let effective_hash = cfg.admin_password.as_str();
            request
                .header("Authorization")
                .and_then(|v| http::basic_auth_password(v))
                .as_deref()
                .map(|pwd| {
                    if effective_hash.is_empty() {
                        tokens_equal(pwd, DEFAULT_ADMIN_PASSWORD)
                    } else {
                        self.hal.hasher.verify(pwd, effective_hash)
                    }
                })
                .unwrap_or(false)
        } else {
            // All other routes require a Bearer token.
            let effective_token = if cfg.token.is_empty() {
                DEFAULT_TOKEN
            } else {
                cfg.token.as_str()
            };
            request
                .header("Authorization")
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|tok| tokens_equal(tok, effective_token))
                .unwrap_or(false)
        };
        if !authorized {
            http::write_json(socket, 401, &serde_json::json!({"error": "Unauthorized"}))
                .await
                .ok();
            return;
        }

        // Provisioning gate: non-admin routes (except status and config GET)
        // are blocked until the robot has been provisioned with a config.
        if !is_admin_route {
            let state = self.hal.status.state().await;
            let allowed_in_provisioning = PROVISIONING_ALLOWED
                .iter()
                .any(|(m, p)| *m == method && *p == path);
            if state == RobotState::Provisioning && !allowed_in_provisioning {
                http::write_json(
                    socket,
                    503,
                    &serde_json::json!({
                        "error": "Service Unavailable",
                        "hint": "Robot is in provisioning mode. POST /v1/config/restore to provision."
                    }),
                )
                .await
                .ok();
                return;
            }
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
                    &self.hal.hasher,
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
                let config = self.hal.config.get_active_config().await;
                handlers::dispense::handle_create_job(
                    &mut self.hal.dispense,
                    &config,
                    &request,
                    socket,
                )
                .await;
            }
            ("GET", "/v1/dispense/jobs") => {
                handlers::dispense::handle_list_jobs(&self.hal.dispense, socket).await;
            }

            // ----- cleaning -----
            ("POST", "/v1/cleaning/start") => {
                handlers::cleaning::handle_start(&self.hal.status, &mut self.hal.cleaning, socket)
                    .await;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// PATCH /v1/config is an admin route: a Bearer token must not satisfy it.
    #[test]
    fn patch_config_is_admin_route() {
        assert!(
            ADMIN_ROUTES
                .iter()
                .any(|(m, p)| *m == "PATCH" && *p == "/v1/config"),
            "PATCH /v1/config must be in ADMIN_ROUTES"
        );
    }

    /// GET /v1/config/backup is an admin route.
    #[test]
    fn backup_is_admin_route() {
        assert!(ADMIN_ROUTES
            .iter()
            .any(|(m, p)| *m == "GET" && *p == "/v1/config/backup"),);
    }

    /// POST /v1/config/restore is an admin route.
    #[test]
    fn restore_is_admin_route() {
        assert!(ADMIN_ROUTES
            .iter()
            .any(|(m, p)| *m == "POST" && *p == "/v1/config/restore"),);
    }

    /// Bearer-style header yields None from basic_auth_password, so the auth
    /// check for admin routes evaluates to false (→ 401).
    #[test]
    fn bearer_token_rejected_on_admin_route() {
        let bearer_header = "Bearer some-token";
        let password = http::basic_auth_password(bearer_header);
        assert!(
            password.is_none(),
            "Bearer header must not produce a Basic Auth password"
        );
    }

    /// Correct Basic Auth header yields the password, and tokens_equal confirms
    /// it matches DEFAULT_ADMIN_PASSWORD (→ 200 when no hash is stored).
    #[test]
    fn correct_basic_auth_accepted_on_admin_route() {
        // "admin:changeme" base64 → "YWRtaW46Y2hhbmdlbWU="
        let basic_header = "Basic YWRtaW46Y2hhbmdlbWU=";
        let password = http::basic_auth_password(basic_header);
        assert!(password.is_some(), "Valid Basic Auth header must decode");
        assert!(
            tokens_equal(password.as_deref().unwrap(), DEFAULT_ADMIN_PASSWORD),
            "Correct password must match DEFAULT_ADMIN_PASSWORD"
        );
    }

    /// Wrong password in Basic Auth must be rejected.
    #[test]
    fn wrong_basic_auth_rejected() {
        // "admin:wrongpassword" base64 → "YWRtaW46d3JvbmdwYXNzd29yZA=="
        let basic_header = "Basic YWRtaW46d3JvbmdwYXNzd29yZA==";
        let password = http::basic_auth_password(basic_header);
        assert!(password.is_some());
        assert!(
            !tokens_equal(password.as_deref().unwrap(), DEFAULT_ADMIN_PASSWORD),
            "Wrong password must not match DEFAULT_ADMIN_PASSWORD"
        );
    }
}
