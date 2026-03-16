# mock-server

A fully stateful mock HTTP server for testing CocktailBotHAL clients.

Exposes the same OpenAPI 3.1.0 REST contract as a real robot, but runs entirely
on the host using `std::net`. All HAL stubs share a single `MockState` that
advances via a background ticker thread every 100 ms.

## Features

- Realistic `RobotState` transitions: `Off → SelfTest → Idle → Working → Idle`
- Time-driven dispense job progress and cleaning cycle timers
- SSE push stream at `GET /v1/events`
- Runtime glass-state and error injection via `POST /mock/control`

## Running

```sh
cargo run --example mock-server
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--port N` | `8000` | TCP port to listen on |
| `--glass-present` | off | Start with a glass on the sensor |
| `--glass-absent` | off | Start without a glass (explicit, overrides `--glass-present`) |
| `--dispense-duration-secs N` | `10` | Simulated time for each dispense job to complete |
| `--liquids <path>` | built-in defaults | JSON file to configure the initial liquid set |

Example:

```sh
cargo run --example mock-server -- --port 9000 --glass-present --dispense-duration-secs 3

cargo run --example mock-server -- --liquids examples/mock-server/liquids.json
```

### Liquids JSON format

The file must be a JSON array of `LiquidConfig` objects:

```json
[
  { "id": "gin",   "name": "Gin",         "position": 0, "calibration": { "factor": 1.0 } },
  { "id": "tonic", "name": "Tonic Water", "position": 1, "calibration": { "factor": 1.0 } }
]
```

A sample file is provided at `examples/mock-server/liquids.json`. The liquids can
also be updated at runtime via `PATCH /config` (see below).


## Authentication

All requests to `/v1/…` require one of:

| Method | Value |
|--------|-------|
| Bearer token | `Authorization: Bearer changeme` |
| Admin (Basic) | `Authorization: Basic YWRtaW46Y2hhbmdlbWU=` (`admin:changeme`) |

## Configuration

The server boots with a built-in default configuration (defined in `state.rs`):

| Field | Default value |
|-------|---------------|
| Bearer token | `changeme` (empty string → server falls back to compile-time default) |
| Admin password | `changeme` (same fallback) |
| Liquids | vodka (pos 0), orange juice (pos 1), sparkling water (pos 2) |
| Glass types | short 100 ml, medium 150 ml, long 200 ml |
| Glass wait timeout | 60 s |
| Drink ready timeout | 300 s |
| Capabilities version | `0.6.0-mock` |

### Read active config

```sh
curl http://127.0.0.1:8000/config \
     -H 'Authorization: Bearer changeme'
```

### Update config at runtime

Use `PATCH /config` with admin credentials. All fields are optional — send only
what you want to change. Supplying a non-empty `admin_password` sets a new
admin password (hashed before storage).

```sh
curl -X PATCH http://127.0.0.1:8000/config \
     -H 'Authorization: Basic YWRtaW46Y2hhbmdlbWU=' \
     -H 'Content-Type: application/json' \
     -d '{
       "liquids": [
         {"id": "gin",    "name": "Gin",          "position": 0, "calibration": {"factor": 1.0}},
         {"id": "tonic",  "name": "Tonic Water",  "position": 1, "calibration": {"factor": 1.0}}
       ],
       "glass_types": [
         {"id": "rocks", "volume": 120.0},
         {"id": "highball", "volume": 200.0}
       ],
       "token": "mysecrettoken",
       "admin_password": "newpassword",
       "glass_wait_timeout_secs": 30,
       "drink_ready_timeout_secs": 120
     }'
```

Config changes are applied immediately to `MockState` and survive for the
lifetime of the process. They are **not** persisted to disk — the server always
resets to defaults on restart.

### Backup and restore

`GET /config/backup` and `POST /config/restore` are also available but backed
by `MockStorage` (RAM only). The backup is lost when the process exits.

## Mock Control Endpoint

`POST /mock/control` lets you inject state at runtime without going through the
public API. It is **not** part of the OpenAPI spec.

**Set glass sensor state:**

```sh
curl -X POST http://127.0.0.1:8000/mock/control \
     -H 'Content-Type: application/json' \
     -d '{"glass": "present"}'

curl -X POST http://127.0.0.1:8000/mock/control \
     -H 'Content-Type: application/json' \
     -d '{"glass": "absent"}'
```

**Inject an error (transitions robot to `Error` state):**

```sh
curl -X POST http://127.0.0.1:8000/mock/control \
     -H 'Content-Type: application/json' \
     -d '{"inject_error": "SENSOR_FAULT"}'
```

Both fields may be sent together in a single request.

## SSE Stream

```sh
curl -N http://127.0.0.1:8000/v1/events \
     -H 'Authorization: Bearer changeme'
```

The server spawns a thread per SSE client so the main accept loop stays
unblocked.

## Architecture Notes

- **Sequential connections** — one request is served at a time (sufficient for
  integration testing; not for load testing).
- `_embassy_time_now` / `_embassy_time_schedule_wake` linker stubs are provided
  in `main.rs` so the binary links without an embassy-net time driver.
- `StdTcpStream` is a thin `embedded-io-async` adapter over `std::net::TcpStream`
  that lets `ApiServer::handle_request` work on the host.
