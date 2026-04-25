///! This module contains aliases for the crate, which are used in various places throughout the codebase.
///!
/// ! The `ListenerMode` trait defines the behavior contract for listener storage backends, allowing for flexible callback and lifetime management across different listener modes.


pub(crate) mod local;
pub use self::local::*;

/// Shared constants that require Send + Sync for asynchronous and/or thread safety
///
/// They are only included if the multi_threaded feature or an async feature is enabled
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub(crate) mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use self::shared::*;
