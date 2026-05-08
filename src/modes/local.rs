//! Local single-threaded event mode primitives.
//!
//! This module provides `LocalPayload` and `LocalCallback`, which intentionally avoid `Send`
//! and `Sync` requirements for sync-only use and local async task spawning.

use super::EventMode;
use crate::Rc;

/// Type alias for a single-threaded synchronous event payload pointer.
///
/// Uses `Rc<T>`
///
/// # Example
/// ```
/// use std::rc::Rc;
/// use rs_events::{LocalCallback, LocalPayload};
///
/// let payload: LocalPayload<String> = LocalPayload::new(String::from("Emitting value"));
/// ```
///
/// For mutable event data, use `Rc<RefCell<T>>` or `Rc<Cell<T>>` instead:
/// - `Rc<RefCell<T>>` for non-`Copy` types and runtime borrow checking.
/// - `Rc<Cell<T>>` for `Copy` types and simple value replacement.
pub type LocalPayload<T> = Rc<T>;

/// Type alias for a single-threaded synchronous callback pointer.
///
/// - Uses `Rc<dyn Fn(&LocalPayload<T>) + 'static>`
///
/// # Example
/// ```The other 
/// use std::rc::Rc;
/// use rs_events::{LocalCallback, LocalPayload};
///
/// let callback: LocalCallback<String> = Rc::new(move |payload: &LocalPayload<String>| {
///     println!("Received event: {}", payload);
/// });
/// ```
pub type LocalCallback<T> = Rc<dyn Fn(&LocalPayload<T>) + 'static>;

/// Single-threaded event mode for synchronous listeners.
pub struct LocalMode;
impl EventMode for LocalMode {
    /// Payload type for local mode: [`LocalPayload`](LocalPayload).
    type Payload<T> = LocalPayload<T>;
    /// Callback type for local mode: [`LocalCallback`](LocalCallback).
    type Callback<T> = LocalCallback<T>;

    /// Invokes a local callback.
    ///
    /// # Example
    /// ```rust
    /// use std::rc::Rc;
    /// use rs_events::{LocalCallback, LocalPayload, LocalMode, EventMode};
    ///
    /// let payload: LocalPayload<String> = Rc::new(String::from("invoke-local"));
    /// let callback: LocalCallback<String> = Rc::new(|payload: &LocalPayload<String>| {
    ///     // print the inner string
    ///     println!("payload={}", &**payload);
    /// });
    /// LocalMode::invoke_callback(&callback, &payload);
    /// ```
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>) {
        callback(payload);
    }
}
