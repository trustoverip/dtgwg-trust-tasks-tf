---
slug: vta/webs/dids/create
version: "0.2"
title: "VTA did:webs DIDs — Create"
summary: "An administrator creates a did:webs relationship, persona or service DID (a node's durable identity, alpha) in a context, from explicit members or a DID template: the VTA derives the keys, writes the KERI v2 events, and publishes keri.cesr through a hosting server."
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
  - service did
  - did template
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
  rationale: "Creating a DID commits an identity third parties will resolve and rely on, under keys the VTA alone will hold; a service DID becomes the identity a node is known by. The VTA must attribute it to a specific administrator in the audit record independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed create incepts a second AID and publishes a second identifier the operator did not ask for. Key events cannot be withdrawn, only superseded."
sideEffects:
  level: mutating
  rationale: "Derives keys, writes an inception event and its designations, and publishes them to a hosting server. The AID, the domain, the slot and the DID's class are fixed here."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "The request carries control members, a template name and variables, and public service entries; the response carries the new DID, its public keys and the VTA's ids for them. No key material moves in either direction."
retention:
  class: durable
  rationale: "The VTA keeps the DID's record, its class and its keys for the DID's life, and the audit row records who created it in which context, from which template."
errorCodes:
  - code: "vta/webs/dids/create:methodDisabled"
    meaning: "This VTA does not create did:webs DIDs: creation is off while the integration is alpha, and is turned on in its configuration. Existing did:webs DIDs are unaffected."
    retryable: false
  - code: "vta/webs/dids/create:serviceDidNotPermitted"
    meaning: "`didClass` is `service`, and this VTA's configuration does not enable did:webs service DIDs. A service did:webs is optional and alpha (CONVENTIONS.md §1.1), and is enabled separately from did:webs creation. A relationship or persona DID is unaffected."
    retryable: false
  - code: "vta/webs/dids/create:didClassConflict"
    meaning: "The DID's class disagrees with the identity it would become or replace: a relationship or persona DID that would be a node's or a service's identity, or a service DID that would replace a persona DID as a context's primary. A DID belongs to one class only (VTI-KEY-193)."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        requiredClass: { type: string, enum: [relationship, persona, service] }
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
  - code: "vta/webs/dids/create:templateNotFound"
    meaning: "No template of that name is visible in the requested scope, the global scope or the built-ins."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        template: { type: string }
        templateContext: { type: string }
  - code: "vta/webs/dids/create:templateVarsInvalid"
    meaning: "The template's `requiredVars` are not all supplied in `templateVars`, a variable the template does not declare was supplied, or a placeholder stayed unresolved. Nothing was derived."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        missing: { type: array, items: { type: string } }
        unknown: { type: array, items: { type: string } }
  - code: "vta/webs/dids/create:templateNotRepresentable"
    meaning: "The template describes a document the did:webs derivation cannot reproduce: a key-role template, a key slot or algorithm the key state cannot hold, a further verification method or relationship, a service or alias the designations cannot carry, or a member the derivation does not render (CONVENTIONS.md §9.2). Nothing was derived; `details` names the rule and the members."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [reason]
      properties:
        reason:
          type: string
          enum: [keyRoleTemplate, keyAlgorithm, documentId, controller, verificationMethod, verificationRelationship, keyAgreementRequired, service, alias, documentMember]
        members:
          type: array
          items: { type: string }
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
  - vta/webvh/dids/create
  - vta/did-templates/get
  - did-management/did/register
  - vta/dids/create
---

## Abstract

**VTA did:webs DIDs — Create** makes a `did:webs` DID in a context. The VTA derives
one Ed25519 signing key and one pre-rotated successor, writes a KERI v2 inception
event, designates the DID's aliases, its key agreement key and its services in an
interaction event, and publishes the resulting `keri.cesr` through a hosting server,
which derives the DID document from it.

Every DID is created in a stated **DID class** — `relationship`, `persona` or `service` —
at key assurance level 1. A `service` DID is the durable identity of the VTA itself, a
VTC, a VTN, a host service or an integration: an **optional, alpha** use of `did:webs`
that a VTA enables separately, and that peers which have not declared `did:webs`
resolution may not be able to verify. A key-role identity (level 2 or 3) is created by
[`vta/dids/create`](../../../../dids/create/1.1/spec.md) with the `webs` method profile,
not here.

The DID's services, aliases and integration kind come either from explicit members or
from a **DID template**, as for [`vta/webvh/dids/create`](../../../../webvh/dids/create/1.2/spec.md).
A `did:webs` document is derived, not authored, so the VTA maps the template's document
onto the designations the derivation expresses and refuses one it cannot reproduce. The
keys are always the VTA's.

