// std imports ========================
#[cfg(not(feature = "no_std"))]
mod _std;
#[cfg(not(feature = "no_std"))]
pub(crate) use self::_std::*;
#[cfg(all(not(feature = "no_std"), not(feature = "multi-thread")))]
pub(crate) use std::collections::BTreeMap as Map;

// No std imports ========================
#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
mod no_std;
#[cfg(feature = "no_std")]
pub(crate) use self::no_std::*;

#[cfg(all(feature = "no_std", not(feature = "multi-thread")))]
pub(crate) use alloc::collections::BTreeMap as Map;

// Multi - Thread Map ========================
#[cfg(all(not(feature = "no_std"), feature = "multi-thread"))]
pub(crate) use dashmap::DashMap as Map;