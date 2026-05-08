//! Shared thread-safe event mode primitives.
//!
//! This module provides `SharedPayload` and `SharedCallback`, which require `Send + Sync`
//! to support async and multi-threaded emission.

use super::EventMode;
use crate::Arc;

/// Type alias for a shared event payload pointer for asynchronous and/or multi-threaded environments.
///
/// Uses `Arc<T>`
///
/// # Example
/// ```rust
/// use rs_events::{SharedPayload};
///
/// let payload: SharedPayload<String> = SharedPayload::new(String::from("Emitting value"));
/// ```
pub type SharedPayload<T> = Arc<T>;

/// Type alias for a shared callback pointer.
///
/// Uses `Arc<dyn Fn(&SharedPayload<T>) + Send + Sync>` and requires `Send + Sync` for thread safety.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use rs_events::{SharedPayload, SharedCallback};
///
/// let callback: SharedCallback<String> = Arc::new(move |payload: &SharedPayload<String>| {
///     println!("Received event: {}", payload);
/// });
/// ```
pub type SharedCallback<T> = Arc<dyn Fn(&SharedPayload<T>) + Send + Sync>;

/// Shared event mode for async and multi-threaded runtimes.
pub struct SharedMode;

impl EventMode for SharedMode {
    /// Payload type for shared mode: [`SharedPayload`](SharedPayload).
    type Payload<T> = SharedPayload<T>;

    /// Callback type for shared mode: [`SharedCallback`](SharedCallback).
    type Callback<T> = SharedCallback<T>;

    /// Invokes a shared callback.
    ///
    /// # Example
    /// ```rust
    /// use std::sync::Arc;
    /// use rs_events::{SharedCallback, SharedPayload, SharedMode, EventMode};
    ///
    /// let payload: SharedPayload<String> = Arc::new(String::from("invoke-shared"));
    /// let callback: SharedCallback<String> = Arc::new(|payload: &SharedPayload<String>| {
    ///     println!("payload={}", &**payload);
    /// });
    /// SharedMode::invoke_callback(&callback, &payload);
    /// ```
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>) {
        callback(payload);
    }
}
