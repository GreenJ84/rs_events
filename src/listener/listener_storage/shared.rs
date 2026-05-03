//! Shared listener mode is used when listeners must be safe to share across tasks and/or
//! threads.
//!
//! It is backed by `Arc` and atomics.

use super::ListenerStorage;
use crate::{Arc, AtomicUsize, Ordering, SharedMode};

/// Tag type for shared listener mode
/// - uses `Arc<String>` for shared ownership and efficient cloning.
pub type SharedTag = Arc<String>;
/// Lifetime type for shared listener mode
/// - uses `Arc<AtomicUsize>` for shared ownership and atomic call counting.
pub type SharedLifetime = Arc<AtomicUsize>;

impl ListenerStorage for SharedMode {
    /// Tag type for shared listener mode
    /// - uses `Arc<String>` for shared ownership and efficient cloning.
    type Tag = SharedTag;

    /// Creates an optional tag from a string-like input.
    ///
    /// # Parameters
    /// - `tag: Option<impl Into<String>>`: Optional string-like input to create the tag from.
    ///
    /// # Returns
    /// - `Option<Self::Tag>`
    ///     - `Some(Arc<String>)` if a tag was created from the input string.
    ///     - `None` if no tag was provided.
    fn new_tag(tag: Option<impl Into<String>>) -> Option<Self::Tag> {
        tag.map(|t| Arc::new(t.into()))
    }

    /// Gets the tag as a string slice, if it exists.
    ///
    /// # Parameters
    /// - `tag: &Option<Self::Tag>`: The optional tag to get as a string slice.
    ///
    /// # Returns
    /// - `Option<&str>`
    ///     - `Some(&str)` if the tag exists
    ///     - `None` if not set
    fn get_tag(tag: &Option<Self::Tag>) -> Option<&str> {
        tag.as_ref().map(|t| t.as_str())
    }

    /// Sets or updates the tag based on a string-like input.
    ///
    /// # Parameters
    /// - `tag: &mut Option<Self::Tag>`: The optional tag to set or update based on the new string-like input.
    /// - `new_tag: Option<impl Into<String>>`: The new string-like input to create the tag from, or `None` to remove the tag.
    fn set_tag(tag: &mut Option<Self::Tag>, new_tag: Option<impl Into<String>>) {
        *tag = new_tag.map(|t| Arc::new(t.into()));
    }


    type Lifetime = SharedLifetime;

    /// Creates a shared atomic lifetime counter.
    ///
    /// # Parameters
    /// - `limit: Option<usize>`: Optional call limit for the listener. *`None` and `Some(0)` produce no counter (unlimited listener)*
    ///
    /// # Returns
    /// - `Option<Self::Lifetime>`
    ///     - `None` if no lifetime counter was created (unlimited listener).
    ///     - `Some(Arc<AtomicUsize>)` if a lifetime counter was created with the specified limit.
    ///
    fn new_lifetime(limit: Option<usize>) -> Option<Self::Lifetime> {
        match limit {
            Some(0) | None => None,
            Some(n) => Some(Arc::new(AtomicUsize::new(n))),
        }
    }

    /// Gets the number of remaining calls for a lifetime-limited listener.
    ///
    /// # Parameters
    /// - `lifetime: &Option<Self::Lifetime>`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    /// - `Option<usize>`
    ///     - `None` if the listener is unlimited (no lifetime counter).
    ///     - `Some(u64)` if the listener has a lifetime limit, representing the remaining call count.
    fn get_lifetime(lifetime: &Option<Self::Lifetime>) -> Option<usize>  {
        lifetime.as_ref().map(|a| a.load(Ordering::Acquire))
    }

    /// Sets or updates the lifetime counter based on a new listener limit.
    ///
    /// # Parameters
    /// - `lifetime: &mut Option<Self::Lifetime>`: The optional lifetime counter to set or update based on the new listener limit.
    /// - `new_life: Option<usize>`: The new listener limit to create the lifetime counter from; `Some(0)` or `None` to produce no counter (unlimited listener).
    fn set_lifetime(lifetime: &mut Option<Self::Lifetime>, new_life: Option<usize>) {
        *lifetime = match new_life {
            Some(0) | None => None,
            Some(n) => Some(Arc::new(AtomicUsize::new(n))),
        };
    }

    /// Checks whether the listener has reached its call limit.
    ///
    /// # Parameters
    /// - `lifetime:  &Option<Self::Lifetime>`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    /// - `bool`
    ///     - `true` if the listener has a lifetime limit and has reached it (0 calls remaining).
    ///     - `false` if the listener is unlimited or has remaining calls.
    fn at_limit(lifetime: &Option<Self::Lifetime>) -> bool {
        lifetime
            .as_ref()
            .is_some_and(|a| a.load(Ordering::Acquire) == 0)
    }

    /// Atomically decrements the call counter when possible.
    ///
    /// # Parameters
    /// - `lifetime: &mut Option<Self::Lifetime>`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    /// - `bool`
    ///     - `true` if the listener is valid for a call and lifetime was decremented (if applicable).
    ///     - `false` if the counter is already `0`.
    fn try_decrement(lifetime: &mut Option<Self::Lifetime>) -> bool {
        match lifetime {
            None => true,
            Some(a) => a
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                    (x > 0).then(|| x - 1)
                })
                .is_ok(),
        }
    }
}
