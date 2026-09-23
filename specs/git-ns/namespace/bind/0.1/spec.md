---
slug: git-ns/namespace/bind
version: "0.1"
title: "Git Namespaces — Bind Namespace"
summary: "A VTC administrator binds the VTC to one owner on one forge, so the VTC governs repositories there. In bridge mode the namespace stays pending until the administrator proves control of the owner on the forge; in manual mode it is bound at once."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - namespace
  - organisation
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Binding makes the VTC the authority over an owner's repositories and grants the binding administrator the namespace's first admin right. It must be attributable to that administrator on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A bind replayed after the namespace was unbound would start binding it again."
sideEffects:
  level: mutating
  rationale: "Records a pending or bound namespace and, once bound, the binding administrator's git.ns.admin. Reversible with git-ns/namespace/unbind."
consequences:
  - "Every right later granted under this namespace is published to the community's Trust Registry, where anyone can read who owns and who may commit to each repository."
  - "The binding administrator receives git.ns.admin on the namespace."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a forge host, an owner name and a mode. The response returns the namespace and, in bridge mode, a single-use binding URL."
retention:
  class: durable
  rationale: "The binding is the root of every right recorded in the namespace."
errorCodes:
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/namespace/bind:alreadyBound
    meaning: "This VTC has already bound, or is already binding, this forge and owner."
    retryable: false
  - code: git-ns/namespace/bind:noBridge
    meaning: "`mode` is `bridge` and no bridge serving this forge is configured for this VTC."
    retryable: false
related:
  - git-ns/namespace/unbind
  - git-ns/bridge/job
  - git-ns/bridge/event
  - git-ns/right/grant
  - git-ns/view
---

## Abstract

A **namespace** binds a VTC to one owner — an organisation or a personal account — on one forge. Once it is bound, the VTC governs who may create repositories under that owner, who owns each one, who maintains it and whose commits its CI check accepts, as the rights described in [the rights model](../../../../git-ns/right/grant/0.1/spec.md#the-rights-model). A VTC may bind several namespaces, on several forges.

This task starts a binding. In **bridge mode** the forge-side proof that the administrator controls the owner is carried by the community's bridge: installing the community's own forge app on the owner (GitHub), or signing in to the forge as an owner of the organisation (Forgejo). The namespace is `pending` until that proof arrives. In **manual mode** there is no automation and no forge-side proof: the namespace is bound at once, and each repository's own CI configuration naming this VTC is what connects it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is the **community-administrator capability** in the VTC's own access control. No git right suffices, because before binding there are none; binding is what creates the first one. In bridge mode a second piece of evidence is required before the namespace is bound: control of the owner on the forge, proved to the bridge by installing the community's app or by signing in as an organisation owner, and reported as a `bindCompleted` event ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.1/spec.md)). The bridge's single-use nonce in `next.url` is what ties that proof to this request; a proof nobody in this VTC started is not bound.

The `proof` on this request establishes which administrator asked, not that they may; that is checked against the VTC's access control ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**`forge`**, **`owner`** — the namespace, as `<forge>/<owner>`, lowercase.

**`mode`** — `bridge` or `manual`, as in the schema.

**`namespace.state`** — `pending` until the forge-side proof arrives, then `bound`. A manual-mode namespace is `bound` at once.

**`next.url`** — where the administrator goes to prove control. It carries the bridge's single-use nonce and is valid only as long as the bridge says ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.1/spec.md) `next.expiresAt`).

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a sender without the community-administrator capability with `permissionDenied`.
2. Refuses a `forge` and `owner` that this VTC has already bound, or is binding, with `git-ns/namespace/bind:alreadyBound`.
3. Evaluates its git-namespace policy (for example, which forges it allows), which may refuse with `git-ns:policyDenied`.
4. In bridge mode, refuses with `git-ns/namespace/bind:noBridge` when no bridge serving this forge is configured. Otherwise records the namespace as `pending`, sends the bridge a `beginBind` job ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.1/spec.md)), and returns the bridge's `next.url`. While completing the proof the bridge also prepares whatever namespace-wide protection its forge needs so that a pull request cannot satisfy its own check — on a GitHub organisation, a bridge-managed workflow repository and an organisation ruleset that runs verify-trust from it at a pinned commit — and the VTC **SHOULD** tell the administrator which forge permissions that takes. When the bridge reports `bindCompleted` for that job, the VTC records the owner's forge id and kind, sets the namespace to `bound`, and records `git.ns.admin` on `<forge>/<owner>` for the administrator who sent this request. When the bridge reports the job `failed` instead (the nonce expired, or the proof did not match), the VTC discards the pending namespace.
5. In manual mode, records the namespace as `bound` and records `git.ns.admin` for the administrator at once.

