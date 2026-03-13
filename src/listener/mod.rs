//! Event listener implementation for the `rs_events` crate.
//!
//! This provides the [`Listener<T>`] type, which wraps a callback
//! and metadata to enable flexible, composable event handling.
//!
//! Listener metadata includes:
//!
//! - **Tags**: Optional string identifiers for tracking or grouping listeners.
//! - **Lifetimes**: Optional call limits (e.g., "call once," "call 3 times").
//! - **Thread safety**: All operations are atomic and safe to share across threads.
//! - **Async extendable**: Spawn callbacks in Tokio tasks or blocking threads. (via 'async' feature)
//!
//! # Quick Start
//!
//! Create a listener with a callback and optional metadata:
//!
//! ```rust
//! use std::sync::Arc;
//! use rs_events::{Listener, EventPayload};
//!
//! // Unlimited calls, tagged for easy identification
//! let listener = Listener::new(
//!     Some("my_handler".to_string()), // tag
//!     Arc::new(|payload: &EventPayload<String>| {
//!         println!("Event: {}", payload);
//!     }),
//!     None, // lifetime
//! );
//! ```
//!
//! # Callback Execution Modes
//!
//! - **Sync** ([`call()`](Listener::call)): Block until the callback completes.
//!
//! If 'async` feature is enabled:
//!
//! - **Async background** ([`background_call()`](Listener::background_call)): Spawn in a Tokio task.
//! - **Async blocking** ([`blocking_call()`](Listener::blocking_call)): Spawn in a blocking thread pool.

#[cfg(feature = "async-tokio")]
mod async_ext;

use crate::{Arc, AtomicU64, String, Ordering};
use crate::{Callback, EventPayload};
use crate::{Debug, FmtResult, Formatter};

/// A thread-safe handle to an event listener callback.
///
/// `Listener<T>` wraps a callback and associates optional metadata:
/// - A **tag** for identification or grouping.
/// - A **lifetime** to limit how many times the callback is called.
///
/// Use `Listener` to manage event handlers flexibly: add them to an `EventEmitter`,
/// check their status, and remove them by reference.
///
/// # Creation
///
/// Create a listener with [`Listener::new()`](Listener::new).
///
/// ```rust
/// use std::sync::Arc;
/// use rs_events::{Listener, EventPayload};
/// let listener = Listener::new(
///     Some("my_handler".to_string()), // tag (optional)
///     Arc::new(|payload: &EventPayload<String>| {
///         println!("Event fired: {}", payload);
///     }),
///     Some(3), // lifetime: call up to 3 times (None/Some(0) = unlimited)
/// );
/// assert_eq!(listener.tag(), Some(&"my_handler".to_string()));
/// assert_eq!(listener.lifetime(), Some(3));
/// ```
///
/// No_Std Example:
///
/// ```rust
/// extern crate alloc;
/// use alloc::{
///     sync::Arc,
///     string::String,
/// };
/// use rs_events::{Listener, EventPayload};
///
/// let listener = Listener::new(Some("my_tag".to_string()), Arc::new(|payload: &EventPayload<String>| {
///     // handle payload
/// }), None);
/// assert_eq!(listener.tag(), Some(&"my_tag".to_string()));
/// assert_eq!(listener.lifetime(), None);
/// ```
///
/// # Thread Safety
///
/// All operations are thread-safe. Lifetimes are managed with atomic counters
/// so multiple threads can call and check a listener concurrently.
pub struct Listener<T> {
    tag: Option<String>,
    callback: Callback<T>,
    lifetime: Option<Arc<AtomicU64>>,
}

