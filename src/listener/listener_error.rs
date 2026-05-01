use crate::{Debug, Display, Error, FmtResult, Formatter};

/// Errors that can occur when operating on a `Listener`.
#[derive(Debug)]
pub enum ListenerError {
    /// The listener has reached its call limit and cannot be invoked.
    Exhausted,

    /// The listener callback could not be invoked.
    InvocationFailure(String),

    #[cfg(any(feature = "async-tokio", feature = "multi-thread"))]
    SpawnFailed(String),

    /// Generic storage/locking error reported by the underlying mode.
    StorageError(String),

    /// Any other possible Errors during Listener Handling
    Other(Box<dyn Error + Send + Sync>),
}

impl PartialEq for ListenerError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ListenerError::InvocationFailure(s1), ListenerError::InvocationFailure(s2))
            | (ListenerError::StorageError(s1), ListenerError::StorageError(s2)) => s1 == s2,
            #[cfg(any(feature = "async-tokio", feature = "multi-thread"))]
            (ListenerError::SpawnFailed(s1), ListenerError::SpawnFailed(s2)) => s1 == s2,
            (ListenerError::Exhausted, ListenerError::Exhausted) => true,
            (ListenerError::Other(a), ListenerError::Other(b)) => {
                format!("{:?}", a) == format!("{:?}", b)
            }
            _ => false,
        }
    }
}
impl Eq for ListenerError {}

impl Display for ListenerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ListenerError::Exhausted => write!(f, "Listener has reached its call limit"),
            ListenerError::InvocationFailure(s) => write!(f, "Listener invocation failed: {}", s),
            #[cfg(any(feature = "async-tokio", feature = "multi-thread"))]
            ListenerError::SpawnFailed(s) => write!(f, "Listener spawn failed: {}", s),
            ListenerError::StorageError(s) => write!(f, "Listener storage error: {}", s),
            ListenerError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Error for ListenerError {}
