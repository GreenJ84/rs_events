//! # Rust Events (rs_events)
//!
//! This crate provides a flexible, modular event system for Rust applications. Components include:
//!
//! - **Listener**: Represents a struct that holds a tag (optional), callback, and lifetime (optional) which can be registered to an event.
//! - **EventEmitter**: Manages event registration and emission.
//! - **EventHandler**: Trait defining the event API.
//!
//! ## Features
//!
//! The crate supports two build modes via a single feature flag:
//!
//! - **`threaded` (default)**: Enables the std/async backend using `tokio` and `dashmap` for
//!   high concurrency and efficient scheduling. Ideal for servers and desktop apps.
//! - **`no_std`/`alloc` (disable defaults)**: Build without the `threaded` feature to use the
//!   minimal backend suitable for embedded or constrained environments. This backend avoids heavy
//!   dependencies and uses `alloc` types.
//!
//! Select the mode in your `Cargo.toml`:
//!
//! ```toml
//! // Threaded (default)
//! [dependencies]
//! rs_events = "0.1.0"
//!
//! // no_std/alloc (disable defaults)
//! [dependencies]
//! rs_events = { version = "0.1.0", default-features = false }
//! ```
//!
//! ## Backends & Modules
//!
//! - When `threaded` is enabled: types are re-exported from the `threaded` module
//!   (e.g., [`threaded::event_emitter::EventEmitter`]) and include async helpers.
//! - When `threaded` is disabled: types are re-exported from the `base` module
//!   (e.g., [`base::event_emitter::EventEmitter`]) with a lean, allocation-focused design.
//!
//! This crate re-exports the same type names (`EventEmitter`, `Listener`, `EventHandler`) at the
//! top level, so consumer code remains identical across backends.
//!
//! ## Error Model
//!
//! The error type [`EventError`] is consistent across backends, with a difference in the
//! representation of the catch-all variant:
//!
//! - Threaded: `EventError::Other(Box<dyn std::error::Error + Send + Sync>)`
//! - no_std + alloc: `EventError::Other(Box<dyn core::error::Error + Send + Sync>)`
//!
//! ## Usage Examples
//!
//! ### **Threaded (default)**
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

mod constants;
pub use crate::constants::*;

mod error;
pub use crate::error::*;

// Base (non-threaded) backend
#[cfg(not(feature = "threaded"))]
#[cfg_attr(docsrs, doc(cfg(not(feature = "threaded"))))]
mod base;

#[cfg(not(feature = "threaded"))]
#[cfg_attr(docsrs, doc(cfg(not(feature = "threaded"))))]
pub use base::{event_emitter::EventEmitter, event_handler::EventHandler, listener::Listener};

// Threaded backend
#[cfg(feature = "threaded")]
#[cfg_attr(docsrs, doc(cfg(feature = "threaded")))]
mod threaded;

#[cfg(feature = "threaded")]
#[cfg_attr(docsrs, doc(cfg(feature = "threaded")))]
pub use threaded::{event_emitter::EventEmitter, event_handler::EventHandler, listener::Listener};
#[cfg(test)]
mod tests;
