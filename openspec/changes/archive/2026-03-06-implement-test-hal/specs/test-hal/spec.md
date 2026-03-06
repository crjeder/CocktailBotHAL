## ADDED Requirements

### Requirement: Transaction-queue mock for each HAL trait
The crate SHALL provide `#[cfg(test)]` mock structs in `src/hal/mock.rs`
implementing all 7 HAL traits (`ControlHal`, `StatusHal`, `ConfigHal`,
`StorageHal`, `SensorHal`, `DispenseHal`, `CleaningHal`).
Each mock SHALL maintain an internal `VecDeque` of pre-loaded
`(expected_args, return_value)` pairs (transactions).
Calling a trait method SHALL pop the next transaction, assert the arguments
match, and return the pre-loaded result.
The mock SHALL provide a `done()` method that panics if unconsumed
transactions remain.

#### Scenario: Correct call sequence consumed
- **WHEN** a test enqueues two transactions on a mock and calls the corresponding method twice
- **THEN** both transactions are consumed in order and `done()` succeeds

#### Scenario: Unexpected call panics
- **WHEN** a test calls a mock method with no transaction enqueued
- **THEN** the mock panics with a descriptive message identifying the method

#### Scenario: Unconsumed transaction detected
- **WHEN** a test enqueues a transaction but never calls the corresponding method
- **THEN** `done()` panics listing the remaining expectations

#### Scenario: Argument mismatch panics
- **WHEN** a test enqueues a transaction with argument `A` but calls the method with argument `B`
- **THEN** the mock panics identifying the mismatch

### Requirement: Mock returns pre-loaded error results
Each mock transaction's return value SHALL be a `Result<T, ErrorInfo>`,
allowing tests to inject both success and failure outcomes.

#### Scenario: Inject error result
- **WHEN** a test enqueues a transaction with `Err(ErrorInfo { ... })` as the return value
- **THEN** the trait method returns that error

#### Scenario: Inject success result
- **WHEN** a test enqueues a transaction with `Ok(value)` as the return value
- **THEN** the trait method returns `Ok(value)`

### Requirement: Mock is available to all test modules in the crate
The mock types SHALL be `pub(crate)` under `#[cfg(test)]` so any
`#[cfg(test)]` module within the crate can import and use them without
a separate crate or feature flag.

#### Scenario: Import from handler test
- **WHEN** a `#[cfg(test)]` block in `src/server/handlers/` imports `crate::hal::mock::MockStatusHal`
- **THEN** it compiles and the mock is usable

### Requirement: Handler integration tests via mock HAL
At least one `#[cfg(test)]` test SHALL exist per server handler module
(`status`, `control`, `config`, `sensors`, `dispense`, `cleaning`),
calling the handler function directly with a mock HAL instance and
asserting the HTTP response.

#### Scenario: Status handler returns robot state
- **WHEN** `MockStatusHal` is loaded with `RobotState::Idle` and the status handler is called
- **THEN** the handler returns an HTTP 200 response body containing `"idle"`

#### Scenario: Handler propagates HAL error as HTTP 500
- **WHEN** a mock HAL method returns `Err(ErrorInfo { ... })` and the corresponding handler is called
- **THEN** the handler returns an HTTP 500 response

### Requirement: Parameterised tests via test-case
The crate SHALL use `test-case = "3.2"` (dev-dependency) for parameterised
HAL trait tests where the same behavior is asserted across multiple inputs.

#### Scenario: Parameterised enum roundtrip
- **WHEN** a `#[test_case]` test verifies JSON serialisation of each `RobotState` variant
- **THEN** all variants round-trip without a single test function per variant

### Requirement: No impact on production build
All mock types and test-helper code SHALL be gated behind `#[cfg(test)]`
or `[dev-dependencies]` only; `cargo build` and
`cargo check --features esp32` SHALL produce zero test-related warnings
or errors.

#### Scenario: Production build unaffected
- **WHEN** `cargo build` is run without the `test` cfg
- **THEN** no mock types, test imports, or test-case macros appear in the compiled output
