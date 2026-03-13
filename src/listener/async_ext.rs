use tokio::task::JoinHandle;

use crate::{Arc};
use crate::{Listener, EventPayload};

impl<T: Send + Sync + 'static> Listener<T> {
    /// Call the callback asynchronously in a Tokio task.
    ///
    /// Spawns the callback as an async task and returns a handle to await completion.
    /// If the listener is at its call limit, returns `None` without spawning.
    ///
    /// # Returns
    /// `Some(JoinHandle)` to await the task, or `None` if at limit.
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use rs_events::{Listener, EventPayload};
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() {
    ///   let mut listener = Listener::new(
    ///       None,
    ///       Arc::new(|_: &EventPayload<String>| {}),
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
    pub fn background_call(&mut self, payload: &EventPayload<T>) -> Option<JoinHandle<()>> {
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
    /// use rs_events::{Listener, EventPayload};
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() {
    ///     let mut listener = Listener::new(
    ///         None,
    ///         Arc::new(|_: &EventPayload<String>| {}),
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
    pub fn blocking_call(&mut self, payload: &EventPayload<T>) -> Option<JoinHandle<()>> {
        if !self.validate_call() {
            return None;
        }
        let callback = Arc::clone(&self.callback);
        let payload = Arc::clone(payload);
        Some(tokio::task::spawn_blocking(move || callback(&payload)))
    }
}
