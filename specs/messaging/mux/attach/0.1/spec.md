---
slug: messaging/mux/attach
version: "0.1"
title: "Messaging — Mux Attach"
summary: "A principal attaches member DIDs to its authenticated connection for this session only, each by a statement the member DID signs with an embedded Data Integrity proof, bound to that one connection; every member is answered on its own."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, attach, session, websocket]
parties:
  - role: Principal (the DID the connection is authenticated as)
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
  - role: Member DID (each attach statement's issuer)
    requirement: REQUIRED
    identifierScope: any
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    An attach exercises each member DID's authority on the principal's behalf (SPEC §7.3 item 8). The authority for each member is that member's own signed attach statement, verified against its DID document; the document proof binds the request — which members, which scopes — to the principal in a form that survives the connection, so an operator auditing who attached what does not depend on transport logs. The response is a per-member outcome the principal acts on immediately.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    An attach changes who receives a DID's live traffic, which makes the task consequential (SPEC §7.3 item 17). Each attach statement is already bound to one connection's nonce, so a replay elsewhere fails; issuedAt bounds a replay on the same connection.
sideEffects:
  level: mutating
  rationale: >-
    Adds members to the connection's member set and routes their live traffic, pickup and (per scope) deletes and sends through it. Reversible: an attach ends on messaging/mux/detach, at its expiry, or when the connection closes, and it changes nothing stored.
consequences:
  - Live delivery for each attached DID moves to this connection; if the DID has its own direct connection, that one keeps precedence and the attach is held inactive.
  - With `delete` in scope, messages the principal collects for the member are deleted from the member's inbox on acknowledgement.
  - The first use of a member DID key that signed none of its active stored grants suspends those grants (the unknown-key rule), and their principals are told to recreate them.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Each attached member's authority is exercised: the principal then receives, collects, deletes and (per scope) sends that DID's traffic as if it were the DID's own connection. The request tells the mediator that the principal and every member DID are held by one party on one device — a correlation that pairwise and peer DIDs exist to prevent (see Correlation). The response discloses only the outcome per member.
retention:
  class: exchange
  rationale: >-
    An attach lives for the connection it is bound to and ends no later than its statement's expiry; nothing of it survives the connection. A mediator's audit record of the attach is its own policy.
maxDocumentBytes:
  request: 4194304
  rationale: >-
    Up to 256 attach statements, each a closed object of a few DIDs, a nonce, a scope and an embedded Data Integrity proof — typically under 1.5 KiB, bounded well below 16 KiB.
errorCodes:
  - code: messaging/mux:notMuxConnection
    meaning: "The request did not arrive on a mux connection \u2014 one the mediator has announced with a `ready` event \u2014 so there is no `sessionId` or `muxNonce` an attach could bind to. Attach over the live connection the members should ride. A whole-request error."
    retryable: false
  - code: messaging/mux:proofPurposeInvalid
    meaning: "Per attachment, in `results[].code`: the embedded proof's `proofPurpose` is not `authentication`."
    retryable: false
  - code: messaging/mux:statementProofInvalid
    meaning: "Per attachment: the embedded Data Integrity proof does not verify \u2014 a bad signature, a verification method not controlled by the statement's `issuer`, or one not in the issuer's current `authentication` relationship."
    retryable: false
  - code: messaging/mux:didUnresolvable
    meaning: "Per attachment: the member DID could not be resolved now, so its proof could not be checked."
    retryable: true
  - code: messaging/mux:audienceMismatch
    meaning: "Per attachment: the statement's `audience` is not this mediator's DID."
    retryable: false
  - code: messaging/mux:principalMismatch
    meaning: "Per attachment: the statement's `principal` is not the DID this connection is authenticated as, or equals the statement's `issuer`."
    retryable: false
  - code: messaging/mux/attach:sessionMismatch
    meaning: "Per attachment: the statement's `sessionId` or `muxNonce` is not this connection's."
    retryable: false
  - code: messaging/mux:notYetValid
    meaning: "Per attachment: the statement's `iat` is more than 60 seconds in the mediator's future."
    retryable: false
  - code: messaging/mux:statementExpired
    meaning: "Per attachment: the statement's `exp` has passed."
    retryable: false
  - code: messaging/mux:lifetimeTooLong
    meaning: "Per attachment: the statement's `exp` is later than the connection's lifetime, or than `iat` plus the mediator's attach maximum."
    retryable: false
  - code: messaging/mux:emptyScope
    meaning: "Per attachment: the signed scope is empty. An empty scope is refused, never read as \"all\"."
    retryable: false
  - code: messaging/mux:accountUnavailable
    meaning: "Per attachment: the member DID has no account at this mediator, or it is blocked."
    retryable: false
  - code: messaging/mux:scopeNotPermitted
    meaning: "Per attachment: the scope exceeds the member DID's own access-control capabilities."
    retryable: false
  - code: messaging/mux/attach:memberQuota
    meaning: "Per attachment: attaching would exceed the members this connection may carry (`maxDidsPerConnection`)."
    retryable: true
related:
  - messaging/mux/detach
  - messaging/mux/grant
  - messaging/mux/list
  - messaging/mux/event
  - messaging/mux/notice
---

## Abstract

The **Messaging — Mux Attach** Trust Task lets one authenticated connection to a mediator — the *principal*'s — carry live delivery and pickup for other DIDs, its *members*, for the rest of that connection's life. Each member signs a short-lived **attach statement** bound to this connection's `sessionId` and `muxNonce`, both announced in the [`messaging/mux/event`](../../event/0.1/spec.md) `ready` event, with an embedded W3C Data Integrity proof. A statement for one connection is therefore useless on any other. The batch is answered member by member, never all-or-nothing.

An attach is the session-only half of DID multiplexing. Its stored counterpart, [`messaging/mux/grant`](../../grant/0.1/spec.md), survives reconnects. An attach costs one round trip per reconnect and leaves nothing stored, which makes it the right choice when a member DID's key is at hand on every connect and its owner does not want a standing delegation.

Each member keeps its own inbox, access-control capabilities, quotas and rate limits. Mux shares the connection, not the account.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.2–§4.3 and §5).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The attach statement

