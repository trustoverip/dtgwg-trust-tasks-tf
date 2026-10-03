---
slug: https
version: "0.3"
title: HTTPS transport binding
summary: Carries Trust Task documents as JSON over HTTP/1.1 POST to a single endpoint; transport-authenticated sender identity comes from a bearer-token mapping to a VID. Adds streamed responses — a success response followed by further documents as Server-Sent Events — for tasks whose specification defines a stream.
status: draft
targetFrameworkVersion: "0.5.0"
bindingURI: https://trusttasks.org/binding/https/0.3
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how *Trust Task documents* are exchanged over HTTP/1.1. A producer sends a *Trust Task document* as the JSON body of an HTTP `POST` to a single well-known path; the server runs the framework's [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) consumer pipeline and either returns a typed `#response`-variant document or a `trust-task-error` error response. Transport-authenticated sender identity is conveyed via HTTP `Authorization` headers; the binding does not constrain the token format, but does define how the bearer-mapped *Verifiable Identifier* feeds the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence.

The binding is named **HTTPS** because every framework *Type URI* uses `https` ([SPEC §6.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#61-type-uri)) and a production deployment **MUST** terminate TLS in front of the receiver. The wire mechanics described here are HTTP/1.1; whether TLS is terminated by a reverse proxy or natively at the server is a deployment concern outside the scope of this binding.

## Status of This Document

`0.3` draft. Targets **framework `0.5.0`** and uses the framework's lowerCamelCase error-code vocabulary ([SPEC §4.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#410-naming-conventions), [§8.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#83-standard-error-codes)).

### Changes from 0.2

This version adds one thing: **streamed responses** ([§2.1](#21-streamed-responses)). A *Trust Task specification* may define a task whose success response is followed by further documents on the same delivery, such as a subscription to live events. This binding now carries that as a [Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html) stream on the response to the ordinary `POST`. The status mapping ([§4](#4-status-mapping)) and the lifecycle mapping ([§4.1](#41-lifecycle-mapping)) gain the rows that stream needs, and the transport security profile ([§5](#5-transport-security-profile)) says what the stream does and does not guarantee. Everything else is unchanged from [0.2](../0.2/spec.md). The change is additive per [§8](#8-versioning): a client that never asks for a stream sees exactly 0.2, and a 0.2 server answering a request for a stream answers it as 0.2 would.

[`trust-tasks-https`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-https) implements everything except §2.1: a typed [`HttpsClient`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/client.rs) (reqwest) and an axum-based [`HttpsServer`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-https/src/server.rs). No library implements streamed responses yet.

## 1. Binding URI

| Resource           | URI                                                |
|--------------------|----------------------------------------------------|
| Binding identifier | `https://trusttasks.org/binding/https/0.3`         |

The binding URI does not appear on the wire — unlike DIDComm, HTTPS has no envelope `type` field. The URI exists solely as the stable identifier this binding is referred to by elsewhere in the framework and in registries.

## 2. Document carriage

A producer **MUST** send a *Trust Task document* as follows:

| HTTP element            | Value                                                                                    |
|-------------------------|------------------------------------------------------------------------------------------|
| Method                  | `POST`                                                                                   |
| Path                    | `/trust-tasks`, appended to the *Trust-Task base* ([§6](#6-endpoint-discovery))           |
| `Content-Type` request  | `application/json`                                                                       |
| Request body            | The *Trust Task document* serialised as a JSON object (UTF-8, no BOM).                   |
| `Accept` request        | `application/json` (recommended), or as [§2.1](#21-streamed-responses) requires for a streamed task. |
| `Authorization` request | `Bearer <token>` where `<token>` identifies the sender (see [§3. Identity mapping](#3-identity-mapping)). |

A conforming server **MUST**:

1. Accept `POST /trust-tasks` and reject every other method/path combination with the appropriate HTTP status (`405 Method Not Allowed` / `404 Not Found`). These responses are not framework error documents.
2. Read `Authorization` (if present) and map the bearer token to a *Verifiable Identifier* via a deployment-defined mechanism (an in-process map, a JWT verifier, a database lookup, …). A request with no `Authorization` header is treated as having no transport-authenticated sender.
3. Parse the request body as JSON, then as a `TrustTask<P>` document for some payload type `P` selected by the document's `type` member.
4. Apply the framework [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) consumer pipeline — `resolve_parties` per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), `validate_basic`, `enforce_audience_binding`, dispatch by canonical *Type URI* per [§4.4.1 item 1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#441-request-and-response-variants), then the registered handler.
5. Return either a `#response`-variant *Trust Task document* (success) or a `trust-task-error` document (rejection) as the response body — both in JSON, both with `Content-Type: application/json` — except where [§2.1](#21-streamed-responses) applies.

Outside the streamed responses of §2.1, this binding defines no multi-message variant: one request per HTTP exchange, one response.

### 2.1 Streamed responses

A *Trust Task specification* **MAY** define a **streamed task**. For such a task, a successful execution is answered with the `#response`-variant document, and the *consumer* then sends further *Trust Task documents* to the *producer* on the same delivery until the stream ends. These are **streamed documents**, and the specification names their *Type URIs*. A streamed document is not a further reply to the request. The `#response` has already closed the request's exchange as a success ([SPEC §4.4.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#441-request-and-response-variants)). Each streamed document is a *Trust Task document* in its own right, flowing from the *consumer* to the *producer*, and is typically fire-and-forget. This section defines how such a task is carried. A request for any other task never receives a stream.

#### 2.1.1 Opening a stream

The *producer* sends the request exactly as [§2](#2-document-carriage) requires — `POST /trust-tasks`, a JSON document body, an `Authorization` header — and asks for a stream with:

```
Accept: text/event-stream, application/json;q=0.5
```

`application/json` stays acceptable at lower preference so that a server that does not stream can still answer in the 0.2 way, rather than with `406 Not Acceptable`.

A server that implements this section, receiving such a request for a streamed task it will execute:

1. **MUST** run the [§2](#2-document-carriage) pipeline in full before it writes anything. A refusal — any `trust-task-error` — is answered exactly as §2 and [§4](#4-status-mapping) define: a JSON body, `Content-Type: application/json`, and the mapped status. **No stream is opened to carry an error**, so a client never has to parse an event stream to learn it was refused.
2. On success, **MUST** answer `200 OK` with `Content-Type: text/event-stream`, and **SHOULD** send `Cache-Control: no-store`. The **first** event of the stream **MUST** carry the `#response`-variant document. Streamed documents follow it, and **MUST NOT** precede it.

A server receiving a request for a stream that it cannot or will not stream — it does not implement this section, or the task is not a streamed task — answers in JSON as §2 defines. A *producer* receiving `application/json` reads it under §4 as an unstreamed answer. For a streamed task, the task's specification says what such an answer means. Typically it is a refusal saying that no stream is available, and the *producer* falls back to whatever the specification provides, such as polling.

#### 2.1.2 Event framing

The stream is a `text/event-stream` as defined by the [WHATWG HTML Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html) specification, UTF-8 without a byte-order mark. Within it:

1. **One document per event, on one `data:` line.** Each event carries exactly one complete *Trust Task document*, serialized as JSON with no line breaks. JSON never needs a raw line break, because line breaks inside strings are escaped. The document is placed in exactly one `data:` field. A server **MUST NOT** split a document across several `data:` fields, and **MUST NOT** put more than one document in an event. A client that receives an event whose data is not exactly one JSON object parseable as a *Trust Task document* **MUST** discard that event, and **SHOULD** close the stream, because a framing error means the stream can no longer be trusted to delimit documents.
2. **No `event:` field.** A server **MUST NOT** set the SSE `event` field. Every event therefore has the default type, and the document's own `type` member is the only discriminator, as [SPEC §4.4.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#441-request-and-response-variants) requires of every reply.
3. **`id:` carries a resume token, where the task defines one.** Where the streamed task's specification defines a resume token for its documents, the server **SHOULD** set each event's SSE `id` field to that document's token (including the `#response`'s, where it carries one), and otherwise **MUST** omit `id`. The value **MUST NOT** contain U+0000, U+000A or U+000D, so a specification defining a resume token constrains its alphabet accordingly.
4. **`retry:` is a hint.** A server **MAY** send the SSE `retry` field. A client **SHOULD** treat it as the floor of its reconnection backoff, never as a schedule.
5. **Comments are heartbeats.** A line beginning with `:` is an SSE comment. A server **SHOULD** send one whenever the stream has otherwise been silent for 30 seconds, or for the shorter interval the task's specification defines. The interval should be shorter than the idle timeouts of the intermediaries in front of the server. A comment carries no document, and a client **MUST NOT** interpret its text. A client that receives nothing on the stream — no event and no comment — for twice the heartbeat interval **MUST** treat the stream as dead and close it.

#### 2.1.3 Reconnecting and resuming

A browser's `EventSource` reconnects by issuing a `GET` with no body and no `Authorization` header. This binding accepts only `POST /trust-tasks` with a document body (§2), so `EventSource`'s automatic reconnection cannot re-open a stream, and this binding does not offer a `GET` it could reconnect to. That is deliberate. A `GET` endpoint for an already-authorized stream would be a bearer capability in a URL, and it would carry no document, no `proof` and no freshness. Instead, every stream is opened by a request document that the server authenticates, validates and authorizes as it would any other.

A client therefore reads the stream with a streaming HTTP client — `fetch` with a readable body, for example — and reconnects by sending a **new** request document, with a fresh `id`, `issuedAt` and, where required, `proof`. Re-sending the original bytes is a [SPEC §8.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#84-retry-semantics) retry of a request already answered, not a reconnection.

Resumption is the streamed task's business, carried in-band:

1. Where the task's specification defines a resume parameter in its request payload, a client resuming places in it the last SSE `id` it received.
2. A client **MAY** also send the `Last-Event-ID` request header with the same value, for intermediaries and diagnostics.
3. A server **MUST** resume from the in-band parameter only. The header is not covered by the document's `proof`. Where both are present and differ, the server **MUST** refuse with `malformedRequest`. This follows the rule of [SPEC §9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies): the in-band member is authoritative and the transport value is a cross-check. A server **MUST NOT** resume from `Last-Event-ID` alone.

#### 2.1.4 Ending a stream

Either side **MAY** end a stream at any time, the client by closing the connection and the server by ending the response.

1. **Ending a stream is not cancellation and not an error.** It ends that delivery. It **MUST NOT** be read as cancellation of anything ([SPEC §11.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#117-transport-level-cancellation-is-not-semantic-cancellation)). It reports no state of the request, whose exchange the `#response` already closed, and no state of any streamed document.
2. **Once the `#response` has been sent, the server MUST NOT send a `trust-task-error` on the stream.** The request's exchange is closed, and an error document would claim a second disposition for it. A server that has to stop — restart, load-shedding, a lifetime bound, a change in the client's entitlement — ends the response, and carries no reason. The client re-sends a request, and any refusal arrives as the answer to that.
3. **The stream does not outlive its authentication.** A server **MUST** end a stream no later than the expiry or revocation of the bearer credential the request was authenticated with ([§3](#3-identity-mapping)), and **SHOULD** bound the life of any one stream, so that a long-lived stream is periodically re-authorized by a fresh request.
4. **No delivery guarantee.** A streamed document not yet written when the stream ends is lost. One written but not yet read may be lost too, and this binding provides no acknowledgement. A streamed task's specification **MUST** be designed so that loss is tolerable: by resumption, by a fallback the *producer* can use, or both.

#### 2.1.5 Identity of streamed documents

Streamed documents travel the same way the response does, from the server to the client that authenticated the request. For each streamed document, the client applies [SPEC §4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) with the *transport-authenticated sender* taken to be the party the request was addressed to, and the *transport-authenticated recipient* taken to be the bearer-mapped VID the request was sent under. A streamed document whose in-band `issuer` or `recipient` disagrees with these values has no return path on which to report `identityMismatch`. The client **MUST** discard it, and **SHOULD** close the stream.

A client **MUST NOT** answer a streamed document on the stream, because there is no return path. Where a streamed task's specification defines a reply to a streamed document, that reply is sent as an ordinary request under §2.

## 3. Identity mapping

The mapping into the framework's [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence is:

| Framework concept                            | HTTPS-derived value                                                                                                  |
|----------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| *Transport-authenticated sender*             | The VID the server maps the bearer token to. Absent for requests with no `Authorization` header or with an unrecognised token. |
| *Transport-authenticated recipient*          | The server's own configured `local_vid`. This is a server-side configuration value, not anything carried in the HTTP request. |
| Producer's *in-band* `issuer` (when set)     | Cross-checked against the transport-authenticated sender per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |
| Producer's *in-band* `recipient` (when set)  | Cross-checked against the transport-authenticated recipient per [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity); mismatch is `identityMismatch`. |

The mapping between bearer token and VID is **deployment-defined**. A demo or test deployment **MAY** use a static `HashMap<token, VID>`; a production deployment **SHOULD** verify a JWT against an issuer-controlled JWKS or otherwise bind tokens to verifiable identifiers under a controlled trust framework. The binding makes no claim about token-revocation, audience-restriction, or replay protection beyond what the chosen mechanism provides.

When applying the §8.1 error-response routing rule under `identityMismatch`, the server **MUST** route its `trust-task-error` response to the bearer-authenticated sender it actually authenticated, and **MUST NOT** carry the contested in-band `issuer` in the response's `recipient` member.

## 4. Status mapping

The server **SHOULD** map the *Trust Task document* response to an HTTP status as follows. The framework error code remains authoritative; the HTTP status is informative (intermediaries and end-user diagnostics).

| Outcome                                                  | HTTP status                           |
|----------------------------------------------------------|---------------------------------------|
| Success (a `#response`-variant document)                 | `200 OK`                              |
| Success for a streamed task, stream opened ([§2.1](#21-streamed-responses)) | `200 OK`, `Content-Type: text/event-stream`; the `#response` is the first event |
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

In every case where the body carries a single Trust Task document — success or `trust-task-error` — the `Content-Type` **MUST** be `application/json`. A stream opened under [§2.1](#21-streamed-responses) is `text/event-stream`, and is only ever opened on success. The `202` and `204` answers carry no body and therefore no `Content-Type`.

A client receiving a non-2xx response with `Content-Type: application/json` **MUST** attempt to deserialise the body as a `trust-task-error` document before falling back to transport-level error handling. A client receiving a non-2xx response with any other `Content-Type` treats the response as an untyped transport-level failure.

### 4.1 Lifecycle mapping

Required by [SPEC §9.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#91-what-a-transport-binding-specifies), which makes the mapping a **MUST** for every binding: each state of [SPEC §4.12 *Document Lifecycle*](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#412-document-lifecycle) is named below against the HTTP event or status that carries it, or declared to have no counterpart here.

Most of the table above already *is* this mapping; what it did not do is say which state each row reports, and that is the gap this section closes. The framework's rule exists because a status that merely sounds like a state gets read as one — and this binding ships the exact status the framework names as the example.

| Lifecycle state | HTTPS counterpart |
|---|---|
| `received` | **No counterpart.** Nothing has been written to the response when the *consumer* holds the bytes. A completed TLS handshake, an accepted TCP connection, and a fully-read request body are transport facts about the *server process*, not about a *Trust Task document*; none is observable to the *producer* as a state and none is emitted. |
| `validated` | **No counterpart.** [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) items 1–10 run entirely between reading the request and writing the status line. Nothing on the wire separates a document that passed them from one still being read. |
| `accepted` | **No counterpart.** Acceptance is established by writing the duplicate-execution record — [§5.1](#51-freshness-and-duplicate-execution) item 3, at the point the *consumer* commits to execute and before dispatch — and this binding emits nothing at that point. **`202 Accepted` does not mean `accepted`.** The table above gives `202` exactly one meaning: a **duplicate** of a document whose *first* execution is still in progress. A first arrival is never answered `202` under this binding, so a `202` is a statement about an *earlier* request and never about the acceptance of the request that received it. |
| `executing` | **No counterpart for the request in hand.** A response is written once, at the end; there is no intermediate status for work under way on the document this request delivered. The `202` row is the only place execution under way is observable at all, and it observes a *previous* request's execution — reported to a duplicate, and only where no response was retained to return instead. |
| `suspended` | **No counterpart on this exchange.** Suspension is carried by a `trust-task-control` document ([SPEC §11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#11-task-control)), which over this binding is a **separate** `POST` with its own response; that response carries the transition. The `POST` that delivered the original document returned long before, and nothing about it changes when the document later suspends. |
| `responded` | `200 OK` carrying the `#response`-variant document; for a streamed task, the **first event** of a `200 OK` `text/event-stream` response, which carries that document — the status line and headers alone, written before it, are not the transition; `204 No Content` where the *Trust Task specification* defines no success response ([SPEC §4.4.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#441-request-and-response-variants)); `200 OK` carrying the retained result where a duplicate is answered with the first execution's response, and `204 No Content` where a completed execution retained none. The framework's empty `{}` `#response` — the positive "received and done" a fire-and-forget *consumer* may now send as a courtesy — is a `200` with that body, not a `204`. |
| `errored` | The rows carrying a `trust-task-error` body: `400` (`malformedRequest`), `403` (`permissionDenied`), `409` (`idConflict`), the flat `422` bucket (`unsupportedType`, `unsupportedVersion`, `proofRequired`, `proofInvalid`, `identityMismatch`, `wrongRecipient`, `taskFailed`), `503` (`unavailable`), `500` (`internalError`). The framework error code in the body is authoritative; the status is informative, and the `422` bucket does not distinguish the codes collapsed into it. |
| `cancelled` | `422` carrying `cancelled`, where the *consumer* stopped of its own accord. For a *producer*-requested cancellation the transition is carried by the response to the separate `trust-task-control` `POST`, exactly as for `suspended`. **No HTTP-level abort is cancellation** — see below. |
| `expired` | `422` carrying `expired`. Expiry reached after the response was written, or on a request that never arrived, produces nothing at all: [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 12 means expiry does not bound a document already `executing`, and a *consumer* that lapses a suspended document announces nothing. |

Three substitutions this binding makes it particularly easy to reach for, stated so that no implementer has to infer them:

* **`202` is not `accepted`.** It is the framework's own worked example of the confusion, and this binding is where it would happen. Read `202` as the table defines it — *this document was already accepted, on an earlier request, and its execution has not finished* — and note that it therefore reports a state strictly **past** `accepted`, on a request that is not the one being answered.
* **A closed connection is not `cancelled`.** An aborted request, a client timeout, a load-balancer idle-timeout, or a server-side connection reset terminates *that delivery*. A document already accepted under [§5.1](#51-freshness-and-duplicate-execution) survives the connection that delivered it and may still be executing; a *producer* that treats the abort as a withdrawal will believe it stopped work that is still running, and a *consumer* that does will stop work the *producer* still wants. Semantic cancellation over this binding is a `trust-task-control` document and nothing else.
* **A response that never comes is not a state.** Where the connection hangs, the timeout fires, or a `502` arrives from an intermediary that never reached the *consumer*, the *producer* has learned nothing: the document may be in any state in the table, including one it never entered. The same holds for `401` and for a bare `500` with no framework error body — both are transport-level refusals carrying no *error response*, and neither establishes that the document reached `errored`. The remedy is [SPEC §8.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#84-retry-semantics)'s bit-for-bit resend, which [§5.1](#51-freshness-and-duplicate-execution) requires the *consumer* to absorb.

**Streamed documents ([§2.1](#21-streamed-responses)).** Each streamed document is a *Trust Task document* sent by the server to the client, and its own lifecycle is not observable on this binding. The client can reply to none of them on the stream. A client that refuses one discards it, and emits nothing. So for a streamed document this binding expresses **no** counterpart for any state: the server learns nothing about it once it is written. The end of a stream — the client closing it, the server ending it, or an intermediary dropping it — corresponds to **no** state of the request, whose exchange the `#response` closed, and to no state of any streamed document. In particular it is not `cancelled`, for the reason the second bullet below gives.

Because the exchange is synchronous and single-shot, this binding expresses **no** counterpart for `received`, `validated`, `accepted`, `executing`, or `suspended`. Every one of them is passed through inside a single request, and the *producer* observes only a terminal state — or nothing. That is a property of the transport, not a defect in it, and a binding that claimed otherwise would be manufacturing the substitution the framework's rule exists to prevent.

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

* **Carry a `proof` for anything consequential.** A *Trust Task* with a mutating, destructive, secret-disclosing, or subject-acting effect (a *consequential Trust Task*, [SPEC §3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#3-terminology)) should carry an in-band `proof` and an in-band `recipient` over this binding, so that producer identity and intended audience survive the intermediaries the binding cannot characterise.
* **Replay protection is the consumer's.** With no transport freshness, [SPEC §7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) item 11 — duplicate-execution protection keyed on the document `id` — is the only thing standing between a captured request and a repeated effect. §5.1 states what this binding requires of that *consumer*.
* **A stream adds no guarantee.** Streamed documents ([§2.1](#21-streamed-responses)) travel the reverse path under the same TLS segments, and every row of the table above applies to them unchanged. Any TLS-terminating intermediary can read, delay, drop, reorder or inject events, and can end the stream. A streamed task whose documents carry data or authority a client will rely on **SHOULD** require `proof` on them. A task whose streamed documents only prompt the client to do something it then authorizes separately — a hint to re-read — limits a forged event's consequence to the cost of that action, and its specification can say so. Either way, the stream's end is not evidence of anything ([§2.1.4](#214-ending-a-stream)).

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

A server **MAY** advertise the set of *Type URIs* it dispatches by registering a handler for `https://trusttasks.org/spec/trust-task-discovery/0.1` ([SPEC §10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#10-discovery-and-capability-negotiation)). Discovery requests **MUST** use the same `POST /trust-tasks` endpoint as every other request; no separate path is defined.

## 8. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision **MUST** remain backwards-compatible with consumers implementing this version: the endpoint path, method, content-type expectations, and identity-mapping shape are preserved, and only additive header conventions, additional status-mapping rows, or stricter rules may be introduced. Breaking changes — a different endpoint path, a different identity-mapping mechanism, an incompatible status mapping — require a `MAJOR` bump and a new binding URI.

`0.3` is such a `MINOR` revision of `0.2`. Streamed responses ([§2.1](#21-streamed-responses)) are reached only through an `Accept: text/event-stream` request for a task defined as streamed. A `0.2` client never sends one, and a `0.2` server answering one answers in JSON, which §2.1.1 tells a `0.3` client how to read.

## 9. References

- [RFC 7235 — Hypertext Transfer Protocol (HTTP/1.1): Authentication](https://datatracker.ietf.org/doc/html/rfc7235).
- [RFC 6750 — The OAuth 2.0 Authorization Framework: Bearer Token Usage](https://datatracker.ietf.org/doc/html/rfc6750).
- [WHATWG HTML — Server-sent events](https://html.spec.whatwg.org/multipage/server-sent-events.html).
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §8, §9, §11.
