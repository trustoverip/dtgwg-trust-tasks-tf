---
slug: vta/webs/dids/rotate-keys
version: "0.1"
title: "VTA did:webs DIDs — Rotate Keys"
summary: "An administrator rotates a did:webs DID to its pre-rotated successor and commits a fresh one. The VTA stages the rotation event, publishes it, then commits it. Alpha."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - did-webs
  - keri
  - did
  - rotate
  - pre-rotation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A rotation changes which key speaks for an identity and spends the commitment that would otherwise recover it. The VTA must attribute it to a specific administrator independently of the transport that carried it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Each rotation spends a successor. A replayed rotation spends the next one, installing a key nobody asked to install and leaving the operator's record of which key is current wrong."
sideEffects:
  level: destructive
  rationale: "Replaces the DID's sole signing key, which is also its change authority: the key that could rotate the DID before this event cannot afterwards. That is rotation of a sole controlling key, and so authority-shifting. The published event is also permanent."
consequences:
  - "The current signing key stops speaking for the DID. Anything it signs from now on fails verification against the current key state."
  - "Installs the committed successor and commits a fresh one, published to the DID's public key event log."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "The request names the DID; the response carries the new public signing key and the VTA's ids for the installed, committed and retired keys. No key material."
retention:
  class: durable
  rationale: "The rotation event is permanent, and the audit row records who rotated the DID's keys and when."
errorCodes:
  - code: "vta/webs/dids/rotate-keys:notFound"
    meaning: "No such did:webs DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webs/dids/rotate-keys:versionConflict"
    meaning: "The DID's latest event no longer matches `expectedVersionId`. Re-read it and decide whether the rotation is still wanted."
    retryable: false
  - code: "vta/webs/dids:deactivated"
    meaning: "The DID is deactivated: it commits no successor, and cannot rotate."
    retryable: false
  - code: "vta/webs/dids:publishRefused"
    meaning: "The hosting server refused the rotation. It stays staged and is retried before anything else is written to the DID; `details.hostError` carries what the server said."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        hostError: { type: string }
related:
  - vta/webs/dids/get
  - vta/webs/dids/update
  - vta/webs/dids/delete
---

## Abstract

**VTA did:webs DIDs — Rotate Keys** moves a `did:webs` DID to the successor its last
establishment event committed, and commits a fresh one. The VTA writes a KERI rotation
event signed by the successor, whose digest the previous event committed, so the
rotation verifies against a commitment made before any compromise of the key it
replaces. The AID does not change: a rotation extends the key event log, it does not
re-create it, and the DID keeps its identity, its designations and its history.

The key model — one signing key (`kt` 1), one committed successor (`nt` 1), both derived
by the VTA — is in [CONVENTIONS.md §3](../../../_shared/0.1/CONVENTIONS.md#3-keys).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration is **alpha**.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST**:

1. Find the DID among the contexts the caller may act in (`notFound`), and refuse a
   deactivated one (`vta/webs/dids:deactivated`).
2. **If a rotation is already staged**, publish that same staged event, commit it, and
   answer with it and `resumed` true. It **MUST NOT** write a second rotation in the same
   request: the caller asked for one rotation, and a staged one is a rotation that has
   begun ([CONVENTIONS.md §6](../../../_shared/0.1/CONVENTIONS.md#6-remote-first)).
3. Otherwise, refuse with `versionConflict` when `expectedVersionId` is present and does
   not name the latest committed event.
4. Derive the next successor, write the rotation event — `k` the committed successor,
   `n` the new successor's digest, `kt` and `nt` 1, signed by the successor — and
   **stage** the event and the new key record before publishing anything.
5. Publish the whole stream to the host, and commit only once the host has accepted it:
   the successor becomes the active signing key, the replaced key is retired, and the new
   successor's record stays inactive. A refusal (`vta/webs/dids:publishRefused`) or an
   unreachable host (the framework's `unavailable`) leaves the rotation staged, and the
   record naming the old key.

A rotation **MUST NOT** change the designations: the key agreement key, the services and
the aliases outlive it. Anything the replaced key signed while it was current remains
verifiable against the key state of its time, and a consumer **MUST NOT** treat the
rotation as invalidating it.

Each request that is not answered by a staged rotation rotates once. A retry is safe only
under the framework's duplicate-execution protection (the same document `id`), or with
`expectedVersionId`, which refuses a second rotation based on the same version.

## Authorization

Authority is the **administrator role over the DID's context**. The successor that signs
the rotation is held by the VTA in that context and is never given to the caller; the role
is what entitles the caller to have it spent.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "d5847206-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webs/dids/rotate-keys/0.1",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T13:00:00Z",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "expectedVersionId": "2:EF3dK9wq2mVb0yQz7Lr5Hn1cXtU8pAaJ6sGe4oWiZ0Ck",
    "label": "scheduled rotation"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-11T13:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z5Yr8XExampleProofValueForWebsRotate"
  }
}
```

## Response

```json
{
  "id": "e6958317-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webs/dids/rotate-keys/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T13:00:02Z",
  "threadId": "d5847206-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "sequenceNumber": 3,
    "versionId": "3:EJ2nQ7cV0aRk5Wm8Yx1Lb4Tg9Pe6Hs3Uo0Zf7Di2Kq5M",
    "signingKeyId": "webs-alice-sign-1",
    "signingKey": "DPm4Rk1cTq7Ww0zX9nB3vY6aH2sL8eJ5uF0gK7dQ1iZo",
    "nextKeyId": "webs-alice-sign-2",
    "retiredKeyId": "webs-alice-sign-0",
    "resumed": false
  }
}
```

## Security & Privacy

### Data carried

The request names the DID, a concurrency precondition and an audit label. The response
carries the new public signing key and the VTA's ids for the installed, committed and
retired keys. No key material moves: the VTA derives the successor, signs with it, and
never returns it. The new key is public once the event is published.

### Correlation

A rotation is visible to everyone who can read the stream, with its time: the key event
log says when each key took over. The new key is unrelated to the old by anything a
verifier can compute, so a rotation adds no link between this DID and any other.

### Retention

The rotation event is permanent. The VTA keeps the retired key's record — it signs nothing
more with it — and the audit record of who rotated the DID and when. Rotation is the
recovery for a compromised key, and it works only because the successor was committed
before the compromise. Against a VTA compromise, which reaches the successor too, it is no
defence; a DID that needs one is a key-role identity, with a recovery key held outside the
VTA ([`vta/dids/*`](../../../../dids/_shared/0.1/CONVENTIONS.md)).

### Consent/purpose

The request is made for one purpose, replacing the key that speaks for the named DID. A
consent surface showing it to a person **SHOULD** say that the current key stops speaking
for the DID once the rotation is published, and that the rotation cannot be undone.
