use crate::{EventEmitter, EventError, EventHandler, EventPayload, Listener};
use std::string::{String, ToString};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// Test adding listeners of different types and verifying their properties
#[test]
fn add_listeners_variety() {
    let mut emitter = EventEmitter::<String>::default();

    let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});

    // Add unlimited listener
    let l1 = emitter
        .add("event", Some("tag1".to_string()), cb.clone())
        .unwrap();
    assert_eq!(l1.tag(), Some(&"tag1".to_string()));
    assert_eq!(l1.lifetime(), None);

    // Add limited listener
    let l2 = emitter
        .add_limited("event", Some("tag2".to_string()), cb.clone(), 2)
        .unwrap();
    assert_eq!(l2.tag(), Some(&"tag2".to_string()));
    assert_eq!(l2.lifetime(), Some(2));

    // Add once listener
    let l3 = emitter
        .add_once("event", Some("tag3".to_string()), cb.clone())
        .unwrap();
    assert_eq!(l3.tag(), Some(&"tag3".to_string()));
    assert_eq!(l3.lifetime(), Some(1));

    assert_eq!(emitter.listener_count("event").unwrap(), 3);
}

/// Test listener at-limit logic and removal after emission
#[test]
fn listener_at_limit_and_removal() {
    let mut emitter = EventEmitter::<String>::default();

    let called = Arc::new(AtomicU64::new(0));
    let called_clone = called.clone();
    let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
        called_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Add a once listener
    emitter.add_once("event", None, cb.clone()).unwrap();
    // Add a limited listener
    emitter.add_limited("event", None, cb.clone(), 3).unwrap();

    // Emit Once: once listener should be removed after first emit
    let _ = emitter.emit("event", Arc::new("payload".to_string()));
    assert_eq!(called.load(Ordering::SeqCst), 2);
    assert_eq!(emitter.listener_count("event").unwrap_or(0), 1);
    // Emit event twice more
    for _ in 0..2 {
        let _ = emitter.emit("event", Arc::new("payload".to_string()));
    }
    // All listeners should be removed
    assert_eq!(called.load(Ordering::SeqCst), 4);
    assert_eq!(emitter.listener_count("event").unwrap_or(0), 0);
}

/// Test event overflow and error handling
#[test]
fn event_overload() {
    let mut emitter = EventEmitter::<String>::new(2);
    let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});
    emitter.add("event", None, cb.clone()).unwrap();
    emitter.add("event", None, cb.clone()).unwrap();
    // Third add should fail
    let res = emitter.add("event", None, cb.clone());
    assert_eq!(res, Err(EventError::OverloadedEvent));
}

#[cfg(test)]
mod removing_listeners {
    use super::*;

    /// Remove a single unlimited listener
    #[test]
    fn remove_single_listener() {
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});

        let mut emitter = EventEmitter::<String>::default();
        // Add & remove infinite listener
        let listener = emitter.add("test", None, cb.clone()).unwrap();
        assert_eq!(emitter.listener_count("test").unwrap(), 1);
        assert!(emitter.remove_listener("test", &listener).is_ok());
        assert_eq!(emitter.listener_count("test").unwrap_or(0), 0);
        // Add & remove limited listener
        let listener = emitter.add_limited("test", None, cb.clone(), 5).unwrap();
        assert_eq!(emitter.listener_count("test").unwrap(), 1);
        assert!(emitter.remove_listener("test", &listener).is_ok());
        assert_eq!(emitter.listener_count("test").unwrap_or(0), 0);
        // Add & remove once listener
        let listener = emitter.add_once("test", None, cb.clone()).unwrap();
        assert_eq!(emitter.listener_count("test").unwrap(), 1);
        assert!(emitter.remove_listener("test", &listener).is_ok());
        assert_eq!(emitter.listener_count("test").unwrap_or(0), 0);
    }

    /// Remove all listeners from an event
    #[test]
    fn remove_all_listeners() {
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});

        let mut emitter = EventEmitter::<String>::default();
        for i in 0..10 {
            match i % 3 {
                0 => emitter.add("test", None, cb.clone()).unwrap(),
                1 => emitter.add_limited("test", None, cb.clone(), 5).unwrap(),
                _ => emitter.add_once("test", None, cb.clone()).unwrap(),
            };
        }
        assert_eq!(emitter.listener_count("test").unwrap(), 10);
        let returned = emitter.remove_all_listeners("test");

        assert!(returned.is_ok());
        assert_eq!(emitter.listener_count("test").unwrap_or(0), 0);
        assert_eq!(returned.unwrap().len(), 10);
    }

    /// Removing a non-existent listener returns ListenerNotFound
    #[test]
    fn remove_invalid_listener_throws_error() {
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add("test", Some("tag1".to_string()), cb.clone())
            .unwrap();

        let fake = Listener::new(None, Arc::new(|_| {}), None);
        assert_eq!(
            emitter.remove_listener("test", &fake),
            Err(EventError::ListenerNotFound)
        );
    }

    /// Removing from a non-existent event returns EventNotFound
    #[test]
    fn remove_from_invalid_event_throws_error() {
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});

        let mut emitter = EventEmitter::<String>::default();
        emitter.add("test", None, cb.clone()).unwrap();

        let fake = Listener::new(None, Arc::new(|_| {}), None);
        assert_eq!(
            emitter.remove_listener("not_test", &fake),
            Err(EventError::EventNotFound)
        );
    }
}

