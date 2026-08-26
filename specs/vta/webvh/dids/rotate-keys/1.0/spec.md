---
slug: vta/webvh/dids/rotate-keys
version: "1.0"
title: "VTA WebVH DIDs — Rotate Keys"
summary: "An administrator rotates a did:webvh's keys by appending a log entry that moves to the pre-committed successor."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - rotate
  - keys
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
  rationale: "A rotation changes which keys speak for an identity. The VTA must attribute the change to a specific administrator independently of the transport that carried it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Rotation retires the DID's current keys. Replayed, it retires the keys installed by a later rotation, which restores the very key material the operator was rotating away from.
sideEffects:
  level: mutating
  rationale: "Appends a log entry and replaces the DID's active keys. The previous keys stop being authoritative."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/dids/rotate-keys:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
related:
  - vta/webvh/dids/update
  - vta/webvh/dids/get
  - vta/webvh/dids/create
---

## Abstract

**VTA WebVH DIDs — Rotate Keys** replaces the keys that speak for a `did:webvh`
by appending a log entry that moves to the successor committed by the previous
entry.

The SCID does not change: rotation appends to a log, it does not re-create one,
so the DID keeps its identity and its history.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** change the DID's `scid`; the
`newScid` member of the response reports the existing one.

When `serverless` is true in the response, the consumer has appended the entry
locally and **MUST NOT** represent the rotation as published — resolvers see the
old keys until the caller serves `newLogEntry`.

`preRotationCount: 0` **MUST** be honoured as an instruction to stop committing
successors, not treated as "unspecified".

## Authorization

Authority is the **administrator role over the DID's context**.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/rotate-keys/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "preRotationCount": 3
  },
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

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/rotate-keys/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "newVersionId": "5-QmNewEntryHash",
    "newScid": "QmScidAbCdEfGh",
    "newLogEntry": "{\"versionId\":\"5-QmNewEntryHash\"}",
    "updateKeysCount": 1,
    "preRotationKeyCount": 3,
    "serverless": false
  }
}
```

## Security & Privacy

Rotation is the recovery mechanism for a compromised key, and it only works if a
successor was committed *before* the compromise. A DID running with
`preRotationCount: 0` cannot be recovered this way: whoever holds the stolen key
can append a rotation of their own, and a resolver has no basis to prefer the
owner's.

After a rotation, anything signed by the previous key remains verifiable against
the log entry that was current when it was signed. A consumer **MUST NOT** treat
a rotation as invalidating past signatures.
