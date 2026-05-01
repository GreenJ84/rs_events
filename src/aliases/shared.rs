///! For asynchronous or multi-threaded event we use aliases that require Send + Sync for asynchronous and/or thread safety
///!
///! They are only included if the multi_threaded feature or an async feature is enabled, so they do not impose Send + Sync bounds on payloads and callbacks for single-threaded synchronous listeners that do not need them.

use crate::Arc;
use super::EventMode;

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
pub type SharedPayload<T: Send + Sync> = Arc<T>;

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
pub type SharedCallback<T: Send + Sync> = Arc<dyn Fn(&SharedPayload<T>) + Send + Sync>;


pub struct SharedMode;
impl EventMode for SharedMode {
    /// Payload type for shared mode is a thread-safe reference-counted pointer to the event data.
    type Payload<T: Send + Sync> = SharedPayload<T>;
    /// Callback type for shared mode is a thread-safe reference-counted pointer to a function that takes a reference to the payload.
    type Callback<T: Send + Sync> = SharedCallback<T>;

    /// Invokes a shared callback.
    ///
    /// # Arguments
    /// - `callback`: The local callback handle to invoke.
    /// - `payload`: The payload to pass to the callback.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>) {
        callback(payload);
    }
}