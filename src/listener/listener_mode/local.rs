//! Local listener mode backed by `Rc` and `Cell`.
//!
//! This mode is intended for single-threaded listener usage where atomic
//! synchronization is unnecessary.

use super::ListenerMode;
use crate::{Cell, LocalCallback, Rc};

/// Single-threaded listener mode.
///
/// - Callback type utilizes [`LocalCallback`].
/// - Lifetime type utilizes `Rc<Cell<u64>>`.
pub struct LocalMode;
impl ListenerMode for LocalMode {
    type Callback<T> = LocalCallback<T>;
    type Lifetime = Rc<Cell<u64>>;

    /// Creates a local lifetime counter.
    ///
    /// `None` and `Some(0)` produce no counter (unlimited listener).
    fn new_lifetime(limit: Option<u64>) -> Option<Self::Lifetime> {
        match limit {
            Some(0) | None => None,
            Some(n) => Some(Rc::new(Cell::new(n))),
        }
    }

    /// Reads the current remaining call count.
    fn remaining(l: &Option<Self::Lifetime>) -> Option<u64> {
        l.as_ref().map(|c| c.get())
    }

    /// Checks whether the listener has reached its call limit.
    fn at_limit(l: &Option<Self::Lifetime>) -> bool {
        l.as_ref().is_some_and(|c| c.get() == 0)
    }

    /// Decrements the call counter when possible.
    ///
    /// Returns `false` if the counter is already `0`.
    fn try_decrement(l: &mut Option<Self::Lifetime>) -> bool {
        match l {
            None => true,
            Some(c) if c.get() > 0 => {
                c.set(c.get() - 1);
                true
            }
            Some(_) => false,
        }
    }
}
