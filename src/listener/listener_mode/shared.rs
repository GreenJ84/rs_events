//! Shared listener mode is used when listeners must be safe to share across tasks and/or
//! threads.
//!
//! It is backed by `Arc` and atomics.

use super::ListenerMode;
use crate::{Arc, AtomicU64, Ordering, SharedCallback, SharedPayload};

/// Shared listener mode for async and multi-threaded runtimes.
///
/// - Callback type utilizes [`SharedCallback`](crate::SharedCallback).
/// - Lifetime type utilizes `Arc<AtomicU64>`.
pub struct SharedMode;
impl ListenerMode for SharedMode {
    type Payload<T> = SharedPayload<T>;

    type Callback<T> = SharedCallback<T>;

    /// Compares two shared callback handles by pointer identity.
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
        Arc::ptr_eq(left, right)
    }

    /// Invokes a shared callback.
    ///
    /// # Arguments
    ///
    /// - `callback`: The shared callback handle to invoke.
    /// - `payload`: The payload to pass to the callback.
    fn invoke_callback<T>(callback: &Self::Callback<T>, payload: &Self::Payload<T>) {
        callback(payload);
    }

    type Lifetime = Arc<AtomicU64>;

    /// Creates a shared atomic lifetime counter.
    ///
    /// # Arguments
    ///
    /// - `limit`: Optional call limit for the listener. *`None` and `Some(0)` produce no counter (unlimited listener)*
    ///
    /// # Returns
    ///
    /// - `None` if no lifetime counter was created (unlimited listener).
    /// - `Some(Arc<AtomicU64>)` if a lifetime counter was created with the specified limit.
    ///
    fn new_lifetime(limit: Option<u64>) -> Option<Self::Lifetime> {
        match limit {
            Some(0) | None => None,
            Some(n) => Some(Arc::new(AtomicU64::new(n))),
        }
    }

    /// Reads the current remaining call count atomically.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    ///
    /// - `None` if the listener is unlimited (no lifetime counter).
    /// - `Some(u64)` if the listener has a lifetime limit, representing the remaining call count.
    fn remaining(lifetime: &Option<Self::Lifetime>) -> Option<u64> {
        lifetime.as_ref().map(|a| a.load(Ordering::SeqCst))
    }

    /// Checks whether the listener has reached its call limit.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    ///
    /// - `true` if the listener has a lifetime limit and has reached it (0 calls remaining).
    /// - `false` if the listener is unlimited or has remaining calls.
    fn at_limit(lifetime: &Option<Self::Lifetime>) -> bool {
        lifetime
            .as_ref()
            .is_some_and(|a| a.load(Ordering::SeqCst) == 0)
    }

    /// Atomically decrements the call counter when possible.
    ///
    /// # Arguments
    ///
    /// - `lifetime`: The optional lifetime counter to decrement if possible.
    ///
    /// # Returns
    ///
    /// - `true` if the listener is valid for a call and lifetime was decremented (if applicable).
    /// - `false` if the counter is already `0`.
    fn try_decrement(lifetime: &mut Option<Self::Lifetime>) -> bool {
        match lifetime {
            None => true,
            Some(a) => a
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                    (x > 0).then(|| x - 1)
                })
                .is_ok(),
        }
    }
}
