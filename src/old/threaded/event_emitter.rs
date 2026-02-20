
#[cfg(feature = "async-tokio")]
use futures_util::future::{join_all, BoxFuture};

use crate::{Arc, Map};
use crate::{Callback, EventError, EventHandler, EventPayload, Listener};

/// An event emitter that manages listeners and event emissions for a given payload type.
///
/// This struct implements the `EventHandler` trait and provides thread-safe, concurrent event management using a `DashMap` for storage.
///
/// # Type Parameters
/// * `T` - The payload type for events. Must be `Send + Sync + 'static`.
///
/// # Implementation Notes
/// - Uses `DashMap` for concurrent, lock-free event storage.
/// - All listeners for an event are stored in a `Vec` under the event's name.
/// - Designed for high-performance, multi-threaded event-driven applications.
#[derive(Clone)]
pub struct EventEmitter<T>
where
    T: Send + Sync + 'static,
{
    /// The maximum number of listeners allowed per event.
    max_listeners: usize,
    /// The concurrent map of event names to their listeners.
    events: Arc<Map<String, Vec<Listener<T>>>>,
}

impl<T: Send + Sync> EventEmitter<T> {
    /// Creates a new `EventEmitter<T>` from a passed max listeners value.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::rs_events::{EventEmitter, EventHandler};
    ///
    /// let emitter = EventEmitter::<String>::new(20);
    /// assert_eq!(emitter.max_listeners(), 20);
    /// ```
    pub fn new(max_listeners: usize) -> Self {
        Self {
            max_listeners,
            events: Arc::new(DashMap::<String, Vec<Listener<T>>>::new()),
        }
    }

    /// Get a reference to the underlying DashMap of events.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// let events_map = emitter.events();
    /// assert!(events_map.is_empty());
    /// emitter.add("test_event", None, Arc::new(|_|{})).expect("Failed to add listener");
    ///
    /// let events_map = emitter.events();
    /// assert_eq!(events_map.len(), 1);
    /// assert!(events_map.contains_key("test_event"));
    /// ```
    pub fn events(&self) -> &DashMap<String, Vec<Listener<T>>> {
        &self.events
    }
}

