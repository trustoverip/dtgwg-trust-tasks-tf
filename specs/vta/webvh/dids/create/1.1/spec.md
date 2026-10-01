---
slug: vta/webvh/dids/create
version: "1.1"
title: "VTA WebVH DIDs — Create"
summary: "An administrator mints a did:webvh in a context: the VTA generates the keys, writes the log's first entry, and publishes it through a hosting server or hands it back to be served."
status: draft
targetFrameworkVersion: "0.5.0"
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

1.1 differs from [1.0](../1.0/spec.md) only in its response, which now states `serverless`. In 1.0 a client could tell a serverless DID only by an absent `serverId`, so a client that missed the absence went on to resolve a DID nobody serves, and failed later with an error that named the DID rather than the cause. The new member is REQUIRED, which is why this is a new version rather than an edit to 1.0, whose response refuses unknown members.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** treat `portable` and the resulting
`scid` as fixed once the first log entry is written. Neither is editable by any
task in this family.

When `serverId` is absent the DID is **serverless**: the consumer **MUST**
return the first `logEntry`, and **MUST NOT** represent the DID as resolvable —
it does not resolve until the caller serves that entry at `url`.

The response **MUST** carry `serverless`. It is `true` exactly when the request
named no `serverId`, and then the response **MUST** carry `logEntry` and **MUST
NOT** carry `serverId`. It is `false` exactly when the DID was published through
a hosting server, and then the response **MUST** carry `serverId`. A *producer*
**SHOULD** read `serverless` rather than infer it from an absent `serverId`, and
**SHOULD NOT** treat a `serverless` DID as resolvable until it has served
`logEntry` itself.

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
  "type": "https://trusttasks.org/spec/vta/webvh/dids/create/1.1",
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
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/create/1.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "contextId": "personal",
    "serverId": "prod",
    "serverless": false,
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

### Data carried

The request carries control members (the context, the hosting server or URL,
path, portability, pre-rotation count, template choice and variables) and no key
material: the VTA generates the keys. The response carries the new DID, the key
ids the VTA holds for it, and, for a serverless DID, the log's first entry. The
DID document and log entry are public by construction, since they are what
resolvers fetch. A producer **MUST NOT** put secrets in `label` or
`templateVars`, which are recorded for audit and may be rendered into the
published document.

`preRotationCount: 0` disables pre-rotation, and the cost is asymmetric: with
no successor committed in advance, a party who steals the current signing key
can rotate to a key of their own as convincingly as the owner can. There is no
recovery path afterwards, only a dispute. Choose a non-zero count unless
something else provides that guarantee.

### Correlation

Everything in the published document is visible to every resolver, including
its service endpoints, which name the transports and mediators the identity is
reached through. A caller **SHOULD NOT** add services or template variables that
reveal more about the deployment than its counterparties need. The hosting
server learns the DID and its path; a serverless DID discloses nothing to a
hosting server, but is only as reachable as wherever the caller serves it.

### Retention

The DID's log is permanent once published: an entry cannot be retracted, only
superseded by a later one or the DID deactivated. A serverless DID exists in the
VTA and nowhere else until the caller publishes the returned `logEntry`. Until
then it does not resolve, and anything issued under it cannot be verified by a
third party; `serverless: true` says so in the response. The VTA's audit record
keeps who created the DID, in which context.

### Consent/purpose

The request is made for one purpose, minting an identity in the named context,
and the keys it creates belong to that context. A consent surface showing this
task to a person **SHOULD** say whether the DID will be published through a
hosting server or must be served by the caller, since only the first is
resolvable when the task completes.
