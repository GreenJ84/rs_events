///! For asynchronous or multi-threaded event we use aliases that require Send + Sync for asynchronous and/or thread safety
///!
///! They are only included if the multi_threaded feature or an async feature is enabled, so they do not impose Send + Sync bounds on payloads and callbacks for single-threaded synchronous listeners that do not need them.
use crate::Arc;

/// Type alias for a shared event payload pointer for asynchronous and/or multi-threaded environments.
///
/// Uses `Arc<T>` for both default and no standard library builds.
///
/// # Example (default: std)
/// ```rust
/// use rs_events::{SharedPayload};
///
/// let payload: SharedPayload<String> = SharedPayload::new(String::from("Emitting value"));
/// ```
pub type SharedPayload<T> = Arc<T>;

/// Type alias for a callback pointer.
///
/// - Uses `Arc<dyn Fn(&SharedPayload<T>) + Send + Sync>` for both default and no standard library builds.
/// - Requires `Send + Sync` for thread safety.
///
/// # Example (default: std)
/// ```
/// use std::sync::Arc;
/// use rs_events::{SharedPayload, SharedCallback};
///
/// let callback: SharedCallback<String> = Arc::new(move |payload: &SharedPayload<String>| {
///     println!("Received event: {}", payload);
/// });
/// ```
pub type SharedCallback<T> = Arc<dyn Fn(&SharedPayload<T>) + Send + Sync>;
