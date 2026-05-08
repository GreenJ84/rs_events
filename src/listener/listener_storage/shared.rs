//! Shared listener storage for async and multi-threaded listener usage.
//!
//! This mode uses `Arc<String>` for tags and `Arc<AtomicUsize>` for lifetime counters.

use super::ListenerStorage;
use crate::{Arc, AtomicUsize, Ordering, SharedMode};

/// Tag type for shared listener mode.
///
/// Uses `Arc<String>` for shared ownership and efficient cloning.
///
/// # Example
/// ```rust
/// use crate::{listener::SharedTag, Arc};
///
/// let tag: SharedTag = Arc::new(String::from("listener-1"));
/// assert_eq!(tag.as_str(), "listener-1");
/// ```
pub type SharedTag = Arc<String>;

/// Lifetime type for shared listener mode.
///
/// Uses `Arc<AtomicUsize>` for shared ownership and atomic call counting.
///
/// # Example
/// ```rust
/// use crate::{listener::SharedLifetime, Arc, AtomicUsize, Ordering};
///
/// let lifetime: SharedLifetime = Arc::new(AtomicUsize::new(3));
/// assert_eq!(lifetime.load(Ordering::Acquire), 3);
/// ```
pub type SharedLifetime = Arc<AtomicUsize>;

impl ListenerStorage for SharedMode {
    /// Listener tag type in shared mode.
    type Tag = SharedTag;

    /// Creates a tag from a string-like input.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::ListenerStorage, SharedMode};
    ///
    /// let tag = SharedMode::new_tag("listener-1");
    /// assert_eq!(SharedMode::get_tag(&tag), "listener-1");
    /// ```
    fn new_tag(tag: impl Into<String>) -> Self::Tag {
        Arc::new(tag.into())
    }

    /// Gets the tag as a string slice.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::{ListenerStorage, SharedTag}, Arc, SharedMode};
    ///
    /// let tag: SharedTag = Arc::new(String::from("listener-1"));
    /// assert_eq!(SharedMode::get_tag(&tag), "listener-1");
    /// ```
    fn get_tag(tag: &Self::Tag) -> &str {
        tag.as_str()
    }

    /// Listener lifetime type in shared mode.
    type Lifetime = SharedLifetime;

    /// Creates a shared atomic lifetime counter.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::ListenerStorage, SharedMode};
    ///
    /// let lifetime = SharedMode::new_lifetime(2);
    /// assert_eq!(SharedMode::get_lifetime(&lifetime), 2);
    /// ```
    fn new_lifetime(limit: usize) -> Self::Lifetime {
        Arc::new(AtomicUsize::new(limit))
    }

    /// Gets the number of remaining calls for a lifetime handle.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::ListenerStorage, SharedMode};
    ///
    /// let lifetime = SharedMode::new_lifetime(3);
    /// assert_eq!(SharedMode::get_lifetime(&lifetime), 3);
    /// ```
    fn get_lifetime(lifetime: &Self::Lifetime) -> usize {
        lifetime.load(Ordering::Acquire)
    }

    /// Sets or updates the lifetime counter.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::ListenerStorage, SharedMode};
    ///
    /// let lifetime = SharedMode::new_lifetime(2);
    /// SharedMode::set_lifetime(&lifetime, 5);
    /// assert_eq!(SharedMode::get_lifetime(&lifetime), 5);
    /// ```
    fn set_lifetime(lifetime: &Self::Lifetime, new_life: usize) {
        lifetime.store(new_life, Ordering::Release);
    }

    /// Checks whether the listener has reached its call limit.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::ListenerStorage, SharedMode};
    ///
    /// let lifetime = SharedMode::new_lifetime(0);
    /// assert!(SharedMode::at_limit(&lifetime));
    /// ```
    fn at_limit(lifetime: &Self::Lifetime) -> bool {
        lifetime.load(Ordering::Acquire) == 0
    }

    /// Atomically decrements the call counter when possible.
    ///
    /// # Example
    /// ```rust
    /// use crate::{listener::ListenerStorage, SharedMode};
    ///
    /// let lifetime = SharedMode::new_lifetime(1);
    /// assert!(SharedMode::try_decrement(&lifetime));
    /// assert_eq!(SharedMode::get_lifetime(&lifetime), 0);
    /// ```
    fn try_decrement(lifetime: &Self::Lifetime) -> bool {
        lifetime
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                (x > 0).then(|| x - 1)
            })
            .is_ok()
    }
}
