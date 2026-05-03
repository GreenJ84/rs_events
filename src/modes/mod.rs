///! This module contains the base trait [`EventMode`](EventMode) alongside the [`LocalMode`]() and [`SharedMode`]() implementations of the trait. The module re-exports for public usage/implementation, and for the higher-order abstraction overlay in other modules this crate implements.


pub(crate) mod local;
pub use self::local::*;

/// Shared constants that require Send + Sync for asynchronous and/or thread safety
///
/// They are only included if the multi_threaded feature or an async feature is enabled
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub(crate) mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use self::shared::*;

/// This trait defines the type contract for the base payload and callbacks use in the `rs_events` event system across the crate, allowing for flexible callback and lifetime management across different listener modes.
pub trait EventMode {
    /// Mode-specific Payload type for callbacks used in the crate.
    type Payload<T>: Clone;

    /// Mode-specific callback type used in the crate.
    type Callback<T>: Clone;
    /// Invokes the callback with the provided payload.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>);
}