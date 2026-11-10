---
slug: messaging/mux/grant
version: "0.1"
title: "Messaging — Mux Grant"
summary: "Store or replace stored grants: a member DID signs once that a principal may carry its messages, bound to the member's current key and epoch, with an explicit scope and expiry; reconnects then cost one authentication."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, grant, delegation, websocket]
parties:
  - role: Submitter (the member DID, or a principal carrying the member's grant)
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
  - role: Member DID (signs each grant)
    requirement: REQUIRED
    identifierScope: any
  - role: Principal (each grant's delegate)
    requirement: REQUIRED
    identifierScope: any
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    A stored grant lets a principal exercise a member DID's authority over its messages on every future connection (SPEC §7.3 item 8). Each grant statement's embedded proof is the member's authority, and the statement is stored verbatim; the document proof attributes the submission, so the mediator's record of who lodged a delegation does not rest on transport logs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Storing a grant creates durable state, which makes the task consequential (SPEC §7.3 item 17). A grant statement's epoch and the replay floor already defeat a replay of the statement; issuedAt bounds a replay of the submission.
sideEffects:
  level: mutating
  rationale: >-
    Creates or replaces stored grants, attaches the member at once where the principal is connected, and may suspend other grants on the member DID under the unknown-key rule. Reversible: a grant ends at its expiry or on messaging/mux/revoke or messaging/mux/revoke-all.
consequences:
  - The principal receives, collects and (per scope) deletes or sends the member DID's messages on every connection it opens, until the grant expires or is revoked.
  - A grant for the same member and principal replaces the earlier one, in whatever state it was, provided its `iat` is newer; the replaced statement can never be stored again.
  - A grant signed by a key that signed none of the member DID's active grants suspends those grants; their principals are told durably to recreate them.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Each stored grant exercises the member DID's authority: the principal will act on the member's messages in its name. Storing it tells the mediator, durably, that the member and the principal are held by one party — a correlation that pairwise and peer DIDs exist to prevent (see Correlation). The response discloses per-grant outcomes.
retention:
  class: durable
  rationale: >-
    A grant is a standing entitlement: the mediator keeps it, with its signed statement verbatim as the only proof it relies on, while it is active, and a dead grant (suspended, stale, expired, epoch-revoked) for a retention window after its effective expiry so both parties can list it. A revoked or replaced grant is deleted, behind a replay floor that keeps its statement from being stored again.
maxDocumentBytes:
  request: 4194304
  rationale: >-
    Up to 256 grant statements, each a closed object of a few DIDs, a scope, an epoch and an embedded Data Integrity proof — typically under 1.5 KiB.
errorCodes:
  - code: messaging/mux:proofPurposeInvalid
    meaning: "Per grant, in `results[].code`: the embedded proof's `proofPurpose` is not `authentication`."
    retryable: false
  - code: messaging/mux:statementProofInvalid
    meaning: "Per grant, in `results[].code`: the embedded Data Integrity proof does not verify \u2014 a bad signature, a verification method not controlled by the statement's `issuer`, or one not in the issuer's current `authentication` relationship."
    retryable: false
  - code: messaging/mux:didUnresolvable
    meaning: "Per grant, in `results[].code`: the member DID could not be resolved now, so the grant could not be checked."
    retryable: true
  - code: messaging/mux:audienceMismatch
    meaning: "Per grant, in `results[].code`: the statement's `audience` is not this mediator's DID."
    retryable: false
  - code: messaging/mux:principalMismatch
    meaning: "Per grant, in `results[].code`: the statement's `principal` equals its `issuer`."
    retryable: false
  - code: messaging/mux:notYetValid
    meaning: "Per grant, in `results[].code`: the statement's `iat` is more than 60 seconds in the mediator's future."
    retryable: false
  - code: messaging/mux:statementExpired
    meaning: "Per grant, in `results[].code`: the statement's `exp` has passed."
    retryable: false
  - code: messaging/mux:lifetimeTooLong
    meaning: "Per grant, in `results[].code`: `exp \u2212 iat` exceeds the mediator's maximum grant lifetime (`maxGrantLifetimeSeconds`)."
    retryable: false
  - code: messaging/mux:emptyScope
    meaning: "Per grant, in `results[].code`: the signed scope is empty. An empty scope is refused, never read as 'all'."
    retryable: false
  - code: messaging/mux:accountUnavailable
    meaning: "Per grant, in `results[].code`: the member DID has no account at this mediator, or it is blocked."
    retryable: false
  - code: messaging/mux:scopeNotPermitted
    meaning: "Per grant, in `results[].code`: the signed scope exceeds the member DID's own access-control capabilities."
    retryable: false
  - code: messaging/mux/grant:epochMismatch
    meaning: "Per grant, in `results[].code`: the statement's `epoch` is not the member DID's current epoch. The outcome carries `currentEpoch`; the member re-signs under it."
    retryable: false
  - code: messaging/mux/grant:replayed
    meaning: "Per grant, in `results[].code`: the statement's `iat` is not strictly later than the principal's replay floor for this member, or than the `iat` of the grant it would replace \u2014 a revoked, replaced or superseded statement submitted again. The outcome carries `replayFloorAt`; the member re-signs with a later `iat`."
    retryable: false
  - code: messaging/mux/grant:grantIdReused
    meaning: "Per grant, in `results[].code`: the statement's `grantId` is already on the member DID's record."
    retryable: false
  - code: messaging/mux/grant:didQuota
    meaning: "Per grant, in `results[].code`: storing the grant would exceed the active grants allowed on the member DID."
    retryable: false
  - code: messaging/mux/grant:principalQuota
    meaning: "Per grant, in `results[].code`: storing the grant would exceed the grants allowed per principal."
    retryable: true
  - code: messaging/mux/grant:contended
    meaning: "Per grant, in `results[].code`: the member DID's record changed under concurrent writes too many times for the grant to be committed. Nothing was stored; resubmit the same statement."
    retryable: true
related:
  - messaging/mux/attach
  - messaging/mux/list
  - messaging/mux/update
  - messaging/mux/revoke
  - messaging/mux/revoke-all
  - messaging/mux/notice
  - messaging/mux/event
---

## Abstract

The **Messaging — Mux Grant** Trust Task stores **grants**. A grant is a signed **grant statement** in which a member DID states that a named principal may carry its messages at this mediator: live delivery, pickup, deletes and sends, as its scope says, until its expiry. Once stored, every connection the principal authenticates attaches the member automatically ([`messaging/mux/event`](../../event/0.1/spec.md) `attached`, reason `connect`), so a reconnect costs one authentication however many members ride it. A session attach ([`messaging/mux/attach`](../../attach/0.1/spec.md)) costs one round trip per member per reconnect instead.

The statement's own Data Integrity proof is what counts, not who carries it: the member can submit its grants itself, or hand them to the principal to submit. Grants are bound to the member DID's **current** key state and to its **epoch** at the mediator, so a DID that changes hands, or whose keys rotate, stops flowing to the old principal without anyone remembering to revoke. Storing a grant for the same member and principal again **replaces** the earlier one. That is how a grant is renewed or widened: always with a fresh member signature.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.4–§4.5, §5 and §6).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The grant statement

A grant is a closed JSON object, [`GrantStatement`](../../_shared/0.1/mux.schema.json#/$defs/GrantStatement), signed by the member DID with an embedded W3C Data Integrity proof ([`MuxDataIntegrityProof`](../../_shared/0.1/mux.schema.json#/$defs/MuxDataIntegrityProof); cryptosuite `eddsa-jcs-2022` or `eddsa-rdfc-2022`) — the same proof format, and the same verifier, Trust Tasks use for document proofs ([SPEC §4.7](/SPEC.md#47-proof)):

```json
{
  "type": "muxGrant",
  "issuer": "<member DID>",
  "audience": "<mediator DID>",
  "grantId": "<UUID recommended>",
  "principal": "<principal DID>",
  "scope": ["receive", "pickup"],
  "epoch": 3,
  "iat": 1794214800,
  "exp": 1796806800,
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "<member DID>#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z…"
  }
}
```

The proof's `proofPurpose` **MUST** be `authentication`. `iat` and `exp` are JWT NumericDate values ([RFC 7519 §2](https://www.rfc-editor.org/rfc/rfc7519#section-2)), whole seconds, because the signature covers them exactly. `exp` is REQUIRED; a grant without one does not validate. `epoch` is the member DID's current epoch at the mediator, read from [`messaging/mux/list`](../../list/0.1/spec.md) as `member` (0 for a DID that never advanced it).

A conforming **producer** **MUST**:

1. Sign each grant with a key in the member DID's **current** `authentication` relationship, under its current epoch, with a fresh `grantId`, an explicit scope (there is no default), and an `exp` no more than the mediator's `maxGrantLifetimeSeconds` after `iat`.
2. Renew a grant only when its application opted into renewal for that grant; otherwise let it lapse. Re-signing at about three quarters of the lifetime is RECOMMENDED for a grant that is renewed.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled.
2. Check the statements' **shape** and **type** as part of validating the payload ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): each is a closed `GrantStatement` whose `type` is `muxGrant`. A statement that fails is a malformed request, answered with `malformedRequest` for the whole document.
3. Check each statement independently, **in this order**, stop at the first failure, and report that failure's code as that grant's `results[].code`:
   1. **Proof purpose** — `proof.proofPurpose` is `authentication`. Otherwise `messaging/mux:proofPurposeInvalid`.
   2. **Data Integrity verification** — the proof verifies over the statement under its cryptosuite; its `verificationMethod` is controlled by the statement's `issuer`; and that method is listed in the issuer's **current** `authentication` relationship, resolved now. Otherwise `messaging/mux:statementProofInvalid`, or `messaging/mux:didUnresolvable` when the issuer cannot be resolved now.
   3. **Audience** — `audience` is this mediator's DID. Otherwise `messaging/mux:audienceMismatch`.
   4. **Principal** — `principal` differs from `issuer`. Otherwise `messaging/mux:principalMismatch`. (Who submits the grant is not checked: the statement names its principal.)
   5. **Validity window** — `iat` is no more than 60 seconds in the future (otherwise `messaging/mux:notYetValid`); `exp` is in the future (otherwise `messaging/mux:statementExpired`); `exp − iat` is at most `maxGrantLifetimeSeconds` (otherwise `messaging/mux:lifetimeTooLong`).
   6. **Account and scope** — the scope is not empty (otherwise `messaging/mux:emptyScope`); the member DID has an account here and it is not blocked (otherwise `messaging/mux:accountUnavailable`); the scope is within the member's own access-control capabilities (otherwise `messaging/mux:scopeNotPermitted`).
   7. **Store rules**, against the member DID's record:
      1. **Epoch** — `epoch` equals the member DID's current epoch. Otherwise `messaging/mux/grant:epochMismatch`, with `currentEpoch` in the outcome.
      2. **Replay** — `iat` is strictly later than the principal's [replay floor](#replay-floors) for this member DID, and strictly later than the `iat` of any stored grant for the same member and principal. Otherwise `messaging/mux/grant:replayed`, with `replayFloorAt` — the later of the two — in the outcome.
      3. **Identifier** — `grantId` is not already on the member DID's record. Otherwise `messaging/mux/grant:grantIdReused`.
      4. **Quotas** — storing it would not exceed the active grants allowed per member DID (otherwise `messaging/mux/grant:didQuota`) or the grants allowed per principal (otherwise `messaging/mux/grant:principalQuota`), counting a grant it replaces once. The per-principal quota is a resource bound, not an authorization gate: concurrent grants to one principal may overshoot it by the number racing.
      5. **Unknown key** — apply [the unknown-key rule](#the-unknown-key-rule).
4. Only after a grant passes every check, commit it atomically with every other change to the member DID's record — the unknown-key suspensions included — storing the grant as `active` with its verification method, the key's fingerprint, the DID version it was signed under where the method has one (a did:webvh `versionId`, a did:web document hash), and the statement verbatim, proof included. A stored grant for the same member and principal is replaced, whatever its state, its identifier is returned as `replacedGrantId`, and the principal's replay floor rises to the replaced grant's `iat`. Where the record keeps changing under concurrent writes so that the commit cannot complete after a bounded number of attempts, store nothing and report `messaging/mux/grant:contended`.
5. Where the principal holds a connection — on any node of a clustered mediator — attach the member to it at once, emit `attached` with reason `grant`, and report `attachedNow: true`. A connection that already carries `maxDidsPerConnection` members does not take the member: the grant stays stored, and that connection is sent a `detached` event for it with reason `connectionFull`. `attachedNow` is `true` only when some connection took the member.
6. Respond with one `GrantOutcome` per grant, in request order. A refused grant is an outcome, not an error: the batch is never all-or-nothing.

While a grant is stored, the mediator **MUST** re-validate it on every connect of its principal, on a timer while it is attached (RECOMMENDED every 900 seconds, through the resolver cache), and on any revocation affecting the member DID. A grant whose signing key has left the member DID's current `authentication` relationship moves to `staleKey`, and its member is detached with reason `keyRotated`. It **MUST NOT** return to `active` if the key reappears: rotating a key away and back does not re-authorise an old grant. A change of DID version with the key still present does not invalidate the grant; it only forces the re-check. A grant that fails a check on connect is reported in that connection's `detached` event with its reason. An active grant past its effective expiry is reported `expired`; that state is computed when the grant is read, never stored. Every dead grant — `suspended`, `staleKey`, `expired`, `epochRevoked` — stays listable for the mediator's retention window after its effective expiry (RECOMMENDED 7 days), and is then pruned. Removing either party's account removes the grants between them.

Dead grants are never resumed. The member signs a new grant under its current epoch, with an `iat` later than the replay floor, which replaces the old one.

### Replay floors

The mediator keeps, per member DID and principal, a **replay floor**: an instant at or before which no grant for that pair is accepted. It rises to the moment of revocation when a grant is revoked with [`messaging/mux/revoke`](../../revoke/0.1/spec.md), and to the replaced grant's `iat` when a grant is replaced. Together with the rule that a replacement's `iat` must be strictly later than the grant it replaces, this makes every revoked, replaced or superseded statement unusable, whatever its `grantId`: replaying it is `messaging/mux/grant:replayed`. A floor MAY be forgotten once it lies more than `maxGrantLifetimeSeconds` in the past, because no grant signed at or before it can still be unexpired.

### The unknown-key rule

The *known keys* of a member DID are read from its currently active grants, and are of two kinds, because a proof of control is compared only with keys of the relationship it proves:

- **signing keys** — the `authentication` verification methods that signed the active grants;
- **key-agreement keys** — the `keyAgreement` verification methods the member DID's document listed when each active grant was stored. The mediator records them with the grant, identified by their key material, not only by their identifier. A document that lists none records an empty set. A grant whose member DID cannot be resolved to read them is refused with `messaging/mux:didUnresolvable`; it is never stored without them.

The rule fires when the member DID proves control with a key **outside** the matching set, through any of:

- a grant submitted with this task, or a session attach ([`messaging/mux/attach`](../../attach/0.1/spec.md)) — the statement's signing key, compared with the signing keys;
- authenticating directly as the DID over DIDComm — the authcrypt sender key, a key-agreement key, compared with the key-agreement keys;
- authenticating directly as the DID over TSP — the sender VID's signing key, compared with the signing keys.

On a direct authentication the mediator **MUST** apply the rule, commit any suspension and detach its members before the authentication succeeds. Where it cannot — the sender key does not resolve, or the store is unavailable — it **MUST** refuse the authentication as retryable rather than complete it unchecked. A DID with no active grants is not affected.

Comparing an authcrypt sender key with signing keys would fire on every direct authentication, because the two relationships hold different keys. With the rule as stated, a key the member DID already listed when it granted is known, so a member that also authenticates directly does not suspend its own grants; a key added since — a new controller's, or a rotation — is unknown. A member DID that listed no key-agreement keys when it granted has none known, so its first one is unknown. The comparison is with the keys every active grant recorded together: a grant that recorded none does not exempt the others.

When it fires, every active grant on that DID that was **not** signed by the new verification method moves to `suspended`, with reason `suspendedUnknownKey`. A DID whose set is empty — no active grants yet — never fires it. The known keys are read from the grants themselves, so nothing separate can drift from them. Suspension does **not** advance the epoch — a grant the new key signs at the same moment must land active, which it could not under an epoch just invalidated — but `suspended` is terminal, so for the old grants the effect is that of revocation. It goes through the same synchronous detach as revocation: no frame for the DID reaches a suspended principal once the suspension is committed. Then the mediator:

- stores a `grantsSuspended` [`messaging/mux/notice`](../../notice/0.1/spec.md) in each affected principal's inbox, and emits a `detached` event to it where it is connected;
- stores a `grantsPresent` notice for the member DID, so a new owner learns what delegations existed;
- writes an audit record.

If the trigger was a grant signed by the new key, that grant is stored as `active` in the same commit, so a principal is never suspended by its own new grant. To make a hand-over final, the new controller advances the epoch itself with [`messaging/mux/revoke-all`](../../revoke-all/0.1/spec.md).

The rule's cost is accepted: a member DID with several keys, used from several devices, suspends the other devices' grants the first time a new key is used. The notice tells each client exactly what to do, and a client that opted into recreation re-signs the grant if it still holds the member's key.

## Authorization

**Authority: the member DID's own signature, made with a key currently in its `authentication` relationship, under its current epoch.** A principal is entitled to a member's messages because the member signed a grant naming that principal, this mediator, a scope, an expiry and the epoch in force. Whoever submits the grant — the member or the principal — gains no authority by doing so; the document `proof` and the transport identify the submitter only ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). Consumer rule 2 enforces the entitlement, verification first and every check before any state changes.

A grant is bounded by its scope, by the member's own access-control capabilities, by its expiry, and by the member's continued control: a key rotation (`staleKey`), a new key (`suspended`), an epoch advance (`epochRevoked`) or a revocation (deletion) each ends it without the principal's cooperation. A copied static key — a did:key or did:peer whose private key the old owner kept — cannot be told apart from its rightful holder by any protocol; DIDs meant to change hands should use a method with key rotation.

## Definitions

- **`grants`** — REQUIRED. 1–256 [`GrantStatement`](../../_shared/0.1/mux.schema.json#/$defs/GrantStatement)s; the member DID is each one's `issuer`.
- **Epoch** — a per-member-DID counter at the mediator, starting at 0, advanced only by messaging/mux/revoke-all. Every grant embeds the epoch it was signed under; a grant whose epoch is not the current one is dead.
- **Replay floor** — per member DID and principal, the instant a new grant's `iat` must be strictly later than; see [Replay floors](#replay-floors).
- **`results`** (response) — one `GrantOutcome` per grant: `status` (`stored` | `refused`), the `grantId`, `did` (the statement's issuer) and `principal` from its statement, `code` for a refusal with `currentEpoch` (for `epochMismatch`) or `replayFloorAt` (for `replayed`), and for a stored grant `attachedNow` and, where it replaced one, `replacedGrantId`.
- `GrantState`, `MuxScope` — see [`mux.schema.json`](../../_shared/0.1/mux.schema.json).

## Request

The member DID, or a principal carrying its grant, sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A principal lodges a grant its member signed

```json
{
  "id": "urn:uuid:2c3d4e5f-6a7b-4c8d-9e0f-1a2b3c4d5e01",
  "type": "https://trusttasks.org/spec/messaging/mux/grant/0.1",
  "issuer": "did:webvh:QmPrinc1pal:phone.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T09:00:10Z",
  "payload": {
    "grants": [
      {
        "type": "muxGrant",
        "issuer": "did:webvh:QmA1ice:alice.example:work",
        "audience": "did:web:mediator.example",
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "scope": [
          "receive",
          "pickup"
        ],
        "epoch": 3,
        "iat": 1794214800,
        "exp": 1796806800,
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-1",
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
    "created": "2026-11-09T09:00:10Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

The statement grants `receive` and `pickup` to the principal under epoch 3, for 30 days from 2026-11-09T09:00:00Z, signed by the member's own `#key-1`; the document proof is the principal's, as submitter. The proof values are placeholders.

## Response

The mediator responds with one outcome per grant, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures of the request as a whole use `trust-task-error` rather than a `#response` document.

### Stored, and attached at once

```json
{
  "id": "urn:uuid:2c3d4e5f-6a7b-4c8d-9e0f-1a2b3c4d5e02",
  "type": "https://trusttasks.org/spec/messaging/mux/grant/0.1#response",
  "threadId": "urn:uuid:2c3d4e5f-6a7b-4c8d-9e0f-1a2b3c4d5e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:00:10Z",
  "payload": {
    "results": [
      {
        "status": "stored",
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "did": "did:webvh:QmA1ice:alice.example:work",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "attachedNow": true
      }
    ]
  }
}
```

### Refused: signed under an epoch the member has since advanced

```json
{
  "id": "urn:uuid:2c3d4e5f-6a7b-4c8d-9e0f-1a2b3c4d5e03",
  "type": "https://trusttasks.org/spec/messaging/mux/grant/0.1#response",
  "threadId": "urn:uuid:2c3d4e5f-6a7b-4c8d-9e0f-1a2b3c4d5e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T09:00:10Z",
  "payload": {
    "results": [
      {
        "status": "refused",
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "did": "did:webvh:QmA1ice:alice.example:work",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "code": "messaging/mux/grant:epochMismatch",
        "currentEpoch": 4
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Each grant statement carries the mediator, the member and principal DIDs, a grant identifier, a scope, the member's epoch and a validity window, with the member's Data Integrity proof. A statement is signed, not secret, but it is a standing authority: anyone who holds the principal's keys while it is valid collects the member's messages within scope. No message content or key material is carried. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

A grant is a durable, signed statement that the member DID and the principal are held by one party, kept by the mediator until the grant's expiry. For pairwise or peer DIDs that link defeats their purpose at the mediator, and it outlasts any one connection. Mitigations are the producer's: grant only DIDs whose owner opted into mux, keep scopes minimal and expiries short, use one principal per device, and spread DIDs across principals (*mux groups*) so no single principal links them all. Grant listings are visible only to the two parties and the operator. The mediator's identifier scope is public by design: every grant names it as `aud`, which is what makes a grant useless at any other mediator.

### Retention

**Durable.** The mediator keeps each grant, with its signed statement verbatim, while it is active, and keeps a dead grant listable for a retention window after its effective expiry, so both parties can see what existed; a revoked or replaced grant is deleted at once, leaving only a replay floor. The signed statement is the only evidence the mediator relies on for the delegation; keeping it verbatim lets an auditor re-verify it. A principal **SHOULD** keep its grants only while it uses them.

### Consent/purpose

The purpose is connection sharing: one principal carrying several DIDs' messages, so a reconnect costs one authentication. Mux is opted into at every layer: no grant is signed, renewed or recreated unless the application asks, and no DID is muxed because of what kind of DID it is. Using stored grants for anything but routing those DIDs' traffic — linking accounts, profiling devices — is outside this task's purpose.