A VTC **MUST** make clear to the administrator, before the namespace is bound, that rights published under it are public: anyone can read from the Trust Registry who owns and who may commit to each repository in it. How it does so is its own.

### Binding a GitHub organisation through the bridge

```json
{
  "id": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1701",
  "type": "https://trusttasks.org/spec/git-ns/namespace/bind/0.1",
  "threadId": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1701",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "forge": "github.com",
    "owner": "acme",
    "mode": "bridge"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zLDdgsoRSHNLhQUJF9qMYB3YE7p41YyJRe4rB9CbQ8g8jivGKgNnHxAeKMPXtCFfwoPkGBzLjq6Wmbx8C597fCB"
  }
}
```

### Declaring a Codeberg organisation in manual mode

```json
{
  "id": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1703",
  "type": "https://trusttasks.org/spec/git-ns/namespace/bind/0.1",
  "threadId": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1703",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "forge": "codeberg.org",
    "owner": "acme",
    "mode": "manual"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5ey7bYryGdKep1V7KEgjY8qja6ZgMSoKkyMr467YWQv8F6iRCmFvJxY89aXftvDR4tuXKn7pbiFe3QsuP7y9S8"
  }
}
```

## Response

The VTC, now responding, returns the namespace, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). `next` is present exactly when the namespace is `pending`. Refusals use `trust-task-error`.

### Pending: send the administrator to install the app

```json
{
  "id": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1702",
  "type": "https://trusttasks.org/spec/git-ns/namespace/bind/0.1#response",
  "threadId": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1701",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "namespace": {
      "id": "ns_01J8Z6Q4M2",
      "forge": "github.com",
      "owner": "acme",
      "mode": "bridge",
      "state": "pending"
    },
    "next": {
      "url": "https://github.com/apps/acme-vgi/installations/new?state=b7Hq2xP9LmR4"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zr9epKdKkNUNA8UWbWNZV8SH7ee2n9h87SkQbbc6CA99tfThEUj8Nc1PyV5oBwBBLk1CFoLc523MBEtWvwX1wXH"
  }
}
```

### Bound at once

```json
{
  "id": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1704",
  "type": "https://trusttasks.org/spec/git-ns/namespace/bind/0.1#response",
  "threadId": "urn:uuid:4fa07603-6606-4b9a-a5c9-824c073d1703",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "namespace": {
      "id": "ns_01J8Z7C1TX",
      "forge": "codeberg.org",
      "owner": "acme",
      "mode": "manual",
      "state": "bound"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zE9R8jFQGXqHf2rD4b2Y5wjvcPsSoKndPn6yA1D99qYdAQvgrKS7ZDHUVetfRhxWy5DjfsDybJBLMBuWn5yGcDk"
  }
}
```

## Security & Privacy

### Data carried

A forge host, an owner name and a mode in; the namespace record and a URL out. None of it is personal. `next.url` carries a single-use nonce, which a VTC **SHOULD** hand only to the administrator who asked and **MUST NOT** log where others can read it: whoever completes the forge-side step with it binds the namespace to this VTC.

### Correlation

The VTC declares `identifierScope: public`, because it is the authority every repository in the namespace names and every published right carries. The administrator declares `pairwise`. Binding publicly associates the VTC with the forge owner from the moment the first right is published.

### Retention

Durable. The binding, who made it and when, is the root of every right later recorded in the namespace and is kept for as long as the namespace is bound, and in the audit history after.

### Consent/purpose

The purpose is to let this VTC govern repositories under the owner. Whether binding warrants a step-up or a second confirmation is the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13); this specification only requires that the public consequence is stated first.
