---
slug: vtc/join-requests/wallet-session/start
version: "0.1"
title: VTC Join-Requests — Wallet Session Start
summary: An applicant opens a short-lived OpenID4VP presentation session for one join criterion, and receives the wallet URI a swiyu or EUDI wallet scans to present credentials as evidence for the applicant's join.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - oid4vp
  - dcql
  - wallet
  - presentation
  - eudi
  - swiyu
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The session is bound to the applicant, and the only thing that names the applicant is the document proof's signer. Whatever a wallet later presents into the session is recorded against that signer's join, so a session opened without a proof would be evidence with nobody to attach it to — or, worse, attached to whoever the transport said was asking.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Each accepted document creates a session holding a nonce and a key. A captured start replayed later would open sessions nobody asked for, bound to the applicant, and count against their limit of open sessions; the window is what lets the consumer refuse it.
sideEffects:
  level: mutating
  rationale: Creates a short-lived presentation session at the community holding a single-use nonce, a per-session ephemeral key for the encrypted wallet response, the signed request object, and the binding to the applicant, the criterion and, when given, the join request. Nothing about the applicant's join changes until a wallet answers.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response returns a session handle, the community's verifier client identifier, and the DCQL query of a criterion the manifest already publishes — the community's own policy, not data about any person. The request carries a criterion digest the community published and, optionally, an identifier of a join request it minted; nothing about the applicant travels inbound that the community did not already hold."
retention:
  class: exchange
  rationale: 'The session — nonce, ephemeral private key, request object, binding — lives until a wallet answers it or it expires, minutes after it opens, and is then dropped; the ephemeral private key is deleted when the session closes. What survives is not this task''s: the evidence a wallet''s presentation yields is recorded on the applicant''s join request (or held briefly for their next submission) and lives as long as that request''s record does.'
errorCodes:
  - code: vtc/join-requests/wallet-session/start:criterionNotFound
    meaning: No criterion the community publishes, or still honours within its `requirementsGrace`, has the supplied `requirementsDigest`.
    retryable: false
  - code: vtc/join-requests/wallet-session/start:noWalletPresentation
    meaning: The criterion states no credential query a wallet can answer — it requires no credential, or only credentials in formats the community does not accept over OpenID4VP.
    retryable: false
  - code: vtc/join-requests/wallet-session/start:verifierNotConfigured
    meaning: The community holds no OpenID4VP verifier identity for the requested `clientIdPrefix`, or, when none was requested, for any prefix.
    retryable: false
  - code: vtc/join-requests/wallet-session/start:requestNotFound
    meaning: No join request with the supplied `requestId` exists, or it does not belong to the proof signer. The same code for both.
    retryable: false
  - code: vtc/join-requests/wallet-session/start:requestClosed
    meaning: The supplied `requestId` names the signer's own join request, but it has been decided or withdrawn and can take no more evidence.
    retryable: false
  - code: vtc/join-requests/wallet-session/start:tooManySessions
    meaning: The applicant already has as many open wallet sessions as the community allows. Retry once one has been answered or has expired.
    retryable: true
related:
  - vtc/join-requests/wallet-session/status
  - vtc/join-requests/manifest
  - vtc/join-requests/submit
  - vtc/join-requests/status
  - credential-exchange/query
  - credential-exchange/present
---

## Abstract

The **VTC Join-Requests — Wallet Session Start** Trust Task lets an applicant ask a community to collect evidence for one of its join criteria from a credential wallet — a swiyu or EU EUDI wallet holding an SD-JWT VC, say — that cannot take part in a Trust Task itself.

Such a wallet has no DID agent. It signs no Trust Task and receives no pushed document; it speaks [OpenID4VP 1.0](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) over HTTPS, as a verifier's client, and nothing else. The applicant, who does sign Trust Tasks, therefore starts the exchange on the wallet's behalf: this task opens a **presentation session** at the community, bound to the applicant, and returns a `walletUri` the applicant shows as a QR code or opens as a deep link. The wallet follows it to two OpenID4VP endpoints the community serves for that session alone — a `request_uri` returning a signed request object, and a `response_uri` receiving the encrypted response — and the applicant reads the outcome with [`vtc/join-requests/wallet-session/status`](../../status/0.1/spec.md).

