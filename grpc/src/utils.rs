pub(crate) use imp::*;

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::time::Duration;
    use tokio::time::interval;
    pub(crate) struct Interval(tokio::time::Interval);

    impl Interval {
        pub(crate) async fn new(dur: Duration) -> Self {
            let mut inner = interval(dur);

            // In Tokio the first tick returns immediately, so we
            // consume to it to create an identical cross-platform
            // behavior.
            inner.tick().await;

            Interval(inner)
        }

        pub(crate) async fn tick(&mut self) {
            self.0.tick().await;
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use futures::StreamExt;
    use gloo_timers::future::IntervalStream;
    use send_wrapper::SendWrapper;
    use std::time::Duration;

    pub(crate) struct Interval(SendWrapper<IntervalStream>);

    impl Interval {
        pub(crate) async fn new(dur: Duration) -> Self {
            // If duration was less than a millisecond, then make
            // it 1 millisecond.
            let millis = u32::try_from(dur.as_millis().max(1)).unwrap_or(u32::MAX);

            Interval(SendWrapper::new(IntervalStream::new(millis)))
        }

        pub(crate) async fn tick(&mut self) {
            self.0.next().await;
        }
    }
}

/// Create a new javascript `Object` with given properties
#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
macro_rules! make_object {
    ($( $prop:expr => $val:expr ),+) => {{
        let object = ::js_sys::Object::new();
        $(
            ::js_sys::Reflect::set(
                &object,
                &$prop.into(),
                &$val,
            )
            .expect("setting field on new object");
        )+
        object
    }};
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
pub(crate) use make_object;
