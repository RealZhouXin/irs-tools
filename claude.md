# claude.md

## 1. Project Context

This is a Windows desktop application built with:

* Frontend: Svelte + TypeScript + Vite (Bun)
* Backend: Rust + Tauri v2
* Device communication: Windows DLL (`CommDllv2.dll`) loaded dynamically via `libloading`

The project is configuration-driven.

Test definitions and thresholds are defined in TOML files.

The architecture must remain layered and clean.

---

## 2. Architecture Rules (STRICT)

The project follows strict layered architecture:

Frontend

→ commands.rs (thin command layer)

→ test_service.rs (orchestration layer)

→ test_runner.rs (execution logic)

→ device_gateway.rs (abstraction)

→ comm_dll.rs (DLL binding layer)

### Non-Negotiable Rules

* ❌ Business logic must NOT be placed in `commands.rs`
* ❌ `comm_dll.rs` must NOT contain orchestration logic
* ❌ UI must NOT depend on device layer details
* ❌ `test_runner.rs` must NOT directly depend on DLL
* ❌ No direct DLL calls outside `comm_dll.rs`

Dependency direction must always point inward (DIP).

---

## 3. Configuration Model Constraints

The system is configuration-driven:

* `threshold.toml` → connection config
* `tests.toml` → test groups, stage definitions, checks

Rules:

* Do NOT hardcode thresholds in Rust
* Do NOT embed stage logic in frontend
* Stage execution order comes only from TOML
* `TestGroup`, `TestResult`, and `TestConfig` must remain serializable

---

## 4. Command Layer Rules

All Tauri commands:

* Must remain thin
* Must delegate to service layer
* Must return structured `CommandResult<T>`
* Must NOT panic
* Must use structured `AppError`

---

## 5. Error Handling Policy

* No unwrap()
* No expect()
* All fallible operations return Result
* Use structured AppError
* Convert DLL return codes into readable errors

---

## 6. Event Model Rules

Events (e.g. `test-group-complete`) are:

* Defined centrally in `events.rs`
* Must not use magic strings
* Must carry structured payloads
* Must include stage information

---

## 7. Device Gateway Abstraction Rules

* All hardware access goes through `DeviceGateway`
* `DeviceGatewayFactory` is the only creation entry
* Fake gateway must remain supported for unit tests
* No direct DLL instantiation inside service layer

---

## 8. Code Generation Requirements

When generating Rust code:

* Provide complete compilable files
* Include necessary imports
* Do not omit error types
* Do not introduce new dependencies unless justified
* Follow Rust idiomatic style

When modifying architecture:

* Explain reasoning
* Do not refactor unrelated modules
* Do not simplify layered structure

---

## 9. Frontend Constraints

Frontend:

* Must call backend only via `src/services/tauri.ts`
* Must not embed business rules
* Must group results by `stage`
* Must not assume execution order outside config

---

## 10. Performance & Safety

* DLL must be dynamically loaded safely
* No blocking UI thread
* Long-running tests must remain async
* Avoid global mutable state

---

## 11. Things Claude Must NOT Do

* ❌ Collapse service and runner layers
* ❌ Move orchestration into command layer
* ❌ Inline DLL calls into business logic
* ❌ Remove stage grouping logic
* ❌ Replace structured errors with strings
* ❌ Add random async runtimes
