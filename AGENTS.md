# AGENTS.md — rs_events

Agent workflow guidelines for working on the `rs_events` crate.

## Developer commands

```text
cargo fmt -- --check               # Formatted (CI fails without this)
cargo clippy -- -D warnings        # Lint (CI fails without this)
cargo check --all-features         # Fast compile check for all features
cargo test --all-features          # All tests (local + shared + shared-multi + doc tests)
```

**CI order:** `fmt --check` → `clippy -- -D warnings` → `cargo check --all-features` → `test --all-features`.

Always run the full suite before submitting changes.

## Agent expectations

- **Do**: Ask before making file renames or major structural changes.
- **Do**: Keep trait and module-level comments in sync with implementation.
- **Do**: Utilize sub-agents for complex refactors or new feature branches, acting as coordinator with greater oversight on shared code areas.
- **Do**: Work incrementally and validate with cargo commands after each logical change.
- **Do**: Update docs, doctests, and examples alongside code changes.
- **Do**: Report progress after 3–5 tool calls or after editing multiple files.

- **Do not**: Modify `src/old/` — it is legacy code kept for reference.
- **Do not**: Modify `tests/old/` — it is legacy test code.
- **Do not**: Make assumptions about internal storage types or trait boundaries without confirming with the architecture reference and existing code. If cannot confirm, ask for clarification before proceeding.
- **Do not**: Introduce new dependencies without discussion, especially for core structs, traits, or implementations.
- **Do not**: Skip testing edge cases or error scenarios, especially around lifetime exhaustion and concurrent emissions.
- **Do not**: Change public API traits or struct signatures without documenting the new usage pattern and confirming compile status.
- **Do not**: Overlook code security or efficiency implications of trait bounds, interior mutability, or atomic operations. Always validate with tests and code review.

## Language & conventions

- **Imports**: Users import all deps from `rs_events::`, not the imports submodule. `imports.rs` consolidates all external dependencies and re-exports them for internal use. Do not import directly from external crates in other modules.
- **Module Comments**: Each module must have a doc comment describing its purpose and key types. Use `//!` for module-level comments. submodules should only have a brief comment.
- **Doc comments**: Use doc comments `///` with `/// ```rust` doctests for all items. Trait methods, enum variants, etc can stay minimal; Structs and implementations must be descriptive for user docs. All examples must stay compile-ready.
- **Trait names**:
  - `EventMode` — base payload/callback contract.
  - `ListenerStorage` — listener tag and lifetime storage contract.
  - `ListenerApi` — listener operations trait.
  - `EmitterStorage` — event map and max_listeners contract.
  - `EmitterApi` — emitter operations trait (also called `EventHandler` in legacy code).
- **Struct re-exports**: `Listener`, `Emitter`, `LocalMode`, `SharedMode` are main exports. Storage trait impls are in submodules.

## Feature flags

| Feature | What it does |
| ------- | ------------ |
| `default` | None — local sync is the default mode |
| `async-tokio` | Adds `SharedMode` with `Arc<Mutex<BTreeMap>>` for async/single-threaded runtime |
| `multi-thread` | Replaces `Arc<Mutex<BTreeMap>>` with `Arc<DashMap>` for concurrent multi-threaded emission |
| `threaded` | Deprecated alias for `multi-thread` |

The crate has three compilation modes:

1. **Default (empty features)** — `LocalMode` + `Rc<RefCell<BTreeMap>>`. Local sync emission. No external deps beyond `spin`.
2. **async-tokio only** — `LocalMode` + `SharedMode` with `Arc<Mutex<BTreeMap>>`. Local + shared async.
3. **multi-thread (with or without async-tokio)** — `LocalMode` + `SharedMode` with `Arc<DashMap>`. Local + shared multi-threaded. `DashMap` overrides the BTreeMap storage.

`--all-features` must compile (all features together, not mutually exclusive).

## Testing

Tests mirror the feature-gating: `local/` always compiled, `shared/` gated on async/multi-thread, `shared-multi/` gated on multi-thread.

- **Location**: `src/tests/mod.rs` — cfg-gated entry point
  - `#[cfg(not(any(feature = "async-tokio", feature = "multi-thread")))]` — `mod local`
  - `#[cfg(any(feature = "async-tokio", feature = "multi-thread"))]` — `mod shared`
  - `#[cfg(feature = "multi-thread")]` — `mod shared_multi`
- **Structure**: Each mode tests every tier (EventMode, Listener, Emitter) with submodules for create/read/update/emit/remove operations. Storage tier has its own `listener_storage/` and `emitter_storage/` submodules per mode.
- **shared/** must include sync smoke tests to validate SharedMode works for non-async contexts.
- **shared-multi/** only tests where DashMap concurrency deviates from BTreeMap behavior.
- **examples/**: `src/examples/` has real-world patterns (embedded_sensor.rs, gui_button.rs, threaded_server.rs).

## Change reporting

When making changes to architecture or trait boundaries:

1. Note what tier changed (EventMode, ListenerStorage, EmitterStorage).
2. List affected files (storage impls, trait, concrete structs).
3. Confirm compile status via `cargo check` or `cargo test`.
4. If breaking user code, the breaking change should have been approved and needs documentation of the new usage pattern.
5. Record the change in a local log file such as `.local_logs.txt` for in-progress work or non-release changes.
6. Use `CHANGELOG.md` only for major version changes, release notes, or other user-facing history worth shipping.

## Gotchas & edge cases

- `emit_final` drains all listeners in one pass and removes the event key. Do not access the emitter after calling it.
- `emit_async(parallel=true)` uses concurrent tasks; `parallel=false` is sequential. Ensure proper synchronization if using shared state in callbacks.
- Lifetime exhaustion is normal: listeners are returned in the emission result when they hit their limit.
- `Emitter::Other` uses debug-format string comparison. Do not rely on `PartialEq` for variant-level equivalence.
- `#[cfg(test)]` mod is commented out in `lib.rs`. Tests live in `src/tests/` with their own `mod.rs`.
- In `lib.rs`, the re-exported `EventEmitter` name will be transitioned to `Emitter` in the refactor.
- `EventHandler` in `src/emitter/event_handler/mod.rs` will be deprecated in favor of `EmitterApi`.
- Storage `Map` types must provide interior mutability since they are stored behind `Arc`/`Rc`. Static trait methods receive maps by reference.

## Architecture reference

See [@@PROJECT.md](./PROJECT.md) for detailed architecture, trait hierarchy, file layout, and extensibility patterns.