impl<T: Send + Sync + 'static> Listener<T> {
    /// Create a new listener with an optional tag and lifetime.
    ///
    /// # Arguments
    ///
    /// - `tag`: Optional string identifier (useful for tracking or removing specific listeners).
    /// - `callback`: The function to invoke when the listener is called.
    /// - `lifetime`: Call limit. Use `None` or `Some(0)` for unlimited; `Some(n)` to stop after `n` calls.
    ///
    /// # Examples
    ///
    /// Unlimited listener:
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let listener = Listener::new(
    ///     Some("my_tag".to_string()),
    ///     Arc::new(|_: &EventPayload<String>| {}),
    ///     None, // Some(0) is also unlimited
    /// );
    /// assert_eq!(listener.lifetime(), None);
    /// ```
    ///
    /// Limited to 3 calls:
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let listener = Listener::new(
    ///     None,
    ///     Arc::new(|_: &EventPayload<String>| {}),
    ///     Some(3),
    /// );
    /// assert_eq!(listener.lifetime(), Some(3));
    /// ```
    pub fn new(tag: Option<String>, callback: Callback<T>, lifetime: Option<u64>) -> Self {
        Self {
            tag,
            callback,
            lifetime: match lifetime {
                Some(0) | None => None,
                Some(limit) => Some(Arc::new(AtomicU64::new(limit))),
            },
        }
    }

    /// Get the tag associated with this listener, if set.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let listener = Listener::new(
    ///     Some("my_tag".to_string()),
    ///     Arc::new(|_: &EventPayload<String>| {}),
    ///     None,
    /// );
    /// assert_eq!(listener.tag(), Some(&"my_tag".to_string()));
    /// ```
    pub fn tag(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    /// Get the callback function for this listener.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Callback, Listener, EventPayload};
    /// let callback: Callback<String> = Arc::new(|_: &EventPayload<String>| {});
    /// let listener = Listener::new(
    ///     None,
    ///     callback.clone(),
    ///     None,
    /// );
    /// let _callback = listener.callback();
    /// assert!(Arc::ptr_eq(&callback, _callback));
    /// ```
    pub fn callback(&self) -> &Callback<T> {
        &self.callback
    }

    /// Get the number of remaining calls for this listener.
    ///
    /// Returns `None` for unlimited listeners; `Some(n)` for listeners with `n` calls left.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let mut listener = Listener::new(
    ///     None,
    ///     Arc::new(|_: &EventPayload<String>| {}),
    ///     Some(2),
    /// );
    /// assert_eq!(listener.lifetime(), Some(2));
    ///
    /// listener.call(&Arc::new("test".to_string()));
    /// assert_eq!(listener.lifetime(), Some(1));
    ///
    /// listener.call(&Arc::new("test".to_string()));
    /// assert_eq!(listener.lifetime(), Some(0));
    /// ```
    pub fn lifetime(&self) -> Option<u64> {
        self.lifetime.as_ref().map(|l| l.load(Ordering::SeqCst))
    }

    /// Check if this listener has reached its call limit.
    ///
    /// Returns `true` if the listener is exhausted and will not accept further calls.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let mut listener = Listener::new(
    ///     None,
    ///     Arc::new(|_: &EventPayload<String>| {}),
    ///     Some(1),
    /// );
    /// assert!(!listener.at_limit());
    ///
    /// listener.call(&Arc::new("test".to_string()));
    /// assert!(listener.at_limit());
    /// ```
    #[inline]
    pub fn at_limit(&self) -> bool {
        match self.lifetime {
            None => false,
            Some(ref lifetime) => lifetime.load(Ordering::SeqCst) == 0,
        }
    }

    /// Validate that the listener can be called (not at limit) and decrement lifetime if applicable.
    fn validate_call(&mut self) -> bool {
        if let Some(ref lifetime) = self.lifetime {
            if lifetime
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                    if x > 0 {
                        Some(x - 1)
                    } else {
                        None
                    }
                })
                .is_err()
            {
                return false;
            }
        }
        true
    }

    /// Call the callback synchronously (blocking).
    ///
    /// The callback is invoked immediately. If the listener has a limited lifetime,
    /// it is decremented. If the listener has reached its limit, there is a no operation.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let mut listener = Listener::new(
    ///     None,
    ///     Arc::new(|payload: &EventPayload<String>| {
    ///         println!("{}", payload);
    ///     }),
    ///     Some(1),
    /// );
    /// listener.call(&Arc::new("test".to_string()));
    /// ```
    #[inline]
    pub fn call(&mut self, payload: &EventPayload<T>) {
        if !self.validate_call() {
            return;
        }
        (self.callback)(payload);
    }
}

impl<T: Send + Sync + 'static> Clone for Listener<T> {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            callback: Arc::clone(&self.callback),
            lifetime: self.lifetime.as_ref().map(Arc::clone),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.tag = source.tag.clone();
        self.callback = Arc::clone(&source.callback);
        self.lifetime = source.lifetime.as_ref().map(Arc::clone);
    }
}

impl<T: Send + Sync + 'static> Default for Listener<T> {
    /// Returns a default listener with a no-op callback and a single call limit.
    ///
    /// # Example
    /// ```
    /// use rs_events::Listener;
    /// let _ = Listener::<String>::default();
    /// ```
    fn default() -> Self {
        Self::new(None, Arc::new(|_: &EventPayload<T>| {}), Some(1))
    }
}

impl<T> Debug for Listener<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Listener")
            .field("tag", &self.tag)
            .field(
                "lifetime",
                &self.lifetime.as_ref().map(|a| a.load(Ordering::SeqCst)),
            )
            .finish()
    }
}

impl<T> PartialEq for Listener<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback) && self.tag == other.tag
    }
}
impl<T> Eq for Listener<T> {}
