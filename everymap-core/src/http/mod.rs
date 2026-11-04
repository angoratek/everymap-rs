//! Backend-neutral HTTP request, response, and transport types.
//!
//! Provider code builds an [`HttpRequest`] and the shared client sends it
//! through an [`HttpTransport`]. Native builds use `reqwest`; WebAssembly
//! builds use `window.fetch`. Neither type names a backend.

mod request;
mod response;
mod transport;

pub use request::{HttpBody, HttpRequest, Method, QueryPair, CONTENT_TYPE_HEADER};
pub use response::{HttpHeaders, HttpResponse};
pub use transport::{default_transport, HttpTransport};

#[cfg(not(target_arch = "wasm32"))]
pub use transport::ReqwestTransport;

#[cfg(target_arch = "wasm32")]
pub use transport::FetchTransport;
