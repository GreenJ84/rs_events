extern crate alloc;
use alloc::{string::String, vec::Vec};

use crate::{Callback, EventError, EventPayload, Listener};

/// Defines the contract for event-driven types that manage listeners and emit events.
///
/// This trait provides a flexible, extensible API for adding, removing, and querying listeners, as well as emitting events synchronously.
/// Implementors can be used as event-driven components in any no_std environment, and users may implement their own custom event handlers by conforming to this trait.
///
/// # Type Parameters
/// * `T`: The payload type for events.
///
pub trait EventHandler<T> {
    /// Gets the names of events that currently have one or more active listeners.
    ///
    /// # Returns
    /// A `Vec<String>` containing the names of all events with at least one registered listener.
    fn event_names(&self) -> Vec<String>;

    /// Sets the maximum number of listeners allowed per event.
    ///
    /// # Parameters
    /// * `max` - The maximum number of listeners allowed for any single event.
    fn set_max_listeners(&mut self, max: usize);

    /// Gets the current maximum number of listeners allowed per event.
    ///
    /// # Returns
    /// The maximum number of listeners as a `usize`.
    fn max_listeners(&self) -> usize;

    /// Creates and adds a listener with unlimited lifetime to the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `tag_name` - An optional tag name for the listener.
    /// * `callback` - The callback to invoke when the event is emitted.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if adding the listener would exceed the maximum allowed listeners for the event.
    fn add(
        &mut self,
        event_name: &str,
        tag_name: Option<String>,
        callback: Callback<T>,
    ) -> Result<Listener<T>, EventError>;

    /// Creates and adds a listener with a limited number of allowed calls to the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `tag_name` - An optional tag name for the listener.
    /// * `callback` - The callback to invoke when the event is emitted.
    /// * `limit` - The maximum number of times the listener will be called before being removed.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if adding the listener would exceed the maximum allowed listeners for the event.
    fn add_limited(
        &mut self,
        event_name: &str,
        tag_name: Option<String>,
        callback: Callback<T>,
        limit: u64,
    ) -> Result<Listener<T>, EventError>;

    /// Creates and adds a listener that will be called only once for the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `tag_name` - An optional tag name for the listener.
    /// * `callback` - The callback to invoke when the event is emitted.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if adding the listener would exceed the maximum allowed listeners for the event.
    fn add_once(
        &mut self,
        event_name: &str,
        tag_name: Option<String>,
        callback: Callback<T>,
    ) -> Result<Listener<T>, EventError>;

    /// Adds a listener for the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `listener` - The listener to add.
    ///
    /// # Returns
    /// * `Ok(())` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if adding the listener would exceed the maximum allowed listeners for the event.
    fn add_listener(&mut self, event_name: &str, listener: Listener<T>) -> Result<(), EventError>;

    /// Gets the number of listeners currently registered to the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to query.
    ///
    /// # Returns
    /// * `Ok(usize)` - The number of listeners registered to the event.
    /// * `Err(EventError::EventNotFound)` - If the event is not registered.
    fn listener_count(&self, event_name: &str) -> Result<usize, EventError>;

    /// Returns `true` if the specified event has any registered listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to check.
    ///
    /// # Returns
    /// * `Ok(true)` if there is at least one listener for the event.
    /// * `Ok(false)` if there are no listeners for the event.
    /// * `Err(EventError::EventNotFound)` if the event is not registered.
    fn has_listener(&self, event_name: &str) -> Result<bool, EventError> {
        Ok(self.listener_count(event_name)? > 0)
    }

    /// Removes a specific listener from the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event.
    /// * `listener` - The listener to remove.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was removed successfully.
    /// * `Err(EventError::EventNotFound)` if the event is not registered.
    /// * `Err(EventError::ListenerNotFound)` if the listener is not found for the event.
    fn remove_listener(
        &mut self,
        event_name: &str,
        listener: &Listener<T>,
    ) -> Result<Listener<T>, EventError>;

    /// Removes all listeners from the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event.
    ///
    /// # Returns
    /// * `Ok(Vec<Listener<T>>)` with all removed listeners if successful.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    fn remove_all_listeners(&mut self, event_name: &str) -> Result<Vec<Listener<T>>, EventError>;

    /// Emits the specified event synchronously, invoking all registered listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to emit.
    /// * `payload` - The payload to pass to each listener.
    ///
    /// # Returns
    /// * `Ok(())` if the event was emitted successfully.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    fn emit(
        &mut self,
        event_name: &str,
        payload: EventPayload<T>,
    ) -> Result<Vec<Listener<T>>, EventError>;

    /// Emits the specified event synchronously for the last time, then removes all listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to emit.
    /// * `payload` - The payload to pass to each listener.
    ///
    /// # Returns
    /// * `Ok(Vec<Listener<T>>)` if the event was emitted and listeners were removed successfully.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    fn emit_final(
        &mut self,
        event_name: &str,
        payload: EventPayload<T>,
    ) -> Result<Vec<Listener<T>>, EventError>;
}
