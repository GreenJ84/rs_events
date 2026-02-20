use crate::{Arc, AtomicU64, Ordering};
use crate::{Debug, Formatter, FmtResult};
use crate::{Callback, EventPayload};

/// A handle for an event listener callback.
///
/// `Listener<T>` wraps a callback, an optional tag, and an optional lifetime counter, allowing for one-shot, limited, or unlimited event listeners.
///
/// # Tagging
///
/// The `tag` field (`Option<String>`) allows users to associate arbitrary metadata or an identifier with the listener. This is useful for tracking, grouping, or refreshing listeners in user code.
///
/// # Examples
///
/// Basic usage with tag:
/// ```
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
/// ```
pub struct Listener<T> {
    tag: Option<String>,
    callback: Callback<T>,
    lifetime: Option<Arc<AtomicU64>>,
}

impl<T> Listener<T> {
    /// Create a new listener with an optional tag and lifetime.
    ///
    /// # Parameters
    /// * `tag` - Optional string tag for user metadata or identification.
    /// * `callback` - The callback function to invoke when the event is emitted.
    /// * `lifetime` - Optional call limit. If `None` or `Some(0)`, the listener is unlimited. If `Some(n)`, the listener will be called at most `n` times.
    ///
    /// # Examples
    ///
    /// Unlimited:
    /// ```
    /// extern crate alloc;
    /// use alloc::{
    ///     sync::Arc,
    ///     string::String,
    /// };
    /// use rs_events::{Listener, EventPayload};
    ///
    /// let listener = Listener::new(Some("my_tag".to_string()), Arc::new(|_: &EventPayload<String>| {}), None);
    /// ```
    ///
    /// Limited:
    /// ```
    /// extern crate alloc;
    /// use alloc::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let listener = Listener::new(None, Arc::new(|_: &EventPayload<String>| {}), Some(3));
    /// ```
    pub fn new(tag: Option<String>, callback: Callback<T>, lifetime: Option<u64>) -> Self {
        match lifetime {
            Some(0) | None => Self {
                tag,
                callback,
                lifetime: None,
            },
            Some(limit) => Self {
                tag,
                callback,
                lifetime: Some(Arc::new(AtomicU64::new(limit))),
            },
        }
    }

    /// Returns a reference to the tag associated with this listener, if any.
    ///
    /// # Returns
    /// * `Some(&String)` if a tag was set for this listener.
    /// * `None` if no tag was set.
    ///
    /// # Example
    /// ```
    /// extern crate alloc;
    /// use alloc::{
    ///     sync::Arc,
    ///     string::String,
    /// };
    /// use rs_events::{Listener, EventPayload};
    ///
    /// let listener = Listener::new(Some("tag".to_string()), Arc::new(|_: &EventPayload<String>| {}), None);
    /// assert_eq!(listener.tag(), Some(&"tag".to_string()));
    /// ```
    pub fn tag(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    /// Returns a reference to the callback function for this listener.
    ///
    /// # Returns
    /// * `&Callback<T>` - The callback function that will be invoked when the event is emitted.
    ///
    /// # Example
    /// ```
    /// extern crate alloc;
    /// use alloc::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    ///
    /// let listener = Listener::new(None, Arc::new(|_: &EventPayload<String>| {}), None);
    /// let cb = listener.callback();
    /// // cb can be called with a payload
    /// ```
    pub fn callback(&self) -> &Callback<T> {
        &self.callback
    }

    /// Returns the lifetime remaining for this listener, if it has a limited lifetime.
    ///
    /// # Returns
    /// * `Some(n)` if the listener was created with a limited lifetime, where `n` is the number of calls left before it is at limit.
    /// * `None` if the listener is unlimited.
    ///
    /// # Example
    /// ```
    /// extern crate alloc;
    /// use alloc::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    ///
    /// let mut listener = Listener::new(None, Arc::new(|_: &EventPayload<String>| {}), Some(2));
    /// assert_eq!(listener.lifetime(), Some(2));
    ///
    /// listener.call(&Arc::new("payload".to_string()));
    /// assert_eq!(listener.lifetime(), Some(1));
    /// ```
    pub fn lifetime(&self) -> Option<u64> {
        self.lifetime.as_ref().map(|l| l.load(Ordering::SeqCst))
    }

    /// Returns whether the listener has reached its call limit.
    ///
    /// # Returns
    /// * `true` if the listener was created with a limited lifetime and has been called the maximum number of times.
    /// * `false` if the listener is unlimited or has remaining calls.
    ///
    /// # Example
    /// ```
    /// extern crate alloc;
    /// use alloc::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    ///
    /// let mut listener = Listener::new(None, Arc::new(|_: &EventPayload<String>| {}), Some(1));
    /// assert!(!listener.at_limit());
    /// listener.call(&Arc::new("payload".to_string()));
    /// assert!(listener.at_limit());
    /// ```
    #[inline]
    pub fn at_limit(&self) -> bool {
        match self.lifetime {
            None => false,
            Some(ref lifetime) => lifetime.load(Ordering::SeqCst) == 0,
        }
    }

    /// Synchronously invoke the callback with the given payload.
    ///
    /// If the listener has a limited lifetime, it is decremented. If at limit, the callback is not invoked.
    ///
    /// # Panics
    /// Panics if the atomic counter is poisoned (should not happen).
    ///
    /// # Example
    /// ```
    /// extern crate alloc;
    /// use alloc::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// let mut listener = Listener::new(None, Arc::new(|_: &EventPayload<String>| {}), Some(1));
    /// listener.call(&Arc::new("payload".to_string()));
    /// ```
    #[inline]
    pub fn call(&mut self, payload: &EventPayload<T>) {
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
                return;
            }
        }
        (self.callback)(payload);
    }
}

impl<T> Clone for Listener<T> {
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
impl<T> Default for Listener<T> {
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
