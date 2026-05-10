# PROJECT.md — rs_events

Detailed architecture, design philosophy, and project structure for the `rs_events` event emitter crate.

## Project overview

Single lib crate (`rs_events`). Public API entry: `src/lib.rs`.

**Design goal**: Ergonomic, mode-driven event primitives with composable abstractions. Async is an extension, not a trade-off.

## Design philosophy

The crate provides two concrete modes that users can plug into generic `Listener<M>` and `Emitter<M>` types:

- **`LocalMode`** — single-threaded. Uses `Rc`/`Cell`/`RefCell`. Intended for sync-only emission; cannot be used in async contexts.
- **`SharedMode`** — thread-safe. Uses `Arc`/atomics/`Mutex`/`DashMap`. Safe for both sync and async emission.

**Key principle**: `SharedMode` listeners work in local (sync) emission, but `LocalMode` listeners **cannot** run in async emission. This is enforced at the type level via trait bounds.

**Extensibility**: Users can implement custom `EventMode`, `ListenerStorage`, and `EmitterStorage` implementations and plug them into the generic types. API traits (`ListenerApi`, `EmitterApi`) are intentionally public so users may implement only the traits without using provided concrete structs (optional trait-only feature possible).

## Cargo features

| Feature        | What it does                          |
|---------------|---------|---------|-------|
| `default`    | None — local sync is the default mode |
| `async-tokio`| Adds `SharedMode` with `Arc<Mutex<BTreeMap>>` for async/single-threaded runtime |
| `multi-thread` | Replaces `Arc<Mutex<BTreeMap>>` with `Arc<DashMap>` for concurrent multi-threaded emission |
| `threaded`   | Deprecated alias for `multi-thread`   |

`spin` is always included (unconditional dep).

Three compilation modes:

1. **Default (empty features)** — `LocalMode` + `Rc<RefCell<BTreeMap>>`. Local sync emission. No external deps beyond `spin`.
2. **async-tokio only** — `LocalMode` + `SharedMode` with `Arc<Mutex<BTreeMap>>`. Local + shared async.
3. **multi-thread (with or without async-tokio)** — `LocalMode` + `SharedMode` with `Arc<DashMap>`. `DashMap` overrides the BTreeMap storage.

`--all-features` must compile (all features together, not mutually exclusive).

## Core tiers (trait-based static contracts)

Each tier is expressed as a trait-based static contract with pluggable implementations. Storage traits use static methods; concrete structs delegate to them rather than mutating storage directly.

### Tier 1: EventMode

**Location**: `src/modes/mod.rs`

**Contract**:

```rust
pub trait EventMode {
    type Payload<T>: Clone;
    type Callback<T>: Clone;
    
    fn invoke_callback<T>(...);
}
```

**Concrete implementations**:

- `LocalMode` — single-threaded primitives (`Rc`, `Cell`, `RefCell`).
- `SharedMode` — thread-safe primitives (`Arc`, atomics, `Mutex`/`DashMap`).

**Notes**:

- `SharedMode` callbacks must be `Send + Sync + 'static`.
- `LocalMode` callbacks may hold non-Send/non-Sync data.

### Tier 2: ListenerStorage (with Listener struct)

**Location**: `src/listener/listener_storage/mod.rs`

**Contract**: Static storage helpers for listener tag and concrete lifetime type operations.

```rust
pub trait ListenerStorage: EventMode {
    type Tag;
    type Lifetime;
    
    fn new_lifetime(limit: usize) -> Self::Lifetime;
    fn get_lifetime(lifetime: &Self::Lifetime) -> usize;
    fn set_lifetime(lifetime: &mut Self::Lifetime, count: usize);
    fn at_limit(lifetime: &Self::Lifetime) -> bool;
    fn try_decrement(lifetime: &Self::Lifetime) -> bool;
}
```

**Concrete implementations**:

- `src/listener/listener_storage/local.rs` — `LocalMode`:
  - `Tag = Rc<String>`
  - `Lifetime = Rc<Cell<usize>>`
- `src/listener/listener_storage/shared.rs` — `SharedMode`:
  - `Tag = Arc<String>`
  - `Lifetime = Arc<AtomicUsize>`

**Listener struct** (`src/listener/mod.rs`):

- `Listener<T, M: ListenerStorage>` owns `Option<M::Lifetime>`, a `Option<M::Tag>`, and a callback of type `M::Callback<T>`.
- Owns option handling; delegates `M::` type operations to `M::` static methods.
- Implements `ListenerApi` trait (public API contract).

**ListenerApi** (`src/listener/listener_api.rs`):

