use crate::{Cell, Rc, RefCell, String, Vec};
use crate::{EventMode, LocalCallback, LocalMode, LocalPayload};

#[test]
fn test_callback_creation_with_u32() {
    let callback: LocalCallback<u32> = Rc::new(|payload: &LocalPayload<u32>| {
        assert_eq!(payload.as_ref(), &42);
    });

    let payload: LocalPayload<u32> = Rc::new(42);
    LocalMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_creation_with_string() {
    let callback: LocalCallback<String> = Rc::new(|payload: &LocalPayload<String>| {
        assert_eq!(payload.as_ref(), &String::from("test"));
    });

    let payload: LocalPayload<String> = Rc::new(String::from("test"));
    LocalMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_creation_with_vec() {
    let callback: LocalCallback<Vec<u8>> = Rc::new(|payload: &LocalPayload<Vec<u8>>| {
        assert_eq!(payload.as_ref(), b"test");
    });

    let payload: LocalPayload<Vec<u8>> = Rc::new(Vec::from(b"test".as_ref()));
    LocalMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_with_custom_struct() {
    #[derive(Debug, PartialEq)]
    struct EventData {
        value: u32,
    }

    let callback: LocalCallback<EventData> = Rc::new(|payload: &LocalPayload<EventData>| {
        assert_eq!(payload.value, 123);
    });

    let payload: LocalPayload<EventData> = Rc::new(EventData { value: 123 });
    LocalMode::invoke_callback(&callback, &payload);
}

#[test]
fn test_callback_mutable_state_via_cell() {
    use core::cell::Cell;

    let counter = Rc::new(Cell::new(0));

    let counter2 = Rc::clone(&counter);
    let callback: LocalCallback<u32> = Rc::new(move |payload: &LocalPayload<u32>| {
        assert_eq!(payload.as_ref(), &1);
        counter2.set(counter2.get() + 1);
    });

    let payload: LocalPayload<u32> = LocalPayload::new(1);
    LocalMode::invoke_callback(&callback, &payload);
    assert_eq!(counter.get(), 1);

    LocalMode::invoke_callback(&callback, &payload);
    assert_eq!(counter.get(), 2);
}

#[test]
fn test_callback_mutable_state_via_refcell() {
    let data = Rc::new(RefCell::new(Vec::new()));
    let data2 = data.clone();

    let callback: LocalCallback<u32> = Rc::new(move |payload: &LocalPayload<u32>| {
        data2.borrow_mut().push(*payload.as_ref());
    });

    LocalMode::invoke_callback(&callback, &LocalPayload::new(1));
    LocalMode::invoke_callback(&callback, &LocalPayload::new(2));
    LocalMode::invoke_callback(&callback, &LocalPayload::new(3));

    assert_eq!(*data.borrow(), vec![1, 2, 3]);
}

#[test]
fn test_callback_not_invoked_if_payload_doesnt_match() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let callback: LocalCallback<u32> = Rc::new(|payload: &LocalPayload<u32>| {
        assert_eq!(payload.as_ref(), &99);
    });

    let wrong_payload: LocalPayload<u32> = LocalPayload::new(67);

    let result = catch_unwind(AssertUnwindSafe(|| {
        LocalMode::invoke_callback(&callback, &wrong_payload);
    }));

    assert!(result.is_err());
}

#[test]
fn test_callback_clone_preserves_reference() {
    let data = Rc::new(Cell::new(0));
    let data2 = Rc::clone(&data);

    let callback: LocalCallback<u32> = Rc::new(move |_| {
        data2.set(data2.get() + 1);
    });

    let callback2 = Rc::clone(&callback);
    let payload: LocalPayload<u32> = LocalPayload::new(0);

    LocalMode::invoke_callback(&callback, &payload);
    assert_eq!(data.get(), 1);

    LocalMode::invoke_callback(&callback2, &payload);
    assert_eq!(data.get(), 2);
}
