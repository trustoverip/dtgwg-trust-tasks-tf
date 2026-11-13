---
slug: vta/webs/dids/update
version: "0.1"
title: "VTA did:webs DIDs — Update"
summary: "An administrator replaces a did:webs DID's designated services, key agreement key or designated aliases. Each is a designation anchored by an interaction event; the key state does not change. Alpha."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - did-webs
  - keri
  - did
  - update
  - designation
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
  rationale: "The VTA signs, with a key the caller never holds, an event that becomes part of a public, append-only identity history and changes where the DID is reached and what it is known as. It must attribute the change to a specific administrator independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An update anchors an event that cannot be withdrawn. Replayed later, it would restore services, aliases or a key agreement key a later update replaced."
sideEffects:
  level: mutating
  rationale: "Issues designations, revokes the ones they replace, and anchors both in a published interaction event. No key of the key state changes, so the change authority is not shifted."
consequences:
  - "Publishes an interaction event to the DID's public key event log. The replaced designations stay in its history, revoked."
  - "Replacing the key agreement key changes the key every peer encrypts to; messages encrypted to the old key after they next resolve the DID fail."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "The request carries public service entries and aliases; the response carries the event's version and the designations in force. No key material."
retention:
  class: durable
  rationale: "The published event is permanent, and the audit row records who changed the DID's designations."
errorCodes:
  - code: "vta/webs/dids/update:notFound"
    meaning: "No such did:webs DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webs/dids/update:versionConflict"
    meaning: "The DID's latest event no longer matches `expectedVersionId`. Re-read it and re-apply the change."
    retryable: false
  - code: "vta/webs/dids/update:noChange"
    meaning: "The request would replace no designation, or replace each with one equal to the one in force. Nothing is published."
    retryable: false
  - code: "vta/webs/dids:serviceIdConflict"
    meaning: "An additional service uses `#didcomm` or `#tsp`, an id a KERI endpoint reply of the AID establishes, or an id another service in the set uses."
    retryable: false
  - code: "vta/webs/dids:deactivated"
    meaning: "The DID is deactivated. Nothing can be anchored in its log."
    retryable: false
  - code: "vta/webs/dids:publishRefused"
    meaning: "The hosting server refused the publication. Nothing was committed; `details.hostError` carries what the server said."
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

**VTA did:webs DIDs — Update** changes what a `did:webs` DID's document says beyond its
keys: the services peers reach it at, the key they encrypt to, and the other identifiers
it is known as. Each of the three is a **designation** — an ACDC the AID issues in its
own registry — and replacing one issues a new designation, revokes the old one, and
anchors both in one interaction event signed by the current signing key
([CONVENTIONS.md §4](../../../_shared/0.1/CONVENTIONS.md#4-designations)).

None of it touches the key state. The signing key and its committed successor stay as
they were: a routine change never spends the commitment that recovers the DID. Keys
change only through [`rotate-keys`](../../rotate-keys/0.1/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration is **alpha**, and the key agreement and service designations are proposals to the `did:webs` specification.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Find the DID among the contexts the caller may act in (`notFound`), refuse a
   deactivated one (`vta/webs/dids:deactivated`), and complete any staged rotation
   before going on ([CONVENTIONS.md §6](../../../_shared/0.1/CONVENTIONS.md#6-remote-first)).
2. Refuse with `versionConflict` when `expectedVersionId` is present and does not name the
   latest committed event.
3. Compute each replacement the request asks for — `services` replaces the whole service
   set, `alsoKnownAs` the further aliases (the DID itself stays first), and
   `rotateKeyAgreementKey` a freshly derived X25519 key — and refuse with `noChange` when
   none differs from the designation in force. It **MUST NOT** publish an event that
   changes nothing.
4. Refuse a service set whose ids conflict (`vta/webs/dids:serviceIdConflict`).
5. Issue the new designations and revoke those they replace in **one** interaction event,
   publish the whole stream to the host, and commit only once the host has accepted it.

A consumer **MUST NOT** change `k` or `n` in this task, and **MUST NOT** designate as key
agreement key the X25519 key converted from a signing key. It **MAY** keep a replaced key
agreement key to decrypt messages already encrypted to it, and **MUST NOT** designate it
again.

Updating is **setting a state**, but an update that sets the state already in force is
refused with `noChange` rather than answered as a success, because answering it would
mean either publishing an empty event or reporting a `versionId` the request did not
produce.

## Authorization

Authority is the **administrator role over the DID's context**. The VTA signs the event
with the DID's current signing key, which it holds in that context and the caller never
does; the role is what entitles the caller to have it signed.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

Withdraw TSP, keep DIDComm, add a file store, and drop the did:webvh alias:

```json
{
  "id": "b3625084-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webs/dids/update/0.1",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T12:00:00Z",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "services": {
      "mediator": true,
      "tsp": false,
      "additional": [
        { "id": "#files", "type": "FileStore", "serviceEndpoint": "https://files.example.com/alice" }
      ]
    },
    "alsoKnownAs": [],
    "expectedVersionId": "1:EHmLpA0OPWQbm6Ua2A8m2c4XUbJkfR7zTG4MBxwdXNbk",
    "label": "file store; unlink webvh"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-11T12:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z4Xq7WExampleProofValueForWebsUpdate"
  }
}
```

## Response

```json
{
  "id": "c4736195-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webs/dids/update/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T12:00:02Z",
  "threadId": "b3625084-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "replaced": ["aliases", "services"],
    "sequenceNumber": 2,
    "versionId": "2:EF3dK9wq2mVb0yQz7Lr5Hn1cXtU8pAaJ6sGe4oWiZ0Ck",
    "keyAgreementKeyId": "webs-alice-ka-0",
    "keyAgreementKey": "CBwO9bU8Qk3n2YQm0xTNu0R1tq2Hk7f1pWx0aJc3Kd5e",
    "alsoKnownAs": [
      "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe"
    ],
    "serviceIds": ["#didcomm", "#files"]
  }
}
```

## Security & Privacy

### Data carried

The request carries the DID, the service set and aliases to stand, a flag asking for a
new key agreement key, the concurrency precondition and an audit label. It carries no key
material: the VTA derives any new key and signs with the key it holds. The response
carries the event's version and the designations in force. All of it but the key id and
the label is published. A producer **MUST NOT** put secrets in `label` or in a service
entry.

### Correlation

Withdrawing an alias or a service ends the link only from the event that withdraws it.
The designations it replaces stay in the stream, revoked, and any copy of the stream —
at a watcher, in a verifier's cache — keeps showing that the DID was once known by that
alias or reached at that endpoint. A subject that must keep two identifiers apart
**SHOULD** never designate the link, rather than designate it and withdraw it.

### Retention

The event is permanent once published: it can be superseded by a later one, never
retracted. The VTA's audit record keeps who asked for the change, against which version,
and with what label.

### Consent/purpose

The request is made for one purpose, changing how the named DID is reached and what it
is known as. A consent surface showing it to a person **SHOULD** show the services and
aliases before and after, and **SHOULD** say that the earlier ones remain visible in the
DID's history.