- Public listener operations: `get_tag()`, `set_tag()`, `get_callback()`, `get_lifetime()`, `set_lifetime()`, `call()`, `at_limit()`, etc.

**ListenerError** (`src/listener/listener_error.rs`):

- Listener-specific errors: `Exhausted`, `InvocationFailure`, `StorageError`, etc.

### Tier 3: EmitterStorage (with Emitter struct)

**Location**: `src/emitter/emitter_storage/mod.rs`

**Contract**: Static storage helpers for event maps, max_listeners handling, and CRUD operations.

```rust
pub trait EmitterStorage: ListenerStorage {
    type Map<T>;
    type MaxListeners;
    
    fn new_events() -> Self::Map<T>;
    fn add_listener<T>(...) -> Result<(), EmitterError>;
    fn add_listeners<T>(...) -> Result<(), EmitterError>;
    fn get_listeners<T>(...) -> Result<Vec<Listener<T>>, EmitterError>;
    fn get_listeners_by_tag<T>(...) -> Result<Vec<Listener<T>>, EmitterError>;
    fn has_listener<T>(...) -> Result<bool, EmitterError>;
    fn remove_listener<T>(...) -> Result<(), EmitterError>;
    fn remove_listeners<T>(...) -> Result<(), EmitterError>;
    fn set_max_listeners(...);
    fn get_max_listeners(...) -> usize;
}
```

**Concrete implementations** (three provided):

1. **Local sync** (`src/emitter/emitter_storage/local.rs`):
   - `Map<T> = Rc<RefCell<BTreeMap<String, Vec<Listener<T>>>>>`
   - `MaxListeners = Rc<Cell<usize>>`
   - Single-threaded, sync-only emission.

2. **Shared single-thread async** (`src/emitter/emitter_storage/shared_single.rs`):
   - `Map<T> = Arc<Mutex<BTreeMap<String, Vec<Listener<T>>>>>`
   - `MaxListeners = Arc<AtomicUsize>`
   - Allows async but single-threaded runtime (e.g., `tokio::task::block_in_place`).

3. **Shared multi-thread** (`src/emitter/emitter_storage/shared_multi.rs`):
   - `Map<T> = Arc<DashMap<String, Vec<Listener<T>>>>`
   - `MaxListeners = Arc<AtomicUsize>`
   - Concurrent, multi-threaded emission.
   - Mutually exclusive with `shared_single` via `#[cfg]` feature gating.

**Emitter struct** (`src/emitter/mod.rs`):

- `Emitter<T, M: EmitterStorage>` owns `Option<M::Map<T>>`, M::MaxListeners, and delegates to `M::` static methods for all storage operations.
- Delegates storage operations to `M::` static methods.
- Implements `EmitterApi` trait (public API contract).

**EmitterApi** (`src/emitter/emitter_api.rs` or legacy `event_handler.rs`):

- Public emitter operations: `add_listener()` [`on()`, `once()`, etc aliases], `emit()`, `emit_async()`, `emit_final()`, `remove_all_listeners()`, etc.
- May be named `EventHandler` in legacy code; transitioning to `EmitterApi`.

**EmitterError** (`src/emitter/error.rs` or `src/error.rs`):

- Emitter-level errors: `EventNotFound`, `ListenerNotFound`, `MaxListenersExceeded`, etc.
- May be named `EventError` in legacy code; transitioning to `EmitterError` for clarity.

## Code safety & efficiency

- **Trait bounds matter**: `LocalMode` listeners must never compile in async contexts. Use type-level enforcement via trait bounds (e.g., `M: SharedMode`).
- **Interior mutability**: Storage types are passed by reference to static trait methods, so map types must provide interior mutability (`Rc<RefCell<...>>`, `Arc<Mutex<...>>`, `Arc<DashMap<...>>`).
- **Option handling**: Structs own `Option<>` wrappers. Storage methods operate on the concrete types, not the Option. Storage never manipulates the Option.
- **Callback cloning**: Callbacks are wrapped in `Rc` or `Arc` for cheap sharing across listeners. Do not require `Clone` beyond the wrapper.
- **Atomic operations**: `SharedMode` lifetime counters use `Arc<AtomicUsize>` with safe increment/decrement. Validate all `Ordering` choices and test underflow scenarios.

## File layout (high level)