#[cfg(test)]
mod emitting_events {
    use super::*;

    /// Emit event multiple times and verify all listeners are called
    #[test]
    fn emit_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            assert_eq!(payload.as_ref(), "Test");
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter.add("count", None, cb.clone()).unwrap();
        for _ in 0..10 {
            assert!(emitter.emit("count", Arc::new("Test".to_string())).is_ok());
        }
        assert_eq!(
            called.load(Ordering::SeqCst),
            10,
            "Event callbacks unsuccessful"
        );
    }

    /// Emit one event with multiple listeners and verify all are called
    #[test]
    fn emit_multiple_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            assert_eq!(payload.as_ref(), "Test");
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        for _ in 0..10 {
            emitter.add("count", None, cb.clone()).unwrap();
        }
        assert!(emitter.emit("count", Arc::new("Test".to_string())).is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            10,
            "Event callbacks unsuccessful"
        );
    }

    /// Emit event for once listener and verify drop-off
    #[test]
    fn once_listener_emission_drop_off_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_once("count", Some("tag1".to_string()), cb.clone())
            .unwrap();

        let falloff = emitter.emit("count", Arc::new("Increment".to_string()));
        assert!(falloff.is_ok(), "Emit failed");
        assert_eq!(
            called.load(Ordering::SeqCst),
            1,
            "Event callbacks unsuccessful"
        );

        // Falloff should contain the emitted event that fell off
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );

        // Listener should be removed after 1 call
        assert_eq!(emitter.listener_count("count").unwrap_or(0), 0);
    }

    /// Emit event for limited listener and verify drop-off
    #[test]
    fn limited_listener_emission_drop_off_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_limited("count", Some("tag1".to_string()), cb.clone(), 5)
            .unwrap();
        for _ in 0..4 {
            assert!(emitter.emit("count", Arc::new("Test".to_string())).is_ok());
        }
        // Fifth emit should return falloff
        let falloff = emitter.emit("count", Arc::new("Test".to_string()));
        assert!(falloff.is_ok(), "Emit failed");
        assert_eq!(
            called.load(Ordering::SeqCst),
            5,
            "Event callbacks unsuccessful"
        );

        // Falloff should contain the emitted event that has fired for the last time
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );

        // Listener should be removed after 5 calls
        assert_eq!(emitter.listener_count("count").unwrap_or(0), 0);
    }
}

#[cfg(test)]
mod emitting_final_events {
    use super::*;

    /// Emit final for unlimited listener and verify removal
    #[test]
    fn emit_final_drops_infinite_listener_and_event() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add("count_final", Some("tag1".to_string()), cb.clone())
            .unwrap();
        assert!(
            emitter
                .emit("count_final", Arc::new("Test".to_string()))
                .is_ok(),
            "Regular emit failed"
        );

