---
slug: auth/oob/respond
version: "0.4"
title: "Auth — OOB Respond"
summary: "The lock holder of an out-of-band request delivers the member's signed decision by carrying an auth/oob/grant, and the service records the request as approved or declined; no session material ever comes back to the approver."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, grant, wallet]
parties:
  - role: Approver (lock key)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof by the lock key ties the carried grant to the device that held the lock and saw step 2; the grant names that key, so no other device can deliver it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task records a decision that lets a key act as the member, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Moves the request from identified to approved, storing the grant, or to declined. Each request allows one decision.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The carried grant authorizes the starter's key to act as the member's DID for one session, and may let it sign documents of named task families in the member's name for that session; recording it exercises the member's own authority. The response is a bare status and discloses nothing.
retention:
  class: durable
  rationale: >-
    The stored grant is the audit record of the sign-in decision and is kept with the session it armed.
errorCodes:
  - code: auth/oob/respond:contextMismatch
    meaning: The grant's purpose, sessionKey, origin, contextDigest or signingScope does not match what the service stored for the request. The request is declined.
    retryable: false
  - code: auth/oob/respond:scopeExceedsRequest
    meaning: A deviceEnrol grant's aclScope is missing on approve, present on decline, or outside the requestedScope step 2 showed — a capability not requested, an additive or ceiling capability scope, a wider purpose or key scope, an expiresAt past the requested lifetime, or a consumerKind other than the client's. The request is declined.
    retryable: false
  - code: auth/oob:notClaimant
    meaning: The respond is not signed by the request's lock key.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The decision window has passed, or the request has otherwise ended.
    retryable: false
  - code: auth/oob:alreadyDecided
    meaning: The request has already been approved, declined or cancelled.
    retryable: false
  - code: auth/oob:notAuthorized
    meaning: The carried grant failed verification, is not from the identified DID or does not name the lock, or the member is no longer active — for deviceEnrol, the issuer is no longer an administrator of the VTA or could not itself write the entry the grant describes. Deliberately generic. The request is declined.
    retryable: false
related:
  - auth/oob/prove
  - auth/oob/grant
  - auth/oob/redeem
---

## Abstract

After step 2 the member approves (with user verification) or declines, and the member's VTA signs an [`auth/oob/grant/0.4`](../../grant/0.4/spec.md). The wallet delivers it in this task, signed by the lock key. The service checks the grant against everything it stored for the request and records the decision. The response is only a status: the authority the grant confers goes to the starter's key, and only through [`auth/oob/redeem`](../../redeem/0.3/spec.md).

### Changes from 0.1

- **The carried grant is [`auth/oob/grant/0.2`](../../grant/0.2/spec.md)**, which may carry a `signingScope` — the task families the starter key may sign documents for in the member's name for the session. The service checks it against the scope it offered in step 2 ([`auth/oob/prove/0.2`](../../prove/0.2/spec.md)) along with the other members, so a grant can never delegate more, or other, signing than the member was shown. Nothing else changes.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.3

- The carried grant is [`auth/oob/grant/0.4`](../../grant/0.4/spec.md), which
  states its `purpose` and, for `deviceEnrol`, carries the `aclScope` the
  approver chose. The service checks the purpose (item 3) and, for
  `deviceEnrol`, the scope against step 2's `requestedScope` and the approver's
  own authority (items 3a and 4).
- A new error code, **`auth/oob/respond:scopeExceedsRequest`**, for a
  `deviceEnrol` grant whose scope is missing or outside the ceiling.

The request and response payloads are unchanged, and a `login` respond is
handled exactly as in 0.3. Not declared as `extends: "0.3"`,
so that a consumer that implements only `login` 0.3 never routes a
`deviceEnrol` document to a `login` handler
([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A10). `0.3` remains
published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **approver**:

1. **MUST** sign with the lock key and **SHOULD** set `parentThreadId` to the `requestId` ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)).
2. **MUST** check that the grant its VTA returned is the one it sent, apart from the added proof, before carrying it.
3. **MUST** discard the lock key after this task, and **MUST NOT** resend once the request has ended.

A conforming **service**, in this order:

