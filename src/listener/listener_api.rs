use crate::Debug;

use super::{ListenerMode, ListenerError};

pub trait ListenerApi<T: 'static, M: ListenerMode>: Clone + PartialEq + Debug {
    /// Create a new listener with an optional tag and lifetime.
    ///
    /// # Required Parameters
    /// - `tag: Option<impl Into<String>>` - Optional string-able identifier (useful for tracking or removing specific listeners).
    /// - `callback: M::Callback<T>` - The function to invoke when the listener is called (mode specific).
    /// - `lifetime: Option<usize>` - Call limit.
    ///   - Use `None` or `Some(0)` for unlimited
    ///   - `Some(n)` to stop after `n` calls.
    ///
    /// # Required Return
    /// - `Self` - A new `ListenerApi` instance with the specified metadata and callback.
    fn new(tag: Option<impl Into<String>>, callback: M::Callback<T>, lifetime: Option<usize>) -> Self;


    /// Get the tag associated with this listener, if set.
    ///
    /// # Required Return
    /// - `Option<&str>` - The tag name if it exists, or `None` if no tag was set.
    fn get_tag(&self) -> Option<&str>;

    /// Get the tag associated with this listener, if set.
    ///
    /// # Required Parameters
    /// - `tag: Option<impl Into<String>>` - The string-able tag name to set for this listener, or `None` to remove the tag.
    fn set_tag(&mut self, tag: Option<impl Into<String>>);


    /// Get the number of remaining calls for this listener.
    ///
    /// # Required Return
    /// - `Option<usize>` - The number of calls remaining before the listener is exhausted.
    ///   - `Some(n)` if the listener has n calls remaining.
    ///   - `None` if the listener is unlimited.
    fn get_lifetime(&self) -> Option<usize>;

    /// Set the number of remaining calls for this listener.
    ///
    /// # Required Parameters
    /// - `new_life: Option<usize>` - The number of calls the listener can take; `Some(0)` or `None` if the listener is unlimited.
    fn set_lifetime(&mut self, new_life: Option<usize>);

    /// Check if the listener has reached its call limit.
    ///
    /// # Required Return
    /// - `bool`
    ///   - `true` if the listener has reached its call limit (0 calls remaining).
    ///   - `false` if the listener is unlimited or has remaining calls.
    fn at_limit(&self) -> bool;


    /// Get the callback function for this listener.
    ///
    /// # Required Return
    /// - `M::Callback<T>` - A reference to the callback handle, which may be an `Rc` or `Arc` depending on the mode.
    fn get_callback(&self) -> M::Callback<T>;

    /// Set the callback function for this listener.
    ///
    /// # Required Parameters
    /// - `M::Callback<T>` - A reference to the callback handle, which may be an `Rc` or `Arc` depending on the mode.
    fn set_callback(&mut self, callback: M::Callback<T>);



    /// Call the callback synchronously (blocking).
    ///
    /// # Required Parameters
    /// - `payload: &M::Payload<T>` - The mode-specific payload reference to pass to the callback.
    ///
    /// # Required Return
    /// - `Result<(), ListenerError>` - Whether the call was successful or not.
    ///   - `Ok(())` if the call was successful
    ///   - `Err(ListenerError)` if the listener has reached its call limit and cannot be called.
    fn call(&mut self, payload: &M::Payload<T>) -> Result<(), ListenerError>;
}
