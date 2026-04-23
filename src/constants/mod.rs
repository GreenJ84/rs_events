///! This module contains constants for the crate, which are used in various places throughout the codebase.
///!
///! The constants are organized into submodules based on their functionality and the features they are associated with.
/// ! The main constants module re-exports the relevant constants based on the enabled features, ensuring that only the necessary constants are included in the final build.

/// By default, we use single-threaded constants which do not require Send or Sync for simplicity.
///
/// Even with async-tokio enabled, we use single-threaded constants to avoid unnecessary Send + Sync bounds on payloads and callbacks that do not need them with local async task spawning.
pub(crate) mod local;
pub use self::local::*;

/// Shared constants that require Send + Sync for asynchronous and/or thread safety
///
/// They are only included if the multi_threaded feature or an async feature is enabled
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub(crate) mod shared;
#[cfg(any(feature = "multi-thread", feature = "async-tokio"))]
pub use self::shared::*;
