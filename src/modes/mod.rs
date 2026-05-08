//! Base event mode contracts and the concrete `LocalMode` and `SharedMode` implementations.
//!
//! This module defines [`EventMode`], the shared trait contract for payload and callback types,
//! and re-exports the concrete mode implementations used throughout the crate.

pub(crate) mod local;
pub use self::local::*;

#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub(crate) mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use self::shared::*;

/// Trait contract for the base payload and callback types used across the crate.
pub trait EventMode {
    /// Mode-specific payload type used in the crate.
    type Payload<T>: Clone;

    /// Mode-specific callback type used in the crate.
    type Callback<T>: Clone;

    /// Invokes the callback with the provided payload.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>);
}
