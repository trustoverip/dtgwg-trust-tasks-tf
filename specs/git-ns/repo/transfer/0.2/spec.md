---
slug: git-ns/repo/transfer
version: "0.2"
title: "Git Namespaces — Transfer Repository Ownership"
summary: "An owner hands their ownership of a repository to someone else: the recipient becomes an owner and the caller's own record is revoked, atomically."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repository
  - ownership
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
  rationale: "A transfer changes who governs a repository. It must be attributable to the transferring owner on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A transfer replayed after ownership came back would hand it away again."
sideEffects:
  level: mutating
  rationale: "Grants git.repo.own to the recipient and revokes the caller's. The recipient can transfer it back."
consequences:
  - "The caller loses ownership of the repository, and with it the maintainer and committer rights ownership implied, unless they hold those by separate grants."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a resource and a DID; the response the repository record."
retention:
  class: durable
  rationale: "Both halves of the transfer are part of the repository's audit history."
errorCodes:
  - code: git-ns:unknownRepo
    meaning: "The resource names no repository this VTC records."
    retryable: false
  - code: git-ns:repoNotActive
    meaning: "The repository's state does not allow this operation; each task says which states it accepts."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/repo/transfer:notOwner
    meaning: "The caller holds no explicit `git.repo.own` record on this repository to hand over."
    retryable: false
  - code: git-ns/repo/transfer:selfTransfer
    meaning: "`to` is the caller."
    retryable: false
related:
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/view
---

## Abstract

An owner hands their ownership of a repository to someone else. `to` is recorded as an owner and the caller's own `git.repo.own` record is revoked, in one step, so the repository never has fewer owners than before. Ownership moves between people; the repository stays where it is on the forge. Moving a repository to another forge owner is a forge operation, which the bridge reports as `repoTransferred` ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.2/spec.md)).

## Changes from 0.1

The schema pins [`git-ns/_shared/0.3`](../../../_shared/0.3/git-ns.schema.json), which narrows `Did` to the syntax of [W3C DID Core §3.1](https://www.w3.org/TR/did-core/#did-syntax). `0.1` pinned `_shared/0.1`, whose `Did` accepted anything without whitespace after `did:<method>:` — shell metacharacters, quotes, backticks, `/`, `?` and `#` included — so a value that was not a DID at all validated, and travelled on to every surface that later displayed, logged or quoted it. The members affected here are the request's `to`, and `repo.owners` in the response. Each now carries a bare DID only: `did:`, a method name of lowercase letters and digits, and a method-specific id of colon-separated segments drawn from `A-Z a-z 0-9 . - _` and percent-encoded octets, the last one non-empty. A DID URL — a path, a query, or a `#` fragment such as a verification-method id — is refused: every one of these members names a party, never a key.

Narrowing a constraint is a breaking change, released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules). A `0.1` document is a valid `0.2` document exactly when every DID it carries is well-formed. Everything else is unchanged from `0.1` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **an explicit `git.repo.own` record on the repository** — this task hands over the caller's own record, and a namespace admin whose ownership is only implied has none to hand over (they use [`git-ns/right/grant`](../../../../git-ns/right/grant/0.2/spec.md) instead). A caller without one is refused with `git-ns/repo/transfer:notOwner`. Receiving ownership is a grant of `git.repo.own` to `to` and is subject to the same [fixed rules](../../../../git-ns/right/grant/0.2/spec.md#the-fixed-rules) and policy as [`git-ns/right/grant`](../../../../git-ns/right/grant/0.2/spec.md).

## Definitions

**`resource`** — the repository.

**`to`** — who receives ownership. It may already be a co-owner, in which case the transfer only removes the caller.

## Request

The owner sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a resource it does not record with `git-ns:unknownRepo`, and one that is not `active` or `orphaned` with `git-ns:repoNotActive`.
2. Checks the authorization above; refuses `to` equal to the caller with `git-ns/repo/transfer:selfTransfer`; evaluates policy for `to`, which may refuse with `git-ns:policyDenied`.
3. Records `git.repo.own` for `to` (unless already recorded), with `grantedBy` set to the caller, and revokes the caller's `git.repo.own` record, as one atomic change. The caller's other records on the repository are untouched; the rights their ownership implied end with it.
4. Publishes the change and projects it to the forge.

### Alice hands `widgets` to Bob

```json
{
  "id": "urn:uuid:71a75dfb-a35b-43b3-ab76-df9cbb52db01",
  "type": "https://trusttasks.org/spec/git-ns/repo/transfer/0.1",
  "threadId": "urn:uuid:71a75dfb-a35b-43b3-ab76-df9cbb52db01",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "resource": "github.com/acme/widgets",
    "to": "did:webvh:QmBobScid2:acme-vtc.example:bob"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zTMFRSHsScYJV8oiMWKqHeXQ9eCjdAirGE9ANQGgue74WuCj6D6VMSoL6wGqy6x6PWtPy4WcTAcQLcSSefc8irf"
  }
}
```

## Response

The VTC, now responding, returns the repository, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### After the transfer

```json
{
  "id": "urn:uuid:71a75dfb-a35b-43b3-ab76-df9cbb52db02",
  "type": "https://trusttasks.org/spec/git-ns/repo/transfer/0.1#response",
  "threadId": "urn:uuid:71a75dfb-a35b-43b3-ab76-df9cbb52db01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "repo": {
      "resource": "github.com/acme/widgets",
      "forgeId": "812736451",
      "visibility": "public",
      "state": "active",
      "owners": [
        "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "did:webvh:QmBobScid2:acme-vtc.example:bob"
      ],
      "bootstrap": {
        "workflow": true,
        "keyring": true,
        "variables": true,
        "requiredCheck": true
      },
      "sync": {
        "state": "pending",
        "drift": []
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zWE85YvMf1gmRJBufjs48toPbuer8ascUGzayhy5FssGWNKrkf5mqEnZ9WFiFnrNhRRh26mGzBKQTRZF4di7z73"
  }
}
```

## Security & Privacy

### Data carried

A resource and a DID in; the repository record out.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority; the owner `pairwise`. The change of owner is visible to anyone who queries the registry.

### Retention

Durable. Both halves of the transfer stay in the repository's audit history.

### Consent/purpose

The purpose is to move ownership. Whether `to` must accept first, and whether a transfer warrants a step-up, are the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
