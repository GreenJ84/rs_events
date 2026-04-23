use crate::Arc;

/// Type alias for a shared event payload pointer for asynchronous and/or multi-threaded environments.
///
/// Uses `Arc<T>` for both default and no standard library builds.
///
/// # Example (default: std)
/// ```rust
/// use std::sync::Arc;
/// // Or `use rs_events::Arc` if using the crate's re-export
/// use rs_events::{SharedCallback, SharedPayload};
///
/// let payload: SharedPayload<String> = Arc::new(String::from("Emitting value"));
/// ```
/// # Example (no_std)
/// ```
/// extern crate alloc;
/// use alloc::sync::Arc;
/// // Or `use rs_events::Arc` if using the crate's re-export
/// use rs_events::{SharedCallback, SharedPayload};
///
/// let payload: SharedPayload<String> = Arc::new(String::from("Emitting value"));
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
/// // Or `use rs_events::Arc` if using the crate's re-export
/// use rs_events::{SharedCallback, SharedPayload};
///
/// let callback: SharedCallback<String> = Arc::new(move |payload: &SharedPayload<String>| {
///     println!("Received event: {}", payload);
/// });
/// ```
///
/// # Example (no_std)
/// ```
/// extern crate alloc;
/// use alloc::sync::Arc;
/// // Or `use rs_events::Arc` if using the crate's re-export
/// use rs_events::{SharedCallback, SharedPayload};
///
/// let callback: SharedCallback<String> = Arc::new(move |payload: &SharedPayload<String>| {
///     // handle payload
/// });
/// ```
pub type SharedCallback<T> = Arc<dyn Fn(&SharedPayload<T>) + Send + Sync>;
