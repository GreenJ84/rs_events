//! Local listener mode is intended for single-threaded listener usage where atomic
//! synchronization is unnecessary.
//!
//! It is backed by Local aliases, `Rc` and `Cell`.

use super::ListenerMode;
use crate::{Cell, LocalCallback, LocalPayload, Rc};

/// Single-threaded listener mode.
///
/// - Callback type utilizes [`LocalCallback`](crate::LocalCallback).
/// - Lifetime type utilizes `Rc<Cell<u64>>`.
pub struct LocalMode;
impl ListenerMode for LocalMode {
    type Payload<T> = LocalPayload<T>;


    type Callback<T> = LocalCallback<T>;

    /// Compares two local callback handles by Rc pointer identity.
    ///
    /// # Arguments
    ///
    /// - `left`: The first callback handle to compare.
    /// - `right`: The second callback handle to compare.
    ///
    /// # Returns
    ///
    /// - `true` if the two callback handles point to the same allocation (are clones).
    /// - `false` if the two callback handles point to different allocations.
    fn callback_ptr_eq<T>(left: &Self::Callback<T>, right: &Self::Callback<T>) -> bool {
        LocalCallback::ptr_eq(left, right)
    }

    /// Invokes a local callback.
    ///
    /// # Arguments
    ///
    /// - `callback`: The local callback handle to invoke.
    /// - `payload`: The payload to pass to the callback.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>) {
        callback(payload);
    }


    type Lifetime = Rc<Cell<u64>>;

    /// Creates a local lifetime counter.
    ///
    /// # Arguments
    ///
    /// - `limit`: Optional call limit for the listener. *`None` and `Some(0)` produce no counter (unlimited listener)*
    ///
    /// # Returns
    ///
    /// - `None` if no lifetime counter was created (unlimited listener).
    /// - `Some(Rc<Cell<u64>>)` if a lifetime counter was created with
    fn new_lifetime(limit: Option<u64>) -> Option<Self::Lifetime> {
        match limit {
            Some(0) | None => None,
            Some(n) => Some(Rc::new(Cell::new(n))),
        }
    }

    /// Reads the current remaining call count.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: The optional lifetime counter to read.
    ///
    /// # Returns
    ///
    /// - `None` for unlimited listeners
    /// - `Some(n)` for listeners with `n` calls left.
    fn remaining(lifetime: &Option<Self::Lifetime>) -> Option<u64> {
        lifetime.as_ref().map(|c| c.get())
    }

    /// Checks whether the listener has reached its call limit.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: The optional lifetime counter to check.
    ///
    /// # Returns
    ///
    /// - `true` if the listener has a lifetime limit and has reached it (0 calls remaining).
    /// - `false` if the listener is unlimited or has remaining calls.
    fn at_limit(lifetime: &Option<Self::Lifetime>) -> bool {
        lifetime.as_ref().is_some_and(|c| c.get() == 0)
    }

    /// Decrements the call counter when possible.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
     ///
     /// - `true` if the listener is valid for a call (not at limit or unlimited) and lifetime was decremented if applicable.
     /// - `false` if the listener is at limit and cannot be called.
    fn try_decrement(lifetime: &mut Option<Self::Lifetime>) -> bool {
        match lifetime {
            None => true,
            Some(c) if c.get() > 0 => {
                c.set(c.get() - 1);
                true
            }
            Some(_) => false,
        }
    }
}
