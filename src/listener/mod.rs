//! Event listener implementation for the `rs_events` crate.
//!
//! This module provides the generic [`Listener<T, M>`] wrapper struct and the listener mode
//! abstractions that decides how payloads, callbacks,  and lifetimes are stored.
//!
//! Modes [M] include:
//! - [`LocalMode`] for single-threaded use
//! - [`SharedMode`] for multi-threaded and async-tokio use
//!
//! [`Listener<T>`] defaults to [`LocalMode`].
//!
//! Listener metadata includes:
//!
//! - **Tags**: Optional string identifiers for tracking or grouping listeners.
//!
//! # Quick Start
//!
//! Create a local listener with a callback and optional metadata:
//!
//! ```rust
//! use std::rc::Rc;
//! use rs_events::{Listener, LocalPayload};
//!
//! // Unlimited calls, tagged for easy identification.
//! let listener = Listener::local(
//!     Some("my_handler".to_string()), // tag
//!     Rc::new(|payload: &LocalPayload<String>| {
//!         println!("Event: {}", payload);
//!     }), // callback
//!     None, // lifetime
//! );
//! ```
//!
//! Create a shared listener with a callback and optional metadata:
//!
//! ```rust
//! #[cfg(any(feature = "async-tokio", feature = "multi-thread"))]
//! {
//! use std::sync::Arc;
//! use rs_events::{Listener, SharedPayload};
//!
//! // Unlimited calls, tagged for easy identification.
//! let listener = Listener::shared(
//!     Some("my_handler".to_string()), // tag
//!     Arc::new(|payload: &SharedPayload<String>| {
//!         println!("Event: {}", payload);
//!     }), // callback
//!     None, // lifetime
//! );
//! }
//! ```
//!
//! # Callback Execution Modes
//!
//! - **Sync** ([`call()`](Listener::call)): Block until the callback completes.
//!
//! If the `async-tokio` feature is enabled, a [`SharedMode`] listener can also
//! be called asynchronously:
//!
//! - **Async background** ([`background_call()`](Listener::background_call)): Spawn in a Tokio task.
//! - **Async blocking** ([`blocking_call()`](Listener::blocking_call)): Spawn in a blocking thread pool.

pub mod listener_mode;
pub use listener_mode::*;

#[cfg(feature = "async-tokio")]
mod async_ext;

use crate::{Debug, FmtResult, Formatter, LocalCallback, String};

/// A generic handle to an event listener callback.
///
/// `Listener<T, M: ListenerMode>` stores callback and lifetime state using the
/// mode selected by `M`.
///
/// Use [`Listener<T, LocalMode>`] or [`Listener<T>`] for local listeners or [`Listener<T, SharedMode>`]
/// for shared listeners.
///
/// A listener carries two pieces of metadata:
/// - a **tag** for identification or grouping
/// - a mode-specific **lifetime** for call limits
///
/// Note:
/// - Cloning a listener preserves the tag, callback, and lifetime state.
/// - Cloned handles refer to the same logical listener [Same callback and lifetime reference containers].
/// - For an identical listener with an independent lifetime, create a new listener with [`Listener::clone_detached`](Listener::clone_detached).
///
/// # Construction
///
/// Local listener:
///
/// - Use [`Listener::local()`](Listener::local)
/// - use the default local alias [`Listener::new()`](Listener::new).
///
/// Shared listener (requires `async-tokio` or 'multi-thread' features):
///
/// - Use [`Listener::shared()`](Listener::shared)
/// - Use generics with the new constructor, [`Listener::<T, SharedMode>::new()`](Listener::new)
///
/// # Mode Notes
///
/// [`LocalMode`] uses `Rc` and `Cell` and is intended for single-threaded use.
/// [`SharedMode`] uses `Arc` and atomics and is intended for async or
/// multi-threaded use.
pub struct Listener<T: 'static, M: ListenerMode = LocalMode> {
    tag: Option<String>,
    callback: M::Callback<T>,
    lifetime: Option<M::Lifetime>,
}

