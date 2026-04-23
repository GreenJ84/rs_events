use crate::Rc;

/// Type alias for a single threaded synchronous event payload pointer.
///
/// Uses `Rc<T>` for both default and no standard library builds.
///
/// # Example (default: std)
/// ```
/// use std::rc::Rc;
/// // Or `use rs_events::Rc` if using the crate's re-export
/// use rs_events::{LocalCallback, LocalPayload};
///
/// let payload: LocalPayload<String> = Rc::new(String::from("Emitting value"));
/// ```
///
/// # Example (no_std)
/// ```
/// extern crate alloc;
/// use alloc::rc::Rc;
/// // Or `use rs_events::Rc` if using the crate's re-export
/// use rs_events::{LocalCallback, LocalPayload};
///
/// let payload: LocalPayload<String> = Rc::new(String::from("Emitting value"));
/// ```
///
/// For mutable event data, use `Rc<RefCell<T>>` or `Rc<Cell<T>>` instead:
/// - `Rc<RefCell<T>>` for non-`Copy` types and runtime borrow checking.
/// - `Rc<Cell<T>>` for `Copy` types and simple value replacement.
pub type LocalPayload<T> = Rc<T>;

/// Type alias for a single threaded synchronous callback pointer.
///
/// - Uses `Rc<dyn Fn(&LocalPayload<T>) + 'static>` for both default and no standard library builds.
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
///
/// # Example (no_std)
/// ```
/// extern crate alloc;
/// use alloc::rc::Rc;
/// use rs_events::{LocalCallback, LocalPayload};
///
/// let callback: LocalCallback<String> = Rc::new(move |payload: &LocalPayload<String>| {
///     // handle payload
/// });
/// ```
pub type LocalCallback<T> = Rc<dyn Fn(&LocalPayload<T>) + 'static>;