What the family shares — the DID classes, the key model, designations, hosting,
remote-first publication and the template mapping — is in the family's
[CONVENTIONS.md](../../../_shared/0.2/CONVENTIONS.md), and this specification relies on it
throughout.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration it drives is **alpha**: expect breaking changes as the KERI and `did:webs` specifications settle, and a DID created under it may have to be created again. A service `did:webs` follows the amendment of VTI-KEY-001 and VTI-KEY-003 that names `did:webs` an optional durable-identity method ([dtgwg-vti-spec#120](https://github.com/trustoverip/dtgwg-vti-spec/issues/120)); until that amendment is merged, a VTA creating one diverges from VTI-KEY-001 as it stands, and the divergence belongs in that specification's divergence register (Appendix F).

## Changes from 0.1

1. **`didClass` is REQUIRED.** 0.1 created "persona, relationship and integration" DIDs
   without saying which, and refused every service DID with `serviceDidNotPermitted`. 0.2
   creates a DID in a stated class, records it, and returns it. An integration's identity
   is a service DID (VTI-KEY-192), so `service` covers what 0.1 called an integration DID.
2. **Service DIDs are allowed** (CONVENTIONS.md §1.1). `serviceDidNotPermitted` now means
   only that the VTA's configuration does not enable `did:webs` service DIDs. The class
   rules that remain are `didClassConflict` (CONVENTIONS.md §1.2).
3. **DID templates.** `template`, `templateContext` and `templateVars`, with
   `templateNotFound`, `templateVarsInvalid` and `templateNotRepresentable`, and the
   mapping of CONVENTIONS.md §9. A template excludes the explicit document members. The
   response names the template, the scope it resolved in and its `kind`.
4. The shared definitions move to [`vta/webs/_shared/0.2`](../../../_shared/0.2/webs-records.schema.json),
   which adds `WebsDidClass`; the slot path and service shapes are unchanged.

A 0.1 request is answered as 0.1 specifies, including its refusal of a service DID. A VTA
that serves both **MUST NOT** infer a class for a 0.1 request; it records none, which the
family's other tasks read as "not a service DID".

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse with `methodDisabled` when its configuration does not enable `did:webs`
   creation, before anything else is derived or published. The setting gates this task
   only: every other task of the family works for an existing DID with it off.
2. Refuse with `serviceDidNotPermitted` a request whose `didClass` is `service` when its
   configuration does not enable `did:webs` service DIDs
   ([CONVENTIONS.md §1.1](../../../_shared/0.2/CONVENTIONS.md#11-service-dids-optional-and-alpha)).
3. Refuse with `didClassConflict` a request whose class disagrees with the identity it
   would become or replace
   ([CONVENTIONS.md §1.2](../../../_shared/0.2/CONVENTIONS.md#12-one-class-only)), with
   `details.requiredClass` when one class would be accepted.
4. Resolve `serverId` to a hosting server it has registered (`serverNotFound`), and
   `domain`, or the server's default, to a domain that server serves for it
   (`domainNotServed`), before incepting: the domain is part of the DID the aliases
   designation names.
5. With `template`: resolve and render it, and check every member of the rendered document
   against [CONVENTIONS.md §9](../../../_shared/0.2/CONVENTIONS.md#9-did-templates)
   (`templateNotFound`, `templateVarsInvalid`, `templateNotRepresentable`) **before** it
   derives a key. Rendering needs the DID, which needs the AID; the VTA computes the
   inception event, renders, checks, and only then commits the keys it was computed with,
   so a refused template leaves no key, record or published event.
6. Derive the keys itself (§3 of the conventions), write the inception event with
   `kt` 1, `nt` 1, the signing key in `k` and the successor's digest in `n`, then one
   interaction event anchoring the designations: the aliases, the DID first, then
   `alsoKnownAs` or the template's; the key agreement key; and the services, from
   `addMediatorService`, `addTspService` and `additionalServices`, or from the template.
7. Stage the record, with its class, and with the template's name, scope and kind when one
   was used; publish the stream by `did-management/did/register/0.1` with `method` `webs`;
   and commit only once the host has accepted it
   ([CONVENTIONS.md §6](../../../_shared/0.2/CONVENTIONS.md#6-remote-first)). A refusal
   because the host does not host `did:webs` is `hostDoesNotHostWebs`; a refused slot path
   is `pathRefused`; any other refusal is `vta/webs/dids:publishRefused`. A refused
   publication leaves no DID, no committed record and no active key.
8. Return `didClass` as created, and `template`, `templateScope` and `templateKind`
   exactly when the request named a template.

The schema refuses a request carrying `template` with any of `addMediatorService`,
`addTspService`, `additionalServices` or `alsoKnownAs`, and `templateContext` or
`templateVars` without `template` ([CONVENTIONS.md §9.3](../../../_shared/0.2/CONVENTIONS.md#93-template-or-explicit-members-not-both)).
A VTA **MUST NOT** merge a template with explicit members.

Every request creates a new AID: a create is never answered with an existing DID.
Retrying one is safe only under the framework's duplicate-execution protection (the same
document `id`); a new request makes a second DID.

A **producer** creating a service DID **SHOULD** check that the peers the node will
deal with have declared `did:webs` resolution, and keep a `did:webvh` or `did:scid:vh:1`
identity for those that have not. A producer creating a VTA's own identity, a VTC or a
VTN **SHOULD** consider level 2 or higher (VTI-KEY-196), which this task does not create.

## Authorization

Authority is the **administrator role over the context** named in `contextId`. The DID's
keys are derived in that context and belong to it, so the authority that governs the
context governs what identities it brings into being. Using `serverId` needs no further
authority: the server is one the VTA's operator registered. Using a template needs no
further authority than reading it in the scope it resolves in: a context's templates are
visible to that context's administrators, the global templates and the built-ins to every
administrator.

Creating a `service` DID needs no further role, because it is not, by itself, adopted as
anyone's identity: making it the VTA's own DID, or a community's, is the task that adopts
it, under that task's authority (CONVENTIONS.md §1.2 item 3). `setPrimary` on a context
provisioned for a node changes which DID that context presents, and is the context
administrator's to make.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

### A persona DID with explicit members

A persona DID under `people/alice`, reachable over DIDComm and TSP through the VTA's
mediator, also known by its holder's did:webvh:

```json
{
  "id": "5d0c9a2e-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/vta/webs/dids/create/0.2",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T10:00:00Z",
  "payload": {
    "contextId": "alice",
    "serverId": "prod",
    "didClass": "persona",
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

### A witness host service's identity from a template

The durable identity of a witness, shaped by the built-in `dtg-witness` template: one
signing key in `authentication` and `assertionMethod`, a key agreement key, and `#tsp`
and `#didcomm` services at the mediator. Every member of that template is one the
derivation expresses, so it is honoured whole:

```json
{
  "id": "5d0c9a2e-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/vta/webs/dids/create/0.2",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T10:05:00Z",
  "payload": {
    "contextId": "witness-1",
    "serverId": "prod",
    "didClass": "service",
    "path": "witnesses/w1",
    "label": "Witness 1",
    "template": "dtg-witness",
    "templateVars": { "MEDIATOR_DID": "did:webvh:QmMediatorScid:mediator.example" },
    "setPrimary": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-11T10:05:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQExampleProofValueForWebsCreateTemplate"
  }
}
```

## Response

The persona DID: where its artifacts are served, and the ids of the three keys the VTA
holds for it:

```json
{
  "id": "6e1d0b3f-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/vta/webs/dids/create/0.2#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T10:00:02Z",
  "threadId": "5d0c9a2e-0000-4000-8000-000000000011",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "aid": "ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
    "contextId": "alice",
    "didClass": "persona",
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

The witness's identity, shaped by the built-in template:

```json
{
  "id": "6e1d0b3f-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/vta/webs/dids/create/0.2#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T10:05:02Z",
  "threadId": "5d0c9a2e-0000-4000-8000-000000000012",
  "payload": {
    "did": "did:webs:example.com:witnesses:w1:EAmWkL0hB3k9cY2pTqZx8RvN1sG4uJ6oD5fH7iQ0eMnc",
    "aid": "EAmWkL0hB3k9cY2pTqZx8RvN1sG4uJ6oD5fH7iQ0eMnc",
    "contextId": "witness-1",
    "didClass": "service",
    "serverId": "prod",
    "domain": "example.com",
    "slot": "witnesses/w1/EAmWkL0hB3k9cY2pTqZx8RvN1sG4uJ6oD5fH7iQ0eMnc",
    "keriCesrUrl": "https://example.com/witnesses/w1/EAmWkL0hB3k9cY2pTqZx8RvN1sG4uJ6oD5fH7iQ0eMnc/keri.cesr",
    "didJsonUrl": "https://example.com/witnesses/w1/EAmWkL0hB3k9cY2pTqZx8RvN1sG4uJ6oD5fH7iQ0eMnc/did.json",
    "signingKeyId": "webs-witness-1-sign-0",
    "signingKey": "DPq3vX0mZr7Lk2sW9eT4yU1oI8aS6dF5gH3jK0lZxCvB",
    "nextKeyId": "webs-witness-1-sign-1",
    "keyAgreementKeyId": "webs-witness-1-ka-0",
    "keyAgreementKey": "CMn8bV7cX6zL5kJ4hG3fD2sA1qW0eR9tY8uI7oP6aS5d",
    "sequenceNumber": 1,
    "versionId": "1:EKr0p9oQ8iW7uY6tR5eE4wQ3aS2dF1gH0jK9lZ8xC7vB",
    "alsoKnownAs": [
      "did:webs:example.com:witnesses:w1:EAmWkL0hB3k9cY2pTqZx8RvN1sG4uJ6oD5fH7iQ0eMnc"
    ],
    "serviceIds": ["#tsp", "#didcomm"],
    "primary": true,
    "template": "dtg-witness",
    "templateScope": "builtin",
    "templateKind": "dtg-witness",
    "createdAt": "2026-11-11T10:05:02Z"
  }
}
```

The built-in `swiyu-verifier` template keeps its Ed25519 key out of the document
(`publish: false`) and publishes a P-256 key in `authentication` and `assertionMethod`.
Neither is something the key state of a level-1 `did:webs` can hold, so it is refused
before any key is derived:

```json
{
  "id": "6e1d0b3f-0000-4000-8000-000000000013",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T10:06:01Z",
  "threadId": "5d0c9a2e-0000-4000-8000-000000000013",
  "payload": {
    "code": "vta/webs/dids/create:templateNotRepresentable",
    "message": "template 'swiyu-verifier' needs an unpublished signing key and a P-256 key; a did:webs key state holds one published Ed25519 key. Create this identity as did:webvh.",
    "retryable": false,
    "details": {
      "reason": "keyAlgorithm",
      "members": ["/keys/signing", "/keys/verifier"]
    },
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/webs/dids/create/0.2",
      "id": "5d0c9a2e-0000-4000-8000-000000000013"
    }
  }
}
```

A VTA whose configuration does not enable `did:webs` service DIDs answers a `service`
request with the error, and creates nothing:

```json
{
  "id": "6e1d0b3f-0000-4000-8000-000000000014",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-11-11T10:05:01Z",
  "threadId": "5d0c9a2e-0000-4000-8000-000000000012",
  "payload": {
    "code": "vta/webs/dids/create:serviceDidNotPermitted",
    "message": "did:webs service DIDs are off on this VTA (alpha). Peers that have not declared did:webs resolution may not verify one; enable it with [dids] webs_service_dids = true, or create the identity as did:webvh.",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vta/webs/dids/create/0.2",
      "id": "5d0c9a2e-0000-4000-8000-000000000012"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries control members — the context, the DID's class, the hosting server,
domain and path, a label, which services and aliases to designate or which template to
shape them from and its variables — and no key material: the VTA derives every key, and a
template cannot supply one. The response carries the DID, its class, its public signing
and key agreement keys, the VTA's ids for the three keys it holds, where the artifacts are
served, and the template used. Everything published is public by construction, since it
is what resolvers fetch: the services name the transports and the mediator the DID is
reached through, and the aliases name every other identifier the subject answers to. A
producer **MUST NOT** put secrets in `label`, which is recorded for audit, or in
`additionalServices` or `templateVars`, which are published.

### Correlation

A persona or relationship DID exists to be kept apart from the subject's other
identifiers, and every alias undoes that: it tells every resolver that the two are one
subject. A producer **SHOULD** list in `alsoKnownAs`, or let a template list, only
identifiers the subject already gives out to the same audience. The services correlate
too: two DIDs designating the same mediator are reachable at one place, and two DIDs made
from one template share its shape. The `path` is published in the DID, so it **SHOULD NOT**
carry a name the subject would not put in the DID itself. A service DID is public by
design (VTI-KEY-007), and its shape says what kind of node it is. The hosting server learns
the DID, its slot, and the times it changes. `didClass` and the template's kind are not
published.

### Retention

A published key event is permanent: the stream can be superseded and the DID deactivated,
but nothing withdraws what a watcher or a verifier has already copied. The VTA keeps the
record, its class and the keys for the DID's life, and after deactivation keeps the final
stream as the evidence anything the DID signed is checked against. Its audit record keeps
who created the DID, in which context, in which class and from which template.

### Consent/purpose

The request is made for one purpose, bringing an identity of one class into being in the
named context, and the keys it creates belong to that context. A consent surface showing
this task to a person **SHOULD** say that the integration is alpha, and that the DID's key
agreement key and services are designations that resolve in full only for verifiers
whose `did:webs` resolver implements them
([CONVENTIONS.md §4](../../../_shared/0.2/CONVENTIONS.md#4-designations)). For a
`service` DID it **SHOULD** also say that peers which have not declared `did:webs`
resolution may not be able to verify the node's identity, and that the identity is at key
assurance level 1.