```text
src/
├── lib.rs                           # Public API re-exports
├── modes/
│   └── mod.rs                       # EventMode trait; LocalMode/SharedMode types
├── listener/
│   ├── mod.rs                       # Listener<T, M> struct
│   ├── listener_api.rs              # ListenerApi trait
│   ├── listener_error.rs            # ListenerError enum
│   └── listener_storage/
│       ├── mod.rs                   # ListenerStorage trait
│       ├── local.rs                 # Local: Rc<Cell<usize>>, Rc<String>
│       └── shared.rs                # Shared: Arc<AtomicUsize>, Arc<String>
├── emitter/
│   ├── mod.rs                       # Emitter<T, M> struct
│   ├── emitter_api.rs               # EmitterApi trait (or event_handler.rs)
│   ├── error.rs                     # EmitterError enum
│   └── emitter_storage/
│       ├── mod.rs                   # EmitterStorage trait
│       ├── local.rs                 # Local: Rc<RefCell<BTreeMap>>, Rc<Cell<usize>>
│       ├── shared_single.rs         # Shared (single-thread): Arc<Mutex<BTreeMap>>, Arc<AtomicUsize>
│       └── shared_multi.rs          # Shared (multi-thread): Arc<DashMap>, Arc<AtomicUsize>
├── tests/
│   └── mod.rs                       # Test suites
│   ...
└── old/                             # Legacy pre-refactor code (do not modify)
```

## Extensibility patterns

All tiers are designed for user extension. Users should be able to create a custom mode implementing `EventMode`, `ListenerStorage`, and `EmitterStorage` traits and plug them into the generic `Listener<T, M>` and `Emitter<T, M>` structs. Users should be able to create a custom wrapper that can forward and intercept specific implementations on the defined modes.

API traits are public for users who want to implement their own concrete types without using the provided structs.

### Implementing custom EventMode

Users can implement their own `EventMode` by defining the associated types and `invoke_callback` logic:

```rust
pub trait EventMode {
    type Payload<T>: Clone;
    type Callback<T>: Clone;

    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: Self::Payload<T>) -> Result<(), EventError>;
}
```

Then plug into `Listener<T, M>` and `Emitter<T, M>`:

```rust
let listener = Listener::<MyEvent, MyMode>::new(...);
let emitter = Emitter::<MyEvent, MyMode>::new();
```

### Implementing custom ListenerStorage

Users can provide custom `Tag` and `Lifetime` types by implementing `ListenerStorage`:

```rust
pub trait ListenerStorage: EventMode {
    type Tag;
    type Lifetime;
    
    fn new_lifetime(limit: usize) -> Self::Lifetime;
    fn get_lifetime(lifetime: &Self::Lifetime) -> usize;
    fn set_lifetime(lifetime: &mut Self::Lifetime, count: usize);
    fn at_limit(lifetime: &Self::Lifetime) -> bool;
    fn try_decrement(lifetime: &Self::Lifetime) -> bool;
}
```

Then plug into `Listener<T, M>`:

```rust
let listener = Listener::<MyEvent, MyMode>::new(...);
```

### Implementing custom EmitterStorage

Users can provide custom `Map<T>` and `MaxListeners` types by implementing `EmitterStorage`:

```rust
pub trait EmitterStorage: ListenerStorage {
    type Map<T>;
    type MaxListeners;
    
    fn new_events() -> Self::Map<T>;
    // ... CRUD and max_listeners methods
}
```

Then plug into `Emitter<T, M>`:

```rust
let emitter = Emitter::<MyEvent, MyMode>::new();
```

### Using API traits only

If users prefer to implement their own `Listener`/`Emitter` types without using the concrete structs, they can conform to `ListenerApi` and `EmitterApi` traits directly:

```rust
pub trait ListenerApi<T>: Sized {
    fn get_tag(&self) -> String;
    fn get_callback(&self) -> Self::Callback;
    fn call(&self) -> Result<(), ListenerError>;
    // ... other methods
}

pub trait EmitterApi<T>: Sized {
    fn on<C>(&mut self, event: impl Into<String>, callback: C);
    fn once<C>(&mut self, event: impl Into<String>, callback: C);
    fn emit(&mut self, event: impl Into<String>, payload: T) -> Result<(), EmitterError>;
    // ... other methods
}
```

Possible trait-only feature flag to exclude concrete structs if desired.

## Key constraints and conventions

1. **Async is optional**: `SharedMode` is safe for async emission. `LocalMode` cannot be used in async contexts — enforce via type-level trait bounds.
2. **Storage is static**: Trait methods on storage implementations receive references (not `&mut`), so map types must provide interior mutability.
3. **Option ownership**: `Listener` owns `Option<M::Lifetime>`; storage methods operate on the concrete `Lifetime`. Storage never manipulates the Option.
4. **Callback wrapping**: Callbacks are wrapped in `Rc` (LocalMode) or `Arc` (SharedMode) for cheap sharing.
5. **Interior mutability required**: Maps stored behind `Arc` must provide interior mutability (`Mutex`, `DashMap`, etc.).

## Testing

