//! Cross-target asynchronous sleep used by rate limiting and retry backoff.

use std::time::Duration;

/// Wait for the given duration.
///
/// Native builds use the `tokio` timer. WebAssembly builds use a `setTimeout`
/// promise. The WebAssembly version spawns the promise on the local executor
/// and awaits a channel, so the returned future stays `Send` and can be used
/// from the crate's `Send` async traits.
#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    use futures_channel::oneshot;
    use wasm_bindgen_futures::{spawn_local, JsFuture};

    let milliseconds = duration.as_millis().min(i32::MAX as u128) as i32;
    let (sender, receiver) = oneshot::channel();
    spawn_local(async move {
        if let Some(window) = web_sys::window() {
            let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                let _ = window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, milliseconds);
            });
            let _ = JsFuture::from(promise).await;
        }
        let _ = sender.send(());
    });
    let _ = receiver.await;
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn sleeps_at_least_the_requested_duration() {
        let start = Instant::now();
        sleep(Duration::from_millis(10)).await;
        assert!(start.elapsed() >= Duration::from_millis(10));
    }
}