impl<T, M: ListenerMode> Listener<T, M> {
    /// Create a new local listener with an optional tag and lifetime.
    ///
    /// # Arguments
    ///
    /// - `tag`: Optional string identifier (useful for tracking or removing specific listeners).
    /// - `callback`: The function to invoke when the listener is called.
    /// - `lifetime`: Call limit. Use `None` or `Some(0)` for unlimited; `Some(n)` to stop after `n` calls.
    ///
    /// # Returns
    ///
    /// - A new `Listener` instance with the specified metadata and callback.
    ///
    /// # Examples
    ///
    /// Unlimited listener:
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    ///
    /// let listener = Listener::<String>::new(
    ///     Some("my_tag".to_string()),
    ///     Rc::new(|_: &LocalPayload<String>| {}),
    ///     None, // Some(0) is also unlimited
    /// );
    /// assert_eq!(listener.lifetime(), None);
    /// ```
    ///
    /// Limited to 3 calls:
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    ///
    /// let listener = Listener::<String>::new(
    ///     None,
    ///     Rc::new(|_: &LocalPayload<String>| {}),
    ///     Some(3),
    /// );
    /// assert_eq!(listener.lifetime(), Some(3));
    /// ```
    ///
    /// For a better ergonomics when creating local listeners, use the [`Listener::local()`](Listener::local) constructor:
    ///
    /// For shared listeners, use [`Listener::<T, SharedMode>::new()`] or the [`Listener::shared()`](Listener::shared)method. *[`async-tokio`] feature must be enabled for shared listener support.*
    ///
    /// ```
    /// #[cfg(any(feature = "async-tokio", feature = "multi-thread"))]
    /// {
    /// use std::sync::Arc;
    /// use rs_events::{Listener, SharedPayload, listener::SharedMode};
    /// // let listener = Listener::shared(
    /// let listener: Listener<String> = Listener::<String, SharedMode>::new(
    ///     Some("my_tag".to_string()),
    ///     Arc::new(|_: &SharedPayload<String>| {}),
    ///     Some(5),
    /// );
    /// assert_eq!(listener.lifetime(), Some(5));
    /// }
    /// ```
    pub fn new(tag: Option<String>, callback: M::Callback<T>, lifetime: Option<u64>) -> Self {
        Self {
            tag,
            callback,
            lifetime: M::new_lifetime(lifetime),
        }
    }

    /// Create a new listener from an existing one, optionally overriding the lifetime.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: Call limit.
    ///     - Use `None` to create a new independent lifetime initialized from
    ///       the current remaining count.
    ///     - Use `Some(0)` for a new unlimited lifetime override
    ///     - Use `Some(n)` for a new `n` limited override.
    ///
    /// # Returns
    ///
    /// - A new `Listener` instance with the same tag and callback as `other` and an independent lifetime determined by `lifetime`.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    ///
    /// let mut original = Listener::<String>::new(
    ///     Some("original".to_string()),
    ///     Rc::new(|_: &LocalPayload<String>| {}),
    ///     Some(3),
    /// );
    ///
    /// let mut clone = original.clone();
    /// assert_eq!(clone.lifetime(), Some(3));
    ///
    /// original.call(&Rc::new("test".to_string()));
    /// assert_eq!(clone.lifetime(), Some(2));
    /// assert_eq!(original.lifetime(), Some(2));
    ///
    /// let detached = Listener::clone_detached(&original, Some(5));
    /// assert_eq!(detached.lifetime(), Some(5));
    /// ```
    pub fn clone_detached(other: &Self, lifetime: Option<u64>) -> Self {
        Self {
            tag: other.tag.clone(),
            callback: other.callback.clone(),
            lifetime: match lifetime {
                None => M::new_lifetime(M::remaining(&other.lifetime)),
                Some(0) => None,
                Some(n) => M::new_lifetime(Some(n)),
            },
        }
    }

