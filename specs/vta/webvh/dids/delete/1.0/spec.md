---
slug: vta/webvh/dids/delete
version: "1.0"
title: "VTA WebVH DIDs — Delete"
summary: "An administrator deletes a did:webvh the VTA holds; the published log may outlive the deletion, and the response says when it has."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - delete
  - destructive
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
  rationale: "Destroying a DID's keys ends an identity third parties may still be relying on. The audit record is what remains, so the VTA must attribute it independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Deleting a webvh DID makes it unresolvable, and the log entry recording the deletion cannot be withdrawn. Replayed after the identifier was re-created, it deletes the new one.
sideEffects:
  level: destructive
  rationale: "Destroys the VTA's record and keys for the DID. Anything issued under it becomes unverifiable against a live document."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/dids/delete:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
related:
  - vta/webvh/dids/get
  - vta/webvh/dids/list
  - vta/webvh/dids/create
---

## Abstract

**VTA WebVH DIDs — Delete** removes a `did:webvh` from the VTA: its record and
the keys that signed its log.

What it cannot reliably remove is the *published* log. That lives on a hosting
server, and the response says so when the server did not confirm.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** set `daemonCleanupError` when it
removed its own record but did not confirm removal of the published log, and
**MUST NOT** report such a deletion as complete. A consumer of the response
**MUST** surface it: the DID may still resolve.

`deleted: false` in a success response means the VTA declined to act.

## Authorization

Authority is the **administrator role over the DID's context**.

The required `proof` attributes the deletion. Once the keys are gone the audit
record is the only remaining account of what was destroyed and on whose
instruction.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/delete/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": { "did": "did:webvh:QmScidAbCdEfGh:example.com:alice" },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T11:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

A deletion the hosting server did not confirm — reported as success, with the caveat:

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/delete/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "deleted": true,
    "daemonCleanupError": "hosting server prod returned 503"
  }
}
```

## Security & Privacy

Deleting a DID does not retract anything issued under it. Credentials naming it
as issuer remain in circulation; they simply stop being verifiable against a
resolvable document, which to a relying party looks identical to an outage.
Revoking what the DID issued is a separate act, and it must happen **before**
the keys are destroyed — afterwards there is nothing left to sign a revocation
with.
