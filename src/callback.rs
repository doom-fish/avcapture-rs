use core::ffi::c_void;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub(crate) struct ArcContext<T>(Arc<T>);

impl<T> ArcContext<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(crate) fn as_ptr(&self) -> *mut c_void {
        Arc::as_ptr(&self.0).cast_mut().cast()
    }

    pub(crate) fn into_raw(self) -> *mut c_void {
        Arc::into_raw(self.0).cast_mut().cast()
    }

    pub(crate) unsafe fn get<'a>(ptr: *mut c_void) -> Option<&'a T> {
        ptr.cast::<T>().as_ref()
    }

    pub(crate) unsafe fn retain(ptr: *mut c_void) {
        if !ptr.is_null() {
            Arc::increment_strong_count(ptr.cast::<T>());
        }
    }

    pub(crate) unsafe fn release(ptr: *mut c_void) {
        if !ptr.is_null() {
            Arc::decrement_strong_count(ptr.cast::<T>());
        }
    }
}

struct SerializedCallbackInner<T> {
    callback: Option<Box<dyn FnMut(T) + Send + 'static>>,
    queued: VecDeque<T>,
    dispatching: bool,
}

pub(crate) struct SerializedCallback<T> {
    inner: Mutex<SerializedCallbackInner<T>>,
}

impl<T> SerializedCallback<T> {
    pub(crate) fn new<F>(callback: F) -> Self
    where
        F: FnMut(T) + Send + 'static,
    {
        Self {
            inner: Mutex::new(SerializedCallbackInner {
                callback: Some(Box::new(callback)),
                queued: VecDeque::new(),
                dispatching: false,
            }),
        }
    }

    pub(crate) fn dispatch(&self, source: &'static str, event: T) {
        let callback = {
            let mut inner = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inner.queued.push_back(event);
            if inner.dispatching {
                return;
            }
            inner.dispatching = true;
            inner
                .callback
                .take()
                .expect("serialized callback missing while idle")
        };
        let mut callback = Some(callback);

        loop {
            let next = {
                let mut inner = self
                    .inner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                match inner.queued.pop_front() {
                    Some(event) => Some(event),
                    None => {
                        inner.callback = callback.take();
                        inner.dispatching = false;
                        None
                    }
                }
            };

            let Some(event) = next else {
                return;
            };
            doom_fish_utils::panic_safe::catch_user_panic(source, || {
                callback
                    .as_mut()
                    .expect("serialized callback missing while dispatching")(event);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ArcContext, SerializedCallback};
    use std::sync::{Arc, Mutex, Weak};

    #[test]
    fn nested_dispatch_is_queued_without_recursive_callback_aliasing() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let callback_holder = Arc::new(Mutex::new(Weak::<SerializedCallback<u32>>::new()));
        let events_for_callback = Arc::clone(&events);
        let holder_for_callback = Arc::clone(&callback_holder);
        let callback = Arc::new(SerializedCallback::new(move |event| {
            events_for_callback
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(event);
            if event == 1 {
                holder_for_callback
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .upgrade()
                    .expect("callback should remain alive")
                    .dispatch("nested_dispatch", 2);
            }
        }));
        *callback_holder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(&callback);

        callback.dispatch("nested_dispatch", 1);

        assert_eq!(
            *events
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![1, 2]
        );
    }

    #[test]
    fn transferred_arc_context_lives_until_native_owner_release() {
        let value = Arc::new(());
        let weak = Arc::downgrade(&value);
        let raw = ArcContext::new(value).into_raw();
        assert!(weak.upgrade().is_some());

        unsafe { ArcContext::<Arc<()>>::release(raw) };

        assert!(weak.upgrade().is_none());
    }
}
