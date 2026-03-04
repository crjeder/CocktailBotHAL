// src/server/mod.rs

use embassy_net::tcp::TcpSocket;
use embedded_io_async::{Read, Write};

use crate::hal::*;

pub mod http;
pub mod sse;

pub struct RobotHal<'a>
{
    pub control: &'a mut dyn ControlHal,
    pub status: &'a mut dyn StatusHal,
    pub config: &'a mut dyn ConfigHal,
    pub storage: &'a mut dyn StorageHal,
    pub sensors: &'a mut dyn SensorHal,
    pub dispense: &'a mut dyn DispenseHal,
    pub cleaning: &'a mut dyn CleaningHal,
}

pub struct ApiServer<'a>
{
    pub hal: RobotHal<'a>,
}

impl<'a> ApiServer<'a>
{
    /// Main server loop — accepts connections on port 80.
    pub async fn run(
        &mut self,
        net_stack: embassy_net::Stack<'_>,
    )
    {
        loop
        {
            let mut socket = TcpSocket::new(&net_stack);
            socket.accept(80).await.unwrap();

            self.handle_connection(&mut socket).await;
        }
    }

    async fn handle_connection<S: Read + Write + Unpin>(
        &mut self,
        socket: &mut S,
    )
    {
        let request =
            match http::read_http_request(socket).await
            {
                Ok(r) => r,
                Err(_) => return,
            };

        let method = request.method.as_str();
        let path = request.path.as_str();

        match (method, path)
        {
            // ----- status -----
            ("GET", "/v1/status") =>
            {
                handlers::status::handle_status(
                    &self.hal, socket,
                )
                .await;
            }

            // ----- control -----
            ("POST", "/v1/control/power") =>
            {
                handlers::control::handle_power(
                    &mut self.hal,
                    &request,
                    socket,
                )
                .await;
            }
            ("POST", "/v1/control/power-save") =>
            {
                handlers::control::handle_power_save(
                    &mut self.hal,
                    &request,
                    socket,
                )
                .await;
            }
            ("POST", "/v1/control/reset") =>
            {
                handlers::control::handle_reset(
                    &mut self.hal,
                    socket,
                )
                .await;
            }
            ("POST", "/v1/control/reload-config") =>
            {
                handlers::control::handle_reload_config(
                    &mut self.hal,
                    socket,
                )
                .await;
            }

            // ----- config -----
            ("GET", "/v1/config") =>
            {
                handlers::config::handle_config_get(
                    &self.hal, socket,
                )
                .await;
            }
            ("PATCH", "/v1/config") =>
            {
                handlers::config::handle_config_patch(
                    &mut self.hal,
                    &request,
                    socket,
                )
                .await;
            }

            // ----- storage -----
            ("GET", "/v1/storage/config") =>
            {
                handlers::config::handle_storage_read(
                    &self.hal, socket,
                )
                .await;
            }
            ("POST", "/v1/storage/config") =>
            {
                handlers::config::handle_storage_write(
                    &mut self.hal,
                    &request,
                    socket,
                )
                .await;
            }

            // ----- sensors -----
            ("GET", "/v1/sensors/glass") =>
            {
                handlers::sensors::handle_glass(
                    &self.hal, socket,
                )
                .await;
            }
            ("GET", "/v1/sensors/levels") =>
            {
                handlers::sensors::handle_levels(
                    &self.hal, socket,
                )
                .await;
            }

            // ----- dispense (collection) -----
            ("POST", "/v1/dispense/jobs") =>
            {
                handlers::dispense::handle_create_job(
                    &mut self.hal,
                    &request,
                    socket,
                )
                .await;
            }
            ("GET", "/v1/dispense/jobs") =>
            {
                handlers::dispense::handle_list_jobs(
                    &self.hal, socket,
                )
                .await;
            }

            // ----- cleaning -----
            ("POST", "/v1/cleaning/start") =>
            {
                handlers::cleaning::handle_start(
                    &mut self.hal,
                    socket,
                )
                .await;
            }
            ("POST", "/v1/cleaning/stop") =>
            {
                handlers::cleaning::handle_stop(
                    &mut self.hal,
                    socket,
                )
                .await;
            }

            // ----- dynamic paths & fallback -----
            _ =>
            {
                // /v1/dispense/jobs/{job_id}
                if let Some(job_id) =
                    path.strip_prefix("/v1/dispense/jobs/")
                {
                    match method
                    {
                        "GET" =>
                        {
                            handlers::dispense::handle_job_status(
                                &self.hal, job_id, socket,
                            )
                            .await;
                        }
                        "POST" =>
                        {
                            handlers::dispense::handle_cancel_job(
                                &mut self.hal,
                                job_id,
                                socket,
                            )
                            .await;
                        }
                        _ =>
                        {
                            http::write_json(
                                socket,
                                405,
                                &serde_json::json!({"error": "method not allowed"}),
                            )
                            .await
                            .ok();
                        }
                    }
                }
                else
                {
                    http::write_json(
                        socket,
                        404,
                        &serde_json::json!({"error": "not found"}),
                    )
                    .await
                    .ok();
                }
            }
        }
    }
}

pub mod handlers
{
    pub mod cleaning;
    pub mod config;
    pub mod control;
    pub mod dispense;
    pub mod sensors;
    pub mod status;
}
