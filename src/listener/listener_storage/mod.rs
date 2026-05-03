//! This module defines the [`ListenerStorage`] type and type
//! interaction contract trait, which lets a generic listener
//! choose mode-specific callback and lifetime representations
//! without duplicating listener logic.
//!
//! [`ListenerStorage`] abstracts callback and lifetime storage selection and interactions and requires a EventMode implementation for defined payload and callback types.

use crate::EventMode;


mod local;
pub use local::*;

#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use shared::*;

/// [`Listener`](crate::Listener) storage type and storage interaction contract.
///
/// Each mode chooses:
/// - tag type with invocation semantics
/// - lifetime counter storage type with creation and decrement semantics
pub trait ListenerStorage: EventMode {
    /// Listener tag type for tracking or grouping listeners.
    type Tag: Clone;
    /// Creates an optional tag from a string-like input.
    fn new_tag(tag: Option<impl Into<String>>) -> Option<Self::Tag>;
    /// Gets the tag as a string slice, if it exists.
    fn get_tag(tag: &Option<Self::Tag>) -> Option<&str>;
    /// Sets or updates the tag based on a string-like input.
    fn set_tag(tag: &mut Option<Self::Tag>, new_tag: Option<impl Into<String>>);

    /// Mode-specific lifetime counter storage type.
    type Lifetime: Clone;
    /// Creates an optional lifetime counter from a listener limit.
    fn new_lifetime(limit: Option<usize>) -> Option<Self::Lifetime>;
    /// Returns the remaining call count for a lifetime-limited listener.
    fn get_lifetime(lifetime: &Option<Self::Lifetime>) -> Option<usize>;
    /// Sets or updates the lifetime counter based on a new listener limit.
    fn set_lifetime(lifetime: &mut Option<Self::Lifetime>, new_life: Option<usize>);
    /// Returns whether or not the listener has reached its call limit.
    fn at_limit(lifetime: &Option<Self::Lifetime>) -> bool;
    /// Attempts to decrement the remaining call counter.
    fn try_decrement(lifetime: &mut Option<Self::Lifetime>) -> bool;
}
