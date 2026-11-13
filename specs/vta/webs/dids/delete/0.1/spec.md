---
slug: vta/webs/dids/delete
version: "0.1"
title: "VTA did:webs DIDs — Delete"
summary: "An administrator deactivates a did:webs DID: the VTA publishes a rotation to no next keys, commits it, then removes the slot from the hosting server. Irreversible. Alpha."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - did-webs
  - keri
  - did
  - delete
  - deactivate
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
  rationale: "Deactivation ends an identity third parties may still rely on, and nothing can revive it. The audit record is what remains, so the VTA must attribute the deactivation independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Deactivation is irreversible. A delete that cannot be placed in time ends an identity at a moment nobody chose."
sideEffects:
  level: destructive
  rationale: "Publishes a rotation to no next keys, after which the AID can never rotate or change again, and removes the DID's slot from the host. Nothing the DID signs from then on verifies as current, and no task revives it."
consequences:
  - "The DID is deactivated for good: every copy of its key event log says so, and no later event can be anchored in it."
  - "The DID's slot is removed from the hosting server, and the DID stops resolving there."
  - "Anything the DID issued stays in circulation. Revoke it before deleting: afterwards nothing can sign a revocation as this DID."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
  rationale: "The request names the DID; the response reports the deactivating event and whether the slot was removed."
retention:
  class: durable
  rationale: "The VTA keeps the deactivated record and the final stream as evidence for what the DID signed while it was live, and the audit row records who deactivated it."
errorCodes:
  - code: "vta/webs/dids/delete:notFound"
    meaning: "No such did:webs DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webs/dids/delete:versionConflict"
    meaning: "The DID's latest event no longer matches `expectedVersionId`. Re-read it and decide whether the deletion is still wanted."
    retryable: false
  - code: "vta/webs/dids:publishRefused"
    meaning: "The hosting server refused the deactivating rotation. Nothing was committed and the DID is live; `details.hostError` carries what the server said."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        hostError: { type: string }
related:
  - vta/webs/dids/get
  - vta/webs/dids/rotate-keys
  - vta/webs/dids/create
---

## Abstract

**VTA did:webs DIDs — Delete** ends a `did:webs` DID. It does so in two steps, in this
order ([CONVENTIONS.md §7](../../../_shared/0.1/CONVENTIONS.md#7-deactivation)):

1. **Deactivate.** The VTA rotates to the committed successor and commits **no next
   keys** (`n` empty, `nt` 0). Once that event is published, the AID can never rotate or
   anchor anything again, and every copy of the stream says it is deactivated.
2. **Remove the slot** from the hosting server.

The order makes a failure safe. If the slot cannot be removed, the DID still resolves —
as deactivated. Once this task has answered, the DID never resolves live.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration is **alpha**.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST**:

1. Find the DID among the contexts the caller may act in (`notFound`).
2. For a DID not yet deactivated: complete any staged rotation first, refuse with
   `versionConflict` when `expectedVersionId` is present and does not name the latest
   committed event, then stage, publish and commit the deactivating rotation exactly as
   [`rotate-keys`](../../rotate-keys/0.1/spec.md) stages, publishes and commits a rotation,
   but committing no successor. A refused or unreachable publication leaves the DID live,
   and the task answers with the error.
3. Only once the deactivation is committed, ask the host to remove the slot. A failure
   here is **not** an error: the task answers with `slotRemoved` false and
   `hostCleanupError`, which a consumer of the response **MUST** surface.
4. Keep its record of the DID, marked deactivated, with the final stream, and sign
   nothing more as the DID. It **MAY** destroy the DID's private keys once the
   deactivation is committed; nothing can use them.

Deleting is **setting a state**. A request for a DID already deactivated succeeds, with
`newlyDeactivated` false, and retries the slot's removal if it had failed; it publishes
nothing.

## Authorization

Authority is the **administrator role over the DID's context**. The deactivating rotation
is signed by the committed successor, which the VTA holds in that context; the role is
what entitles the caller to have the identity ended.

The required `proof` attributes the deactivation. Once the DID is deactivated, the audit
record is the only account of whose instruction ended it.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "f7a69428-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webs/dids/delete/0.1",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T14:00:00Z",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "expectedVersionId": "3:EJ2nQ7cV0aRk5Wm8Yx1Lb4Tg9Pe6Hs3Uo0Zf7Di2Kq5M",
    "label": "persona retired"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-11T14:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z6Zs9YExampleProofValueForWebsDelete"
  }
}
```

## Response

The deactivation was published and committed, but the host did not confirm the slot's
removal — a success, with the caveat:

```json
{
  "id": "08b7a539-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webs/dids/delete/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T14:00:03Z",
  "threadId": "f7a69428-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "deactivated": true,
    "newlyDeactivated": true,
    "versionId": "4:EAq8Wn2Lb6Yc0Tx4Rm9Kd1Hs7Pe3Uf5Zo2Gi8Jv0Qk6N",
    "slotRemoved": false,
    "hostCleanupError": "hosting server prod returned 503"
  }
}
```

## Security & Privacy

### Data carried

The request names the DID, a concurrency precondition and an audit label; the response
reports the deactivating event and the slot's removal. No key material moves.

### Correlation

Deactivation is public and timed: the stream says when the DID ended. Removing the slot
stops the host serving the DID, but not the copies watchers and verifiers already hold,
which keep the DID's whole history, its designated aliases included.

### Retention

The deactivating event is permanent. The VTA keeps the deactivated record and the final
stream, because they are what anything the DID signed while it was live is still checked
against, and keeps the audit record of who ended the DID. Deleting a DID does not retract
anything issued under it: credentials naming it as issuer stay in circulation and still
verify against the key state of their time. Revoking them is a separate act, and it must
happen **before** the DID is deactivated, since afterwards nothing can sign as it.

### Consent/purpose

The request is made for one purpose, ending the named identity. A consent surface showing
it to a person **MUST** say that the deactivation is irreversible, and **SHOULD** say that
anything the DID issued stays in circulation unless revoked first.
