//! Local listener storage for single-threaded listener usage.
//!
//! This mode uses `Rc<String>` for tags and `Rc<Cell<usize>>` for lifetime counters.

use super::ListenerStorage;
use crate::{Cell, LocalMode, Rc};

/// Type alias for listener tags in local mode.
///
/// Uses `Rc<String>` for shared ownership.
///
/// # Example
/// ```rust
/// extern crate alloc;
/// use alloc::rc::Rc;
/// use rs_events::{listener::LocalTag};
///
/// let tag: LocalTag = Rc::new(String::from("listener-1"));
/// assert_eq!(tag.as_str(), "listener-1");
/// ```
pub type LocalTag = Rc<String>;

/// Type alias for listener lifetime counters in local mode.
///
/// Uses `Rc<Cell<usize>>` for interior mutability and shared ownership.
///
/// # Example
/// ```rust
/// extern crate alloc;
/// use alloc::rc::Rc;
/// use core::cell::Cell;
/// use rs_events::{listener::LocalLifetime};
///
/// let lifetime: LocalLifetime = Rc::new(Cell::new(3));
/// assert_eq!(lifetime.get(), 3);
/// ```
pub type LocalLifetime = Rc<Cell<usize>>;

impl ListenerStorage for LocalMode {
    /// Listener tag type in local mode.
    type Tag = LocalTag;

    /// Creates a tag from a string-like input.
    ///
    /// # Example
    /// ```rust
    /// use rs_events::{listener::ListenerStorage, LocalMode};
    ///
    /// let tag = LocalMode::new_tag("listener-1");
    /// assert_eq!(LocalMode::get_tag(&tag), "listener-1");
    /// ```
    fn new_tag(tag: impl Into<String>) -> Self::Tag {
        Rc::new(tag.into())
    }

    /// Gets the tag as a string slice.
    ///
    /// # Example
    /// ```rust
    /// extern crate alloc;
    /// use alloc::{rc::Rc};
    /// use rs_events::{listener::{ListenerStorage, LocalTag}, LocalMode};
    ///
    /// let tag: LocalTag = Rc::new(String::from("listener-1"));
    /// assert_eq!(LocalMode::get_tag(&tag), "listener-1");
    /// ```
    fn get_tag(tag: &Self::Tag) -> &str {
        tag.as_str()
    }

    /// Listener lifetime type in local mode.
    type Lifetime = LocalLifetime;

    /// Creates a local lifetime counter.
    ///
    /// # Example
    /// ```rust
    /// use rs_events::{listener::ListenerStorage, LocalMode};
    ///
    /// let lifetime = LocalMode::new_lifetime(2);
    /// assert_eq!(LocalMode::get_lifetime(&lifetime), 2);
    /// ```
    fn new_lifetime(limit: usize) -> Self::Lifetime {
        Rc::new(Cell::new(limit))
    }

    /// Sets the local lifetime counter value.
    ///
    /// # Example
    /// ```rust
    /// use rs_events::{listener::ListenerStorage, LocalMode};
    ///
    /// let lifetime = LocalMode::new_lifetime(2);
    /// LocalMode::set_lifetime(&lifetime, 5);
    /// assert_eq!(LocalMode::get_lifetime(&lifetime), 5);
    /// ```
    fn set_lifetime(lifetime: &Self::Lifetime, limit: usize) {
        lifetime.set(limit);
    }

    /// Reads the current remaining call count.
    ///
    /// # Example
    /// ```rust
    /// use rs_events::{listener::ListenerStorage, LocalMode};
    ///
    /// let lifetime = LocalMode::new_lifetime(3);
    /// assert_eq!(LocalMode::get_lifetime(&lifetime), 3);
    /// ```
    fn get_lifetime(lifetime: &Self::Lifetime) -> usize {
        lifetime.get()
    }

    /// Checks whether the listener has reached its call limit.
    ///
    /// # Example
    /// ```rust
    /// use rs_events::{listener::ListenerStorage, LocalMode};
    ///
    /// let lifetime = LocalMode::new_lifetime(0);
    /// assert!(LocalMode::at_limit(&lifetime));
    /// ```
    fn at_limit(lifetime: &Self::Lifetime) -> bool {
        lifetime.get() == 0
    }

    /// Decrements the call counter when possible.
    ///
    /// # Example
    /// ```rust
    /// use rs_events::{listener::ListenerStorage, LocalMode};
    ///
    /// let lifetime = LocalMode::new_lifetime(1);
    /// assert!(LocalMode::try_decrement(&lifetime));
    /// assert_eq!(LocalMode::get_lifetime(&lifetime), 0);
    /// ```
    fn try_decrement(lifetime: &Self::Lifetime) -> bool {
        let current = lifetime.get();
        if current > 0 {
            lifetime.set(current - 1);
            true
        } else {
            false
        }
    }
}
