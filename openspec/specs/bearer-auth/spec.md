## Requirements

### Requirement: All requests require a valid Bearer token
The server SHALL check the `Authorization` header on every incoming HTTP request
before dispatching to any handler. If the header is absent, malformed, or contains
an incorrect token, the server SHALL return `401 Unauthorized` with a JSON error
body and SHALL NOT call any HAL method.

#### Scenario: Request with valid token is accepted
- **WHEN** a request arrives with `Authorization: Bearer <correct-token>`
- **THEN** the server dispatches the request to the appropriate handler

#### Scenario: Request with missing Authorization header is rejected
- **WHEN** a request arrives with no `Authorization` header
- **THEN** the server returns HTTP 401 with body `{"error":"Unauthorized"}`

#### Scenario: Request with wrong token is rejected
- **WHEN** a request arrives with `Authorization: Bearer wrongtoken`
- **THEN** the server returns HTTP 401 with body `{"error":"Unauthorized"}`

#### Scenario: Request with malformed Authorization header is rejected
- **WHEN** a request arrives with `Authorization: Basic abc123` (wrong scheme)
- **THEN** the server returns HTTP 401 with body `{"error":"Unauthorized"}`

### Requirement: Accepted token is read from RobotConfig
The server SHALL read the accepted token from `RobotConfig::token` via
`ConfigHal::get_active_config()`. If the token field is empty, the server SHALL
fall back to the compile-time default token `"changeme"`.

#### Scenario: Token from config is used for validation
- **WHEN** `RobotConfig::token` is set to `"mysecret"` and a request arrives
  with `Authorization: Bearer mysecret`
- **THEN** the request is accepted

#### Scenario: Default token is used when config token is empty
- **WHEN** `RobotConfig::token` is empty and a request arrives with
  `Authorization: Bearer changeme`
- **THEN** the request is accepted

### Requirement: Token comparison is constant-time
The server SHALL compare the incoming token to the accepted token using a
comparison that does not short-circuit on the first differing byte, preventing
timing-based token enumeration.

#### Scenario: Timing is consistent regardless of match position
- **WHEN** tokens of equal length are compared
- **THEN** comparison time does not vary based on the position of the first
  differing byte