impl<T: Send + Sync> EventHandler<T> for EventEmitter<T> {
    /// Returns a vector of all event names that currently have one or more registered listeners.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// emitter.add("event_one", None, Arc::new(|_|{})).expect("Failed to add");
    /// emitter.add("event_two", None, Arc::new(|_|{})).expect("Failed to add");
    ///
    /// let event_names = emitter.event_names();
    /// assert_eq!(event_names.len(), 2);
    /// assert!(event_names.contains(&"event_one".to_string()));
    /// assert!(event_names.contains(&"event_two".to_string()));
    /// ```
    fn event_names(&self) -> Vec<String> {
        self.events
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Sets the maximum number of listeners allowed for any single event.
    ///
    /// # Example
    /// ```
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(5);
    /// emitter.set_max_listeners(3);
    /// assert_eq!(emitter.max_listeners(), 3);
    /// ```
    fn set_max_listeners(&mut self, max: usize) {
        self.max_listeners = max;
    }

    /// Gets the current maximum number of listeners allowed for any event.
    ///
    /// # Example
    /// ```
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let emitter = EventEmitter::<String>::new(7);
    /// assert_eq!(emitter.max_listeners(), 7);
    /// ```
    fn max_listeners(&self) -> usize {
        self.max_listeners
    }

    /// Returns the number of listeners currently registered for the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to query.
    ///
    /// # Returns
    /// * `Ok(usize)` - The number of listeners registered to the event.
    /// * `Err(EventError::EventNotFound)` - If the event is not registered.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// emitter.add("test_event", None, Arc::new(|_|{})).expect("Failed to add");
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    /// ```
    fn listener_count(&self, event_name: &str) -> Result<usize, EventError> {
        self.events
            .get(event_name)
            .map(|entry| entry.len())
            .ok_or(EventError::EventNotFound)
    }

    /// Adds a new listener to the specified event with an optional tag and unlimited lifetime.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `tag_name` - Optional tag for identifying or grouping the listener.
    /// * `callback` - The callback function to invoke when the event is emitted.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if the event has reached its listener limit.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// let listener = emitter.add("test_event", Some("tag1".to_string()), Arc::new(|_|{})).unwrap();
    ///
    /// assert_eq!(listener.tag(), Some(&"tag1".to_string()));
    /// assert_eq!(listener.lifetime(), None);
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    /// ```
    fn add(
        &mut self,
        event_name: &str,
        tag_name: Option<String>,
        callback: Callback<T>,
    ) -> Result<Listener<T>, EventError> {
        self.add_limited(event_name, tag_name, callback, 0)
    }

    /// Adds a new listener to the specified event with an optional tag and a limited number of allowed calls.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `tag_name` - Optional tag for identifying or grouping the listener.
    /// * `callback` - The callback function to invoke when the event is emitted.
    /// * `limit` - The maximum number of times the listener will be called before being removed. Use 0 for unlimited.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if the event has reached its listener limit.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// let listener = emitter.add_limited("test_event", Some("tag2".to_string()), Arc::new(|_|{}), 3).unwrap();
    ///
    /// assert_eq!(listener.lifetime(), Some(3));
    /// assert_eq!(listener.tag(), Some(&"tag2".to_string()));
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    /// ```
    fn add_limited(
        &mut self,
        event_name: &str,
        tag_name: Option<String>,
        callback: Callback<T>,
        limit: u64,
    ) -> Result<Listener<T>, EventError> {
        let mut entry = self.events.entry(event_name.to_string()).or_default();
        if entry.len() < self.max_listeners {
            let listener = Listener::new(
                tag_name,
                callback,
                if limit > 0 { Some(limit) } else { None },
            );
            entry.push(listener.clone());
            return Ok(listener);
        }
        Err(EventError::OverloadedEvent)
    }

    /// Adds a new listener to the specified event with an optional tag that will be called only once.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `tag_name` - Optional tag for identifying or grouping the listener.
    /// * `callback` - The callback function to invoke when the event is emitted.
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if the event has reached its listener limit.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// let listener = emitter.add_once("test_event", Some("tag3".to_string()), Arc::new(|_|{})).unwrap();
    ///
    /// assert_eq!(listener.lifetime(), Some(1));
    /// assert_eq!(listener.tag(), Some(&"tag3".to_string()));
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    /// ```
    fn add_once(
        &mut self,
        event_name: &str,
        tag_name: Option<String>,
        callback: Callback<T>,
    ) -> Result<Listener<T>, EventError> {
        self.add_limited(event_name, tag_name, callback, 1)
    }

    /// Adds a pre-constructed `Listener` to the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to listen to.
    /// * `listener` - The `Listener<T>` instance to add.
    ///
    /// # Returns
    /// * `Ok(())` if the listener was added successfully.
    /// * `Err(EventError::OverloadedEvent)` if the event has reached its listener limit.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler, Listener, EventPayload};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// let listener = Listener::new(Some("tag4".to_string()), Arc::new(|_: &EventPayload<String>| {}), Some(4));
    ///
    /// emitter.add_listener("test_event", listener).unwrap();
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    /// ```
    fn add_listener(&mut self, event_name: &str, listener: Listener<T>) -> Result<(), EventError> {
        let mut entry = self.events.entry(event_name.to_string()).or_default();
        if entry.len() < self.max_listeners {
            entry.push(listener);
            return Ok(());
        }
        Err(EventError::OverloadedEvent)
    }

    /// Removes a specific listener from the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event.
    /// * `listener` - The listener to remove (must match by tag and callback).
    ///
    /// # Returns
    /// * `Ok(Listener<T>)` if the listener was removed successfully.
    /// * `Err(EventError::EventNotFound)` if the event is not registered.
    /// * `Err(EventError::ListenerNotFound)` if the listener is not found for the event.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler, Listener, EventPayload};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// let listener = emitter.add("test_event", Some("tag".to_string()), Arc::new(|_: &EventPayload<String>| {})).ok().unwrap();
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    ///
    /// let removed = emitter.remove_listener("test_event", &listener).unwrap();
    /// assert_eq!(removed.tag(), Some(&"tag".to_string()));
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 0);
    /// ```
    fn remove_listener(
        &mut self,
        event_name: &str,
        other: &Listener<T>,
    ) -> Result<Listener<T>, EventError> {
        if let Some(mut entry) = self.events.get_mut(event_name) {
            let original_len = entry.len();
            entry.retain(|listener| !listener.eq(other));

            return if entry.len() < original_len {
                Ok(other.clone())
            } else {
                Err(EventError::ListenerNotFound)
            };
        }
        Err(EventError::EventNotFound)
    }

