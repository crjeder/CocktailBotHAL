// src/server/mod.rs

use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;
use crate::hal::*;

pub struct RobotHal<'a> {
    pub control: &'a mut dyn ControlHal,
    pub status: &'a mut dyn StatusHal,
    pub config: &'a mut dyn ConfigHal,
    pub storage: &'a mut dyn StorageHal,
    pub sensors: &'a mut dyn SensorHal,
    pub dispense: &'a mut dyn DispenseHal,
    pub cleaning: &'a mut dyn CleaningHal,
}

pub struct ApiServer<'a> {
    pub hal: RobotHal<'a>,
}

impl<'a> ApiServer<'a> {
    /// Hauptserver-Loop
    pub async fn run(&mut self, net_stack: embassy_net::Stack<'_>) {
        loop {
            let mut socket = TcpSocket::new(&net_stack);
            socket.accept(80).await.unwrap();

            self.handle_connection(&mut socket).await;
        }
    }

    async fn handle_connection<W: Write + Unpin>(&mut self, socket: &mut W) {
        let request = http_smol::read_http_request(socket).await.unwrap();
        let path = request.path.as_str();

        match (request.method.as_str(), path) {
            ("GET", "/v1/status") => {
                handlers::status::handle_status(&self.hal, socket).await;
            }

            ("POST", "/v1/control/power") => {
                handlers::control::handle_power(&mut self.hal, request, socket).await;
            }

            ("PATCH", "/v1/config") => {
                handlers::config::handle_config_patch(&mut self.hal, request, socket).await;
            }

            ("GET", "/v1/storage/config") => {
                handlers::config::handle_storage_read(&self.hal, socket).await;
            }

            ("POST", "/v1/storage/config") => {
                handlers::config::handle_storage_write(&mut self.hal, request, socket).await;
            }

            ("POST", "/v1/dispense/jobs") => {
                handlers::dispense::handle_create_job(&mut self.hal, request, socket).await;
            }

            ("GET", "/v1/dispense/jobs") => {
                handlers::dispense::handle_list_jobs(&self.hal, socket).await;
            }

            ("POST", "/v1/cleaning/start") => {
                handlers::cleaning::handle_start(&mut self.hal, socket).await;
            }

            // TODO: weitere Endpunkte 1:1 nach OpenAPI

            _ => {
                http_smol::write_json(
                    socket,
                    404,
                    &serde_json::json!({ "error": "not found" }),
                )
                .await
                .ok();
            }
        }
    }
}

pub mod handlers {
    pub mod status;
    pub mod control;
    pub mod config;
    pub mod sensors;
    pub mod dispense;
    pub mod cleaning;
}
