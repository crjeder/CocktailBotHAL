## ADDED Requirements

### Requirement: README has a project overview section
The README SHALL open with a one-line crate description followed by a short
prose paragraph (2–4 sentences) explaining what CocktailBotHAL does, who it
is for, and its target platform.

#### Scenario: Reader opens README
- **WHEN** a developer opens README.md
- **THEN** the first section explains the crate purpose, target platform (ESP32),
  and intended audience (hardware vendors and API clients) without requiring
  prior knowledge of the codebase

### Requirement: README documents the architecture
The README SHALL contain an **Architecture** section that explains the two
top-level modules (`hal` and `server`), the static generic dispatch pattern
used by `ApiServer`, and the relationship between the HAL traits and the HTTP
handlers.

#### Scenario: Hardware vendor reads architecture section
- **WHEN** a hardware vendor reads the Architecture section
- **THEN** they understand that they must implement the HAL traits and that the
  HTTP server is provided by the library

### Requirement: README lists all HAL traits with their responsibilities
The README SHALL include a section that lists all seven HAL traits
(`ControlHal`, `StatusHal`, `ConfigHal`, `StorageHal`, `SensorHal`,
`DispenseHal`, `CleaningHal`) with a one-line description of each trait's
responsibility.

#### Scenario: Implementor identifies which trait to implement
- **WHEN** a hardware implementor reads the HAL traits section
- **THEN** they can determine which traits are relevant to their hardware
  without reading `src/hal/mod.rs`

### Requirement: README documents key public types
The README SHALL document the following types with their role:
`RobotState`, `RobotConfig`, `LiquidConfig`, `JobItem`, `JobStatus`,
`Capabilities`, `ApiServer`.

#### Scenario: Client developer looks up RobotState
- **WHEN** an API client developer reads the key types section
- **THEN** they find a description of `RobotState` variants and understand
  what each state means at the robot level

### Requirement: README includes build and run commands
The README SHALL contain a **Build & Run** section with the exact Cargo
commands needed to check, build, run the dev example, and build the ESP32
example.

#### Scenario: New contributor sets up the project
- **WHEN** a new contributor reads the Build & Run section
- **THEN** they can run the development server without consulting CLAUDE.md or
  other internal documentation

### Requirement: README includes a HAL implementor quick-start guide
The README SHALL contain a short guide (bullet list or numbered steps) showing
hardware vendors how to implement the HAL traits and wire them into `ApiServer`.

#### Scenario: Vendor follows the implementor guide
- **WHEN** a vendor follows the HAL implementor guide
- **THEN** they know which structs to create, which traits to implement on
  them, and how to construct `ApiServer` with their implementations

### Requirement: README includes a usage code example
The README SHALL contain a **Usage** section with a minimal Rust code snippet
showing a stub HAL implementation (at minimum one trait) being constructed and
passed to `ApiServer`. The example SHALL compile or be clearly marked as
illustrative pseudo-code.

#### Scenario: Developer copies the usage example
- **WHEN** a developer copies the usage snippet
- **THEN** they have a working skeleton to start their own HAL implementation
  with concrete method signatures visible

### Requirement: README states the project status
The README SHALL contain a **Project Status** section noting that the crate is
pre-1.0 and under active development, and that the HAL trait interface may
change with semver-breaking releases.

#### Scenario: Prospective adopter reads project status
- **WHEN** a developer considers adopting the library
- **THEN** they understand the stability guarantees and versioning policy

### Requirement: README includes a support section
The README SHALL contain a **Support** section pointing to the GitHub Issues
tracker for bug reports and questions.

#### Scenario: User encounters a problem
- **WHEN** a user encounters a problem and reads the README
- **THEN** they know where to file an issue or ask a question

### Requirement: README includes a contributing section
The README SHALL contain a **Contributing** section stating that contributions
are welcome, with instructions to run `cargo test` and `cargo fmt` before
submitting a pull request.

#### Scenario: Contributor prepares a PR
- **WHEN** a contributor reads the Contributing section
- **THEN** they know the minimum quality bar (tests pass, code formatted) before
  opening a pull request

### Requirement: README states the license
The README SHALL end with a **License** section stating that the project is
licensed under GNU GPL v3 and pointing to the `LICENSE` file or SPDX
identifier.

#### Scenario: License section is present
- **WHEN** a developer reads to the end of the README
- **THEN** they see the GPL v3 license statement
