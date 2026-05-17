---
slug: https
version: "0.1"
title: HTTPS transport binding
summary: Carries Trust Task documents as JSON over HTTP/1.1 POST to a single endpoint; transport-authenticated sender identity comes from a bearer-token mapping to a VID.
status: draft
targetFrameworkVersion: "0.1"
bindingURI: https://trusttasks.org/binding/https/0.1
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged over HTTP/1.1. A producer sends a *Trust Task document* as the JSON body of an HTTP `POST` to a single well-known path; the server runs the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) consumer pipeline and either returns a typed `#response`-variant document or a `trust-task-error/0.1` error response. Transport-authenticated sender identity is conveyed via HTTP `Authorization` headers; the binding does not constrain the token format, but does define how the bearer-mapped *Verifiable Identifier* feeds the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

The binding is named **HTTPS** because every framework *Type URI* uses `https` ([SPEC §6.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#61-type-uri)) and a production deployment **MUST** terminate TLS in front of the receiver. The wire mechanics described here are HTTP/1.1; whether TLS is terminated by a reverse proxy or natively at the server is a deployment concern outside the scope of this binding.

## Status of This Document

`0.1` draft. Tracks `SPEC.md 0.1`. The binding is implemented by [`trust-tasks-https`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-https) — a typed [`HttpsClient`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/client.rs) (reqwest) and an axum-based [`HttpsServer`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/server.rs).

## 1. Binding URI

| Resource           | URI                                                |
|--------------------|----------------------------------------------------|
| Binding identifier | `https://trusttasks.org/binding/https/0.1`         |

The binding URI does not appear on the wire — unlike DIDComm, HTTPS has no envelope `type` field. The URI exists solely as the stable identifier this binding is referred to by elsewhere in the framework and in registries.

## 2. Document carriage

A producer **MUST** send a *Trust Task document* as follows:

| HTTP element            | Value                                                                                    |
|-------------------------|------------------------------------------------------------------------------------------|
| Method                  | `POST`                                                                                   |
| Path                    | `/trust-tasks`                                                                           |
| `Content-Type` request  | `application/json`                                                                       |
| Request body            | The *Trust Task document* serialised as a JSON object (UTF-8, no BOM).                   |
| `Accept` request        | `application/json` (recommended).                                                        |
| `Authorization` request | `Bearer <token>` where `<token>` identifies the sender (see [§3. Identity mapping](#3-identity-mapping)). |

A conforming server **MUST**:

1. Accept `POST /trust-tasks` and reject every other method/path combination with the appropriate HTTP status (`405 Method Not Allowed` / `404 Not Found`). These responses are not framework error documents.
2. Read `Authorization` (if present) and map the bearer token to a *Verifiable Identifier* via a deployment-defined mechanism (an in-process map, a JWT verifier, a database lookup, …). A request with no `Authorization` header is treated as having no transport-authenticated sender.
3. Parse the request body as JSON, then as a `TrustTask<P>` document for some payload type `P` selected by the document's `type` member.
4. Apply the framework [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) consumer pipeline — `resolve_parties` per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), `validate_basic`, `enforce_audience_binding`, dispatch by canonical *Type URI* per [§4.4.1 item 1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#441-request-and-response-variants), then the registered handler.
5. Return either a `#response`-variant *Trust Task document* (success) or a `trust-task-error/0.1` document (rejection) as the response body — both in JSON, both with `Content-Type: application/json`.

This binding does not define a streaming or multi-message variant. One request per HTTP exchange, one response.

## 3. Identity mapping

The mapping into the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence is:

| Framework concept                            | HTTPS-derived value                                                                                                  |
|----------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| *Transport-authenticated sender*             | The VID the server maps the bearer token to. Absent for requests with no `Authorization` header or with an unrecognised token. |
| *Transport-authenticated recipient*          | The server's own configured `local_vid`. This is a server-side configuration value, not anything carried in the HTTP request. |
| Producer's *in-band* `issuer` (when set)     | Cross-checked against the transport-authenticated sender per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identity_mismatch`. |
| Producer's *in-band* `recipient` (when set)  | Cross-checked against the transport-authenticated recipient per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identity_mismatch`. |

The mapping between bearer token and VID is **deployment-defined**. A demo or test deployment **MAY** use a static `HashMap<token, VID>`; a production deployment **SHOULD** verify a JWT against an issuer-controlled JWKS or otherwise bind tokens to verifiable identifiers under a controlled trust framework. The binding makes no claim about token-revocation, audience-restriction, or replay protection beyond what the chosen mechanism provides.

When applying the §8.1 error-response routing rule under `identity_mismatch`, the server **MUST** route its `trust-task-error/0.1` response to the bearer-authenticated sender it actually authenticated, and **MUST NOT** carry the contested in-band `issuer` in the response's `recipient` member.

## 4. Status mapping

The server **SHOULD** map the *Trust Task document* response to an HTTP status as follows. The framework error code remains authoritative; the HTTP status is informative (intermediaries and end-user diagnostics).

| Outcome                                                  | HTTP status                           |
|----------------------------------------------------------|---------------------------------------|
| Success (a `#response`-variant document)                 | `200 OK`                              |
| `malformed_request`                                      | `400 Bad Request`                     |
| `unauthenticated`                                        | `401 Unauthorized`                    |
| `unauthorized` (a.k.a. `permission_denied`)              | `403 Forbidden`                       |
| `unsupported_type` / `unsupported_version`               | `422 Unprocessable Entity`            |
| `expired`                                                | `422 Unprocessable Entity`            |
| `proof_required` / `proof_invalid` / `identity_mismatch` | `422 Unprocessable Entity`            |
| `task_failed`                                            | `422 Unprocessable Entity`            |
| Internal server error (transport-level, no error doc)    | `500 Internal Server Error`           |
| Server temporarily unavailable                           | `503 Service Unavailable`             |

In every case where the body carries a Trust Task document — success or `trust-task-error/0.1` — the `Content-Type` **MUST** be `application/json`.

A client receiving a non-2xx response with `Content-Type: application/json` **MUST** attempt to deserialise the body as a `trust-task-error/0.1` document before falling back to transport-level error handling. A client receiving a non-2xx response with any other `Content-Type` treats the response as an untyped transport-level failure.

## 5. Discovery wiring

A server **MAY** advertise the set of *Type URIs* it dispatches by registering a handler for `https://trusttasks.org/spec/trust-task-discovery/0.1` ([SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-discovery-and-capability-negotiation)). Discovery requests **MUST** use the same `POST /trust-tasks` endpoint as every other request; no separate path is defined.

## 6. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the endpoint path, method, content-type expectations, and identity-mapping shape are preserved, and only additive header conventions, additional status-mapping rows, or stricter rules may be introduced. Breaking changes — a different endpoint path, a different identity-mapping mechanism, an incompatible status mapping — require a `MAJOR` bump and a new binding URI.

## 7. References

- [RFC 7235 — Hypertext Transfer Protocol (HTTP/1.1): Authentication](https://datatracker.ietf.org/doc/html/rfc7235).
- [RFC 6750 — The OAuth 2.0 Authorization Framework: Bearer Token Usage](https://datatracker.ietf.org/doc/html/rfc6750).
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §8, §9, §11.