What the wallet presents becomes **evidence on the applicant's join**, never the applicant's identity. The applicant is, and stays, the DID that signed this task; the key that held the presented credentials is recorded beside it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### What this task covers, and what it does not

This task **begins** an OpenID4VP exchange; it does not carry one. The two wallet-facing calls — fetching the request object from `request_uri` and posting the response to `response_uri` — are OpenID4VP 1.0, with the request passed by reference as a signed JWT-Secured Authorization Request ([RFC 9101](https://www.rfc-editor.org/rfc/rfc9101)) and the response returned by `direct_post.jwt`. Their wire is OpenID4VP's, fixed by the wallet, and outside this specification. The session they act on, and the outcome they produce, are reached only through this task and [`wallet-session/status`](../../status/0.1/spec.md).

### Producer (applicant)

1. **MUST** carry a proof; its signer is the applicant the session is bound to.
2. **MUST** name the criterion by the `requirementsDigest` [`vtc/join-requests/manifest/0.3`](../../../manifest/0.3/spec.md) published for it, and **SHOULD** check that the criterion states a credential query before asking.
3. **MAY** supply `requestId` to record the evidence on a join request it has already submitted — typically one answered `deferred` because a credential was missing. Without it the evidence is held for its next [`submit`](../../../submit/0.3/spec.md).
4. **SHOULD** show the `walletUri` only to the person who will answer it, and **SHOULD** stop showing it at `expiresAt` or once [`wallet-session/status`](../../status/0.1/spec.md) reports the session is no longer `awaitingWallet`.

### Consumer (community)

1. **MUST** resolve `criterion` to a criterion it publishes, or one it still honours within that version's `requirementsGrace` ([`manifest/0.3`](../../../manifest/0.3/spec.md) item 5), and refuse otherwise with `criterionNotFound`. It **MUST** refuse with `noWalletPresentation` when that criterion states no credential query a wallet can answer.
2. When `requestId` is supplied, **MUST** confirm the request exists and belongs to the proof signer, and refuse with `requestNotFound` if either fails — the same code for both, so the task cannot be used to probe for other applicants' requests. It **MUST** refuse with `requestClosed` a request of the signer's that has been decided or withdrawn.
3. **MUST** choose the verifier identity named by `clientIdPrefix`, or its default when absent, and refuse with `verifierNotConfigured` when it holds none. The identity is the one OpenID4VP's client identifier prefix names: a DID for `decentralized_identifier`, an X.509 certificate for `x509_hash`. It **SHOULD** be a key dedicated to signing OpenID4VP requests, not the community's own DID.
4. **MUST** bind the session to the proof signer, to the resolved criterion version, and to `requestId` when supplied. Nothing a wallet later sends can change any of the three.
5. **MUST** generate, for the session alone, a single-use `nonce` bound to it and to its own client identifier, and a fresh ephemeral key pair for the encrypted response, whose public key it publishes in the request object's `client_metadata`. It **MUST NOT** reuse either across sessions.
6. **MUST** ask the wallet for the criterion's published query and nothing else: `dcqlQuery` in the response, and `dcql_query` in the request object, are that query unchanged. A community cannot use this task to ask a wallet for more than its manifest told the applicant it would.
7. **MUST** serve the `request_uri` and `response_uri` named in `walletUri` for this session only. It **SHOULD** answer the request object once and refuse a second fetch, and **MUST** accept exactly one wallet response; a session that has received one is closed to every later one.
8. **MUST** close the session at `expiresAt` if no wallet has answered it, and **SHOULD** set `expiresAt` minutes, not hours, after the session opens.
9. **MUST** verify a wallet response as OpenID4VP and the credential's own format require — the key-binding JWT's `nonce` equal to the session nonce and its `aud` equal to `clientId` with its prefix, the holder key the credential's confirmation key, the issuer's signature and status — before recording anything, and **MUST** record the outcome against the session for [`wallet-session/status`](../../status/0.1/spec.md) to read.
10. **MUST NOT** treat the wallet's holder key as the applicant. It records the holder — for example the `did:key` of the credential's `cnf` key — **beside** the applicant DID that signed this task, and never instead of it. The presentation is evidence that the holder of that key holds that credential; it is not evidence that the holder is the applicant.
11. With `requestId`, records the evidence on that request. Without it, **MUST** hold the evidence only for the applicant's next [`submit`](../../../submit/0.3/spec.md) signed by the same DID that names the same criterion (or names none and is decided under it), and only for a period of its own policy, which it **MUST** bound and **SHOULD** keep to hours rather than days; unclaimed evidence is then discarded. [`wallet-session/status`](../../status/0.1/spec.md) reports that deadline as `heldUntil`.
12. **SHOULD** bound how many open sessions one applicant may hold, and refuse beyond it with `tooManySessions`, and **SHOULD** rate-limit this task per applicant.

## Authorization

The authority this task assumes is **being the applicant**: the party seeking to join, who wants evidence collected for their own application. It needs no membership — applicants are not members yet — and no grant. The `proof` does not confer the authority; it names the signer so the consumer can bind the session to them, and it is that binding, held by consumer rule 4, that gives the authority its limits. Only the signer can read the session ([`wallet-session/status`](../../status/0.1/spec.md) answers anyone else `notFound`), only the signer's own join request can receive its evidence (consumer rule 2), and nothing a wallet sends can re-bind it.

The wallet holds no authority here at all. It answers a request the community signed; what it presents is checked as evidence and recorded against the applicant's join, where the community decides under the criterion's `admission` as it would decide any other evidence. A consumer rate-limits the task (consumer rule 12) because opening a session costs it a key, a signed request and two live endpoints, and the task is open to anyone who can sign.

## Definitions

**Presentation session** — the state the community holds for one wallet exchange: the applicant DID, the criterion version, the join request if any, the DCQL query, the nonce, the ephemeral response key, the verifier identity that signs the request object, and, once a wallet answers, the outcome.

**`criterion`** — the `requirementsDigest` of the criterion the evidence is for, as [`manifest/0.3`](../../../manifest/0.3/spec.md) publishes it and [`submit/0.3`](../../../submit/0.3/spec.md) carries it. The digest names one version of the criterion, so the query a wallet is asked to satisfy is the one the applicant read, even if the community changes the criterion while the session is open.

**`requestId`** — a join request of the applicant's, as returned by [`submit`](../../../submit/0.3/spec.md), to record the evidence on.

**`clientIdPrefix`** — the OpenID4VP 1.0 client identifier prefix the community should present itself with: `decentralized_identifier` (a DID; what swiyu wallets expect) or `x509_hash` (the SHA-256 hash of an X.509 certificate; what EUDI wallets expect). The values are OpenID4VP's and are carried verbatim. Absent: the community's default.

**`sessionId`** — an opaque, unguessable identifier for the session, carrying at least 128 bits of entropy. It is what [`wallet-session/status`](../../status/0.1/spec.md) takes.

**`walletUri`** — the OpenID4VP invocation URI, carrying `client_id` and `request_uri` as its query parameters, for the applicant to render as a QR code (cross-device) or open as a deep link (same device). The scheme is the community's choice for the wallet ecosystem — `openid4vp://`, `swiyu-verify://`, `haip-vp://` — and a client renders it without interpreting it.

**`clientId`** — the full client identifier the request object names, prefix included. It is the audience the wallet's key-binding JWT will carry, and is returned so the applicant's agent can show which verifier the wallet will see.

**`dcqlQuery`** — the DCQL query ([OpenID4VP 1.0 §6](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html)) the wallet will be asked to satisfy: the query the criterion publishes as its `presentationDefinition`, unchanged. Opaque to this specification; its members keep DCQL's own snake_case names.

**`expiresAt`** — when the session closes if no wallet has answered it.

## Request

The applicant sends the request to the community, signed; the payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Evidence for a deferred request, from a swiyu wallet

The applicant's request was deferred for want of a credential. They open a session for that criterion, asking the community to identify itself by DID.

```json
{
  "id": "urn:uuid:3c1f9a52-6e4b-4d8a-9b27-5f0e8c1d7a43",
  "type": "https://trusttasks.org/spec/vtc/join-requests/wallet-session/start/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-11T09:00:00Z",
  "threadId": "urn:uuid:7b2e4d91-0c3a-4f65-8e1d-2a9c6b5f0e17",
  "payload": {
    "criterion": "zQmaf3wtkysZWg8vCzdetBwzjiHACZz8JAxTVr4W66chiXV",
    "requestId": "6f1d2c3b-9a4e-4b8f-a7d6-0e5c1b2a3f49",
    "clientIdPrefix": "decentralized_identifier"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-11T09:00:00Z",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKiZsQoXEhTaD3XJhN"
  }
}
```

## Response

The community answers with the session; the payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). `sessionId` is what the applicant polls; `walletUri` is what they show; `clientId` and `dcqlQuery` say which verifier the wallet will see and what it will be asked for; `expiresAt` is when the session closes unanswered. A refusal is a `trust-task-error` document carrying one of the codes declared above, not a `#response`.

### The session, ready for a wallet

```json
{
  "id": "urn:uuid:9e4a7c13-2b5d-4f80-a6c9-1d3e8f0b2c74",
  "type": "https://trusttasks.org/spec/vtc/join-requests/wallet-session/start/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-11T09:00:01Z",
  "threadId": "urn:uuid:7b2e4d91-0c3a-4f65-8e1d-2a9c6b5f0e17",
  "payload": {
    "sessionId": "q7Xw2LmN9pRt4VbK8sYc3Hd6",
    "walletUri": "swiyu-verify://?client_id=decentralized_identifier%3Adid%3Awebvh%3AQmVerifierScid%3Averifier.community.example&request_uri=https%3A%2F%2Fcommunity.example%2Fv1%2Foid4vp%2Frequest%2Fq7Xw2LmN9pRt4VbK8sYc3Hd6",
    "clientId": "decentralized_identifier:did:webvh:QmVerifierScid:verifier.community.example",
    "dcqlQuery": {
      "credentials": [
        {
          "id": "residence",
          "format": "dc+sd-jwt",
          "meta": { "vct_values": ["https://example.org/credentials/residence-permit"] },
          "claims": [ { "path": ["address", "country"] } ]
        }
      ]
    },
    "expiresAt": "2026-10-11T09:10:01Z"
  }
}
```

### Refused — no identity for the requested prefix

The community verifies only with an X.509 access certificate, and the applicant asked for a DID.

```json
{
  "id": "urn:uuid:5d8b1e27-4c6a-4f93-b0e2-7a1c9d3f6b58",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-11T09:00:01Z",
  "threadId": "urn:uuid:7b2e4d91-0c3a-4f65-8e1d-2a9c6b5f0e17",
  "payload": {
    "code": "vtc/join-requests/wallet-session/start:verifierNotConfigured",
    "message": "This community has no verifier identity for client identifier prefix decentralized_identifier.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a criterion digest the community published, optionally a join-request identifier the community minted, and optionally a client identifier prefix: nothing about the applicant that the community did not already hold or publish. The applicant's identity is the proof's signer and is not repeated in the payload.

The response carries the community's own policy and plumbing. `dcqlQuery` is the criterion's published query; `clientId` is the verifier identity the community presents to every wallet; `walletUri` combines that identity with a session-specific `request_uri`; `sessionId` and `expiresAt` describe the session. None of it is data about a natural person. `sessionId` is the one member that is sensitive in the ordinary sense — it is the handle the applicant polls under, and appears in the `request_uri` — and a client **SHOULD** treat it as it treats the QR code: shown to the person answering, not logged or forwarded.

Personal data enters later, from the wallet, over OpenID4VP and not over this task: the presented claims, and the holder key that held them. Where the community keeps them, and what of them [`wallet-session/status`](../../status/0.1/spec.md) returns (no claim values), is stated there. A producer **MUST NOT** put personal data in `ext`; this task has no use for any.

### Correlation

The applicant declares `identifierScope: pairwise`, for the reason [`submit`](../../../submit/0.3/spec.md) gives: the identifier needs to persist for the life of this application and no further. The community declares `public` because the applicant must be able to confirm it is opening a session with the community whose manifest it read, and a pairwise community identifier would leave it unable to.

The session is a deliberate join between two identifiers that otherwise have nothing in common: the applicant's DID, and the wallet's holder key. That join is the point of the task, and consumer rule 10 limits what it means — the community learns that this applicant had a session answered by that key, and records both, never one in place of the other. A holder key that is stable across verifiers (a hardware-bound key a wallet reuses) lets the community recognise the same wallet in a later session, by this applicant or another. That is a property of the wallet's key management, not of this task, and the reason [`wallet-session/status`](../../status/0.1/spec.md) returns the holder only to the applicant who began the session.

The QR code is the correlation risk particular to cross-device presentation. Anyone who sees `walletUri` — over a shoulder, on a shared screen, in a screenshot — can answer it with their own wallet, and the community will receive a valid presentation of somebody else's credential into the applicant's session. OpenID4VP cannot prevent this: the wallet and the applicant's agent share no key, and the session is exactly as strong as the secrecy of the URI. The mitigations are those this specification states and those a community may add. Sessions expire in minutes and accept one response (consumer rules 7 and 8), so a relayed QR code is a narrow window, and a second answer cannot overwrite the first. The request object shows the wallet the community and the purpose, so a person answering a code they did not expect sees whom they are presenting to. And a community **MAY** refer wallet evidence to review rather than admit on it automatically, or require a claim to match something the applicant has declared — both criterion policy, outside this task. Applicants **SHOULD** prefer the same-device flow where the wallet is on the device the applicant is using.

### Retention

The session lives minutes: until a wallet answers or `expiresAt` passes. Its ephemeral private key **MUST** be deleted when it closes, so a response captured in transit can no longer be decrypted once the session is gone; its nonce is single-use and has no value after. A community keeps the session record no longer than [`wallet-session/status`](../../status/0.1/spec.md) needs to report its outcome, which is a matter of its own policy and **SHOULD** be short.

The evidence the session yields is not this task's to retain. With `requestId` it is recorded on the join request and lives as long as the community keeps that request's record — the record it would keep of any other evidence on which it decided. Without `requestId` it is held for the applicant's next submission for a bounded period (consumer rule 11) and discarded if unclaimed; evidence nobody submitted is evidence nobody decided on, and has no reason to persist.

### Consent/purpose

The purpose is narrow: to collect, from the applicant's wallet, the evidence one published criterion asks for, so the community can decide the applicant's join under it. Consumer rule 6 is the limit: the wallet is asked for the criterion's published query unchanged, so the community cannot ask the wallet for more than its manifest told the applicant it would, and an applicant who read the manifest knows before scanning exactly what the wallet will be asked to disclose.

The wallet asks its holder's agreement before presenting anything; that is OpenID4VP's and the wallet's, and this task neither replaces nor relies on it beyond receiving its result. Whether a community requires anything further before acting on wallet evidence — review, a matching declaration, a human decision — is its own policy, expressed in the criterion's `admission` and outside this specification. The evidence is for the join it was collected for and **SHOULD NOT** be reused for any other purpose.
