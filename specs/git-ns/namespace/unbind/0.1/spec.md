---
slug: git-ns/namespace/unbind
version: "0.1"
title: "Git Namespaces — Unbind Namespace"
summary: "A namespace admin ends the VTC's governance of a namespace: every right inside it is revoked and withdrawn from the Trust Registry, and every repository in it becomes detached."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - namespace
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Unbinding withdraws every right in a namespace at once. It must be attributable to the actor on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An unbind replayed after the namespace was bound again would withdraw the new binding's rights."
sideEffects:
  level: destructive
  rationale: "Revokes every right recorded in the namespace and withdraws it from the Trust Registry. Binding again restores none of them, and from then on every repository whose CI check names this VTC fails it."
consequences:
  - "Every right recorded in the namespace is revoked and withdrawn from the Trust Registry; binding again restores none of them."
  - "Every repository in the namespace whose CI check names this VTC fails that check from then on."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a namespace identifier; the response two counts."
retention:
  class: durable
  rationale: "The unbinding and the revocations it caused are part of the namespace's audit history."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
related:
  - git-ns/namespace/bind
  - git-ns/right/revoke
  - git-ns/view
---

## Abstract

Ends a VTC's governance of a namespace bound with [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md). Every right recorded on a resource inside the namespace is revoked and withdrawn from the Trust Registry, every repository in it becomes `detached`, and the namespace record is removed. Unbinding a `pending` namespace cancels the binding.

Unbinding does not touch the forge: the repositories stay, and so does the forge app or bot if nobody removes it. But every repository in the namespace whose CI check names this VTC will fail that check from then on, because the rights it queries are gone. That is the honest outcome of withdrawing trust, and the reason this task is destructive.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **`git.ns.admin` on the namespace**, or the **community-administrator capability**, which could bind it again anyway. Anyone else is refused with `permissionDenied`. The `proof` establishes who asked, never that they may.

## Definitions

**`namespace`** — the namespace's identifier.

**`rightsRevoked`**, **`reposDetached`** — what unbinding removed, so the actor can see the extent of it.

## Request

The actor sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses an identifier it does not know with `git-ns:unknownNamespace`.
2. Checks the authorization above, and evaluates its policy, which may refuse with `git-ns:policyDenied`.
3. Revokes every record whose resource the namespace contains, including every `git.ns.admin` — the last-owner and last-admin invariants do not apply to a namespace that is going away — withdraws them from its registry projection, and records each as revoked by the actor.
4. Sets every repository in the namespace to `detached` and stops projecting to the forge. It sends the bridge no job: removing the app or bot is for a forge owner to do.
5. Removes the namespace. A later [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md) of the same owner is a new binding with a new identifier and no rights.

### Unbinding a manual-mode namespace

```json
{
  "id": "urn:uuid:f4913732-7b1a-4820-a110-407bab2e4601",
  "type": "https://trusttasks.org/spec/git-ns/namespace/unbind/0.1",
  "threadId": "urn:uuid:f4913732-7b1a-4820-a110-407bab2e4601",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2027-01-10T09:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z7C1TX"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2027-01-10T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zeSLNHXSyA3v4j1EdFCiMWs6d827zNL9GYSJB5yR3CSQC1hGxhtWCAng2dwo3zDjcRdJeFmTizFKUtJNWsmF7Vt"
  }
}
```

## Response

The VTC, now responding, returns what was removed, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### What unbinding removed

```json
{
  "id": "urn:uuid:f4913732-7b1a-4820-a110-407bab2e4602",
  "type": "https://trusttasks.org/spec/git-ns/namespace/unbind/0.1#response",
  "threadId": "urn:uuid:f4913732-7b1a-4820-a110-407bab2e4601",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2027-01-10T09:00:02Z",
  "payload": {
    "namespace": "ns_01J8Z7C1TX",
    "rightsRevoked": 14,
    "reposDetached": 3
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2027-01-10T09:00:02Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zA7mUBg4joL9kaAHiz6TuANDMNghEfRkuixzHJYudsMeK55B8NBWbAcTABqELEViNj8RZbmbU8w4Vb3x4ZpaSp2"
  }
}
```

## Security & Privacy

### Data carried

A namespace identifier in; the identifier and two counts out.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority whose rights are being withdrawn; the actor `pairwise`. The withdrawal is visible to anyone who queries the registry before and after.

### Retention

Durable. The unbinding, and every revocation it caused, stay in the VTC's audit history: commits made while the namespace was bound remain explicable.

### Consent/purpose

The purpose is to end the VTC's authority over the namespace. Whether unbinding warrants a step-up or a second confirmation is the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
