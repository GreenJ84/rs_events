//! Local listener mode is intended for single-threaded listener usage where atomic
//! synchronization is unnecessary.
//!
//! It is backed by Local aliases, `Rc` and `Cell`.

use super::ListenerStorage;
use crate::{Cell, LocalMode, Rc};

/// Type alias for listener tags in local mode
/// - uses `Rc<String>` for shared ownership.
pub type LocalTag = Rc<String>;
/// Type alias for listener lifetime counters in local mode
/// - uses `Rc<Cell<usize>>` for interior mutability and shared ownership.
pub type LocalLifetime = Rc<Cell<usize>>;

impl ListenerStorage for LocalMode {
    /// Type alias for listener tags in local mode
    /// - uses `Rc<String>` for and shared ownership.
    type Tag = LocalTag;
    /// Creates an optional tag from a string-like input.
    ///
    /// # Parameters
    /// - `tag: Option<impl Into<String>>`: Optional string-like input to create the tag from.
    ///
    /// # Returns
    /// - `Option<Self::Tag>`
    ///    - `Some(Tag)` if a tag was created from the input string.
    ///    - `None` if no tag was provided.
    fn new_tag(tag: Option<impl Into<String>>) -> Option<Self::Tag> {
        tag.map(|t| Rc::new(t.into()))
    }

    /// Gets the tag as a string slice, if it exists.
    ///
    /// # Parameters
    /// - `tag: &Option<Self::Tag>`: The optional tag to get as a string slice.
    ///
    /// # Returns
    /// - `Option<&str>`
    ///   - `Some(tag)` if the tag exists
    ///   - `None` if not set
    fn get_tag(tag: &Option<Self::Tag>) -> Option<&str> {
        tag.as_ref().map(|t| t.as_str())
    }

    /// Sets or updates the tag based on a string-like input.
    ///
    /// # Parameters
    /// - `tag: &mut Option<Self::Tag>`: The optional tag to set or update based on the input string.
    /// - `new_tag: Option<impl Into<String>>`: Optional string-like input to set the tag to.
    ///     - If `Some(t)`, the tag will be set to `t`.
    ///     - If `None`, the tag will be removed (set to `None`).
    fn set_tag(tag: &mut Option<Self::Tag>, new_tag: Option<impl Into<String>>) {
         *tag = new_tag.map(|t| Rc::new(t.into()));
    }


    /// Type alias for listener lifetime counters in local mode
    /// - uses `Rc<Cell<usize>>` for interior mutability and shared ownership.
    type Lifetime = LocalLifetime;

    /// Creates a local lifetime counter.
    ///
    /// # Parameters
    /// - `limit: Option<usize>`: Optional call limit for the listener. *`None` and `Some(0)` produce no counter (unlimited listener)*
    ///
    /// # Returns
    /// - `Option<Self::Lifetime>`
    ///     - `None` if no lifetime counter was created (unlimited listener).
    ///     - `Some(Rc<Cell<usize>>)` if a lifetime counter was created with
    fn new_lifetime(limit: Option<usize>) -> Option<Self::Lifetime> {
        match limit {
            Some(0) | None => None,
            Some(n) => Some(Rc::new(Cell::new(n))),
        }
    }

    /// Set a new local lifetime counter (or erase the lifetime for unlimited).
    /// # Parameters
    /// - `lifetime:  &mut Option<Self::Lifetime`: The optional lifetime counter to set or update based on the input limit.
    /// - `limit: Option<usize>`: Optional call limit for the listener. *`None` and `Some(0)` produce no counter (unlimited listener)*
    fn set_lifetime(lifetime: &mut Option<Self::Lifetime>, limit: Option<usize>) {
        match limit {
            Some(0) | None => {
                *lifetime = None;
            }
            Some(n) => *lifetime = Some(Rc::new(Cell::new(n))),
        }
    }

    /// Reads the current remaining call count.
    ///
    /// # Parameters
    /// - `lifetime: &Option<Self::Lifetime>`: The optional lifetime counter to read.
    ///
    /// # Returns
    /// - `Option<usize>`
    ///     - `None` for unlimited listeners
    ///     - `Some(n)` for listeners with `n` calls left.
    fn get_lifetime(lifetime: &Option<Self::Lifetime>) -> Option<usize> {
        lifetime.as_ref().map(|c| c.get())
    }

    /// Checks whether the listener has reached its call limit.
    ///
    /// # Parameters
    /// - `lifetime: &Option<Self::Lifetime>`: The optional lifetime counter to check.
    ///
    /// # Returns
    /// - `bool`
    ///     - `true` if the listener has a lifetime limit and has reached it (0 calls remaining).
    ///     - `false` if the listener is unlimited or has remaining calls.
    fn at_limit(lifetime: &Option<Self::Lifetime>) -> bool {
        lifetime.as_ref().is_some_and(|c| c.get() == 0)
    }

    /// Decrements the call counter when possible.
    ///
    /// # Parameters
    /// - `lifetime: &Option<Self::Lifetime>`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    /// - `bool`
    ///     - `true` if the listener is valid for a call (not at limit or unlimited) and lifetime was decremented if applicable.
    ///     - `false` if the listener is at limit and cannot be called.
    fn try_decrement(lifetime: &mut Option<Self::Lifetime>) -> bool {
        match lifetime {
            None => true,
            Some(c) if c.get() > 0 => {
                c.set(c.get() - 1);
                true
            }
            Some(_) => false,
        }
    }
}
