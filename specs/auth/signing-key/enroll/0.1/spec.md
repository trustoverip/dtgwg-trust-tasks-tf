---
slug: auth/signing-key/enroll
version: "0.1"
title: "Auth — Signing Key Enroll"
summary: A key a holder has just generated — typically a non-extractable browser key — is enrolled as a delegation of an identity, so documents it signs are authorized by that identity's standing without the identity's own key.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - signing-key
  - delegation
  - console
  - did-key
  - step-up
parties:
  - role: key being enrolled
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: consumer holding the identity's standing
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request's proof is the proof of possession: it is made by the key being enrolled, and a delegation written for a key nobody demonstrated holding would let an enrolling party point the identity's authority at a key someone else controls. The response is the holder's record of what was enrolled and on what terms; a proof lets it be relied on after the transport session is gone.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. It is also the substantive need: an enrolment replayed after the key was revoked is refused by the tombstone, but one replayed before it was ever used would re-open a delegation its holder had abandoned, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Writes one delegation record naming a key and the identity it acts for. It confers no role, scope or capability — the identity's own standing, read at execution time, remains the only authority a signed document can reach — and it is reversed by auth/signing-key/revoke. Not destructive: nothing is removed and no authority moves away from anyone.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a public key, an identity DID and an optional label; the response echoes the stored delegation. Nothing secret moves in either direction — the private half of the key never leaves the holder, and the authority evidence the consumer asks for is a separate exchange. The label is free text the holder chose, and is the one member that can carry personal detail (a device owner's name).
retention:
  class: durable
  rationale: >-
    The delegation is the record every later document signed by the key is authorized against, so it lives until revoked or expired, and is kept as a tombstone after revocation so a burned key can never be enrolled again and a holder can see why a device stopped working.
errorCodes:
  - code: auth/signing-key/enroll:keyNotIssuer
    meaning: "`signingKeyDid` is not the document's `issuer`, or the proof was not made by it. The enrolment must be signed by the key being enrolled — that signature is the proof of possession."
    retryable: false
  - code: auth/signing-key/enroll:selfDelegation
    meaning: "`signingKeyDid` equals `identityDid`. A delegation names a separate key; an identity already signs as itself."
    retryable: false
  - code: auth/signing-key/enroll:keyHoldsStanding
    meaning: "The key already holds standing of its own at the consumer (for example an access-control entry). It is an identity rather than a key, and a delegation to it would take effect only once that standing was removed — turning a de-escalation into a hand-over."
    retryable: false
  - code: auth/signing-key/enroll:alreadyEnrolled
    meaning: "The key is already enrolled — for this identity or another. Re-pointing a key at a different identity is a hand-over, not an edit; revoke it and generate a new key."
    retryable: false
  - code: auth/signing-key/enroll:keyRevoked
    meaning: "The key was enrolled and revoked. A revoked key is burned and cannot be enrolled again; generate a new one."
    retryable: false
  - code: auth/signing-key/enroll:expiryInPast
    meaning: "`expiresAt` is not in the future, so the delegation would authorize nothing."
    retryable: false
  - code: auth/signing-key/enroll:tooManyKeys
    meaning: "The identity already holds as many active delegations as the consumer allows. Revoke one (auth/signing-key/revoke) before enrolling another. `details.maxActiveKeys` MAY state the cap."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      properties:
        maxActiveKeys:
          type: integer
          minimum: 1
related:
  - auth/signing-key/list
  - auth/signing-key/revoke
  - auth/step-up/approve-request
  - auth/step-up/approve-response
  - auth/passkey/enroll/start
---

## Abstract

A party that holds standing at a consumer — a community administrator, say — often works from a surface that cannot hold that identity's signing key: a web console, whose only credentials are a session cookie and a passkey. Such a surface cannot author a signed Trust Task document, so every operation it performs has to travel over a bearer session instead, which is exactly the channel the framework's sender-bound documents exist to replace.

This task closes that gap without handing the surface the identity's key. The surface generates a key of its own — in a browser, a **non-extractable** WebCrypto Ed25519 key — derives a `did:key` from its public half, and enrols it here as a **delegation**: one sentence, *"key K may act as identity D, for scope S, until T"*. A delegation carries no role and no capability of its own: its `scope` only narrows which of D's operations K may be used for, and its expiry, which the consumer sets and caps, bounds how long. A document signed by K is authorized by D's standing, read fresh at execution time, so it can never reach further than D can, and removing or demoting D's standing disarms every key delegated from D at once.

The document is issued and signed by K itself. That signature proves K is held by the party asking; what it cannot prove is that the party asking is D, which is why the payload names D and why the consumer establishes it from separate authority evidence — in the reference implementation, a user-verified passkey gesture by D bound to this one enrolment (see [Authorization](#authorization)).

It is a sibling of `auth/passkey/*` because the thing managed is the same kind of thing: a credential of an identity, enrolled by its holder, listed, and individually revocable. It is not `device/register`, which registers a device as a consumer in its own right with its own row and its own capability set — exactly the independent authority a delegation is designed not to be.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the holder of the new key):

1. **MUST** generate the key itself and **SHOULD** make its private half non-extractable where the platform allows. The key is possession of a device profile; it is one factor, and exporting it would make it possession of a file.
2. **MUST** set `issuer` and `payload.signingKeyDid` to the key's `did:key`, and sign the document with that key.
3. **MUST** name in `identityDid` the identity the key is to act for, and **MUST** be prepared to supply whatever authority evidence the consumer requires for that identity — typically by answering an inline `stepUpRequest` (see Authorization).

A conforming **consumer**:

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, and **MUST** refuse with `keyNotIssuer` unless `payload.signingKeyDid` equals `issuer` and the proof's verification method resolves to that DID.
2. **MUST** refuse with `selfDelegation`, `keyHoldsStanding`, `alreadyEnrolled`, `keyRevoked` or `expiryInPast` when the corresponding rule in the error code table holds. These concern the key, not the identity, and **SHOULD** be checked first, so a refusal on them reveals nothing about `identityDid`.
3. **MUST NOT** write the delegation until it has established, from authority evidence it accepts, that the enrolling party controls `identityDid` and that `identityDid` holds the standing the delegation is meant to borrow (see Authorization). The key's own proof is never that evidence.
4. **The evidence binds this exact enrolment.** **MUST NOT** accept evidence of control over `identityDid` unless that evidence is bound to a digest of the whole enrolment document — its `type` and its complete `payload`, `signingKeyDid`, `identityDid`, `scope` and `expiresAt` included — and **MUST** verify, by recomputing that digest over the document as received, that the document it is about to enrol is exactly the one the evidence covers. Evidence bound to anything less — a session, the identity alone, or a payload with any member changed — **MUST NOT** be accepted for an enrolment. A captured approval then cannot enrol a different key, the same key for a different identity, or the same key on wider terms.
5. **No prompts to the identity's devices before standing is known.** Where a consumer answers an enrolment whose standing it has not yet checked with a request for evidence, that request **MUST** be one the requester completes on an authenticator it presents itself — an inline request, answered by the producer that sent the enrolment — and **MUST NOT** cause any prompt to be delivered to the identity's registered devices, wallet or other channels. A consumer that obtains the evidence by delivering an approval prompt to the identity (a push to its devices, a notification, a message to its wallet) **MUST** first establish that `identityDid` holds the standing the delegation would borrow, and **MUST** rate-limit such prompts per identity. See [Approval fatigue](#approval-fatigue).
6. **MUST** give every delegation an expiry: the earlier of the requested `expiresAt` and the consumer's own maximum lifetime, or the consumer's default lifetime when none was requested. It **MUST NOT** enrol a delegation without one, **SHOULD NOT** allow a lifetime longer than 30 days, and **MUST** return the expiry it set in the response and in [`auth/signing-key/list`](../../list/0.1/spec.md).
7. **MUST** accept a document signed by the key only for operations within the delegation's `scope`, and only while the delegation is active. For `console`, that is the operations the identity performs through the consumer's administration surface, each still authorized by the identity's own standing. **MUST NOT** accept a proof by a delegated key anywhere a specification requires an `assertionMethod` attestation — in particular [`auth/step-up/approve-response`](../../../step-up/approve-response/0.5/spec.md), [`task-consent/decision`](../../../../task-consent/decision/0.1/spec.md) and [`confirm/response`](../../../../confirm/response/0.1/spec.md) — whatever its scope. A delegated key is never listed under the identity's `assertionMethod` relationship, for the reason [`auth/authenticate/0.2`](../../../authenticate/0.2/spec.md) gives for its `sessionKey`: it is an operational credential, and an approver's decision is an attestation.
8. **SHOULD** cap the number of active delegations per identity, refusing beyond it with `tooManyKeys`, and **SHOULD** rate-limit enrolment attempts per source and per `identityDid`, answering with `unavailable` and a `retryAfter`, because an unauthenticated caller can start the evidence flow of item 5 with a throwaway key.
9. **MUST** serialise the check that the key is not already enrolled with the write that enrols it, so that two concurrent enrolments of one key — for two different identities — cannot both succeed and silently re-point it.
10. **MUST** record `identityDid` as the identity the delegation acts for, and **MUST** thereafter resolve a document signed by the key to that identity's standing read at execution time, never to a copy of it taken at enrolment.
11. **MUST** answer a document signed by a key that holds standing of its own from that standing alone, and **MUST NOT** fall back to a delegation when that standing refuses. A demoted identity's key must not route around the demotion.
12. **MUST** audit the enrolment against `identityDid`, naming the key as the acting credential.

## Authorization

The authority this task presupposes is **control of `identityDid`**, and the standing of `identityDid` that the delegation will borrow — at a community, an administrator's access-control entry. The key's proof establishes neither. It establishes only that the party asking holds the key, which is necessary (so an identity's authority cannot be pointed at somebody else's key) and nowhere near sufficient (anyone can generate a key and name any identity).

What establishes control is a consumer's policy decision, and this specification does not make it. The reference implementation — the VTC admin console — establishes it with an **operation-bound step-up**: a user-verified WebAuthn assertion from a passkey registered to `identityDid`, bound by digest to this one enrolment document and consumed by it. Its shape is [`auth/step-up/approve-request/0.3`](../../../step-up/approve-request/0.3/spec.md) *Inline delivery*:

1. The consumer receives the enrolment, runs Conformance items 1–2, and finds no approval bound to it. It refuses with `permissionDenied`, carrying `details.stepUpRequest` — an `approve-request/0.3` payload whose `subject` is `identityDid`, whose `boundTo` is the digest of the document's type and whole payload (Conformance item 4) salted with the request's `challenge`, and which has no `sessionId`. It is delivered only inline, to the producer that sent the enrolment (Conformance item 5).
2. The producer surfaces the request to the human behind `identityDid`, obtains the gesture, and returns [`auth/step-up/approve-response/0.4`](../../../step-up/approve-response/0.4/spec.md) with `evidence.kind: webauthn`. The consumer verifies the assertion against the parked ceremony, requires user verification, and requires the credential to be registered to `identityDid`. Nothing is elevated; one enrolment is authorized.
3. The producer re-sends the identical enrolment. The consumer recomputes the digest over it, finds the approval, removes it **before** writing, checks that `identityDid` holds the standing it requires, and enrols with an expiry it sets. A payload differing in any member — another key, another identity, a later expiry — has a different digest and finds nothing.

Two properties of that design are worth a reviewer's attention:

- **The passkey is the second factor, and the key never stands in for it.** Possession of a signing key is one factor. A delegation exists only after a gesture by a passkey of the identity, so a script that steals a session cannot leave a durable signing credential behind it.
- **The refusal is not an oracle for who holds standing.** A consumer following this design **SHOULD** answer step 1 identically whether or not `identityDid` holds standing — for instance with discoverable-credential options that name no credentials — and check standing only at step 3, after a gesture only the identity could make. Otherwise an unauthenticated party could learn which DIDs are administrators by enrolling throwaway keys against them. That ordering is safe only because the step-up is completed by the requester on its own authenticator and nothing reaches the identity's devices; a design that pushes an approval prompt to the identity must check standing first (Conformance item 5).

A consumer **MAY** accept other authority evidence for control of `identityDid` — a proof by `identityDid`'s own key over the same payload, delivered in `ext`, for a producer that holds it. Whatever it accepts, the decision is the consumer's alone ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); this section describes what the task assumes, not an obligation to authorize anyone.

## Definitions

- **Delegation** — the consumer's record that `signingKeyDid` may sign documents that are authorized by `identityDid`'s standing. It confers nothing of its own.
- **`signingKeyDid`** — the key being enrolled, as a `did:key`. It is the document's `issuer`.
- **`identityDid`** — the identity the key will act for.
- **Standing** — whatever the consumer authorizes `identityDid` by: at a community, its access-control entry.
- **`scope`** — which operations the key may be accepted for; `console` in this version.
- **Expiry** — when the delegation stops authorizing documents, set and capped by the consumer (Conformance item 6).
- **`deviceLabel`** — a name for where the key lives, for the humans choosing which key to revoke.

## Request

The key being enrolled (`issuer`) sends the request to the consumer (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A browser console enrols its key for an administrator

```json
{
  "id": "urn:uuid:5b0e3a2e-6f4c-4b8f-9d2a-1f0c2b7e4a01",
  "type": "https://trusttasks.org/spec/auth/signing-key/enroll/0.1#request",
  "issuer": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:5b0e3a2e-6f4c-4b8f-9d2a-1f0c2b7e4aff",
  "payload": {
    "signingKeyDid": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "identityDid": "did:web:alice.example",
    "scope": "console",
    "deviceLabel": "Work laptop — Chrome"
  }
}
```

## Response

The consumer (`recipient` of the request, now responding) answers with the sub-schema reachable via `$anchor: "response"`: the delegation as stored, with the expiry the consumer set. A refusal is a `trust-task-error`. The first answer to a well-formed enrolment is, under the reference design, a `permissionDenied` carrying `details.stepUpRequest` — not a failure of the enrolment but the request for its authority evidence.

### The enrolment is recorded

```json
{
  "id": "urn:uuid:5b0e3a2e-6f4c-4b8f-9d2a-1f0c2b7e4a02",
  "type": "https://trusttasks.org/spec/auth/signing-key/enroll/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-09-28T10:00:31Z",
  "threadId": "urn:uuid:5b0e3a2e-6f4c-4b8f-9d2a-1f0c2b7e4aff",
  "payload": {
    "signingKey": {
      "signingKeyDid": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
      "identityDid": "did:web:alice.example",
      "scope": "console",
      "deviceLabel": "Work laptop — Chrome",
      "createdAt": "2026-09-28T10:00:31Z",
      "expiresAt": "2026-10-28T10:00:31Z",
      "active": true
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a public key, an identity DID, a scope, an optional label and an optional requested expiry; the response echoes the stored delegation. The label is the only free-text member: it is bounded at 128 characters, read by the identity and by administrators, retained with the delegation, and untrusted — a surface rendering it **MUST** attribute it to the enrolling identity. A producer **SHOULD NOT** put a person's name or any identifier in it beyond what distinguishes one device from another. The private key never moves.

### Correlation

The consumer is declared with identifier scope `public`: it is the one party every holder of a delegation addresses, and the delegation is meaningful only at the consumer that recorded it, so a caller has to be able to recognise it. A pairwise identifier for it would leave a holder unable to tell which consumer a key is enrolled at.

The `did:key` is generated per device profile and is used only to sign documents to consumers the identity already deals with, so it adds no correlation handle the identity's own DID did not already provide — which is why the key's party is declared `pairwise`. The consumer learns which identity each key acts for; that is the point of the task. A key **SHOULD NOT** be enrolled at more than one consumer: doing so would let those consumers join the identity's activity across them by the key alone.

### Retention

The delegation is kept for as long as it can authorize a document, and afterwards as a tombstone, so a revoked key cannot be enrolled again and a holder can see that a device was disowned rather than never enrolled. A consumer **SHOULD** exclude delegations from backups it can restore onto different infrastructure: a restored consumer must not resurrect signing authority for a device profile that may no longer exist.

### Consent/purpose

The purpose is to let a surface that cannot hold an identity's key author documents in that identity's name, within that identity's existing standing. Reusing a delegation as evidence of anything else — as a second factor, as proof that a human approved a given document, as standing in its own right — is outside that purpose. In particular a consumer that requires a user-verified gesture for some operation **MUST NOT** accept a signature by a delegated key as that gesture.

### Approval fatigue

An enrolment is a request a stranger can make: anyone can generate a key and name any identity. If each such request made the identity's phone buzz with "approve sign-in?", an attacker could send them until a tired or confused holder tapped *approve* — the push-fatigue attack that defeats push-based MFA in practice. This task is designed so that cannot happen. The reference design's step-up is answered on the *requester's* own authenticator, so a stranger's enrolment reaches no device of the identity at all and can be completed only by someone who already holds the identity's passkey. A consumer that nonetheless chooses to push an approval to the identity is held, by Conformance item 5, to checking standing first and rate-limiting the prompts, and **SHOULD** show the holder the key, label, scope and expiry being approved — the values the digest of item 4 binds — so that what the holder approves is exactly what is enrolled.
