use crate::{Arc, Mutex, String, Vec};
use crate::{EventMode, SharedCallback, SharedMode, SharedPayload};

#[test]
fn test_callback_creation_with_u32() {
    let callback: SharedCallback<u32> = Arc::new(|payload: &SharedPayload<u32>| {
        assert_eq!(payload.as_ref(), &42);
    });

    let payload = SharedPayload::new(42);
    SharedMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_creation_with_string() {
    let callback: SharedCallback<String> = Arc::new(|payload: &SharedPayload<String>| {
        assert_eq!(payload.as_ref(), &String::from("test"));
    });

    let payload = SharedPayload::new(String::from("test"));
    SharedMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_creation_with_vec() {
    let callback: SharedCallback<Vec<u8>> = Arc::new(|payload: &SharedPayload<Vec<u8>>| {
        assert_eq!(payload.as_ref(), b"test");
    });

    let payload = SharedPayload::new(Vec::from(b"test".as_ref()));
    SharedMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_with_custom_struct() {
    #[derive(Debug, PartialEq)]
    struct EventData {
        value: u32,
    }

    let callback: SharedCallback<EventData> = Arc::new(|payload: &SharedPayload<EventData>| {
        assert_eq!(payload.value, 123);
    });

    let payload = SharedPayload::new(EventData { value: 123 });
    SharedMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_mutable_state_via_mutex() {
    let counter = Arc::new(Mutex::new(0));
    let counter2 = Arc::clone(&counter);

    let callback: SharedCallback<u32> = Arc::new(move |payload: &SharedPayload<u32>| {
        assert_eq!(payload.as_ref(), &1);
        {
            let mut count = counter2.lock();
            *count += 1;
        }
    });

    let payload = SharedPayload::new(1);
    SharedMode::invoke_callback(&callback, &payload);
    {
        let count = counter.lock();
        assert_eq!(*count, 1);
    }

    SharedMode::invoke_callback(&callback, &payload);
    {
        let count = counter.lock();
        assert_eq!(*count, 2);
    }
}

#[test]
fn test_callback_not_invoked_if_payload_doesnt_match() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let callback: SharedCallback<u32> = Arc::new(|payload: &SharedPayload<u32>| {
        assert_eq!(payload.as_ref(), &99);
    });

    let wrong_payload = SharedPayload::new(67u32);

    let result = catch_unwind(AssertUnwindSafe(|| {
        SharedMode::invoke_callback(&callback, &wrong_payload);
    }));

    assert!(result.is_err());
}

#[test]
fn test_callback_clone_preserves_reference() {
    let data = Arc::new(Mutex::new(0));
    let data2 = Arc::clone(&data);

    let callback: SharedCallback<u32> = Arc::new(move |_: &SharedPayload<u32>| {
        let mut count = data2.lock();
        *count += 1;
    });

    let callback2 = Arc::clone(&callback);
    let payload = SharedPayload::new(0);

    SharedMode::invoke_callback(&callback, &payload);
    {
        let count = data.lock();
        assert_eq!(*count, 1);
    }

    SharedMode::invoke_callback(&callback2, &payload);
    {
        let count = data.lock();
        assert_eq!(*count, 2);
    }
}
