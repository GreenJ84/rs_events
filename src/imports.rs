// STD ========================
#[cfg(not(feature = "no_std"))]
pub(crate) use std::sync::{
  atomic::{AtomicU64, Ordering},
  Arc,
};

#[cfg(not(feature = "no_std"))]
pub(crate) use std::fmt::{Display, Debug, Formatter, Result as FmtResult};

#[cfg(not(feature = "no_std"))]
pub(crate) use std::error::Error;

#[cfg(all(not(feature = "no_std"), not(feature = "multi-thread")))]
pub use std::collections::BTreeMap as Map;


// NO_STD ========================
#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
pub(crate) use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::{
  string::String,
  sync::Arc
};

#[cfg(feature = "no_std")]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "no_std")]
use core::fmt::{Display, Debug, Formatter, Result as FmtResult};

#[cfg(feature = "no_std")]
use core::error::Error;

#[cfg(all(feature = "no_std", not(feature = "multi-thread")))]
pub use alloc::collections::BTreeMap as Map;

// Multi - Thread ========================
#[cfg(feature = "multi-thread")]
pub use dashmap::DashMap as Map;