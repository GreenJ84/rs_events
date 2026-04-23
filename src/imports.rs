use crate::alloc;

pub(crate) use alloc::{boxed::Box, format, rc::Rc, string::String, sync::Arc};

pub(crate) use core::{
    cell::Cell,
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub(crate) use core::error::Error;

#[cfg(not(feature = "multi-thread"))]
pub(crate) use alloc::collections::BTreeMap as Map;

// Multi - Thread Map ========================
#[cfg(feature = "multi-thread")]
pub(crate) use dashmap::DashMap as Map;