    /// Get the tag associated with this listener, if set.
    ///
    /// # Returns
    ///
    /// - `None` if the listener has no tag.
    /// - `Some(&String)` if the listener has a tag.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    /// let listener = Listener::<String>::new(
    ///     Some("my_tag".to_string()),
    ///     Rc::new(|_: &LocalPayload<String>| {}),
    ///     None,
    /// );
    /// assert_eq!(listener.tag(), Some(&"my_tag".to_string()));
    /// ```
    pub fn tag(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    /// Get the callback function for this listener.
    ///
    /// # Returns
    ///
    /// - a reference to the callback handle, which may be an `Rc` or `Arc` depending on the mode.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalCallback, LocalPayload};
    ///
    /// let callback: LocalCallback<String> = Rc::new(|_: &LocalPayload<String>| {});
    /// let mut listener = Listener::<String>::new(
    ///     None,
    ///     callback.clone(),
    ///     None,
    /// );
    /// let _callback = listener.callback();
    /// assert!(Rc::ptr_eq(&callback, _callback));
    /// ```
    pub fn callback(&mut self) -> &M::Callback<T> {
        &self.callback
    }

    /// Get the number of remaining calls for this listener.
    ///
    /// Returns
    ///
    /// - `None` for unlimited listeners
    /// - `Some(n)` for listeners with `n` calls left.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    ///
    /// let mut listener = Listener::<String>::new(
    ///     None,
    ///     Rc::new(|_: &LocalPayload<String>| {}),
    ///     Some(2),
    /// );
    /// assert_eq!(listener.lifetime(), Some(2));
    ///
    /// listener.call(&LocalPayload::new("test".to_string()));
    /// assert_eq!(listener.lifetime(), Some(1));
    ///
    /// listener.call(&LocalPayload::new("test".to_string()));
    /// assert_eq!(listener.lifetime(), Some(0));
    /// ```
    pub fn lifetime(&self) -> Option<u64> {
        M::remaining(&self.lifetime)
    }

    /// Check if this listener has reached its call limit.
    ///
    /// # Returns
    ///
    /// - `true` if the listener is exhausted and will not accept further calls.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    /// let mut listener = Listener::<String>::new(
    ///     None,
    ///     Rc::new(|_: &LocalPayload<String>| {}),
    ///     Some(1),
    /// );
    /// assert!(!listener.at_limit());
    ///
    /// listener.call(&Rc::new("test".to_string()));
    /// assert!(listener.at_limit());
    /// ```
    #[inline]
    pub fn at_limit(&self) -> bool {
        M::at_limit(&self.lifetime)
    }

    /// Validate that the listener can be called (not at limit) and decrement lifetime if applicable.
    fn validate_call(&mut self) -> bool {
        M::try_decrement(&mut self.lifetime)
    }

    /// Call the callback synchronously (blocking).
    ///
    /// The callback is invoked immediately. If the listener has a limited lifetime,
    /// it is decremented. If the listener has reached its limit, there is a no operation.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use rs_events::{Listener, LocalPayload};
    ///
    /// let mut listener = Listener::<String>::new(
    ///     None,
    ///     Rc::new(|payload: &LocalPayload<String>| {
    ///         println!("{}", payload);
    ///     }),
    ///     Some(1),
    /// );
    /// listener.call(&Rc::new("test".to_string()));
    /// ```
    #[inline]
    pub fn call(&mut self, payload: &M::Payload<T>) -> bool {
        if !self.validate_call() {
            return false;
        }
        M::invoke_callback(&self.callback, payload);
        true
    }
}

impl<T> Listener<T, LocalMode> {
    /// Create a local-mode listener with a callback and optional metadata.
    pub fn local(tag: Option<String>, callback: LocalCallback<T>, lifetime: Option<u64>) -> Self {
        Self::new(tag, callback, lifetime)
    }
}

#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
impl<T> Listener<T, SharedMode> {
    /// Create a shared-mode listener with a callback and optional metadata.
    pub fn shared(
        tag: Option<String>,
        callback: crate::SharedCallback<T>,
        lifetime: Option<u64>,
    ) -> Self {
        Self::new(tag, callback, lifetime)
    }
}

impl<T, M: ListenerMode> Clone for Listener<T, M> {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            callback: self.callback.clone(),
            lifetime: self.lifetime.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.tag = source.tag.clone();
        self.callback = source.callback.clone();
        self.lifetime = source.lifetime.clone();
    }
}

impl<T, M: ListenerMode> Debug for Listener<T, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Listener")
            .field("tag", &self.tag)
            .field("lifetime", &M::remaining(&self.lifetime))
            .finish()
    }
}

impl<T, M: ListenerMode> PartialEq for Listener<T, M> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
            && M::remaining(&self.lifetime) == M::remaining(&other.lifetime)
            && M::callback_ptr_eq(&self.callback, &other.callback)
    }
}
impl<T, M: ListenerMode> Eq for Listener<T, M> {}
