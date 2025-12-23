#[cfg(feature = "threaded")]
use std::error::Error;

#[cfg(not(feature = "threaded"))]
use core::error::Error;

/// Errors that can occur in the event system.
///
/// Variants:
/// - `OverloadedEvent`: Too many listeners for an event.
/// - `ListenerNotFound`: Tried to remove or emit to a listener that does not exist.
/// - `EventNotFound`: Tried to remove or emit to an event that does not exist.
/// - `Other`: Any other error
#[derive(Debug)]
pub enum EventError {
    /// Trying to add more than `max_listeners` to an Event.
    ///
    /// Occurs during:
    /// - Adding Listeners
    OverloadedEvent,

    /// Trying to access a specific `Listener` that cannot be found.
    ///
    /// Occurs during:
    /// - Removing Listeners
    ListenerNotFound,

    /// Trying to access a specific `Event` that cannot be found.
    ///
    /// Occurs during:
    /// - Removing Listeners
    /// - Emitting Events
    EventNotFound,

    /// Any other possible Errors during Event Handling
    Other(Box<dyn Error + Send + Sync>),
}
impl PartialEq for EventError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EventError::ListenerNotFound, EventError::ListenerNotFound)
            | (EventError::EventNotFound, EventError::EventNotFound)
            | (EventError::OverloadedEvent, EventError::OverloadedEvent) => true,
            (EventError::Other(a), EventError::Other(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}
impl Eq for EventError {}

impl core::fmt::Display for EventError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EventError::OverloadedEvent => write!(f, "Too many listeners for event"),
            EventError::ListenerNotFound => write!(f, "Listener not found"),
            EventError::EventNotFound => write!(f, "Event not found"),
            EventError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

#[cfg(feature = "threaded")]
impl std::error::Error for EventError {}