    /// Removes all listeners from the specified event.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event.
    ///
    /// # Returns
    /// * `Ok(Vec<Listener<T>>)` with all removed listeners if successful.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler, Listener, EventPayload};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    ///
    /// let listener1 = Listener::new(Some("tag1".to_string()), Arc::new(|_: &EventPayload<String>| {}), None);
    /// emitter.add_listener("test_event", listener1).unwrap();
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 1);
    ///
    /// let listener2 = Listener::new(Some("tag2".to_string()), Arc::new(|_: &EventPayload<String>| {}), None);
    /// emitter.add_listener("test_event", listener2).unwrap();
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 2);
    ///
    /// let removed = emitter.remove_all_listeners("test_event").unwrap();
    /// assert_eq!(removed.len(), 2);
    /// assert_eq!(emitter.listener_count("test_event").unwrap(), 0);
    /// ```
    fn remove_all_listeners(&mut self, event_name: &str) -> Result<Vec<Listener<T>>, EventError> {
        match self.events.get_mut(event_name) {
            Some(mut entry) => Ok(entry.drain(..).collect()),
            None => Err(EventError::EventNotFound),
        }
    }

    /// Emits the specified event synchronously, invoking all registered listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to emit.
    /// * `payload` - The payload to pass to each listener.
    ///
    /// # Returns
    /// * `Ok(Vec<Listener<T>>)` - All listeners that were removed because they reached their call limit after this emit.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    ///
    /// emitter.add("test_event", Some("unlimited".to_string()), Arc::new(|payload| {
    ///     assert_eq!(payload.as_ref(), "hello");
    /// })).unwrap();
    /// emitter.add_once("test_event", Some("once".to_string()), Arc::new(|_| {})).unwrap();
    ///
    /// let removed = emitter.emit("test_event", Arc::new("hello".to_string())).unwrap();
    /// assert_eq!(removed.len(), 1);
    /// assert_eq!(removed[0].tag(), Some(&"once".to_string()));
    /// ```
    fn emit(
        &mut self,
        event_name: &str,
        payload: EventPayload<T>,
    ) -> Result<Vec<Listener<T>>, EventError> {
        if let Some(mut entry) = self.events().get_mut(event_name) {
            for listener in entry.iter_mut() {
                listener.call(&payload);
            }
            // Remove listeners that are at their limit and collect them
            let mut removed = Vec::<Listener<T>>::new();
            let mut i = 0;
            while i < entry.len() {
                if entry[i].at_limit() {
                    removed.push(entry.remove(i));
                } else {
                    i += 1;
                }
            }
            return Ok(removed);
        }
        Err(EventError::EventNotFound)
    }

    /// Emits the specified event synchronously for the last time, then removes all listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to emit.
    /// * `payload` - The payload to pass to each listener.
    ///
    /// # Returns
    /// * `Ok(Vec<Listener<T>>)` if the event was emitted and listeners were removed successfully.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// emitter.add("test_event", Some("unlimited".to_string()), Arc::new(|_| {})).unwrap();
    /// emitter.add_once("test_event", Some("once".to_string()), Arc::new(|_| {})).unwrap();
    ///
    /// let removed = emitter.emit_final("test_event", Arc::new("sync".to_string())).unwrap();
    /// assert_eq!(removed.len(), 2);
    /// ```
    fn emit_final(
        &mut self,
        event_name: &str,
        payload: EventPayload<T>,
    ) -> Result<Vec<Listener<T>>, EventError> {
        let removed: Vec<Listener<T>>;
        if let Some(mut entry) = self.events().get_mut(event_name) {
            // Call all listeners
            for listener in entry.iter_mut() {
                listener.call(&payload);
            }
            // Drain and return all listeners for this event (avoids DashMap deadlock)
            removed = entry.drain(..).collect();
        } else {
            return Err(EventError::EventNotFound);
        }
        self.events().remove(event_name);
        Ok(removed)
    }

    /// Emits the specified event asynchronously, invoking all registered listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to emit.
    /// * `payload` - The payload to pass to each listener.
    /// * `parallel` - If `true`, listeners are called in parallel (spawned as tasks); if `false`, listeners are called sequentially.
    ///
    /// # Returns
    /// A `BoxFuture` that resolves to:
    /// * `Ok(())` if the event was emitted successfully.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use tokio::runtime::Runtime;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// emitter.add("test_event", Some("unlimited".to_string()), Arc::new(|payload| {
    ///     assert_eq!(payload.as_ref(), "async");
    /// })).unwrap();
    /// emitter.add_once("test_event", Some("once".to_string()), Arc::new(|_| {})).unwrap();
    ///
    /// let rt = Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     let removed = emitter.emit_async("test_event", Arc::new("async".to_string()), false).await.unwrap();
    ///
    ///     assert_eq!(removed.len(), 1);
    ///     assert_eq!(removed[0].tag(), Some(&"once".to_string()));
    /// });
    /// ```
    fn emit_async<'a>(
        &'a mut self,
        event_name: &'a str,
        payload: EventPayload<T>,
        parallel: bool,
    ) -> BoxFuture<'a, Result<Vec<Listener<T>>, EventError>> {
        Box::pin(async move {
            if let Some(mut entry) = self.events.get_mut(event_name) {
                let handles = entry.iter_mut().filter_map(|listener| {
                    if parallel {
                        listener.background_call(&payload)
                    } else {
                        listener.blocking_call(&payload)
                    }
                });
                join_all(handles).await;

                // Remove listeners that are at their limit and collect them
                let mut removed = Vec::new();
                let mut i = 0;
                while i < entry.len() {
                    if entry[i].at_limit() {
                        removed.push(entry.remove(i));
                    } else {
                        i += 1;
                    }
                }
                let removed: Vec<Listener<T>> = removed;
                Ok(removed)
            } else {
                Err(EventError::EventNotFound)
            }
        })
    }

    /// Emits the specified event asynchronously for the last time, then removes all listeners.
    ///
    /// # Parameters
    /// * `event_name` - The name of the event to emit.
    /// * `payload` - The payload to pass to each listener.
    /// * `parallel` - If `true`, listeners are called in parallel (spawned as tasks); if `false`, listeners are called sequentially.
    ///
    /// # Returns
    /// A `BoxFuture` that resolves to:
    /// * `Ok(Vec<Listener<T>>)` if the event was emitted and listeners were removed successfully.
    /// * `Err(EventError::EventNotFound)` if the event has not been registered.
    ///
    /// # Example
    /// ```
    /// use std::sync::Arc;
    /// use tokio::runtime::Runtime;
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let mut emitter = EventEmitter::<String>::new(10);
    /// emitter.add("test_event", Some("unlimited".to_string()), Arc::new(|payload| {
    ///     assert_eq!(payload.as_ref(), "async");
    /// })).unwrap();
    /// emitter.add_once("test_event", Some("once".to_string()), Arc::new(|_| {})).unwrap();
    ///
    /// let rt = Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     let removed = emitter.emit_final_async("test_event", Arc::new("async".to_string()), false).await.unwrap();
    ///     assert_eq!(removed.len(), 2);
    ///     assert_eq!(removed[0].tag(), Some(&"unlimited".to_string()));
    ///     assert_eq!(removed[1].tag(), Some(&"once".to_string()));
    /// });
    /// ```
    fn emit_final_async<'a>(
        &'a mut self,
        event_name: &'a str,
        payload: EventPayload<T>,
        parallel: bool,
    ) -> BoxFuture<'a, Result<Vec<Listener<T>>, EventError>> {
        Box::pin(async move {
            if let Some(mut entry) = self.events.get_mut(event_name) {
                let handles = entry.iter_mut().filter_map(|listener| {
                    if parallel {
                        listener.background_call(&payload)
                    } else {
                        listener.blocking_call(&payload)
                    }
                });
                join_all(handles).await;
            } else {
                return Err(EventError::EventNotFound);
            }
            let removed = self
                .events()
                .remove(event_name)
                .map(|(_, listeners)| listeners)
                .unwrap_or_default();
            Ok(removed)
        })
    }
}

impl<T: Send + Sync> Default for EventEmitter<T> {
    /// Creates a new `EventEmitter<T>` with a default max listeners of 10.
    ///
    /// # Example
    ///
    /// ```
    /// use rs_events::{EventEmitter, EventHandler};
    ///
    /// let emitter: EventEmitter<String> = EventEmitter::default();
    /// assert_eq!(emitter.max_listeners(), 10);
    /// ```
    fn default() -> Self {
        Self {
            max_listeners: 10,
            events: Arc::new(DashMap::new()),
        }
    }
}
