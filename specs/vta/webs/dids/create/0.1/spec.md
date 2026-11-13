---
slug: vta/webs/dids/create
version: "0.1"
title: "VTA did:webs DIDs — Create"
summary: "An administrator creates a did:webs persona, relationship or integration DID in a context: the VTA derives the keys, writes the KERI v2 inception and designations, and publishes keri.cesr through a hosting server. Alpha."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - did-webs
  - keri
  - did
  - create
  - persona
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
  rationale: "Creating a DID commits an identity third parties will resolve and rely on, under keys the VTA alone will hold. The VTA must attribute it to a specific administrator in the audit record independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed create incepts a second AID and publishes a second identifier the operator did not ask for. Key events cannot be withdrawn, only superseded."
sideEffects:
  level: mutating
  rationale: "Derives keys, writes an inception event and its designations, and publishes them to a hosting server. The AID, the domain and the slot are fixed here."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "The request carries control members and public service entries; the response carries the new DID, its public keys and the VTA's ids for them. No key material moves in either direction."
retention:
  class: durable
  rationale: "The VTA keeps the DID's record and keys for the DID's life, and the audit row records who created it in which context."
errorCodes:
  - code: "vta/webs/dids/create:methodDisabled"
    meaning: "This VTA does not create did:webs DIDs: creation is off while the integration is alpha, and is turned on in its configuration. Existing did:webs DIDs are unaffected."
    retryable: false
  - code: "vta/webs/dids/create:serverNotFound"
    meaning: "The VTA has no hosting server registered under `serverId`."
    retryable: false
  - code: "vta/webs/dids/create:hostDoesNotHostWebs"
    meaning: "The hosting server does not host did:webs DIDs: it must run with did:webs hosting (`method-webs`) enabled."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        serverId: { type: string }
  - code: "vta/webs/dids/create:domainNotServed"
    meaning: "The hosting server serves no such domain, or none for this VTA."
    retryable: false
  - code: "vta/webs/dids/create:pathRefused"
    meaning: "The hosting server refused the slot path: its grammar, a reserved prefix, or a path it does not let this VTA use."
    retryable: false
  - code: "vta/webs/dids/create:serviceDidNotPermitted"
    meaning: "The DID would be the durable identity of the VTA, a community or another node. A service DID is did:webvh or did:scid:vh:1 (VTI-KEY-001); did:webs is for persona, relationship and integration DIDs."
    retryable: false
  - code: "vta/webs/dids:serviceIdConflict"
    meaning: "An additional service uses `#didcomm` or `#tsp`, or two services share an id."
    retryable: false
  - code: "vta/webs/dids:publishRefused"
    meaning: "The hosting server refused the publication. Nothing was created; `details.hostError` carries what the server said."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        hostError: { type: string }
related:
  - vta/webs/dids/get
  - vta/webs/dids/list
  - vta/webs/dids/update
  - vta/webs/dids/rotate-keys
  - vta/webs/dids/delete
  - vta/webvh/servers/register
  - did-management/did/register
  - vta/dids/create
---

## Abstract

**VTA did:webs DIDs — Create** makes a `did:webs` DID in a context. The VTA derives
one Ed25519 signing key and one pre-rotated successor, writes a KERI v2 inception
event, designates the DID's aliases, its key agreement key and its services in an
interaction event, and publishes the resulting `keri.cesr` through a hosting server,
which derives the DID document from it.

It creates **persona, relationship and integration DIDs** at key assurance level 1. It
never creates a node's durable identity: that is `did:webvh` or `did:scid:vh:1`
(VTI-KEY-001). What the family shares — which identities, the key model, designations,
hosting, remote-first publication — is in the family's
[CONVENTIONS.md](../../../_shared/0.1/CONVENTIONS.md), and this specification relies on it
throughout.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration it drives is **alpha**: expect breaking changes as the KERI and `did:webs` specifications settle, and a DID created under it may have to be created again.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse with `methodDisabled` when its configuration does not enable `did:webs`
   creation, before anything else is derived or published. The setting gates this task
   only: every other task of the family works for an existing DID with it off.
