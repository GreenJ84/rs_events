//! This module defines [`ListenerMode`], which lets a generic listener choose
//! mode-specific callback and lifetime representations without duplicating
//! listener logic.
//!
//! ListenerMode abstracts callback and lifetime storage selection.


pub mod local;
pub use local::LocalMode;

#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use shared::SharedMode;

/// Behavior contract for listener storage backends.
///
/// Each mode chooses:
/// - callback type with invocation semantics
/// - lifetime counter storage type with  creation and decrement semantics
pub trait ListenerMode {
    /// Payload type for the callback, used for type inference in `Listener`.
    type Payload<T>: Clone;

    /// Mode-specific callback handle type.
    type Callback<T>: Clone;
    /// Compares two callback handles for pointer identity.
    fn callback_ptr_eq<T>(left: &Self::Callback<T>, right: &Self::Callback<T>) -> bool;
    /// Invokes the callback with the provided payload.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>);

    /// Mode-specific lifetime counter storage type.
    type Lifetime: Clone;
    /// Creates an optional lifetime counter from a listener limit.
    fn new_lifetime(limit: Option<u64>) -> Option<Self::Lifetime>;
    /// Returns the remaining call count for a lifetime-limited listener.
    fn remaining(lifetime: &Option<Self::Lifetime>) -> Option<u64>;
    /// Returns whether or not the listener has reached its call limit.
    fn at_limit(lifetime: &Option<Self::Lifetime>) -> bool;
    /// Attempts to decrement the remaining call counter.
    fn try_decrement(lifetime: &mut Option<Self::Lifetime>) -> bool;
}
