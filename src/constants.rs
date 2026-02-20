use crate::Arc;
/// Type alias for an event payload pointer.
///
/// Uses `Arc<T>` for both default and no standard library builds.
///
/// # Example (default: std)
/// ```
/// use std::sync::Arc;
/// use rs_events::{Callback, EventPayload};
///
/// let payload: EventPayload<String> = Arc::new(String::from("Emitting value"));
/// ```
/// # Example (no_std)
/// ```
/// extern crate alloc;
/// use alloc::sync::Arc;
/// use rs_events::{Callback, EventPayload};
///
/// let payload: EventPayload<String> = Arc::new(String::from("Emitting value"));
/// ```
pub type EventPayload<T> = Arc<T>;

/// Type alias for a callback pointer.
///
/// - Uses `Arc<dyn Fn(&EventPayload<T>) + Send + Sync>` for both default and no standard library builds.
/// - Requires `Send + Sync` for thread safety.
///
/// # Example (default: std)
/// ```
/// use std::sync::Arc;
/// use rs_events::{Callback, EventPayload};
///
/// let callback: Callback<String> = Arc::new(move |payload: &EventPayload<String>| {
///     println!("Received event: {}", payload);
/// });
/// ```
///
/// # Example (no_std)
/// ```
/// extern crate alloc;
/// use alloc::sync::Arc;
/// use rs_events::{Callback, EventPayload};
///
/// let callback: Callback<String> = Arc::new(move |payload: &EventPayload<String>| {
///     println!("Received event: {}", payload);
/// });
/// ```
pub type Callback<T> = Arc<dyn Fn(&EventPayload<T>) + Send + Sync>;
