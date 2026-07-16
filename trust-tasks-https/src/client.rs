//! Reqwest-based client for emitting Trust Task documents over the
//! HTTPS binding.
//!
//! ```rust,ignore
//! let client = HttpsClient::builder()
//!     .server_url("http://localhost:3000")
//!     .server_vid("did:web:server.example")
//!     .my_vid("did:web:client.example")
//!     .my_token("alice-token")
//!     .build()?;
//!
//! let req = TrustTask::for_payload("urn:uuid:...", AclGrantPayload { ... });
//! let resp: TrustTask<AclGrantResponse> = client.send(req).await?;
//! ```
//!
//! The client serialises the request, attaches the configured bearer
//! token, and POSTs to `<server_url>/trust-tasks`. The response body is
//! deserialised either as the typed `#response`-variant document (HTTP
//! 2xx) or as a `trust-task-error/0.1` document (non-2xx); both surface
//! to the caller as [`ClientError`] variants for ergonomic `?` chains.

use std::time::Duration;

use reqwest::{header, Client, ClientBuilder, Url};
use serde::de::DeserializeOwned;
use thiserror::Error;
use trust_tasks_rs::{ErrorResponse, Payload, TransportHandler, TrustTask};

use crate::handler::HttpsHandler;

/// Default end-to-end request timeout. A trust-task exchange is one small
/// JSON round trip; a peer that takes longer than this is down or wedged.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default connection-establishment timeout.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Builder for [`HttpsClient`].
#[derive(Default)]
pub struct HttpsClientBuilder {
    server_url: Option<String>,
    server_vid: Option<String>,
    my_vid: Option<String>,
    my_token: Option<String>,
    strip_redundant_in_band: bool,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
}

impl HttpsClientBuilder {
    /// Base URL of the server, e.g. `"http://localhost:3000"`. The client
    /// will POST to `<server_url>/trust-tasks`.
    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = Some(url.into());
        self
    }

    /// VID of the server, used as the in-band `recipient` on outbound
    /// documents and cross-checked with what the transport identifies as
    /// the peer.
    pub fn server_vid(mut self, vid: impl Into<String>) -> Self {
        self.server_vid = Some(vid.into());
        self
    }

    /// VID of this client, used as the in-band `issuer`.
    pub fn my_vid(mut self, vid: impl Into<String>) -> Self {
        self.my_vid = Some(vid.into());
        self
    }

    /// Bearer token sent in `Authorization: Bearer <token>`. The server
    /// resolves the token to a VID and uses that as the transport-
    /// authenticated sender for §4.8.1 precedence.
    pub fn my_token(mut self, token: impl Into<String>) -> Self {
        self.my_token = Some(token.into());
        self
    }

    /// If `true`, the client strips in-band `issuer`/`recipient` when
    /// they match `my_vid`/`server_vid` respectively (SPEC §9.2 item 1).
    /// Defaults to `false`: the safe behavior is to leave the in-band
    /// members in place so the document remains self-contained at rest.
    pub fn strip_redundant_in_band(mut self, strip: bool) -> Self {
        self.strip_redundant_in_band = strip;
        self
    }

    /// End-to-end request timeout. Defaults to [`DEFAULT_TIMEOUT`].
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Connection-establishment timeout. Defaults to
    /// [`DEFAULT_CONNECT_TIMEOUT`].
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Build the [`HttpsClient`] or return a configuration error.
    pub fn build(self) -> Result<HttpsClient, ClientError> {
        let server_url = self
            .server_url
            .ok_or_else(|| ClientError::Config("server_url is required".into()))?;
        let base: Url = format!("{}/trust-tasks", server_url.trim_end_matches('/'))
            .parse()
            .map_err(|e| ClientError::Config(format!("server_url is not a valid URL: {e}")))?;

        // Finite timeouts always: an unresponsive peer must surface as an
        // error, never hang the caller. Callers with a genuinely long-running
        // exchange can raise the values explicitly.
        let http = ClientBuilder::new()
            .timeout(self.timeout.unwrap_or(DEFAULT_TIMEOUT))
            .connect_timeout(self.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT))
            .build()
            .map_err(|e| ClientError::Config(e.to_string()))?;

        Ok(HttpsClient {
            http,
            endpoint: base,
            server_vid: self.server_vid,
            my_vid: self.my_vid,
            my_token: self.my_token,
            strip_redundant_in_band: self.strip_redundant_in_band,
        })
    }
}

