//! HTTP transport implementations.
//!
//! [`HttpTransport`] is the only seam between the shared client and an actual
//! HTTP backend. The native build uses `ReqwestTransport`; the WebAssembly
//! build uses `FetchTransport`, which calls `window.fetch`. Both consume and
//! produce the backend-neutral values from [`crate::http`].

use crate::error::EveryMapResult;
use crate::http::{HttpRequest, HttpResponse};
use async_trait::async_trait;
use std::sync::Arc;

/// Sends an [`HttpRequest`] and returns an [`HttpResponse`].
///
/// Implementations must return non-2xx responses as `Ok(HttpResponse)`; the
/// shared client maps them to [`crate::error::EveryMapError`] and decides
/// whether a retry applies. Only transport failures (connection errors,
/// blocked requests, timeouts) are returned as errors.
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Send one request.
    async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse>;
}

/// Create the transport for the current target.
///
/// Native builds get a `ReqwestTransport`; WebAssembly builds get a
/// `FetchTransport`.
pub fn default_transport() -> Arc<dyn HttpTransport> {
    platform::default_transport()
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use super::*;
    use crate::http::{HttpHeaders, Method, CONTENT_TYPE_HEADER};
    use reqwest::Method as ReqwestMethod;

    /// `reqwest`-backed transport for native targets.
    pub struct ReqwestTransport {
        client: reqwest::Client,
    }

    impl ReqwestTransport {
        /// Create a transport with a default `reqwest::Client`.
        pub fn new() -> Self {
            Self {
                client: reqwest::Client::new(),
            }
        }

        /// Create a transport around an already configured `reqwest::Client`.
        pub fn with_client(client: reqwest::Client) -> Self {
            Self { client }
        }
    }

    impl Default for ReqwestTransport {
        fn default() -> Self {
            Self::new()
        }
    }

    fn to_reqwest_method(method: Method) -> ReqwestMethod {
        match method {
            Method::Get => ReqwestMethod::GET,
            Method::Post => ReqwestMethod::POST,
            Method::Put => ReqwestMethod::PUT,
            Method::Patch => ReqwestMethod::PATCH,
            Method::Delete => ReqwestMethod::DELETE,
            Method::Head => ReqwestMethod::HEAD,
        }
    }

    #[async_trait]
    impl HttpTransport for ReqwestTransport {
        async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
            let mut builder = self.client.request(
                to_reqwest_method(request.method()),
                request.url_with_query(),
            );

            for (name, value) in request.headers() {
                builder = builder.header(name, value);
            }

            if let Some(body) = request.body() {
                builder = builder
                    .header(CONTENT_TYPE_HEADER, &body.content_type)
                    .body(body.bytes.clone());
            }

            if let Some(timeout) = request.timeout_value() {
                builder = builder.timeout(timeout);
            }

            let response = builder.send().await?;
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.as_str().to_string(), value.to_string()))
                })
                .collect::<Vec<_>>();
            let body = response.bytes().await?.to_vec();

            Ok(HttpResponse::new(status, HttpHeaders::new(headers), body))
        }
    }

    pub(super) fn default_transport() -> Arc<dyn HttpTransport> {
        Arc::new(ReqwestTransport::new())
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use super::*;
    use crate::error::EveryMapError;
    use crate::http::{HttpHeaders, CONTENT_TYPE_HEADER};
    use futures_channel::oneshot;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::{Headers, Request, RequestInit, RequestMode, Response};

    /// Header names the Fetch standard controls. A page cannot set them, and a
    /// browser that treats one as a custom header preflights the request —
    /// which a tiles endpoint without an `OPTIONS` route answers with 404. They
    /// are dropped before the browser request is built.
    const BROWSER_FORBIDDEN_HEADER_NAMES: [&str; 6] = [
        "user-agent",
        "cookie",
        "host",
        "origin",
        "referer",
        "connection",
    ];

    /// `window.fetch`-backed transport for WebAssembly targets.
    pub struct FetchTransport;

    impl FetchTransport {
        /// Create a fetch transport.
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for FetchTransport {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Convert a JavaScript failure into a transport error.
    fn js_error(error: JsValue) -> EveryMapError {
        let message = error
            .as_string()
            .or_else(|| {
                js_sys::Reflect::get(&error, &JsValue::from_str("message"))
                    .ok()?
                    .as_string()
            })
            .unwrap_or_else(|| "unknown JavaScript error".to_string());
        EveryMapError::TransportError(message)
    }

    /// Build a promise that resolves after `timeout_milliseconds`.
    fn timeout_promise(window: &web_sys::Window, timeout_milliseconds: i32) -> js_sys::Promise {
        js_sys::Promise::new(&mut |resolve, _reject| {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                timeout_milliseconds,
            );
        })
    }

    /// Collect all response headers from the fetch `Headers` object.
    fn collect_headers(headers: &Headers) -> EveryMapResult<Vec<(String, String)>> {
        let mut collected = Vec::new();
        let iterator = headers.entries();
        loop {
            let next = js_sys::Iterator::next(&iterator).map_err(js_error)?;
            if next.done() {
                break;
            }
            let pair: js_sys::Array = next.value().dyn_into().map_err(js_error)?;
            let name = pair.get(0).as_string().unwrap_or_default();
            let value = pair.get(1).as_string().unwrap_or_default();
            collected.push((name, value));
        }
        Ok(collected)
    }

    /// Execute the fetch. This future is intentionally `!Send`: it is spawned
    /// on the single-threaded executor by [`FetchTransport::send`].
    async fn fetch(request: HttpRequest) -> EveryMapResult<HttpResponse> {
        let init = RequestInit::new();
        init.set_method(request.method().as_str());
        init.set_mode(RequestMode::Cors);

        let headers = Headers::new().map_err(js_error)?;
        for (name, value) in request.headers() {
            if BROWSER_FORBIDDEN_HEADER_NAMES
                .iter()
                .any(|forbidden_name| name.eq_ignore_ascii_case(forbidden_name))
            {
                continue;
            }
            headers.set(name, value).map_err(js_error)?;
        }
        if let Some(body) = request.body() {
            headers
                .set(CONTENT_TYPE_HEADER, &body.content_type)
                .map_err(js_error)?;
            let array = js_sys::Uint8Array::from(body.bytes.as_slice());
            init.set_body_opt_u8_array(Some(&array));
        }
        init.set_headers_headers(&headers);

        let web_request =
            Request::new_with_str_and_init(&request.url_with_query(), &init).map_err(js_error)?;
        let window = web_sys::window().ok_or_else(|| {
            EveryMapError::TransportError("no browser window is available".to_string())
        })?;
        let fetch_promise = window.fetch_with_request(&web_request);

        let response_value = if let Some(timeout) = request.timeout_value() {
            let timeout_milliseconds = timeout.as_millis().min(i32::MAX as u128) as i32;
            let timeout_promise = timeout_promise(&window, timeout_milliseconds);
            let race = js_sys::Promise::race(
                js_sys::Array::of2(fetch_promise.as_ref(), timeout_promise.as_ref()).as_ref(),
            );
            JsFuture::from(race).await.map_err(js_error)?
        } else {
            JsFuture::from(fetch_promise).await.map_err(js_error)?
        };

        let response = response_value.dyn_into::<Response>().map_err(|_| {
            EveryMapError::TransportError(
                "request timed out before the browser returned a response".to_string(),
            )
        })?;
        let status = response.status();
        let headers = collect_headers(&response.headers())?;
        let body_value = JsFuture::from(response.array_buffer().map_err(js_error)?)
            .await
            .map_err(js_error)?;
        let body = js_sys::Uint8Array::new(&body_value).to_vec();

        Ok(HttpResponse::new(status, HttpHeaders::new(headers), body))
    }

    #[async_trait]
    impl HttpTransport for FetchTransport {
        async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
            let (sender, receiver) = oneshot::channel();
            spawn_local(async move {
                let _ = sender.send(fetch(request).await);
            });
            receiver.await.map_err(|_| {
                EveryMapError::TransportError("browser fetch task was cancelled".to_string())
            })?
        }
    }

    pub(super) fn default_transport() -> Arc<dyn HttpTransport> {
        Arc::new(FetchTransport::new())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use platform::ReqwestTransport;

#[cfg(target_arch = "wasm32")]
pub use platform::FetchTransport;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpHeaders, HttpResponse, Method};
    use std::sync::Arc;

    struct EchoTransport;

    #[async_trait]
    impl HttpTransport for EchoTransport {
        async fn send(&self, request: HttpRequest) -> EveryMapResult<HttpResponse> {
            Ok(HttpResponse::new(
                200,
                HttpHeaders::new(vec![(
                    "x-method".to_string(),
                    request.method().as_str().to_string(),
                )]),
                request.url_with_query().into_bytes(),
            ))
        }
    }

    #[tokio::test]
    async fn injected_transport_receives_the_request() {
        let transport: Arc<dyn HttpTransport> = Arc::new(EchoTransport);
        let response = transport
            .send(HttpRequest::get("https://example.com/a").query([("q", "b")]))
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.text(), "https://example.com/a?q=b");
        assert_eq!(
            response.headers().get("x-method"),
            Some(Method::GET.as_str())
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn reqwest_transport_reports_connection_failures() {
        let transport = ReqwestTransport::new();
        let error = transport
            .send(
                HttpRequest::get("http://127.0.0.1:1/")
                    .timeout(std::time::Duration::from_millis(200)),
            )
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::EveryMapError::ClientError(_)));
    }
}
