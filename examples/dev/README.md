# dev

A host-side development entry point for CocktailBotHAL.

Runs the [embassy](https://embassy.dev/) spin executor on the host and wires
`ApiServer` with minimal stub HAL implementations. Use this to verify the
library compiles and to iterate on HAL trait changes without real hardware.

## Running

```sh
cargo run --example dev
```

The binary starts the embassy executor and spawns `async_main`, then spins
forever. **It does not actually serve HTTP** — `ApiServer::run()` requires an
`embassy-net` network stack that is not wired up in this example. The server
object is constructed but never driven.

## What the Stubs Do

| Struct | Trait | Behaviour |
|--------|-------|-----------|
| `StubControlHal` | `ControlHal` | All methods `todo!()` (panic on call) |
| `StubStatusHal` | `StatusHal` | Always returns `RobotState::Idle`, no errors |
| `StubConfigHal` | `ConfigHal` | Returns a fixed config (water + 3 glass types); mutations `todo!()` |
| `StubSensorHal` | `SensorHal` | All methods `todo!()` |
| `StubDispenseHal` | `DispenseHal` | `create_job` accepts and echoes the job ID; all other methods return empty/not-found |
| `StubCleaningHal` | `CleaningHal` | All methods `todo!()` |
| `RamStorageHal` | `StorageHal` | Fully functional RAM-backed store (see `storage.rs`) |
| `StubPasswordHasher` | `PasswordHasher` | Stores `stub$<plaintext>` — **not for production** |

## Replacing Stubs

To bring up a real hardware driver:

1. Create a struct that implements the relevant HAL trait from
   `cocktail_bot_hal::hal`.
2. Substitute it for the corresponding stub in `async_main`.
3. Wire `ApiServer::run(&mut server, net_stack).await` once an `embassy-net`
   stack is available.

For ESP32 bring-up, see [`examples/esp32/`](../esp32/).
For a fully stateful mock you can interact with over HTTP, see
[`examples/mock-server/`](../mock-server/).
