---
slug: auth/signing-key/authorize
version: "0.1"
title: "Auth — Signing Key Authorize"
summary: An identity's own signed authorization of one signing-key enrolment, carried inside auth/signing-key/enroll so a holder of the identity's key can authorize a delegation without a passkey gesture.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - signing-key
  - delegation
  - console
  - wallet
parties:
  - role: identity authorizing the delegation
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: consumer that will hold the delegation
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the whole of what this document contributes: it is the identity's own signature over the terms of one enrolment, and it stands in for the user-verified gesture a consumer would otherwise ask for. A proofless authorization authorizes nothing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The authorization arms a delegation of the identity's standing, so it is consequential, and SPEC §7.3 item 17 sets REQUIRED as the floor. Its freshness bounds how long a captured authorization could be carried to the consumer inside an enrolment.
sideEffects:
  level: none
  rationale: >-
    The document does nothing on its own. It is evidence carried inside auth/signing-key/enroll, and the effect — writing a delegation — is that task's.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The payload restates the enrolment's terms: a public key, the identity, a scope, an optional label, an optional expiry and an optional key to replace. Nothing secret moves; the identity's private key signs and stays where it is.
retention:
  class: durable
  rationale: >-
    The consumer keeps the authorization with the delegation it armed, as the record of who authorized it, and keeps its id for as long as its acceptance window lasts so it cannot be spent twice.
errorCodes: []
related:
  - auth/signing-key/enroll
  - auth/signing-key/revoke
---

## Abstract

[`auth/signing-key/enroll`](../../enroll/0.2/spec.md) enrols a key as a delegation of an identity, and needs evidence that the party enrolling controls that identity. Where that party can only offer a passkey, the consumer asks for an operation-bound gesture. Where it can reach the **identity's own key** — a wallet whose signing key is held by the holder's agent, a command-line tool holding the key directly — a stronger and simpler answer is available: the identity signs the terms of the enrolment itself.

This specification defines that signature. An `authorize` document is issued and signed by the identity, addressed to the consumer, and states exactly one enrolment: which key, for which identity, on what scope, label and expiry, replacing which key. It is never sent on its own. It travels as the `authorization` member of the enrolment it authorizes, and the consumer accepts it only if its payload is that enrolment's payload.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the identity, or the agent holding its key on its behalf):

1. **MUST** set `issuer` and `payload.identityDid` to the identity, `recipient` to the consumer the enrolment is addressed to, and sign the document with a key the identity's DID document lists under the relationship its `proofPurpose` names ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)).
2. **MUST** set `payload` to exactly the payload of the enrolment it authorizes, with that enrolment's `authorization` member removed and nothing else changed.
3. **MUST** obtain the identity holder's agreement before signing, and **SHOULD** show them the key, label, scope, expiry and any key being replaced — the terms the signature binds. An agent that signs these without asking hands every page that can reach it the ability to plant a durable signing key.
4. **MUST NOT** send the document on its own. It is embedded in the enrolment.

A conforming **consumer** (the party executing `auth/signing-key/enroll`):

1. **MUST NOT** act on an `authorize` document received on its own, and **SHOULD** answer one with `unsupportedType`.
2. **MUST** verify an embedded `authorize` document as a Trust Task document in its own right — the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, including the proof, the binding of the proof to `issuer` ([SPEC §4.8](/SPEC.md#48-the-issuer-and-recipient-members)), the recipient binding to itself, and its acceptance window on `issuedAt` and `expiresAt` — and **MUST** treat any failure as `auth/signing-key/enroll:authorizationInvalid` on the enrolment.
3. **MUST** require `issuer` to equal `payload.identityDid`, and that to equal the enrolment's `identityDid`.
4. **MUST** require the document's `payload` to equal the enrolment's payload with its `authorization` member removed, compared as [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonical JSON over the payloads **as received**. Any difference — another key, another identity, a wider scope, a later expiry, a different key to replace — **MUST** be refused. This is the binding that makes one authorization arm one enrolment.
5. **MUST** record the document's `id` against reuse for at least its acceptance window ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11), and **MUST** refuse an enrolment carrying an authorization whose `id` it has already accepted.
6. **MUST NOT** accept a proof by a key that is itself a signing-key delegation, of this identity or any other: the authorization must be made by the identity, not by a credential the identity delegated.

## Authorization

Control of `identityDid` is what this document proves, and it proves it the way the framework proves any issuer: the proof verifies against a verification method of `identityDid`, under the relationship the proof names. It does not prove that `identityDid` holds any standing at the consumer; that remains the enrolment's check, made after this evidence is accepted ([`auth/signing-key/enroll/0.2`](../../enroll/0.2/spec.md) Authorization).

## Definitions

The payload members are those of [`auth/signing-key/enroll/0.2`](../../enroll/0.2/spec.md) less `authorization`, with the same meanings. The payload is a copy of the enrolment's terms, so that what the identity signed and what the consumer enrols are compared rather than assumed equal.

## Request

The identity (`issuer`) signs the document for the consumer (`recipient`); the enrolling party carries it inside its enrolment. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet authorizes a browser console's key

```json
{
  "id": "urn:uuid:7f1d2c3b-4a5e-4f60-8a71-b2c3d4e5f601",
  "type": "https://trusttasks.org/spec/auth/signing-key/authorize/0.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T10:00:00Z",
  "payload": {
    "signingKeyDid": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "identityDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "scope": "console",
    "deviceLabel": "Work laptop — Chrome"
  }
}
```

## Security & Privacy

### Data carried

The payload restates the enrolment: a public key, the identity, a scope, an optional label, an optional expiry and an optional key to replace. The label is free text chosen by the identity and is untrusted wherever it is rendered. No secret moves.

### Correlation

The document adds no identifier the enrolment did not already carry. The consumer is declared with identifier scope `public` for the reason [`auth/signing-key/enroll`](../../enroll/0.2/spec.md) gives: the delegation is meaningful only at the consumer that records it. The identity's party is `any`, because the identity is whichever DID already holds standing at that consumer, pairwise or not.

### Retention

The consumer keeps the authorization with the delegation it armed, as the record that the identity itself authorized the key, and keeps its `id` for at least the acceptance window so it cannot be replayed into a second enrolment.

### Consent/purpose

The signature authorizes one delegation on the stated terms and nothing else. An agent signing on the identity's behalf is the one place a holder sees what they are agreeing to, which is why producer item 3 requires their agreement and recommends showing the terms; an agent that signs silently converts any page able to reach it into a party able to enrol keys in the holder's name.
