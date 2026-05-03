///! For local synchronous events we use single-threaded aliases which do not have Send or Sync requirements for simplicity.
///!
///! Even with async-tokio enabled, single-threaded aliases are available to avoid unnecessary Send + Sync bounds on payloads and callbacks that do not need them with local async task spawning.

use crate::Rc;
use super::EventMode;

/// Type alias for a single threaded synchronous event payload pointer.
///
/// Uses `Rc<T>`
///
/// # Example (default: std)
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

/// Type alias for a single threaded synchronous callback pointer.
///
/// - Uses `Rc<dyn Fn(&LocalPayload<T>) + 'static>`
///
/// # Example (default: std)
/// ```
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
    /// Payload type for local mode: A reference-counted pointer to the event data.
    /// - `Rc<T>`
    type Payload<T> = LocalPayload<T>;
    /// Callback type for local mode: A reference-counted pointer to a function that takes a reference to the payload.
    /// - `Rc<dyn Fn(&LocalPayload<T>) + 'static>`
    type Callback<T> = LocalCallback<T>;

    /// Invokes a local callback.
    ///
    /// # Arguments
    /// - `callback`: The local callback to invoke.
    /// - `payload`: The payload to pass to the callback.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>) {
        callback(payload);
    }
}