use tokio::task::JoinHandle;

use crate::{Arc};
use crate::{Listener, SharedPayload, listener::SharedMode};

impl<T: Send + Sync + 'static> Listener<T, SharedMode> {
    /// Call the callback asynchronously in a Tokio task, for non-blocking async work.
    ///
    /// # Arguments
    ///
    /// - `payload`: The payload to pass to the callback. Must be `Send + Sync + 'static` to be used across async boundaries.
    ///
    /// # Returns
    ///
    /// - `None` if the listener has reached its call limit and cannot be called.
    /// - `Some(JoinHandle)` if the call was successfully initiated, allowing the caller to await its completion.
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use rs_events::{Listener, SharedPayload, listener::SharedMode};
    ///
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() {
    ///   let mut listener = Listener::<String, SharedMode>::new(
    ///       None,
    ///       Arc::new(|_: &SharedPayload<String>| {}),
    ///       Some(1),
    ///   );
    ///   let handle = listener.background_call(&Arc::new("test".to_string()));
    ///   assert!(handle.is_some());
    ///
    ///   if let Some(task) = handle {
    ///       task.await.unwrap();
    ///   }
    ///   assert!(listener.background_call(&Arc::new("test".to_string())).is_none());
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn background_call(&mut self, payload: &SharedPayload<T>) -> Option<JoinHandle<()>> {
        if !self.validate_call() {
            return None;
        }
        let callback = Arc::clone(&self.callback);
        let payload = Arc::clone(payload);
        Some(tokio::spawn(async move {
            callback(&payload);
        }))
    }

    /// Call the callback in a Tokio blocking thread pool.
    ///
    /// Use this for CPU-intensive or blocking work that shouldn't starve async tasks.
    /// Returns a handle to await completion, or `None` if at limit.
    ///
    /// # Returns
    /// `Some(JoinHandle)` to await the task, or `None` if at limit.
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use rs_events::{Listener, SharedPayload, listener::SharedMode};
    ///
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() {
    ///     let mut listener = Listener::<String, SharedMode>::new(
    ///         None,
    ///         Arc::new(|_: &SharedPayload<String>| {}),
    ///         Some(1),
    ///     );
    ///
    ///     let handle = listener.blocking_call(&Arc::new("test".to_string()));
    ///     assert!(handle.is_some());
    ///
    ///     if let Some(task) = handle {
    ///         task.await.unwrap();
    ///     }
    ///     assert!(listener.blocking_call(&Arc::new("test".to_string())).is_none());
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn blocking_call(&mut self, payload: &SharedPayload<T>) -> Option<JoinHandle<()>> {
        if !self.validate_call() {
            return None;
        }
        let callback = Arc::clone(&self.callback);
        let payload = Arc::clone(payload);
        Some(tokio::task::spawn_blocking(move || callback(&payload)))
    }
}
