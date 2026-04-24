//! Shared listener mode backed by `Arc` and atomics.
//!
//! This mode is used when listeners must be safe to share across tasks and/or
//! threads.

use super::ListenerMode;
use crate::{Arc, AtomicU64, Ordering, SharedCallback};

/// Shared listener mode for async and multi-threaded runtimes.
///
/// - Callback type utilizes [`SharedCallback`].
/// - Lifetime type utilizes `Arc<AtomicU64>`.
pub struct SharedMode;
impl ListenerMode for SharedMode {
    type Callback<T> = SharedCallback<T>;
    type Lifetime = Arc<AtomicU64>;

    /// Creates a shared atomic lifetime counter.
    ///
    /// `None` and `Some(0)` produce no counter (unlimited listener).
    fn new_lifetime(limit: Option<u64>) -> Option<Self::Lifetime> {
        match limit {
            Some(0) | None => None,
            Some(n) => Some(Arc::new(AtomicU64::new(n))),
        }
    }

    /// Reads the current remaining call count atomically.
    fn remaining(l: &Option<Self::Lifetime>) -> Option<u64> {
        l.as_ref().map(|a| a.load(Ordering::SeqCst))
    }

    /// Checks whether the listener has reached its call limit.
    fn at_limit(l: &Option<Self::Lifetime>) -> bool {
        l.as_ref().is_some_and(|a| a.load(Ordering::SeqCst) == 0)
    }

    /// Atomically decrements the call counter when possible.
    ///
    /// Returns `false` if the counter is already `0`.
    fn try_decrement(l: &mut Option<Self::Lifetime>) -> bool {
        match l {
            None => true,
            Some(a) => a
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                    (x > 0).then_some(x - 1)
                })
                .is_ok(),
        }
    }
}
