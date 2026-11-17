---
slug: vtc/storage/credentials/recipient
version: "0.1"
title: "VTC Storage — Credentials — Recipient"
summary: A room-hosting administrator's client asks the community for the key a sealed storage credential is sealed to — the Ed25519 key it opens with, as its did:key, and where its DID document publishes it — so a client that cannot resolve the community's DID can still seal.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Authority is the signer's `vtc.rooms.admin` capability, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: >-
    Reads the community's own key. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses a public key the community's DID document already publishes, and that document's identifier for it. Never a private key or anything derived from one.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/storage/credentials/recipient:notPublished
    meaning: "The key the community opens sealed blocks with is not the signing key its DID document currently publishes — a key rotation not yet published, or a document changed elsewhere. A block sealed now could not be told apart from one sealed to a stale key, so no key is given."
    retryable: true
related:
  - vtc/storage/credentials/set
  - vtc/storage/configs/get
---

## Abstract

The **VTC Storage — Credentials — Recipient** Trust Task returns the key a [`vtc/storage/credentials/set`](../../../../../vtc/storage/credentials/set/0.3/spec.md) sealed block is sealed to: the Ed25519 key the community opens with, as its `did:key`, and the verification method in the community's DID document that publishes it.

That key is a function of the community's DID document. A client that can resolve and verify the community's DID reads it there. A client that cannot — an administration console running in a browser, where resolving a `did:webs` means verifying a KERI key event stream and a hosted `did:webvh` means fetching and verifying a log from another origin — needs it from the community itself, on the same authenticated channel it sends the credential over. This task is that answer, the same for every DID method.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`.
2. **MUST** answer with the Ed25519 public key whose X25519 derivation opens a sealed block it receives in [`vtc/storage/credentials/set`](../../../../../vtc/storage/credentials/set/0.3/spec.md), and with the absolute identifier of the verification method under which its DID document publishes that key — its signing key, chosen by role. It **MUST NOT** answer with any other key.
3. **MUST** answer `notPublished`, with no key, when it knows the signing key its DID document currently publishes is not that key. A community that cannot resolve its own DID at the time of the request **MAY** answer from its own record of the document it holds.

A conforming **administrator's client** (`issuer`):

4. **MUST** accept the answer only from the community it addressed — a response whose proof, or authenticated sender, is that community — and only when `communityDid` is that community's DID and `verificationMethod` is a DID URL of that DID.
5. **MUST** seal to the X25519 derivation of `recipientKey` and to no other key.
6. A client that resolves and verifies the community's DID itself **MAY** seal to the key the resolved document publishes instead of asking, and **SHOULD** refuse to seal when an answer it did ask for names a different key.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community** — the authority [`vtc/storage/credentials/set`](../../../../../vtc/storage/credentials/set/0.3/spec.md) requires, since this task exists only to serve it. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

The key itself is public — the community's DID document publishes it — so the capability gates the task, not a secret: no caller learns from it anything a resolver could not.

## Definitions

- **`communityDid`** — the community's DID, as a bare DID. It is the response's `issuer`, repeated in the payload so the answer still names its community once separated from its envelope.
- **`verificationMethod`** — the absolute DID URL, in `communityDid`'s document, of the verification method publishing `recipientKey`: the community's signing key, chosen by role (`#key-0` on a `did:webvh`; the key-state or attestation key on a `did:webs`, published as a JSON Web Key under its own identifier).
- **`recipientKey`** — that Ed25519 public key as its `did:key` (multicodec `0xed01`). The recipient of a sealed block is its X25519 derivation.

## Request

A room-hosting administrator's client (`issuer`) asks the community (`recipient`). The payload, the top-level schema in [`payload.schema.json`](payload.schema.json), names nothing: the community answers for itself.

### Ask

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/credentials/recipient/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-17T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000007ff",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The key

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/credentials/recipient/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-17T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000007ff",
  "payload": {
    "communityDid": "did:example:community",
    "verificationMethod": "did:example:community#key-0",
    "recipientKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries one public key and two identifiers the community's DID document already publishes. No private key, and nothing derived from one, is ever returned.

The answer decides who can read the credential sealed next, so its integrity is what matters: a forged key would hand a long-lived storage credential to whoever forged it. Conformance items 4 and 5 bind the answer to the community the client addressed. A client trusting this answer trusts the community for the key exactly as far as it trusts it to open the sealed block — and a console the community serves already trusts it for the code that seals.

### Correlation

Nothing here is not already public in the community's DID document. The request reveals only that an administrator is about to set a sealed credential, which the request that follows reveals anyway.

### Retention

Transient. The community keeps nothing; a client keeps the key only for the seal it is about to make, and asks again for the next one, so a rotation is never sealed past.

### Consent/purpose

The key is asked for to seal a storage credential to the community in [`vtc/storage/credentials/set`](../../../../../vtc/storage/credentials/set/0.3/spec.md), and for no other purpose.
