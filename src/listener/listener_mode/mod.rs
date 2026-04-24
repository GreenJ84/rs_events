//! Listener mode abstractions for selecting callback and lifetime storage.
//!
//! This module defines [`ListenerMode`], which lets a generic listener choose
//! mode-specific callback and lifetime representations without duplicating
//! listener logic.

/// Local (single-threaded) listener mode.
pub mod local;
/// Shared listener mode for multi-threaded and async-tokio configurations.
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub mod shared;

/// Behavior contract for listener storage backends.
///
/// Each mode chooses:
/// - callback type
/// - lifetime counter storage type
/// - lifetime creation and decrement semantics
pub trait ListenerMode {
    /// Mode-specific callback handle type.
    type Callback<T>: Clone;
    /// Mode-specific lifetime counter storage type.
    type Lifetime: Clone;

    /// Creates an optional lifetime counter from a listener limit.
    ///
    /// `None` and `Some(0)` are treated as unlimited by convention.
    fn new_lifetime(limit: Option<u64>) -> Option<Self::Lifetime>;
    /// Returns the remaining call count for a lifetime-limited listener.
    fn remaining(l: &Option<Self::Lifetime>) -> Option<u64>;
    /// Returns `true` when the listener has reached its call limit.
    fn at_limit(l: &Option<Self::Lifetime>) -> bool;
    /// Attempts to decrement the remaining call counter.
    ///
    /// Returns `true` if the listener may continue and `false` when already exhausted.
    fn try_decrement(l: &mut Option<Self::Lifetime>) -> bool;
}
