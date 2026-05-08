use crate::{EventPayload, Listener};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[test]
fn listener_new_and_tag() {
    // Test construction with tag and without
    let cb: Arc<dyn Fn(&EventPayload<String>) + Send + Sync> = Arc::new(|_| {});
    let l1 = Listener::new(Some("tag1".to_string()), cb.clone(), None);
    assert_eq!(l1.tag(), Some(&"tag1".to_string()));
    assert_eq!(l1.lifetime(), None);

    let l2 = Listener::new(None, cb.clone(), Some(3));
    assert_eq!(l2.tag(), None);
    assert_eq!(l2.lifetime(), Some(3));
}

#[test]
fn listener_callback_and_lifetime() {
    let called = Arc::new(AtomicU64::new(0));
    let called2 = called.clone();
    let cb: Arc<dyn Fn(&EventPayload<u32>) + Send + Sync> = Arc::new(move |_| {
        called2.fetch_add(1, Ordering::SeqCst);
    });

    let mut l = Listener::new(None, cb, Some(2));
    assert_eq!(l.lifetime(), Some(2));
    l.call(&Arc::new(42));
    assert_eq!(l.lifetime(), Some(1));
    l.call(&Arc::new(99));
    assert_eq!(l.lifetime(), Some(0));
    // At limit, further calls do not invoke callback
    l.call(&Arc::new(123));
    assert_eq!(called.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn listener_blocking_call() {
    let called = Arc::new(AtomicU64::new(0));
    let called2 = called.clone();
    let cb: Arc<dyn Fn(&EventPayload<u32>) + Send + Sync> = Arc::new(move |_| {
        called2.fetch_add(1, Ordering::SeqCst);
    });

    let mut l = Listener::new(None, cb, Some(2));
    let handle1 = l.blocking_call(&Arc::new(42));
    let handle2 = l.blocking_call(&Arc::new(99));
    let handle3 = l.blocking_call(&Arc::new(123));
    if let Some(h) = handle1 {
        h.await.unwrap();
    }
    if let Some(h) = handle2 {
        h.await.unwrap();
    }
    assert!(handle3.is_none());
    assert_eq!(called.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn listener_background_call() {
    let called = Arc::new(AtomicU64::new(0));
    let called2 = called.clone();
    let cb: Arc<dyn Fn(&EventPayload<u32>) + Send + Sync> = Arc::new(move |_| {
        called2.fetch_add(1, Ordering::SeqCst);
    });

    let mut l = Listener::new(None, cb, Some(2));
    let handle1 = l.background_call(&Arc::new(42));
    let handle2 = l.background_call(&Arc::new(99));
    let handle3 = l.background_call(&Arc::new(123));
    if let Some(h) = handle1 {
        h.await.unwrap();
    }
    if let Some(h) = handle2 {
        h.await.unwrap();
    }
    assert!(handle3.is_none());
    assert_eq!(called.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn listener_blocking_and_background_same_listener() {
    let called = Arc::new(AtomicU64::new(0));
    let called2 = called.clone();
    let cb: Arc<dyn Fn(&EventPayload<u32>) + Send + Sync> = Arc::new(move |_| {
        called2.fetch_add(1, Ordering::SeqCst);
    });

    let mut l = Listener::new(None, cb, Some(2));
    let handle1 = l.blocking_call(&Arc::new(42));
    let handle2 = l.background_call(&Arc::new(99));
    let handle3 = l.blocking_call(&Arc::new(123));
    if let Some(h) = handle1 {
        h.await.unwrap();
    }
    if let Some(h) = handle2 {
        h.await.unwrap();
    }
    assert!(handle3.is_none());
    assert_eq!(called.load(Ordering::SeqCst), 2);
}

#[test]
fn listener_at_limit_logic() {
    let cb: Arc<dyn Fn(&EventPayload<u8>) + Send + Sync> = Arc::new(|_| {});
    let mut l = Listener::new(None, cb.clone(), Some(1));
    assert!(!l.at_limit());
    l.call(&Arc::new(1));
    assert!(l.at_limit());

    let mut l2 = Listener::new(None, cb.clone(), Some(10));
    for _ in 0..=9 {
        assert!(!l2.at_limit());
        l2.call(&Arc::new(1));
    }
    assert!(l2.at_limit());
}

#[test]
fn listener_clone_and_eq() {
    let cb: Arc<dyn Fn(&EventPayload<&'static str>) + Send + Sync> = Arc::new(|_| {});

    let l1 = Listener::new(Some("tag".to_string()), cb.clone(), Some(2));
    let l2 = l1.clone();
    assert_eq!(l1, l2);

    // Different tag, not equal
    let l3 = Listener::new(Some("other".to_string()), cb.clone(), Some(2));
    assert_ne!(l1, l3);

    // Different callback, not equal
    let cb2: Arc<dyn Fn(&EventPayload<&'static str>) + Send + Sync> = Arc::new(|_| {});
    let l4 = Listener::new(Some("tag".to_string()), cb2, Some(2));
    assert_ne!(l1, l4);
}

#[test]
fn listener_default_and_debug() {
    let l: Listener<u32> = Listener::default();

    // Default has Some(1) lifetime, no tag
    assert_eq!(l.lifetime(), Some(1));
    assert_eq!(l.tag(), None);

    // Debug prints lifetime
    let dbg = format!("{:?}", l);
    println!("{}", dbg);
    assert!(dbg.contains("tag: None, lifetime: Some(1)"));
}

#[test]
fn listener_clone_from() {
    let cb1: Arc<dyn Fn(&EventPayload<u8>) + Send + Sync> = Arc::new(|_| {});
    let mut l1 = Listener::new(Some("a".to_string()), cb1, Some(2));

    let cb2: Arc<dyn Fn(&EventPayload<u8>) + Send + Sync> = Arc::new(|_| {});
    let l2 = Listener::new(Some("b".to_string()), cb2, Some(3));
    l1.clone_from(&l2);

    assert_eq!(l1.tag(), Some(&"b".to_string()));
    // Callback pointer equality
    assert!(Arc::ptr_eq(l1.callback(), l2.callback()));
    assert_eq!(l1.lifetime(), l2.lifetime());
}
