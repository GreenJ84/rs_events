//! # Rust Events Crate (rs_events)
//!
//! This crate provides a flexible, modular event system for Rust applications. Components include:
//!
//! - **Listener**: Represents a struct that holds a tag (optional), callback, and lifetime (optional) which can be registered to an event.
//! - **EventEmitter**: Manages event registration and emission.
//! - **EventHandler**: Trait defining the event API.
//!
//! ## Features
//!
//! The crate supports runtime modes via feature flags:
//!
//! - **default** (std): Enables the std/sync backend using the standard library for single-threaded environments.
//! - **no_std**: Utilizes the core library as a replacement for the standard library in environments without std support.
//!
//! The crate also supports optional features for async and multi-threaded extensions:
//! - **async**: Adds async support using
//! `futures-util` and:
//!   - `tokio` for async std task scheduling
//!   - (future) `Embassy` for no_std async utilities
//! - **threaded**: Enables multi-threaded support using `dashmap` for concurrent event storage.
//!
//! ## Error Model
//!
//! The error type [`EventError`] is consistent across backends, with a difference in the
//! representation of the catch-all variant:
//!
//! - std: `EventError::Other(Box<dyn std::error::Error + Send + Sync>)`
//! - no_std + alloc: `EventError::Other(Box<dyn core::error::Error + Send + Sync>)`
//!
//! ## Usage Examples
//!
//! ### **Std (default)**
//!
//! ```toml
//! [dependencies]
//! rs_events = "0.1.0"
//! ```
//!
//! ```rust
//! use rs_events::{EventEmitter, EventPayload, EventHandler};
//! use std::sync::Arc;
//!
//! let mut emitter = EventEmitter::<String>::default();
//!
//! emitter.add("event", None, Arc::new(|payload| {
//!     println!("Received: {}", payload.as_ref());
//! })).unwrap();
//!
//! emitter.emit("event", Arc::new("Hello World".to_string())).unwrap();
//! ```
//!
//! ### **no_std/alloc**
//!
//! ```toml
//! [dependencies]
//! rs_events = { version = "0.1.0", default-features = false }
//! ```
//!
//! ```rust
//! extern crate alloc;
//! use alloc::sync::Arc;
//! use alloc::string::String;
//! use rs_events::{EventEmitter, EventPayload, EventHandler};
//!
//! let mut emitter = EventEmitter::<String>::default();
//!
//! emitter.add("event", None, Arc::new(|payload| {
//!     // Handle event
//! })).unwrap();
//!
//! emitter.emit("event", Arc::new(String::from("Hello no_std!"))).unwrap();
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]


// App Re-exports =============
mod imports;
pub(crate) use imports::*;

mod constants;
pub use crate::constants::*;

mod error;
pub use crate::error::*;

mod listener;
pub use crate::listener::Listener;

mod emitter;
pub use crate::emitter::EventEmitter;

mod handler;
pub use crate::handler::EventHandler;


// Integration Tests ===============
#[cfg(test)]
mod tests;