/// HTTPS Trust Tasks client.
pub struct HttpsClient {
    http: Client,
    endpoint: Url,
    server_vid: Option<String>,
    my_vid: Option<String>,
    my_token: Option<String>,
    strip_redundant_in_band: bool,
}

impl HttpsClient {
    /// Start a new builder.
    pub fn builder() -> HttpsClientBuilder {
        HttpsClientBuilder::default()
    }

    /// Send a typed Trust Task request to the configured server and
    /// receive a typed response.
    ///
    /// The client:
    ///
    /// 1. Populates `issuer` / `recipient` on the outbound document from
    ///    the configured VIDs if they are not already set.
    /// 2. Optionally strips redundant in-band members per
    ///    [`HttpsClientBuilder::strip_redundant_in_band`].
    /// 3. POSTs the JSON body with `Authorization: Bearer <my_token>` if
    ///    a token is configured.
    /// 4. On HTTP 2xx, deserialises the body as `TrustTask<Resp>`.
    /// 5. On non-2xx, deserialises the body as an [`ErrorResponse`] and
    ///    returns it via [`ClientError::TrustTaskError`].
    pub async fn send<Req, Resp>(
        &self,
        mut request: TrustTask<Req>,
    ) -> Result<TrustTask<Resp>, ClientError>
    where
        Req: Payload + serde::Serialize,
        Resp: Payload + DeserializeOwned,
    {
        // Fill in identity defaults if the caller didn't already set them.
        if request.issuer.is_none() {
            request.issuer = self.my_vid.clone();
        }
        if request.recipient.is_none() {
            request.recipient = self.server_vid.clone();
        }
        if request.issued_at.is_none() {
            request.issued_at = Some(chrono::Utc::now());
        }

        // Apply binding-specific outbound prep.
        if self.strip_redundant_in_band {
            HttpsHandler::new(self.my_vid.clone(), self.server_vid.clone())
                .prepare_outbound(&mut request);
        }

        // Build the HTTP request.
        let mut req = self
            .http
            .post(self.endpoint.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request);
        if let Some(token) = &self.my_token {
            req = req.bearer_auth(token);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.bytes().await?;

        if status.is_success() {
            let typed: TrustTask<Resp> = serde_json::from_slice(&body)
                .map_err(|e| ClientError::ResponseDecode(e.to_string()))?;
            Ok(typed)
        } else {
            // Try to parse as a trust-task-error/0.1 document; fall back to
            // a generic transport error if the body isn't one.
            match serde_json::from_slice::<ErrorResponse>(&body) {
                Ok(error_doc) => Err(ClientError::TrustTaskError {
                    http_status: status.as_u16(),
                    error: Box::new(error_doc),
                }),
                Err(_) => Err(ClientError::HttpStatus {
                    http_status: status.as_u16(),
                    body: String::from_utf8_lossy(&body).to_string(),
                }),
            }
        }
    }
}

/// Errors raised by [`HttpsClient::send`].
#[derive(Debug, Error)]
pub enum ClientError {
    /// The client was constructed with missing or invalid configuration.
    #[error("client configuration error: {0}")]
    Config(String),

    /// The HTTP request itself failed (DNS, connect, TLS, etc.).
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The server returned a `trust-task-error/0.1` response document.
    /// `http_status` is the response's HTTP status code.
    #[error("server returned trust-task-error/0.1 (HTTP {http_status}): {error}")]
    TrustTaskError {
        /// The HTTP status code accompanying the error response.
        http_status: u16,
        /// The parsed error-response document.
        error: Box<ErrorResponse>,
    },

    /// Non-2xx HTTP response that did not parse as a `trust-task-error/0.1`
    /// document (e.g. the server replied with plain text or an empty body).
    #[error("non-2xx HTTP response ({http_status}) with non-Trust-Task body: {body}")]
    HttpStatus {
        /// The HTTP status code.
        http_status: u16,
        /// The raw response body, lossy-decoded as UTF-8.
        body: String,
    },

    /// 2xx response whose body did not decode as the expected
    /// `TrustTask<Resp>`.
    #[error("response body did not match expected type: {0}")]
    ResponseDecode(String),
}
