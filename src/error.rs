use crate::{Box};
use crate::{Error, Display, Formatter, Debug, FmtResult, format};

/// Errors that can occur in the event system.
///
/// Variants:
/// - `OverloadedEvent`: Too many listeners for an event.
/// - `ListenerNotFound`: Tried to remove or emit to a listener that does not exist.
/// - `EventNotFound`: Tried to remove or emit to an event that does not exist.
/// - `Other`: Any other error (Boxed trait object).
///
/// Example:
/// ```
/// use rs_events::EventError;
/// let error = EventError::OverloadedEvent;
/// assert_eq!(error, EventError::OverloadedEvent);
/// assert_eq!(error.to_string(), "Too many listeners for event");
/// ```
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
            (EventError::Other(a), EventError::Other(b)) => format!("{:?}", a) == format!("{:?}", b),
            _ => false,
        }
    }
}
impl Eq for EventError {}

impl Display for EventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            EventError::OverloadedEvent => write!(f, "Too many listeners for event"),
            EventError::ListenerNotFound => write!(f, "Listener not found"),
            EventError::EventNotFound => write!(f, "Event not found"),
            EventError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error for EventError {}