        // Final emit should remove listener
        let falloff = emitter.emit_final("count_final", Arc::new("Test".to_string()));
        assert!(falloff.is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Event callbacks unsuccessful"
        );

        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );

        // After final emit, event should be removed
        assert_eq!(
            emitter.emit_final("count_final", Arc::new("Test".to_string())),
            Err(EventError::EventNotFound)
        );
    }

    /// Emit final for limited listener and verify removal
    #[test]
    fn emit_final_drops_limited_listener_and_event() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_limited("count_final", Some("tag1".to_string()), cb.clone(), 5)
            .unwrap();
        assert!(
            emitter
                .emit("count_final", Arc::new("Test".to_string()))
                .is_ok(),
            "Regular emit failed"
        );

        // Final emit should remove listener
        let falloff = emitter.emit_final("count_final", Arc::new("Test".to_string()));
        assert!(falloff.is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Event callbacks unsuccessful"
        );

        // Falloff should contain the emitted event that has fired for the last time
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(falloff[0].lifetime(), Some(3), "Unexpected event lifetime");

        // After final emit, event should be removed
        assert_eq!(
            emitter.emit_final("count_final", Arc::new("Test".to_string())),
            Err(EventError::EventNotFound)
        );
    }
}

#[cfg(test)]
mod emit_async_events {
    use super::*;

