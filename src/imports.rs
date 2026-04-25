use crate::alloc;

pub(crate) use alloc::{boxed::Box, format, rc::Rc, string::String};
pub(crate) use core::{
    cell::Cell,
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

#[cfg(not(feature = "multi-thread"))]
pub(crate) use alloc::collections::BTreeMap as Map;

#[cfg(any(feature = "async-tokio", feature = "multi-thread"))]
pub(crate) use {
    alloc::sync::Arc,
    core::sync::atomic::{AtomicU64, Ordering},
};

// Multi - Thread Map ========================
#[cfg(feature = "multi-thread")]
pub(crate) use dashmap::DashMap as Map;
