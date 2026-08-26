---
slug: https
version: "0.2"
title: HTTPS transport binding
summary: Carries Trust Task documents as JSON over HTTP/1.1 POST to a single endpoint; transport-authenticated sender identity comes from a bearer-token mapping to a VID.
status: draft
targetFrameworkVersion: "0.2"
bindingURI: https://trusttasks.org/binding/https/0.2
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged over HTTP/1.1. A producer sends a *Trust Task document* as the JSON body of an HTTP `POST` to a single well-known path; the server runs the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) consumer pipeline and either returns a typed `#response`-variant document or a `trust-task-error/0.2` error response. Transport-authenticated sender identity is conveyed via HTTP `Authorization` headers; the binding does not constrain the token format, but does define how the bearer-mapped *Verifiable Identifier* feeds the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

The binding is named **HTTPS** because every framework *Type URI* uses `https` ([SPEC §6.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#61-type-uri)) and a production deployment **MUST** terminate TLS in front of the receiver. The wire mechanics described here are HTTP/1.1; whether TLS is terminated by a reverse proxy or natively at the server is a deployment concern outside the scope of this binding.

## Status of This Document

`0.2` draft. Targets **framework `0.2`** and uses the framework's lowerCamelCase error-code vocabulary ([SPEC §4.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#410-naming-conventions), [§8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)). The binding is implemented by [`trust-tasks-https`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-https) — a typed [`HttpsClient`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/client.rs) (reqwest) and an axum-based [`HttpsServer`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/server.rs).

## 1. Binding URI

| Resource           | URI                                                |
|--------------------|----------------------------------------------------|
| Binding identifier | `https://trusttasks.org/binding/https/0.2`         |

The binding URI does not appear on the wire — unlike DIDComm, HTTPS has no envelope `type` field. The URI exists solely as the stable identifier this binding is referred to by elsewhere in the framework and in registries.

## 2. Document carriage

A producer **MUST** send a *Trust Task document* as follows:

| HTTP element            | Value                                                                                    |
|-------------------------|------------------------------------------------------------------------------------------|
| Method                  | `POST`                                                                                   |
| Path                    | `/trust-tasks`, appended to the *Trust-Task base* ([§6](#6-endpoint-discovery))           |
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
| Success for a specification that defines **no** success response ([SPEC §4.4.1](/SPEC.md#441-request-and-response-variants) fire-and-forget) | `204 No Content` |
| Duplicate of a document whose execution is still in progress ([§5.1](#51-freshness-and-duplicate-execution)) | `202 Accepted`, empty body |
| Duplicate of a completed execution for which no response was retained ([§5.1](#51-freshness-and-duplicate-execution)) | `204 No Content` |
| `malformedRequest`                                       | `400 Bad Request`                     |
| Missing / invalid `Authorization` (transport-level, no framework error doc) | `401 Unauthorized` |
| `permissionDenied`                                       | `403 Forbidden`                       |
| `unsupportedType` / `unsupportedVersion`                 | `422 Unprocessable Entity`            |
| `expired`                                                | `422 Unprocessable Entity`            |
| `proofRequired` / `proofInvalid` / `identityMismatch`    | `422 Unprocessable Entity`            |
| `wrongRecipient`                                         | `422 Unprocessable Entity`            |
| `cancelled`                                              | `422 Unprocessable Entity`            |
| `taskFailed`                                             | `422 Unprocessable Entity`            |
| `idConflict`                                             | `409 Conflict`                        |
| `unavailable`                                            | `503 Service Unavailable`             |
| `internalError`                                          | `500 Internal Server Error`           |
| Internal server error (transport-level, no error doc)    | `500 Internal Server Error`           |

A duplicate already accepted under [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 is answered with the response the first execution produced, under `200 OK`, where the *consumer* retained one. The `202` and `204` rows are the two cases where there is no such response to return: an execution still in progress, and a specification that defines none. Neither is an error — §7.2 states that "in no case is a duplicate reported as `taskFailed`; the task did not fail, it already happened" — and a *producer* **MUST NOT** treat either as a failed *Trust Task*.

In every case where the body carries a Trust Task document — success or `trust-task-error/0.2` — the `Content-Type` **MUST** be `application/json`. The `202` and `204` answers carry no body and therefore no `Content-Type`.

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
| **Freshness / replay** | **None at the transport layer**, and the binding still "makes no claim about token-revocation, audience-restriction, or replay protection beyond what the chosen mechanism provides" (§3): a captured request body can be re-sent by anyone holding the token, and nothing in HTTP will stop it arriving. What the binding now requires is that the *consumer* refuse to act on it twice — see [§5.1](#51-freshness-and-duplicate-execution). |
| **Key and credential status** | Whatever the deployment's token mechanism provides; the binding requires nothing and can assume nothing. |
| **Where the guarantee stops** | At the first TLS terminator. Everything past it is deployment topology this binding does not see. |

A bearer token authenticates *whoever presents it*, which is the party that reached the terminator — not necessarily the party that composed the document. That is the case [SPEC §9.1.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#911-permitting-proof-to-be-omitted) forbids treating as grounds for omission: hop authentication tells a *consumer* who handed it the bytes, not who wrote them.

Two consequences worth stating plainly:

* **Carry a `proof` for anything consequential.** A *Trust Task* with a mutating, destructive, secret-disclosing, or subject-acting effect (a *consequential Trust Task*, [SPEC §2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#2-terminology)) should carry an in-band `proof` and an in-band `recipient` over this binding, so that producer identity and intended audience survive the intermediaries the binding cannot characterise.
* **Replay protection is the consumer's.** With no transport freshness, [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 — duplicate-execution protection keyed on the document `id` — is the only thing standing between a captured request and a repeated effect. §5.1 states what this binding requires of that *consumer*.

### 5.1 Freshness and duplicate execution

Because the transport supplies no freshness of its own, a *consumer* over this binding **MUST** implement [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 for every *consequential Trust Task* it serves, and **MUST** apply a bounded acceptance window over `issuedAt` / `expiresAt` (§7.2, *Bounding the record*) so that the record it keeps is finite. Nothing below relaxes any rule stated elsewhere in this document; it names an obligation that §5 previously left to inference.

Specifically, the *consumer*:

1. **MUST** key the record on the document `id` alone, and compare arrivals under a reused `id` on the whole document's canonical serialization, `proof` included — §7.2, *Keying and comparison*. HTTP request identifiers, `Idempotency-Key`-style headers, and any execution handle the *consumer* mints **MUST NOT** substitute for the document `id`.
2. **MUST NOT** cause the consequential effect a second time for a document already accepted under that `id`, and **MUST** reject a *different* document under the same `id` with `idConflict` ([§4](#4-status-mapping): `409 Conflict`).
3. **MUST** record the claim at the point it commits to execute — after every other check this binding requires, and before dispatch. A record written earlier burns the `id` of documents the *consumer* then refuses.
4. **SHOULD** answer a duplicate with the result the first execution produced; where no result was retained, it answers per the `202` / `204` rows of [§4](#4-status-mapping). A duplicate is **never** reported as `taskFailed`.
5. **MUST** fail closed where the record cannot be consulted: answer `unavailable` with `retryable` true ([§4](#4-status-mapping): `503`) and **MUST NOT** execute. A *consumer* that cannot establish whether a document is a duplicate has not satisfied item 11.
6. **MUST**, where it is replicated behind a load balancer or any other fan-out, share the record across every replica. Two replicas each keeping their own record each accept the same document once, which is the failure item 11 exists to prevent.
7. **MUST** retain each record at least as long as it remains willing to execute the document. §7.2 makes the acceptance window and the record's retention **the same bound**; a *consumer* that widens one widens the other.

A *producer* retries by re-sending the **same bytes** ([SPEC §8.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#84-retry-semantics)). Re-signing, re-stamping `issuedAt`, or otherwise altering the body under a reused `id` is not a retry; it is a different document, and item 11 requires the *consumer* to answer it with `idConflict`.

This does not make the binding safe against replay — the request body is still capturable and replayable by anyone who holds the token or terminates TLS. It makes the *effect* happen once.

A future binding version **MAY** define a profile that does support omission — mutually-authenticated TLS terminated at the receiver itself, with the peer certificate mapped to a VID, would be the obvious candidate — but that is a different identity mechanism from the bearer mapping of §3 and, per [§8](#8-versioning), a `MAJOR` bump.

## 6. Endpoint discovery

[§2](#2-document-carriage) fixes the request **path** at `/trust-tasks`. This section defines what that path is relative to, and how a *producer* holding only a *consumer*'s DID obtains it.

### 6.1 The Trust-Task base

A `serviceEndpoint` advertised for this binding denotes the **Trust-Task base**: the origin-and-optional-prefix that `/trust-tasks` is appended to. A producer composing a request **MUST** form the request URL as:

```
<serviceEndpoint> + "/trust-tasks"
```

A trailing `/` on the advertised `serviceEndpoint` **MUST** be ignored rather than producing an empty path segment.

Nothing in [0.1](../0.1/spec.md) said what the advertised endpoint denoted, and the omission is not academic: an implementation reading §2 alone appends `/trust-tasks` to the advertisement, while one that treats the advertisement as an already-complete Trust-Task endpoint posts to it directly. Both readings are defensible against 0.1 and they are not interoperable. Deployments have shipped each of them.

### 6.2 The service entry

A *consumer* reachable over this binding **SHOULD** advertise a DID-document service entry:

| Member            | Value                                                                         |
|-------------------|-------------------------------------------------------------------------------|
| `type`            | `TrustTaskHTTPS`                                                              |
| `serviceEndpoint` | the *Trust-Task base* of [§6.1](#61-the-trust-task-base), an `https:` URL      |

A producer resolving a consumer's DID **MUST** match on the service `type` and **MUST NOT** match on the `id` fragment, which is an arbitrary label chosen by the DID controller.

`TrustTaskHTTPS` names an **interface**, not a product: "this party accepts Trust Task documents over the HTTPS binding". It is deliberately not a service type belonging to any particular application. A party that also exposes an unrelated REST API advertises that separately under its own type — the two claims are different, and a consumer that conflates them will send Trust Tasks to an endpoint that never agreed to accept them.

Sibling bindings do not need this section because their addresses are already owned by another specification: the DIDComm binding resolves `DIDCommMessaging` ([DID Specification Registries](https://www.w3.org/TR/did-spec-registries/#didcommmessaging)), and the TSP binding resolves a VID through TSP's own mechanism. HTTPS is the only binding in this family whose address had no owning specification, which is precisely why two conforming implementations could disagree about it.

### 6.3 Out-of-band configuration

DID-based discovery is not the only way to learn a base, and this section does not make it mandatory. A producer configured with a Trust-Task base directly — a deployment setting, a bootstrap file, an operator-supplied URL — is conformant, and **MUST** compose the request path the same way ([§6.1](#61-the-trust-task-base)).

Where both are available, a producer **SHOULD** prefer the DID-document advertisement, because it is the value the consumer controls and can rotate. A stale configured base outlives the consumer's ability to move.

### 6.4 Relationship to capability discovery

[§7](#7-capability-discovery) answers *which Trust Tasks does this party accept*. This section answers *where is this party*. The second question must be answerable first: a capability-discovery request is itself a Trust Task document sent over this binding, so a producer that cannot compose the URL cannot ask what the consumer supports.

## 7. Capability discovery

A server **MAY** advertise the set of *Type URIs* it dispatches by registering a handler for `https://trusttasks.org/spec/trust-task-discovery/0.1` ([SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-discovery-and-capability-negotiation)). Discovery requests **MUST** use the same `POST /trust-tasks` endpoint as every other request; no separate path is defined.

## 8. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the endpoint path, method, content-type expectations, and identity-mapping shape are preserved, and only additive header conventions, additional status-mapping rows, or stricter rules may be introduced. Breaking changes — a different endpoint path, a different identity-mapping mechanism, an incompatible status mapping — require a `MAJOR` bump and a new binding URI.

## 9. References

- [RFC 7235 — Hypertext Transfer Protocol (HTTP/1.1): Authentication](https://datatracker.ietf.org/doc/html/rfc7235).
- [RFC 6750 — The OAuth 2.0 Authorization Framework: Bearer Token Usage](https://datatracker.ietf.org/doc/html/rfc6750).
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §8, §9, §11.