    #[tokio::test]
    async fn async_emit_blocking_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            assert_eq!(payload.as_ref(), "AsyncTest");
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter.add("async_event", None, cb.clone()).unwrap();
        for _ in 0..10 {
            assert!(emitter
                .emit_async("async_event", Arc::new("AsyncTest".to_string()), false)
                .await
                .is_ok());
        }
        assert_eq!(
            called.load(Ordering::SeqCst),
            10,
            "Async event callbacks unsuccessful"
        );
    }

    #[tokio::test]
    async fn async_emit_parallel_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            assert_eq!(payload.as_ref(), "AsyncTest");
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter.add("async_event", None, cb.clone()).unwrap();
        for _ in 0..10 {
            assert!(emitter
                .emit_async("async_event", Arc::new("AsyncTest".to_string()), true)
                .await
                .is_ok());
        }
        assert_eq!(
            called.load(Ordering::SeqCst),
            10,
            "Async event callbacks unsuccessful"
        );
    }

    #[tokio::test]
    async fn async_emit_multiple_blocking_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            assert_eq!(payload.as_ref(), "AsyncTest");
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        for _ in 0..10 {
            emitter.add("async_event", None, cb.clone()).unwrap();
        }
        assert!(emitter
            .emit_async("async_event", Arc::new("AsyncTest".to_string()), false)
            .await
            .is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            10,
            "Async event callbacks unsuccessful"
        );
    }

    #[tokio::test]
    async fn async_emit_multiple_parallel_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            assert_eq!(payload.as_ref(), "AsyncTest");
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        for _ in 0..10 {
            emitter.add("async_event", None, cb.clone()).unwrap();
        }
        assert!(emitter
            .emit_async("async_event", Arc::new("AsyncTest".to_string()), true)
            .await
            .is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            10,
            "Async event callbacks unsuccessful"
        );
    }

    #[tokio::test]
    async fn async_blocking_emission_drop_off_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_once("async", Some("once".to_string()), cb.clone())
            .unwrap();
        emitter
            .add_limited("async", Some("limited".to_string()), cb.clone(), 5)
            .unwrap();

        // Emit once and once listener should falloff
        let falloff = emitter
            .emit_async("async", Arc::new("OnceTest".to_string()), false)
            .await;
        assert!(falloff.is_ok(), "Async emit failed");
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Async once callback unsuccessful"
        );

        // Falloff should only contain the once listener
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("once".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(
            emitter.listener_count("async").unwrap_or(0),
            1,
            "Limited listener did not remain"
        );

        // Emit 4 more times, limited listener should falloff on 5th
        for _ in 0..3 {
            assert!(emitter
                .emit_async("async", Arc::new("LimitedTest".to_string()), false)
                .await
                .is_ok());
        }

        // Emit 5th time and limited listener should falloff
        let falloff = emitter
            .emit_async("async", Arc::new("OnceTest".to_string()), false)
            .await;
        assert!(falloff.is_ok(), "Async emit failed");
        assert_eq!(
            called.load(Ordering::SeqCst),
            6,
            "Async once callback unsuccessful"
        );

        // Falloff should only contain the limited listener
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("limited".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(emitter.listener_count("async").unwrap_or(0), 0);
    }

    #[tokio::test]
    async fn async_parallel_emission_drop_off_successful() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_once("async", Some("once".to_string()), cb.clone())
            .unwrap();
        emitter
            .add_limited("async", Some("limited".to_string()), cb.clone(), 5)
            .unwrap();

        // Emit once and once listener should falloff
        let falloff = emitter
            .emit_async("async", Arc::new("OnceTest".to_string()), true)
            .await;
        assert!(falloff.is_ok(), "Async emit failed");
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Async once callback unsuccessful"
        );

        // Falloff should only contain the once listener
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("once".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(
            emitter.listener_count("async").unwrap_or(0),
            1,
            "Limited listener did not remain"
        );

        // Emit 4 more times, limited listener should falloff on 5th
        for _ in 0..3 {
            assert!(emitter
                .emit_async("async", Arc::new("LimitedTest".to_string()), true)
                .await
                .is_ok());
        }

        // Emit 5th time and limited listener should falloff
        let falloff = emitter
            .emit_async("async", Arc::new("OnceTest".to_string()), true)
            .await;
        assert!(falloff.is_ok(), "Async emit failed");
        assert_eq!(
            called.load(Ordering::SeqCst),
            6,
            "Async once callback unsuccessful"
        );

        // Falloff should only contain the limited listener
        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("limited".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(emitter.listener_count("async").unwrap_or(0), 0);
    }

    #[tokio::test]
    async fn async_mixed_listeners_and_calls_blocking() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            if payload.as_ref() == "MixTest" {
                called_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let mut emitter = EventEmitter::<String>::default();
        // Add 5 unlimited, 3 limited, 2 once listeners
        for _ in 0..5 {
            emitter.add("mix_event", None, cb.clone()).unwrap();
        }
        for _ in 0..3 {
            emitter
                .add_limited("mix_event", None, cb.clone(), 2)
                .unwrap();
        }
        for _ in 0..2 {
            emitter.add_once("mix_event", None, cb.clone()).unwrap();
        }
        // Emit event 3 times
        for _ in 0..3 {
            assert!(emitter
                .emit_async("mix_event", Arc::new("MixTest".to_string()), false)
                .await
                .is_ok());
        }
        // Unlimited: 5*3, Limited: 3*2, Once: 2*1
        assert_eq!(
            called.load(Ordering::SeqCst),
            15 + 6 + 2,
            "Async mixed callbacks unsuccessful"
        );
        // After all calls, only unlimited listeners remain
        assert_eq!(emitter.listener_count("mix_event").unwrap_or(0), 5);
    }

    #[tokio::test]
    async fn async_mixed_listeners_and_calls_parallel() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |payload| {
            if payload.as_ref() == "MixTest" {
                called_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let mut emitter = EventEmitter::<String>::default();
        // Add 5 unlimited, 3 limited, 2 once listeners
        for _ in 0..5 {
            emitter.add("mix_event", None, cb.clone()).unwrap();
        }
        for _ in 0..3 {
            emitter
                .add_limited("mix_event", None, cb.clone(), 2)
                .unwrap();
        }
        for _ in 0..2 {
            emitter.add_once("mix_event", None, cb.clone()).unwrap();
        }
        // Emit event 3 times in parallel
        for _ in 0..3 {
            assert!(emitter
                .emit_async("mix_event", Arc::new("MixTest".to_string()), true)
                .await
                .is_ok());
        }
        // Unlimited: 5*3, Limited: 3*2, Once: 2*1
        assert_eq!(
            called.load(Ordering::SeqCst),
            15 + 6 + 2,
            "Async mixed callbacks unsuccessful"
        );
        // After all calls, only unlimited listeners remain
        assert_eq!(emitter.listener_count("mix_event").unwrap_or(0), 5);
    }
}

#[cfg(test)]
mod emit_async_final_events {
    use super::*;

    #[tokio::test]
    async fn async_emit_final_drops_infinite_listener_blocking() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add("count_final", Some("tag1".to_string()), cb.clone())
            .unwrap();
        assert!(
            emitter
                .emit_async("count_final", Arc::new("Test".to_string()), false)
                .await
                .is_ok(),
            "Regular async emit failed"
        );

        // Final async emit should remove listener
        let falloff = emitter
            .emit_final_async("count_final", Arc::new("Test".to_string()), false)
            .await;
        assert!(falloff.is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Async event callbacks unsuccessful"
        );

        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );

        // After final emit, event should be removed
        assert_eq!(
            emitter
                .emit_final_async("count_final", Arc::new("Test".to_string()), false)
                .await,
            Err(EventError::EventNotFound)
        );
    }

    #[tokio::test]
    async fn async_emit_final_drops_infinite_listener_parallel() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add("count_final", Some("tag1".to_string()), cb.clone())
            .unwrap();
        assert!(
            emitter
                .emit_async("count_final", Arc::new("Test".to_string()), true)
                .await
                .is_ok(),
            "Regular async emit failed"
        );

        // Final async emit should remove listener
        let falloff = emitter
            .emit_final_async("count_final", Arc::new("Test".to_string()), true)
            .await;
        assert!(falloff.is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Async event callbacks unsuccessful"
        );

        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );

        // After final emit, event should be removed
        assert_eq!(
            emitter
                .emit_final_async("count_final", Arc::new("Test".to_string()), true)
                .await,
            Err(EventError::EventNotFound)
        );
    }

    #[tokio::test]
    async fn async_emit_final_drops_limited_listener_blocking() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_limited("count_final", Some("tag1".to_string()), cb.clone(), 5)
            .unwrap();
        assert!(
            emitter
                .emit_async("count_final", Arc::new("Test".to_string()), false)
                .await
                .is_ok(),
            "Regular async emit failed"
        );

        // Final async emit should remove listener
        let falloff = emitter
            .emit_final_async("count_final", Arc::new("Test".to_string()), false)
            .await;
        assert!(falloff.is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Async event callbacks unsuccessful"
        );

        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(falloff[0].lifetime(), Some(3), "Unexpected event lifetime");

        // After final emit, event should be removed
        assert_eq!(
            emitter
                .emit_final_async("count_final", Arc::new("Test".to_string()), false)
                .await,
            Err(EventError::EventNotFound)
        );
    }

    #[tokio::test]
    async fn async_emit_final_drops_limited_listener_parallel() {
        let called = Arc::new(AtomicU64::new(0));
        let called_clone = called.clone();
        let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mut emitter = EventEmitter::<String>::default();
        emitter
            .add_limited("count_final", Some("tag1".to_string()), cb.clone(), 5)
            .unwrap();
        assert!(
            emitter
                .emit_async("count_final", Arc::new("Test".to_string()), true)
                .await
                .is_ok(),
            "Regular async emit failed"
        );

        // Final async emit should remove listener
        let falloff = emitter
            .emit_final_async("count_final", Arc::new("Test".to_string()), true)
            .await;
        assert!(falloff.is_ok());
        assert_eq!(
            called.load(Ordering::SeqCst),
            2,
            "Async event callbacks unsuccessful"
        );

        let falloff = falloff.unwrap();
        assert_eq!(falloff.len(), 1, "Expected 1 event to falloff");
        assert_eq!(
            falloff[0].tag(),
            Some("tag1".to_string()).as_ref(),
            "Unexpected event payload"
        );
        assert_eq!(falloff[0].lifetime(), Some(3), "Unexpected event lifetime");

        // After final emit, event should be removed
        assert_eq!(
            emitter
                .emit_final_async("count_final", Arc::new("Test".to_string()), true)
                .await,
            Err(EventError::EventNotFound)
        );
    }
}
