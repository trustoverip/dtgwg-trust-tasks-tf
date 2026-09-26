---
slug: git-ns/bridge/event
version: "0.1"
title: "Git Namespaces — Bridge Event"
summary: "The bridge tells the VTC what happened on the forge — renames, transfers, unmanaged or deleted repositories, role and protection changes, lost access, completed links and bindings — in forge-neutral terms, with the drift it now sees."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - bridge
  - webhook
  - drift
parties:
  - role: bridge
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Events move rights after renames and complete namespace bindings and account links. The VTC must be able to attribute each to the bridge that serves the namespace on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An event replayed later would report a forge state that no longer holds."
sideEffects:
  level: mutating
  rationale: "Updates the VTC's record of the forge — repository names and states, drift, pending bindings and links — and may cause the VTC to move or withdraw published rights."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Events carry forge account ids and logins, which identify people and which the VTC can join to members' DIDs."
retention:
  class: durable
  rationale: "Events are the history of what happened on the forge outside the VTC's control."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
related:
  - git-ns/bridge/job
  - git-ns/bridge/result
  - git-ns/namespace/bind
  - git-ns/account/link
  - git-ns/repo/adopt
---

## Abstract

The bridge tells the VTC that something happened on the forge — translated from the forge's webhook, or found by an inspection sweep where the forge has no webhooks — together with any **drift** it now sees between the forge and the VTC's projection. Events are forge-neutral: the adapter translates each forge's webhooks into the nine types below, so the VTC never parses a forge payload.

The VTC is the source of truth, so an event never changes a right directly. It changes what the VTC knows about the forge, and the VTC decides what follows: rewrite rights after a rename, list an unmanaged repository for adoption, re-apply a weakened protection, alert the namespace admins.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the bridge that serves the namespace**. A VTC **MUST** refuse, with `permissionDenied`, an event from any other issuer. The bridge's own authority on the forge — the app installation, the bot's organisation membership — is what lets it observe; the VTC trusts its report of what it observed and nothing more. In particular a `bindCompleted` or `accountLinked` event is accepted only for a `begin*` job the VTC sent this bridge and has not yet seen completed.

## Definitions

**`event.type`** — one of:

| Type | Means | The VTC |
|---|---|---|
| `repoRenamed` | a managed repository was renamed | moves every right keyed by `forgeId` to `to`, withdraws the tuples published for `from` before publishing any for `to`, and never lets a later repository at `from` inherit them |
| `repoTransferred` | a managed repository moved to another forge owner | as for a rename when `to` is inside a bound namespace; otherwise sets the repository `detached` and withdraws its rights |
| `repoCreatedUnmanaged` | a repository exists in the namespace that the VTC did not create or adopt | records it `unmanaged`, for a namespace admin to adopt |
| `repoDeleted` | a repository was deleted | sets it `detached` and withdraws its rights |
| `roleChanged` | someone's forge role changed outside the bridge | records drift; by policy, re-projects or reports |
| `protectionChanged` | branch protection, a ruleset, or the namespace's required workflow changed outside the bridge | records drift; by policy, re-applies (the default when the required check is no longer required) and alerts the namespace admins |
| `installationRemoved` | the bridge lost its access to the namespace | marks every repository's sync `unchecked` and alerts the namespace admins; rights are unaffected |
| `accountLinked` | a member completed a `beginAccountLink` job | completes the link begun with [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md) |
| `bindCompleted` | the forge-side proof for a `beginBind` job arrived | completes the binding begun with [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md) |

**`drift`** — the complete outstanding drift for each repository the event concerns, replacing what the VTC held for them. A repository whose drift is now empty is back in sync.

## Request

The bridge sends the event to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming bridge verifies the forge's webhook signature before translating it and **MUST NOT** report an event from an unverified webhook. It **MUST NOT** report its own namespace-level repositories — a workflow repository it created when the namespace was bound — as `repoCreatedUnmanaged`; they are part of the binding, not repositories to govern. A conforming VTC refuses a namespace it does not have with `git-ns:unknownNamespace`, applies the event as above, and treats a repeated event as harmless: every effect is keyed by forge id and is idempotent.

### A repository renamed on the forge

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af01",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.1",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af01",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T14:20:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "repoRenamed",
      "forgeId": "812736451",
      "from": "github.com/acme/widgets",
      "to": "github.com/acme/widgets-core"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-10-02T14:20:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zX1BNqDY7HHzMVndp638Ye2Qw7s2xvZVrfbz7ZvnMWvVZHKJ1EEQHyUQGNwEB5ZhAzpCNYXntvAbAmk5tm46YaN"
  }
}
```

### Someone made the required check optional

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af03",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.1",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af03",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-03T08:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "protectionChanged",
      "forgeId": "812736990",
      "resource": "github.com/acme/gadgets",
      "requiredCheck": false
    },
    "drift": [
      {
        "type": "requiredCheckMissing",
        "resource": "github.com/acme/gadgets",
        "observed": "Verify commit trust: not required",
        "expected": "Verify commit trust: required"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-10-03T08:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zLKQi5maEpU3bgPaMfECN14do9qszZbcEDatyWhNH5es3Pc9UYi5jmbZ7AsPdE5TQEeoj8LcV5BBYY8w3s78nVS"
  }
}
```

### A GitHub organisation installed the community's app

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af05",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.1",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af05",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:03:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "bindCompleted",
      "jobId": "job_01J8Z6Q4N0",
      "ownerId": "91827364",
      "kind": "organization"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:03:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zErKBa1j26x1aicAi862KvPr5RvRvFaKU5EtMUicEE6BTmNsRMLKgPP6sMorj7tM5Kx2rKHE7KV73KziML7f9Vb"
  }
}
```

## Response

The VTC acknowledges the event with an empty payload, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Recorded

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af02",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.1#response",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258af01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-10-02T14:20:01Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-10-02T14:20:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z5mPrASQ5cpsmsQik4scivNBxCv1fJCdQq7sNCvHL7XE4AHMMM4Emo86yAL6DnBDwF1PdKfcgz9bf6nKXS2KHh1"
  }
}
```

## Security & Privacy

### Data carried

Forge ids, repository names, forge account ids and logins, and role and setting names. Forge accounts are personal data: they identify people, and the VTC can join them to members' DIDs. A bridge **MUST NOT** forward raw webhook payloads, which carry far more — commit messages, email addresses, IP-derived metadata — than any event here needs.

### Correlation

The VTC declares `identifierScope: public`; the bridge `pairwise`. Events for non-members (someone added on the forge outside the VTC) carry forge accounts the VTC has no DID for; the VTC keeps them only as drift, for as long as the drift stands.

### Retention

Durable. Events are the history of what happened on the forge outside the VTC's control, which is exactly what an audit of a weakened protection or an unexpected collaborator needs. Drift items are replaced as they are resolved.

### Consent/purpose

The purpose is to keep the forge and the VTC's projection in agreement. Nothing in an event may be used for anything else — in particular not to monitor members' activity on the forge.