1. **MUST** require the outer issuer to equal the lock, else `auth/oob:notClaimant`; the request to be `identified`, else `auth/oob:alreadyDecided` or `auth/oob:requestExpired`; and the decision window to be open.
2. **MUST** verify the grant as [`auth/oob/grant/0.4`](../../grant/0.4/spec.md) requires, against the issuer's **`assertionMethod`** relationship, and require its issuer to be the identified DID, else `auth/oob:notAuthorized`.
3. **MUST** require the grant's `purpose` to equal the request's purpose, its `approverKey` to equal the lock, its `sessionKey` the starter key, its `origin` the request's origin, its `contextDigest` the stored digest of the signed step 2, and its `signingScope` the scope step 2 offered — both absent, or the same families in the same order — else `auth/oob/respond:contextMismatch`.
3a. For `deviceEnrol`, **MUST** require, on approve, an `aclScope` within step 2's `requestedScope` as [`auth/oob/grant/0.4`](../../grant/0.4/spec.md) producer item 7 states, and on decline no `aclScope`, else `auth/oob/respond:scopeExceedsRequest`. It checks `aclScope.expiresAt` against the grant's `issuedAt` plus `maxLifetimeDays`, within its own clock skew.
4. **MUST** check that the member is still active, else `auth/oob:notAuthorized`. For `deviceEnrol`, **MUST** instead check that the grant's issuer could still write the entry the grant describes by [`acl/grant/0.3`](../../../../acl/grant/0.3/spec.md) — a live ACL entry at this VTA that may write ACL entries in the context `aclScope` names, and that the entry would not exceed on any axis of [ACL CONVENTIONS §9](../../../../acl/_shared/0.3/CONVENTIONS.md#9-bounds-on-the-granter-and-delegation) — else `auth/oob:notAuthorized` ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A7).
5. On success, **MUST** move the request to `approved` (or `declined` for a decline) in one compare-and-set, store the grant, and answer with the status. The stored grant is what the session `auth/oob/redeem` creates carries its signing scope from — or, for `deviceEnrol`, what the ACL entry `auth/oob/redeem` creates takes its scope from. For `deviceEnrol`, `approved` means the entry will be created at redeem; nothing is written to the ACL here. On any failure from step 2 on, **MUST** move the request to `declined`.
6. **MUST NOT** put session material in the response.
7. **MUST** answer an earlier version of this task — `auth/oob/respond/0.3` or before — for a request opened as `deviceEnrol` with `auth/oob:notAuthorized`, and move the request to `declined`: such a request is handled only by the versions that define its purpose ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 3c).

## Authorization

The authority is the member's own, carried in the grant: the identified DID's attestation that the starter key may act as it, and sign for the families its scope names ([`auth/oob/grant/0.4`](../../grant/0.4/spec.md) Authorization). The lock key's proof entitles the device to deliver it, and nothing more. Active membership, read at execution time, is the standing the session needs.

For `deviceEnrol`, the authority is an administrator's, carried the same way: the grant's issuer attests that the VTA may write one ACL entry for the starter key with the scope `aclScope` states, and the VTA checks, at execution time, that the issuer could write that entry itself.

## Definitions

- **`grant`** — a complete signed `auth/oob/grant/0.4` document, verified over its members as received.
- **`status`** (response) — `approved` or `declined`.

## Request

The approver (`issuer`, the lock key) delivers the grant to the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet delivers an approval

```json
{
  "id": "urn:uuid:9e5f6071-8293-4dae-8f10-3b4c5d6e7f01",
  "type": "https://trusttasks.org/spec/auth/oob/respond/0.4",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:31Z",
  "payload": {
    "grant": {
      "id": "urn:uuid:8d4e5f60-7182-4c9d-8e0f-2a3b4c5d6e01",
      "type": "https://trusttasks.org/spec/auth/oob/grant/0.4",
      "issuer": "did:web:alice.example",
      "recipient": "did:web:community.example",
      "issuedAt": "2026-10-10T10:01:30Z",
      "payload": {
        "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
        "purpose": "login",
        "decision": "approve",
        "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "origin": "https://members.community.example",
        "contextDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "sessionEndsAt": "2026-10-10T18:00:00Z",
        "signingScope": ["vtc/directory", "vtc/groups"]
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:web:alice.example#key-1",
        "created": "2026-10-10T10:01:30Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z3kg…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-10T10:01:31Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service answers with the recorded status. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Approved

```json
{
  "id": "urn:uuid:9e5f6071-8293-4dae-8f10-3b4c5d6e7f02",
  "type": "https://trusttasks.org/spec/auth/oob/respond/0.4#response",
  "threadId": "urn:uuid:9e5f6071-8293-4dae-8f10-3b4c5d6e7f01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:31Z",
  "payload": {
    "status": "approved"
  }
}
```

## Security & Privacy

### Data carried

The signed grant: a handle, a purpose, two throwaway keys, an origin, a digest, a time, a decision and, optionally, a list of task families — or, for `deviceEnrol`, the ACL scope chosen for the client. The response is a status only.

### Correlation

The service joins the grant to the request, the identified DID and the starter key, which is the purpose of the task. The lock key is pairwise and discarded after this task. The service has identifier scope `public` because the approver addressed the community's published DID throughout.

### Retention

The service keeps the signed grant as the durable audit record of the sign-in decision, with the session it arms.

### Consent/purpose

The grant is delivered to record one decision for one request. A service **MUST NOT** treat it as consent to anything beyond the session the grant names and the signing that session's scope allows — or, for `deviceEnrol`, beyond the one ACL entry its `aclScope` describes.

## Pending confirmation

This version takes the readings in [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 and will follow the family authors' answer when they give one. Those that bear on this task: **A7**, **A8**, **A10** and **A14**.