Test directory follows the crate's tiered architecture. No trait-only tests — all tests exercise concrete `Listener<M>` and `Emitter<M>` structs within their mode context.

**Location**: `src/tests/` with feature-gated top-level module.

- **`local/`** — `LocalMode` only. Sync emission. Tests every tier under single-threaded assumptions.
- **`shared/`** — `SharedMode`. Primarily async tests, plus smoke tests for sync emission to validate SharedMode compatibility with non-async contexts. Tests concurrent emissions and atomic operations.
- **`shared-multi/`** — `SharedMode` with `Arc<DashMap>`. Only tests where multi-thread behavior deviates from shared-single (e.g., race conditions, concurrent reads/writes under `DashMap` vs `Mutex`).

### Test directory layout

```text
src/tests/
├── mod.rs                           ← cfg-gated: threaded → shared | shared-multi | else → local
│
├── local/                           ← LocalMode: sync-only
│   ├── mod.rs
│   │
│   ├── mode/                        ← EventMode tier: payload + callback ops
│   │   ├── mod.rs                   ← payload creation (various T: u32, String, Vec, custom, etc.)
│   │   └── callback.rs              ← callback creation and invocation
│   │
│   ├── listener/                    ← Listener tier
│   │   ├── mod.rs
│   │   ├── create/                  ← Listener<T, M>::new(), tag, lifetime construction
│   │   ├── reads/                   ← get_tag, get_lifetime, get_callback, at_limit()
│   │   ├── updates/                 ← set_tag, set_lifetime
│   │   ├── removes/                 ← try_decrement, at_limit exhaustion
│   │   └── listener_storage/        ← ListenerStorage tier
│   │       ├── mod.rs               ← tag storage ops (Rc<String>)
│   │       └── lifetime.rs          ← lifetime storage ops (Rc<Cell<usize>>)
│   │
│   └── emitter/                     ← Emitter tier
│       ├── mod.rs
│       ├── create/                  ← Emitter<T, M>::new(), max_listeners init
│       ├── reads/                   ← get_listeners, get_max_listeners, event_names(), etc.
│       ├── updates/                 ← set_max_listeners()
│       ├── emits/                   ← emit(), emit_final() sync operations
│       ├── removes/                 ← remove_listener, remove_listeners, remove_listeners_by_tag()
│       └── emitter_storage/         ← EmitterStorage tier
│           ├── mod.rs               ← max_listeners ops (Rc<Cell<usize>>)
│           └── map.rs               ← map ops (Rc<RefCell<BTreeMap>>)
│
├── shared/                          ← SharedMode: async primary + sync smoke
│   ├── mod.rs
│   │
│   ├── mode/                        ← EventMode tier (payload + callback ops)
│   │   ├── mod.rs
│   │   └── callback.rs
│   │
│   ├── listener/
│   │   ├── mod.rs
│   │   ├── create/
│   │   ├── reads/
│   │   ├── updates/
│   │   ├── removes/
│   │   └── listener_storage/
│   │       ├── mod.rs
│   │       └── lifetime.rs
│   │
│   └── emitter/
│       ├── mod.rs
│       ├── create/
│       ├── reads/
│       ├── updates/
│       ├── emits/                   ← emit_async(blocking), emit_async(parallel), emit_final_async()
│       ├── removes/
│       └── emitter_storage/
│           ├── mod.rs
│           └── map.rs
│
├── shared-multi/                    ← SharedMode + DashMap: concurrency deviations only
│   ├── mod.rs
│   │
│   ├── listener/
│   │   └── listener_storage/
│   │       └── shared_multi.rs      ← Arc<AtomicUsize> concurrency (vs Mutex in shared)
│   │
│   └── emitter/
│       ├── mod.rs
│       ├── creates/                 ← DashMap initialization
│       ├── reads/                   ← concurrent reads vs mutex reads
│       ├── updates/                 ← concurrent set_max_listeners
│       ├── emits/                   ← concurrent emit_parallel edge cases
│       ├── removes/                 ← concurrent removal race conditions
│       └── emitter_storage/
│           └── map.rs               ← Arc<DashMap> concurrent ops
│
└── old/                             ← legacy test suite (base + threaded) before refactor
```

**Current status**: Legacy `#[cfg(test)]` in `lib.rs` is commented out. Legacy tests live in `src/tests/` under `base/` and `threaded/`. These must be moved to `tests/old/` and replaced with the tiered structure above.
**Examples**: `examples/` contains real-world usage patterns (embedded_sensor.rs, gui_button.rs, threaded_server.rs).

## Legacy code

- **Location**: `src/old/` — preserved pre-refactor code.
- **Status**: Do not modify. Reference only if needed to understand old patterns.
