//! SharedMode EventMode tests

/// Tests for SharedCallback<T> creation and invocation
mod callback;

/// Tests for SharedPayload<T> creation across various types (u32, String, Vec<u8>, custom).
mod payload {
    use crate::SharedPayload;
    use crate::{Arc, String, Vec};

    #[test]
    fn test_u32_payload_creation() {
        let payload: SharedPayload<u32> = Arc::new(42u32);
        assert_eq!(*payload, 42);
    }

    #[test]
    fn test_bool_payload_creation() {
        let payload: SharedPayload<bool> = Arc::new(true);
        assert_eq!(*payload, true);

        let payload2 = Arc::new(false);
        assert_eq!(*payload2, false);
    }

    #[test]
    fn test_string_payload_creation() {
        let payload: SharedPayload<String> = Arc::new(String::from("hello"));
        assert_eq!(*payload, String::from("hello"));
    }

    #[test]
    fn test_vec_payload_creation() {
        let payload: SharedPayload<Vec<u8>> = Arc::new(Vec::from(b"test".as_ref()));
        assert_eq!(*payload, b"test");
    }

    #[test]
    fn test_struct_payload_creation() {
        #[derive(Debug, PartialEq)]
        struct CustomPayload {
            message: String,
            value: u32,
        }

        let payload: SharedPayload<CustomPayload> = Arc::new(CustomPayload {
            message: String::from("custom"),
            value: 100,
        });

        assert_eq!(payload.message, String::from("custom"));
        assert_eq!(payload.value, 100);
    }

    #[test]
    fn test_option_payload_creation() {
        let payload: SharedPayload<Option<u32>> = Arc::new(Some(42));
        assert_eq!(*payload, Some(42));

        let payload_none: SharedPayload<Option<u32>> = Arc::new(None);
        assert_eq!(*payload_none, None);
    }

    #[test]
    fn test_payload_sharing_multiple_references() {
        let payload: SharedPayload<u32> = Arc::new(42);
        let r1 = Arc::clone(&payload);
        let r2 = Arc::clone(&payload);
        let r3 = Arc::clone(&payload);

        assert_eq!(*r1, 42);
        assert_eq!(*r2, 42);
        assert_eq!(*r3, 42);
        assert_eq!(Arc::strong_count(&payload), 4);
    }

    #[test]
    fn test_payload_drop_decref() {
        let payload: SharedPayload<u32> = Arc::new(100);
        assert_eq!(Arc::strong_count(&payload), 1);

        {
            let _inner = Arc::clone(&payload);
            assert_eq!(Arc::strong_count(&payload), 2);
        }

        assert_eq!(Arc::strong_count(&payload), 1);
    }
}
