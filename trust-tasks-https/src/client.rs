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

use std::sync::Arc;
use std::time::Duration;

use reqwest::{header, Client, ClientBuilder, Url};
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;
use trust_tasks_rs::{
    erase_verifier, DynProofVerifier, ErrorResponse, Payload, ProofVerifier, TransportHandler,
    TrustTask, TypeUri,
};

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
    response_verifier: Option<Arc<dyn DynProofVerifier>>,
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

    /// Verify the `proof` on every response document.
    ///
    /// Off by default, and off is not the same as safe: with no verifier the
    /// client checks *nothing* cryptographic about what came back. The
    /// correlation checks [`HttpsClient::send`] performs — `threadId`,
    /// `type`, `issuer`, `recipient` — bind a response to its request, but
    /// they are all assertions by whoever answered the socket.
    ///
    /// Configuring a verifier expresses "I require signed responses": a
    /// proofless response is then rejected with
    /// [`ClientError::ResponseProofMissing`] rather than quietly accepted,
    /// because a downgrade that goes unnoticed is worth no more than no
    /// verification at all. Note the bundled [`HttpsServer`](crate::HttpsServer)
    /// does not sign its responses, so this is for peers that do.
    pub fn with_response_verifier<V>(mut self, verifier: V) -> Self
    where
        V: ProofVerifier + Send + Sync + 'static,
    {
        self.response_verifier = Some(erase_verifier(verifier));
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
            response_verifier: self.response_verifier,
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
    response_verifier: Option<Arc<dyn DynProofVerifier>>,
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
    /// 4. On HTTP 2xx, deserialises the body as `TrustTask<Resp>` **and
    ///    binds it to the request** (see below).
    /// 5. On non-2xx, deserialises the body as an [`ErrorResponse`],
    ///    checks its `inResponseTo.id` where present, and returns it via
    ///    [`ClientError::TrustTaskError`].
    ///
    /// # Response binding
    ///
    /// A 2xx body that merely *deserialises* as `TrustTask<Resp>` proves
    /// nothing: it could answer a different request, be a different task
    /// type whose payload happens to be shape-compatible, come from a
    /// party that is not the configured server, or be addressed to someone
    /// else. HTTP's request/response pairing is not a security property —
    /// a proxy, a connection-reuse bug, or anything else in the path can
    /// substitute one body for another. So four equalities are enforced
    /// before the typed response is handed back:
    ///
    /// | Member       | Must equal                                   | On failure |
    /// |--------------|----------------------------------------------|------------|
    /// | `threadId`   | the request's `threadId`, or its `id`        | [`ClientError::ResponseThreadMismatch`] |
    /// | `type`       | the request's `type` with `#response`        | [`ClientError::ResponseTypeMismatch`] |
    /// | `issuer`     | the configured `server_vid`                   | [`ClientError::ResponseIssuerMismatch`] |
    /// | `recipient`  | the configured `my_vid`                       | [`ClientError::ResponseRecipientMismatch`] |
    ///
    /// The `issuer` / `recipient` checks are skipped for a VID the caller
    /// did not configure — there is nothing to compare against. When
    /// [`HttpsClientBuilder::strip_redundant_in_band`] is set, an absent
    /// member is accepted (the client asked for it to be stripped and the
    /// responder echoed the absence); otherwise absence is a mismatch.
    ///
    /// On an error response, `inResponseTo.id` — where the responder
    /// populated it — must be the request's `id`, else
    /// [`ClientError::ErrorResponseMismatch`]. It is legitimately absent
    /// under `identityMismatch` (SPEC §8.1 omits it), so absence is not an
    /// error.
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
            // Parse untyped first: the proof (if any) is verified over the
            // JSON form, and the correlation checks below need no payload
            // types. Only then is the payload downcast to `Resp`.
            let untyped: TrustTask<Value> = serde_json::from_slice(&body)
                .map_err(|e| ClientError::ResponseDecode(e.to_string()))?;

            self.check_response_binding(&request, &untyped)?;
            self.verify_response_proof(&untyped).await?;

            let typed: TrustTask<Resp> = serde_json::from_slice(&body)
                .map_err(|e| ClientError::ResponseDecode(e.to_string()))?;
            Ok(typed)
        } else {
            // Try to parse as a trust-task-error document; fall back to
            // a generic transport error if the body isn't one.
            match serde_json::from_slice::<ErrorResponse>(&body) {
                Ok(error_doc) => {
                    // §8.2 `inResponseTo.id` names the document being
                    // reported on. Where the responder populated it, it must
                    // be *our* document — otherwise an error about some
                    // unrelated exchange would surface to this caller as the
                    // outcome of its own request. It is legitimately absent
                    // under `identityMismatch` (§8.1), so absence passes.
                    if let Some(reported) = error_doc
                        .payload
                        .in_response_to
                        .as_ref()
                        .and_then(|r| r.id.as_deref())
                    {
                        if reported != request.id {
                            return Err(ClientError::ErrorResponseMismatch {
                                expected: request.id.clone(),
                                actual: reported.to_string(),
                            });
                        }
                    }
                    Err(ClientError::TrustTaskError {
                        http_status: status.as_u16(),
                        error: Box::new(error_doc),
                    })
                }
                Err(_) => Err(ClientError::HttpStatus {
                    http_status: status.as_u16(),
                    body: String::from_utf8_lossy(&body).to_string(),
                }),
            }
        }
    }

    /// The four request/response equalities documented on [`Self::send`].
    fn check_response_binding<Req: Payload + serde::Serialize>(
        &self,
        request: &TrustTask<Req>,
        response: &TrustTask<Value>,
    ) -> Result<(), ClientError> {
        // 1. threadId — `respond_with` carries the request's threadId, or
        //    its id when the request opened the thread.
        let expected_thread = request
            .thread_id
            .clone()
            .unwrap_or_else(|| request.id.clone());
        if response.thread_id.as_deref() != Some(expected_thread.as_str()) {
            return Err(ClientError::ResponseThreadMismatch {
                expected: expected_thread,
                actual: response.thread_id.clone(),
            });
        }

        // 2. type — the `#response` variant of what we asked for (§4.4.1).
        let expected_type: TypeUri = request.type_uri.with_response();
        if response.type_uri != expected_type {
            return Err(ClientError::ResponseTypeMismatch {
                expected: expected_type.to_string(),
                actual: response.type_uri.to_string(),
            });
        }

        // 3/4. issuer and recipient — the response comes *from* the server
        //      we addressed and is *for* us. Skipped where the caller
        //      configured no VID to compare against.
        if let Some(expected) = self.server_vid.as_deref() {
            if !self.party_matches(response.issuer.as_deref(), expected) {
                return Err(ClientError::ResponseIssuerMismatch {
                    expected: expected.to_string(),
                    actual: response.issuer.clone(),
                });
            }
        }
        if let Some(expected) = self.my_vid.as_deref() {
            if !self.party_matches(response.recipient.as_deref(), expected) {
                return Err(ClientError::ResponseRecipientMismatch {
                    expected: expected.to_string(),
                    actual: response.recipient.clone(),
                });
            }
        }

        Ok(())
    }

    /// A party member matches when it is present and equal, or when it is
    /// absent and this client asked for redundant in-band members to be
    /// stripped (§9.2 item 1) — the responder then legitimately echoes the
    /// absence back.
    fn party_matches(&self, actual: Option<&str>, expected: &str) -> bool {
        match actual {
            Some(v) => v == expected,
            None => self.strip_redundant_in_band,
        }
    }

    /// Verify the response's `proof` when a response verifier is configured.
    /// A configured verifier makes a signed response mandatory — see
    /// [`HttpsClientBuilder::with_response_verifier`].
    async fn verify_response_proof(&self, response: &TrustTask<Value>) -> Result<(), ClientError> {
        let Some(verifier) = &self.response_verifier else {
            return Ok(());
        };
        if response.proof.is_none() {
            return Err(ClientError::ResponseProofMissing);
        }
        verifier
            .verify_json(response)
            .await
            .map_err(|e| ClientError::ResponseProofInvalid(e.to_string()))
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

    /// The response's `threadId` does not correlate with the request
    /// (SPEC §4.9). The response answers some other exchange.
    #[error("response threadId does not match the request: expected {expected}, got {actual:?}")]
    ResponseThreadMismatch {
        /// The request's `threadId`, or its `id` where the request opened
        /// the thread.
        expected: String,
        /// What the response carried, if anything.
        actual: Option<String>,
    },

    /// The response's `type` is not the `#response` variant of the
    /// request's `type` (SPEC §4.4.1).
    #[error("response type does not match the request: expected {expected}, got {actual}")]
    ResponseTypeMismatch {
        /// The request's Type URI with the `#response` fragment.
        expected: String,
        /// What the response carried.
        actual: String,
    },

    /// The response's `issuer` is not the configured `server_vid` — the
    /// answer did not come from the party this client addressed.
    #[error("response issuer is not the configured server: expected {expected}, got {actual:?}")]
    ResponseIssuerMismatch {
        /// The configured `server_vid`.
        expected: String,
        /// What the response carried, if anything.
        actual: Option<String>,
    },

    /// The response's `recipient` is not the configured `my_vid` — the
    /// answer is addressed to somebody else.
    #[error("response recipient is not this client: expected {expected}, got {actual:?}")]
    ResponseRecipientMismatch {
        /// The configured `my_vid`.
        expected: String,
        /// What the response carried, if anything.
        actual: Option<String>,
    },

    /// An error response named a different document in `inResponseTo.id`
    /// (SPEC §8.2) than the one this client sent.
    #[error("error response reports on a different document: expected {expected}, got {actual}")]
    ErrorResponseMismatch {
        /// The request's `id`.
        expected: String,
        /// The `inResponseTo.id` the error response carried.
        actual: String,
    },

    /// A response verifier is configured but the response carried no
    /// `proof`. See [`HttpsClientBuilder::with_response_verifier`].
    #[error("response carried no proof but this client requires signed responses")]
    ResponseProofMissing,

    /// The response's `proof` failed verification.
    #[error("response proof verification failed: {0}")]
    ResponseProofInvalid(String),
}
