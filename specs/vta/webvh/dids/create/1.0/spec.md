---
slug: vta/webvh/dids/create
version: "1.0"
title: "VTA WebVH DIDs — Create"
summary: "An administrator mints a did:webvh in a context: the VTA generates the keys, writes the log's first entry, and publishes it through a hosting server or hands it back to be served."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - create
  - log
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
  rationale: "Creating a DID commits an identity that third parties will resolve and rely on; the VTA must attribute it to a specific administrator in the audit record independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Creating a webvh DID publishes an identifier to a log others resolve. A replayed create publishes a second identifier the operator did not ask for, and log entries are not retractable.
sideEffects:
  level: mutating
  rationale: "Mints keys and writes a DID log's first entry. The SCID and portability are committed here and cannot be changed later."
subjectPath: /contextId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/dids/create:pathTaken"
    meaning: "The requested path is already in use on the hosting server."
    retryable: false
  - code: "vta/webvh/dids/create:templateNotFound"
    meaning: "The named DID template does not exist in the selected scope."
    retryable: false
related:
  - vta/webvh/dids/get
  - vta/webvh/dids/list
  - vta/webvh/dids/update
  - vta/webvh/dids/rotate-keys
  - vta/webvh/dids/register-with-server
---

## Abstract

**VTA WebVH DIDs — Create** mints a `did:webvh`. The VTA generates the signing
and key-agreement keys inside a context, composes or accepts a DID document,
and writes the first entry of the append-only log that *is* the DID's history.

Two choices made here are permanent. The **SCID** commits to that first entry,
so the DID's identity is bound to it. **Portability** is recorded in it — a DID
created non-portable can never be moved to another domain, whatever an operator
later wishes.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** treat `portable` and the resulting
`scid` as fixed once the first log entry is written. Neither is editable by any
task in this family.

When `serverId` is absent the DID is **serverless**: the consumer **MUST**
return the first `logEntry`, and **MUST NOT** represent the DID as resolvable —
it does not resolve until the caller serves that entry at `url`.

A consumer **MUST** refuse a request supplying both `path` and a `pathMode` of
`explicit` with a different value, rather than choosing between them.

## Authorization

Authority is the **administrator role over the context** named in `contextId`.
The DID's keys are minted in that context and belong to it, so the authority
that governs the context governs what identities it can bring into being.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

Minting a portable DID on a hosting server, with three successor keys committed:

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/create/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "contextId": "personal",
    "serverId": "prod",
    "pathMode": { "mode": "explicit", "path": "alice" },
    "portable": true,
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
  "type": "https://trusttasks.org/spec/vta/webvh/dids/create/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "contextId": "personal",
    "serverId": "prod",
    "mnemonic": "alice",
    "scid": "QmScidAbCdEfGh",
    "portable": true,
    "signingKeyId": "webvh-alice-sign",
    "kaKeyId": "webvh-alice-ka",
    "preRotationKeyCount": 3,
    "createdAt": "2026-08-19T11:00:01Z"
  }
}
```

## Security & Privacy

`preRotationCount: 0` disables pre-rotation, and the cost is asymmetric: with
no successor committed in advance, a party who steals the current signing key
can rotate to a key of their own as convincingly as the owner can. There is no
recovery path afterwards, only a dispute. Choose a non-zero count unless
something else provides that guarantee.

A serverless DID exists in the VTA and nowhere else until the caller publishes
the returned `logEntry`. Until then it does not resolve, and anything issued
under it cannot be verified by a third party.
