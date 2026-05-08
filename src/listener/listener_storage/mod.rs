//! Listener storage contract and concrete mode implementations.
//!
//! [`ListenerStorage`] defines how a mode creates, reads, and updates the
//! listener tag and lifetime values used by [`Listener`](crate::Listener).

use crate::EventMode;

mod local;
pub use local::*;

#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use shared::*;

/// [`Listener`](crate::Listener) storage contract for tag and lifetime values.
pub trait ListenerStorage: EventMode {
    /// Mode-specific listener tag type.
    type Tag: Clone;

    /// Creates a tag from a string-like input.
    fn new_tag(tag: impl Into<String>) -> Self::Tag;

    /// Gets the tag as a string slice.
    fn get_tag(tag: &Self::Tag) -> &str;

    /// Mode-specific lifetime counter storage type.
    type Lifetime: Clone;

    /// Creates a lifetime counter from a listener limit.
    fn new_lifetime(limit: usize) -> Self::Lifetime;

    /// Returns the remaining call count for a lifetime handle.
    fn get_lifetime(lifetime: &Self::Lifetime) -> usize;

    /// Sets or updates the lifetime counter.
    fn set_lifetime(lifetime: &Self::Lifetime, new_life: usize);

    /// Returns whether the listener has reached its call limit.
    fn at_limit(lifetime: &Self::Lifetime) -> bool;

    /// Attempts to decrement the remaining call counter.
    fn try_decrement(lifetime: &Self::Lifetime) -> bool;
}
