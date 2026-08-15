---
slug: https
version: "0.1"
title: HTTPS transport binding
summary: Carries Trust Task documents as JSON over HTTP/1.1 POST to a single endpoint; transport-authenticated sender identity comes from a bearer-token mapping to a VID.
status: draft
targetFrameworkVersion: "0.2"
bindingURI: https://trusttasks.org/binding/https/0.1
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged over HTTP/1.1. A producer sends a *Trust Task document* as the JSON body of an HTTP `POST` to a single well-known path; the server runs the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) consumer pipeline and either returns a typed `#response`-variant document or a `trust-task-error/0.2` error response. Transport-authenticated sender identity is conveyed via HTTP `Authorization` headers; the binding does not constrain the token format, but does define how the bearer-mapped *Verifiable Identifier* feeds the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

The binding is named **HTTPS** because every framework *Type URI* uses `https` ([SPEC §6.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#61-type-uri)) and a production deployment **MUST** terminate TLS in front of the receiver. The wire mechanics described here are HTTP/1.1; whether TLS is terminated by a reverse proxy or natively at the server is a deployment concern outside the scope of this binding.

## Status of This Document

`0.1` draft. Targets **framework `0.2`** and uses the framework's lowerCamelCase error-code vocabulary ([SPEC §4.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#410-naming-conventions), [§8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)). The binding is implemented by [`trust-tasks-https`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-https) — a typed [`HttpsClient`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/client.rs) (reqwest) and an axum-based [`HttpsServer`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/server.rs).

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
5. Return either a `#response`-variant *Trust Task document* (success) or a `trust-task-error/0.2` document (rejection) as the response body — both in JSON, both with `Content-Type: application/json`.

This binding does not define a streaming or multi-message variant. One request per HTTP exchange, one response.

## 3. Identity mapping

The mapping into the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence is:

| Framework concept                            | HTTPS-derived value                                                                                                  |
|----------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| *Transport-authenticated sender*             | The VID the server maps the bearer token to. Absent for requests with no `Authorization` header or with an unrecognised token. |
| *Transport-authenticated recipient*          | The server's own configured `local_vid`. This is a server-side configuration value, not anything carried in the HTTP request. |
| Producer's *in-band* `issuer` (when set)     | Cross-checked against the transport-authenticated sender per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |
| Producer's *in-band* `recipient` (when set)  | Cross-checked against the transport-authenticated recipient per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |

The mapping between bearer token and VID is **deployment-defined**. A demo or test deployment **MAY** use a static `HashMap<token, VID>`; a production deployment **SHOULD** verify a JWT against an issuer-controlled JWKS or otherwise bind tokens to verifiable identifiers under a controlled trust framework. The binding makes no claim about token-revocation, audience-restriction, or replay protection beyond what the chosen mechanism provides.

When applying the §8.1 error-response routing rule under `identityMismatch`, the server **MUST** route its `trust-task-error/0.2` response to the bearer-authenticated sender it actually authenticated, and **MUST NOT** carry the contested in-band `issuer` in the response's `recipient` member.

## 4. Status mapping

The server **SHOULD** map the *Trust Task document* response to an HTTP status as follows. The framework error code remains authoritative; the HTTP status is informative (intermediaries and end-user diagnostics).

| Outcome                                                  | HTTP status                           |
|----------------------------------------------------------|---------------------------------------|
| Success (a `#response`-variant document)                 | `200 OK`                              |
| `malformedRequest`                                       | `400 Bad Request`                     |
| Missing / invalid `Authorization` (transport-level, no framework error doc) | `401 Unauthorized` |
| `permissionDenied`                                       | `403 Forbidden`                       |
| `unsupportedType` / `unsupportedVersion`                 | `422 Unprocessable Entity`            |
| `expired`                                                | `422 Unprocessable Entity`            |
| `proofRequired` / `proofInvalid` / `identityMismatch`    | `422 Unprocessable Entity`            |
| `wrongRecipient`                                         | `422 Unprocessable Entity`            |
| `taskFailed`                                             | `422 Unprocessable Entity`            |
| `unavailable`                                            | `503 Service Unavailable`             |
| `internalError`                                          | `500 Internal Server Error`           |
| Internal server error (transport-level, no error doc)    | `500 Internal Server Error`           |

In every case where the body carries a Trust Task document — success or `trust-task-error/0.2` — the `Content-Type` **MUST** be `application/json`.

A client receiving a non-2xx response with `Content-Type: application/json` **MUST** attempt to deserialise the body as a `trust-task-error/0.2` document before falling back to transport-level error handling. A client receiving a non-2xx response with any other `Content-Type` treats the response as an untyped transport-level failure.

## 5. Transport security profile

Required by [SPEC §9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies), because this binding populates `issuer` and `recipient` from transport context ([§3](#3-identity-mapping)).

**This binding does not permit `proof` to be omitted under [§4.7.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#471-when-to-include-a-proof).** It provides no producer-to-consumer end-to-end guarantee, and a *consumer* over this binding evaluates the `proof` requirement from §4.7.1 and the *Trust Task specification* alone — exactly as it would over a transport with no binding at all.

The name invites the opposite conclusion, which is why this is stated rather than left to inference. "HTTPS with an `Authorization` header" sounds like an authenticated channel, and it is — between the client and *whatever terminates TLS*. That is not the same boundary the framework cares about.

| Property | What this binding provides |
|---|---|
| **Authenticated producer** | A bearer token, mapped to a *VID* by a **deployment-defined** mechanism ([§3](#3-identity-mapping)) — a static map in a test deployment, a verified JWT in a production one. The binding does not constrain the token format and therefore cannot characterise the strength of the authentication. |
| **Mapping to a VID** | Deployment-defined, per §3. Deterministic within a deployment; not interoperable across deployments. |
| **Audience binding** | The server's own configured `local_vid`. This is **server-side configuration, asserted by the receiver about itself** — nothing in the request binds the producer's intent to this recipient. |
| **Integrity across intermediaries** | **None end-to-end.** TLS protects each segment to its terminator. §Abstract places TLS termination — reverse proxy or native — outside this binding's scope, so the binding cannot say how many segments there are. |
| **Re-origination** | **Possible and undetectable.** Any TLS-terminating intermediary — a reverse proxy, a load balancer, an API gateway, a service mesh sidecar — sees plaintext and can modify or re-originate the request body. The server has no signal that it did. |
| **Freshness / replay** | **None.** The binding "makes no claim about token-revocation, audience-restriction, or replay protection beyond what the chosen mechanism provides" (§3). A captured request body can be re-sent by anyone holding the token. |
| **Key and credential status** | Whatever the deployment's token mechanism provides; the binding requires nothing and can assume nothing. |
| **Where the guarantee stops** | At the first TLS terminator. Everything past it is deployment topology this binding does not see. |

A bearer token authenticates *whoever presents it*, which is the party that reached the terminator — not necessarily the party that composed the document. That is the case [SPEC §9.1.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#911-permitting-proof-to-be-omitted) forbids treating as grounds for omission: hop authentication tells a *consumer* who handed it the bytes, not who wrote them.

Two consequences worth stating plainly:

* **Carry a `proof` for anything consequential.** A *Trust Task* with a mutating, destructive, secret-disclosing, or subject-acting effect (a *consequential Trust Task*, [SPEC §2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#2-terminology)) should carry an in-band `proof` and an in-band `recipient` over this binding, so that producer identity and intended audience survive the intermediaries the binding cannot characterise.
* **Replay protection is the consumer's.** With no transport freshness, [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 — duplicate-execution protection keyed on the document `id` — is the only thing standing between a captured request and a repeated effect.

A future binding version **MAY** define a profile that does support omission — mutually-authenticated TLS terminated at the receiver itself, with the peer certificate mapped to a VID, would be the obvious candidate — but that is a different identity mechanism from the bearer mapping of §3 and, per [§7](#7-versioning), a `MAJOR` bump.

## 6. Discovery wiring

A server **MAY** advertise the set of *Type URIs* it dispatches by registering a handler for `https://trusttasks.org/spec/trust-task-discovery/0.1` ([SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-discovery-and-capability-negotiation)). Discovery requests **MUST** use the same `POST /trust-tasks` endpoint as every other request; no separate path is defined.

## 7. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the endpoint path, method, content-type expectations, and identity-mapping shape are preserved, and only additive header conventions, additional status-mapping rows, or stricter rules may be introduced. Breaking changes — a different endpoint path, a different identity-mapping mechanism, an incompatible status mapping — require a `MAJOR` bump and a new binding URI.

## 8. References

- [RFC 7235 — Hypertext Transfer Protocol (HTTP/1.1): Authentication](https://datatracker.ietf.org/doc/html/rfc7235).
- [RFC 6750 — The OAuth 2.0 Authorization Framework: Bearer Token Usage](https://datatracker.ietf.org/doc/html/rfc6750).
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §8, §9, §11.