2. Refuse with `serviceDidNotPermitted` a request that would make the DID a service DID
   ([CONVENTIONS.md §1](../../../_shared/0.1/CONVENTIONS.md#1-which-identities)).
3. Resolve `serverId` to a hosting server it has registered (`serverNotFound`), and
   `domain`, or the server's default, to a domain that server serves for it
   (`domainNotServed`), before incepting: the domain is part of the DID the aliases
   designation names.
4. Derive the keys itself (§3 of the conventions), write the inception event with
   `kt` 1, `nt` 1, the signing key in `k` and the successor's digest in `n`, then one
   interaction event anchoring the designations: the aliases, the DID first, then
   `alsoKnownAs`; the key agreement key; and the services, when any are asked for.
5. Stage the record, publish the stream by `did-management/did/register/0.1` with
   `method` `webs`, and commit only once the host has accepted it
   ([CONVENTIONS.md §6](../../../_shared/0.1/CONVENTIONS.md#6-remote-first)). A refusal
   because the host does not host `did:webs` is `hostDoesNotHostWebs`; a refused slot path
   is `pathRefused`; any other refusal is `vta/webs/dids:publishRefused`. A refused
   publication leaves no DID, no committed record and no active key.

Every request creates a new AID: a create is never answered with an existing DID.
Retrying one is safe only under the framework's duplicate-execution protection (the same
document `id`); a new request makes a second DID.

## Authorization

Authority is the **administrator role over the context** named in `contextId`. The DID's
keys are derived in that context and belong to it, so the authority that governs the
context governs what identities it brings into being. Using `serverId` needs no further
authority: the server is one the VTA's operator registered.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

A persona DID under `people/alice` on a hosting server, reachable over DIDComm and TSP
through the VTA's mediator, also known by its holder's did:webvh:

```json
{
  "id": "5d0c9a2e-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webs/dids/create/0.1",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T10:00:00Z",
  "payload": {
    "contextId": "alice",
    "serverId": "prod",
    "path": "people/alice",
    "label": "Alice — persona",
    "addMediatorService": true,
    "addTspService": true,
    "alsoKnownAs": ["did:webvh:QmAliceScid:example.com:alice"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-11T10:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQExampleProofValueForWebsCreate"
  }
}
```

## Response

The VTA answers with the DID, where its artifacts are served, and the ids of the three
keys it holds for it:

```json
{
  "id": "6e1d0b3f-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webs/dids/create/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T10:00:02Z",
  "threadId": "5d0c9a2e-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "aid": "ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "contextId": "alice",
    "serverId": "prod",
    "domain": "example.com",
    "slot": "people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "keriCesrUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/keri.cesr",
    "didJsonUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/did.json",
    "signingKeyId": "webs-alice-sign-0",
    "signingKey": "DKxy2sgzfplyr-tgwIxS19f2OchFHtLwPWD3v4oYimBx",
    "nextKeyId": "webs-alice-sign-1",
    "keyAgreementKeyId": "webs-alice-ka-0",
    "keyAgreementKey": "CBwO9bU8Qk3n2YQm0xTNu0R1tq2Hk7f1pWx0aJc3Kd5e",
    "sequenceNumber": 1,
    "versionId": "1:EHmLpA0OPWQbm6Ua2A8m2c4XUbJkfR7zTG4MBxwdXNbk",
    "alsoKnownAs": [
      "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
      "did:webvh:QmAliceScid:example.com:alice"
    ],
    "serviceIds": ["#didcomm", "#tsp"],
    "createdAt": "2026-11-11T10:00:02Z"
  }
}
```

A VTA with `did:webs` creation turned off answers with the error, and creates nothing:

```json
{
  "id": "6e1d0b3f-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T10:00:01Z",
  "threadId": "5d0c9a2e-0000-4000-8000-000000000001",
  "payload": {
    "code": "vta/webs/dids/create:methodDisabled",
    "message": "did:webs creation is off on this VTA (alpha). Enable it with [dids] webs_enabled = true.",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/webs/dids/create/0.1",
      "id": "5d0c9a2e-0000-4000-8000-000000000001"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries control members — the context, the hosting server, domain and path,
a label, which services and aliases to designate — and no key material: the VTA derives
every key. The response carries the DID, its public signing and key agreement keys, the
VTA's ids for the three keys it holds, and where the artifacts are served. Everything
published is public by construction, since it is what resolvers fetch: the services name
the transports and the mediator the DID is reached through, and the aliases name every
other identifier the subject answers to. A producer **MUST NOT** put secrets in `label`,
which is recorded for audit, or in `additionalServices`, which are published.

### Correlation

A persona or relationship DID exists to be kept apart from the subject's other
identifiers, and every alias undoes that: it tells every resolver that the two are one
subject. A producer **SHOULD** list in `alsoKnownAs` only identifiers the subject already
gives out to the same audience. The services correlate too: two DIDs designating the same
mediator are reachable at one place. The `path` is published in the DID, so it **SHOULD
NOT** carry a name the subject would not put in the DID itself. The hosting server learns
the DID, its slot, and the times it changes.

### Retention

A published key event is permanent: the stream can be superseded and the DID deactivated,
but nothing withdraws what a watcher or a verifier has already copied. The VTA keeps the
record and the keys for the DID's life, and after deactivation keeps the final stream as
the evidence anything the DID signed is checked against. Its audit record keeps who
created the DID, in which context.

### Consent/purpose

The request is made for one purpose, bringing an identity into being in the named
context, and the keys it creates belong to that context. A consent surface showing this
task to a person **SHOULD** say that the integration is alpha, and that the DID's key
agreement key and services are designations that resolve in full only for verifiers
whose `did:webs` resolver implements them
([CONVENTIONS.md §4](../../../_shared/0.1/CONVENTIONS.md#4-designations)).