An attach statement is a closed JSON object, [`AttachStatement`](../../_shared/0.1/mux.schema.json#/$defs/AttachStatement), signed by the member DID with an embedded W3C Data Integrity proof ([`MuxDataIntegrityProof`](../../_shared/0.1/mux.schema.json#/$defs/MuxDataIntegrityProof); cryptosuite `eddsa-jcs-2022` or `eddsa-rdfc-2022`) — the same proof format, and the same verifier, Trust Tasks use for document proofs ([SPEC §4.7](/SPEC.md#47-proof)):

```json
{
  "type": "muxAttach",
  "issuer": "<member DID>",
  "audience": "<mediator DID>",
  "principal": "<principal DID>",
  "sessionId": "<from the ready event>",
  "muxNonce": "<from the ready event>",
  "scope": ["receive", "pickup", "delete"],
  "iat": 1794214800,
  "exp": 1794218400,
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "<member DID>#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z…"
  }
}
```

The proof's `proofPurpose` **MUST** be `authentication`: an attach proves control of the member DID, and a signature made for any other relationship is never accepted as that. `iat` and `exp` are JWT NumericDate values ([RFC 7519 §2](https://www.rfc-editor.org/rfc/rfc7519#section-2)), whole seconds, because the signature covers them exactly; the Trust Task payload elsewhere carries instants as RFC 3339 as usual.

A conforming **producer** (the principal, holding each member's statement) **MUST**:

1. Send the request on the mux connection the members are to ride, after that connection's `ready` event, with itself as `issuer` and the mediator as `recipient`.
2. For each member, obtain an attach statement signed by a key in the member DID's **current** `authentication` relationship, with `sessionId` and `muxNonce` copied from this connection's `ready` event, `principal` itself, `audience` the mediator, and `exp` no later than the connection's lifetime (`sessionExpiresAt`) and `iat` plus `maxAttachLifetimeSeconds` where the `ready` event states them.
3. Name each member's scope explicitly. There is no default scope; order is not significant and a repeated value collapses.
4. Name each member DID at most once per request, and never the principal itself.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled. Mux is off unless the operator turns it on.
2. Refuse the whole request with `messaging/mux:notMuxConnection` when it did not arrive on a mux connection.
3. Check the statements' **shape** and **type** as part of validating the payload ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): each is a closed `AttachStatement` whose `type` is `muxAttach`. A statement that fails is a malformed request, answered with `malformedRequest` for the whole document, since nothing in it can be trusted to name a member.
4. Check each statement independently, **in this order**, stop at the first failure, and report that failure's code as that member's `results[].code`:
   1. **Proof purpose** — `proof.proofPurpose` is `authentication`. Otherwise `messaging/mux:proofPurposeInvalid`.
   2. **Data Integrity verification** — the proof verifies over the statement under its cryptosuite; its `verificationMethod` is controlled by the statement's `issuer`; and that method is listed in the issuer's **current** `authentication` relationship, resolved now. Otherwise `messaging/mux:statementProofInvalid`, or `messaging/mux:didUnresolvable` when the issuer cannot be resolved now.
   3. **Audience** — `audience` is this mediator's DID. Otherwise `messaging/mux:audienceMismatch`.
   4. **Principal and connection** — `principal` is the DID this connection is authenticated as and differs from `issuer` (otherwise `messaging/mux:principalMismatch`); `sessionId` and `muxNonce` are this connection's (otherwise `messaging/mux/attach:sessionMismatch`).
   5. **Validity window** — `iat` is no more than 60 seconds in the future (otherwise `messaging/mux:notYetValid`); `exp` is in the future (otherwise `messaging/mux:statementExpired`); `exp` is no later than the connection's lifetime or `iat` plus the attach maximum (otherwise `messaging/mux:lifetimeTooLong`).
   6. **Account and scope** — the scope is not empty (otherwise `messaging/mux:emptyScope`); the member DID has an account here and it is not blocked (otherwise `messaging/mux:accountUnavailable`); the scope is within the member's own access-control capabilities (otherwise `messaging/mux:scopeNotPermitted`).
   7. **Limits** — attaching would not exceed the members this connection may carry (`maxDidsPerConnection`). Otherwise `messaging/mux/attach:memberQuota`.
5. Only after a member passes every check, register it with the connection, and then apply the **unknown-key rule**: where the member DID has active stored grants and none of them was signed by this statement's verification method, suspend every active grant on that DID — a terminal state, not an epoch advance — as [`messaging/mux/grant`](../../grant/0.1/spec.md#the-unknown-key-rule) defines. A DID with no active grants never triggers it.
6. Treat an attach for a DID already attached by a session attach on this connection as replacing it (new scope and expiry). A DID that is also a member through a stored grant keeps both: its effective scope is the union of the two, and each lapses on its own expiry.
7. Where the member DID has its own direct connection, accept the attach but hold it inactive (`deliveryActive: false`): the direct connection keeps the member's live traffic. When it ends, activate the attach and emit an `attached` event with reason `reactivated`.
8. Respond with one [`AttachOutcome`](#response) per statement, in request order, and emit one batched `attached` [`messaging/mux/event`](../../event/0.1/spec.md) listing the members attached, with reason `attach`.
9. Detach a member when its attach's `exp` passes, emitting `detached` with reason `expired`. A member's expiry never closes the connection; only the principal's own authentication expiry does.

## Authorization

**Authority: each member DID's own signature, bound to this connection.** The principal is entitled to a member's traffic because the member DID, with a key currently in its `authentication` relationship, signed a statement naming this principal, this mediator and this one connection's nonce, for a bounded time and an explicit scope. The connection authenticates only the principal ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); it carries no authority over any member. The document `proof` likewise identifies only the principal: two Data Integrity proofs are in play, and only the one embedded in each statement carries authority, and only over that statement's member. Consumer rule 4 enforces the entitlement, in the order that keeps verification ahead of any policy lookup and every check ahead of any state change.

The member's authority is bounded twice: by the scope it signed, and by its own access-control capabilities at the mediator, which a delegation can never exceed. Nothing about a member is inferred from the principal's standing.

## Definitions

- **Principal** — the DID the connection is authenticated as. It stays the authenticated identity; members never are.
- **Member DID** — a DID whose traffic rides the principal's connection; the `issuer` of its statement.
- **Mux connection** — an authenticated connection the mediator has announced with a `ready` event carrying its `sessionId` and `muxNonce`. How a connection becomes one is the transport's concern: on a raw-TSP socket, negotiating the `tsp-mux` subprotocol ([`messaging/mux/event`](../../event/0.1/spec.md#raw-tsp-sockets)).
- **`attachments`** — REQUIRED. 1–256 [`AttachStatement`](../../_shared/0.1/mux.schema.json#/$defs/AttachStatement)s.
- **`results`** (response) — one `AttachOutcome` per statement: `did` (its issuer), `status` (`attached` | `refused`), and for `attached` the `scope` in force, `expiresAt`, and `deliveryActive`; for `refused`, `code`.

## Request

The principal sends the request to the mediator on the mux connection the members will ride. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Attach two member DIDs

```json
{
  "id": "urn:uuid:4f1c2a7e-6b3d-4e8a-9c10-2d3e4f5a6b01",
  "type": "https://trusttasks.org/spec/messaging/mux/attach/0.1",
  "issuer": "did:webvh:QmPrinc1pal:phone.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T09:00:05Z",
  "payload": {
    "attachments": [
      {
        "type": "muxAttach",
        "issuer": "did:webvh:QmA1ice:alice.example:work",
        "audience": "did:web:mediator.example",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "sessionId": "ses_7Hq2xV9mK3pL",
        "muxNonce": "Zk3xQ0b9Rw7mT2pN4vYc8LsH1eJ6aUdG5oWiKfB0nXq",
        "scope": [
          "receive",
          "pickup",
          "delete"
        ],
        "iat": 1794214800,
        "exp": 1794218400,
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-1",
          "proofPurpose": "authentication",
          "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
        }
      },
      {
        "type": "muxAttach",
        "issuer": "did:peer:2.Ez6LSexamplePeerDid",
        "audience": "did:web:mediator.example",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "sessionId": "ses_7Hq2xV9mK3pL",
        "muxNonce": "OtherConnectionNonce00000000000000000000000",
        "scope": [
          "receive"
        ],
        "iat": 1794214800,
        "exp": 1794218400,
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:peer:2.Ez6LSexamplePeerDid#key-1",
          "proofPurpose": "authentication",
          "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
        }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmPrinc1pal:phone.example#key-1",
    "created": "2026-11-09T09:00:05Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

The statement for Alice's work DID is signed by that DID's own `#key-1`; the document proof is the principal's. The proof values in these examples are placeholders.

## Response

The mediator responds with one outcome per attachment, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. A refused member is an outcome, not an error. Failures of the request as a whole use `trust-task-error` rather than a `#response` document.

### One member attached, one refused

```json
{
  "id": "urn:uuid:4f1c2a7e-6b3d-4e8a-9c10-2d3e4f5a6b02",
  "type": "https://trusttasks.org/spec/messaging/mux/attach/0.1#response",
  "threadId": "urn:uuid:4f1c2a7e-6b3d-4e8a-9c10-2d3e4f5a6b01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:00:05Z",
  "payload": {
    "results": [
      {
        "did": "did:webvh:QmA1ice:alice.example:work",
        "status": "attached",
        "scope": ["receive", "pickup", "delete"],
        "expiresAt": "2026-11-09T10:00:00Z",
        "deliveryActive": true
      },
      {
        "did": "did:peer:2.Ez6LSexamplePeerDid",
        "status": "refused",
        "code": "messaging/mux/attach:sessionMismatch"
      }
    ]
  }
}
```

The second statement was signed for a different connection's `muxNonce`.

### Refused: not a mux connection

```json
{
  "id": "urn:uuid:4f1c2a7e-6b3d-4e8a-9c10-2d3e4f5a6b03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:4f1c2a7e-6b3d-4e8a-9c10-2d3e4f5a6b01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:00:05Z",
  "payload": {
    "code": "messaging/mux:notMuxConnection",
    "message": "This connection was not opened as a mux connection.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries one attach statement per member: the member DID, the mediator, the principal, the connection's `sessionId` and `muxNonce`, the scope and a validity window, with the member's Data Integrity proof. A statement is signed, not secret: it is useless on any other connection and lapses at `exp`. The response carries an outcome per member. No message content and no key material are carried. A producer **MUST NOT** place anything but extension data the mediator's ecosystem defines in `ext`.

### Correlation

This task exists to tell the mediator that several DIDs are held by one party: every member is linked to the principal and to each other, from the moment of the attach. For pairwise or peer DIDs that link is the very thing they exist to hide, and the mediator learns it as surely as if the DIDs shared a key. Mitigations, all the producer's choice: attach only DIDs whose owner opted into mux for this purpose; spread DIDs over several connections with different principals (*mux groups*), so the mediator sees several smaller groups rather than one; and prefer short `exp` values. Grant and attach listings are visible only to the two parties to them and to the operator. The mediator's identifier scope is public by design: it is the address members, principals and their peers already use to reach it, and every proof must name it as `aud`. An observer of the connection learns batch size and timing only.

### Retention

The attach lives for the connection and no longer than its statement's `exp`; the mediator keeps no record of it after the connection closes, beyond whatever audit log its operator keeps. A principal that keeps a member's attach statement gains nothing by it: the statement is bound to one connection's nonce, which is never reused.

### Consent/purpose

The purpose is connection sharing: cheaper reconnects, fewer keepalives and one reconnect instead of many, for DIDs whose owner chose to have them carried together. Mux is opted into at every layer and nothing is attached by default. Using the member set the mediator learns for anything but routing those DIDs' traffic — profiling, linking accounts for another purpose — is outside this task's purpose. Whether an operator enables mux at all is its policy, not this specification's.
