---
slug: git-ns/projection/show
version: "0.1"
title: "Git Namespaces — Show Trust Registry Projection"
summary: "A community administrator reads what the VTC has published to the Trust Registry for its git namespaces, and how many records the next reconciliation pass will still add, remove or replace."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - trust-registry
  - projection
  - administration
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
  rationale: "The answer discloses the community's whole published rights posture — every entity, action and resource the VTC asserts to the Trust Registry — which is confidential VTC operating detail, not itself the public registry content. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's projection mirror and its current rights; changes nothing and triggers no reconciliation pass."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns whether publishing is configured at all, every record the VTC's mirror says is published (entity, action, resource, context and publish time), and a count of how many records the next pass will change."
retention:
  class: exchange
  rationale: "The request carries at most a resource filter and a paging cursor, and is needed only to answer it."
errorCodes:
  - code: git-ns/projection/show:notCommunityAdministrator
    meaning: "The caller does not hold the community-administrator capability. Publishing configuration and the whole projection mirror are community-wide facts, not namespace-scoped ones, so holding git.ns.admin on a namespace is not enough."
    retryable: false
related:
  - git-ns/view
  - git-ns/right/grant
  - git-ns/right/list
  - git-ns/namespace/list
---

## Abstract

The VTC is the source of truth for every git right; the Trust Registry holds only a projection of it, published so verifiers can answer `git.commit.sign` and the other rights without asking the VTC directly. Nothing in the `git-ns/*` family lets an administrator see that projection — whether it is configured at all, what it currently asserts, or how far it has drifted from the VTC's own records since the last reconciliation pass. This task is that read: what is published, and `pendingChanges`, the size of the gap the next pass will close. It is an admin console's *Trust Registry* panel, a command-line `projection show`.

It is a read. It changes nothing and triggers no publish.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The caller is the DID the document's verified `proof` binds to its `issuer`; an unsigned document is refused with `proofRequired`.

The entitlement is **the community-administrator capability** in the VTC's own access control, exactly as in [`git-ns/right/list`](../../../right/list/0.1/spec.md). Whether publishing is configured, and what the mirror holds, are facts about the whole VTC's relationship with the Trust Registry — not about any one namespace — so holding `git.ns.admin` on a namespace does not entitle a caller to this task, even narrowed by `resource` to that namespace. A caller who does not hold the community-administrator capability is refused with `git-ns/projection/show:notCommunityAdministrator`.

The capability is read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Published record** — one TRQP authorization tuple (`entity`, `action`, `resource`, `context`) the VTC's projection mirror says it last wrote to the Trust Registry. `context` is the projector's own bookkeeping — which framework version made the record, when the underlying right became and stops being active, and, where the record publishes a right implied by another, which right implies it.

**Pending change** — a record the VTC's current rights call for and the mirror does not yet show as published, or a published record the VTC's current rights no longer call for. `pendingChanges` counts both directions across the whole VTC; it is not affected by `resource`, because the reconciliation pass it describes runs over everything regardless of what this read was narrowed to.

**Registry configured** — whether a Trust Registry endpoint and this VTC's own publishing DID are both set. While false, the projector does not run: `published` is whatever the mirror last held from before publishing was unconfigured, and `pendingChanges` still reports honestly against the VTC's current records, which is precisely what tells an administrator how far the registry would fall behind if they left it this way.

## Request

The community administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Confirms the caller holds the community-administrator capability, refusing with `git-ns/projection/show:notCommunityAdministrator` otherwise.
3. Returns `registryConfigured`.
4. Returns `published`: the mirror's records matching `resource` where given, ordered by `resource` then `action` then `entity`, at most `limit` per page (clamped to 1..=500, default 100), with a `nextCursor` when more match.
5. Returns `pendingChanges`: how many records the next reconciliation pass would add, remove or replace, across the whole VTC regardless of `resource`.

A VTC **MUST NOT** change anything on this task, and **MUST NOT** run a reconciliation pass because of it.

### Dana checks the registry projection

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6e01",
  "type": "https://trusttasks.org/spec/git-ns/projection/show/0.1",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6e01",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:25:00Z",
  "payload": {}
}
```

## Response

The VTC answers with the projection state, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/projection/show:notCommunityAdministrator`.

### What Dana sees

One right is published directly and implies a second; one record is a v0.1 role-derived grant. Two changes are pending — an unpublished new grant and a stale record.

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6e02",
  "type": "https://trusttasks.org/spec/git-ns/projection/show/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6e01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-26T09:25:01Z",
  "payload": {
    "registryConfigured": true,
    "published": [
      {
        "entity": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "action": "git.repo.own",
        "resource": "github.com/acme/widgets",
        "context": {
          "framework": "https://trusttasks.org/spec/git-ns/right/grant/0.3",
          "activeFrom": "2026-09-23T09:50:00Z"
        },
        "publishedAt": "2026-09-23T09:50:05Z"
      },
      {
        "entity": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "action": "git.commit.sign",
        "resource": "github.com/acme/widgets",
        "context": {
          "framework": "https://trusttasks.org/spec/git-ns/right/grant/0.3",
          "activeFrom": "2026-09-23T09:50:00Z",
          "impliedBy": "git.repo.own"
        },
        "publishedAt": "2026-09-23T09:50:05Z"
      },
      {
        "entity": "did:webvh:QmFrankScid12:acme-vtc.example:frank",
        "action": "git.commit.sign",
        "resource": "github.com/acme/widgets",
        "context": {
          "framework": "https://trusttasks.org/spec/git-trust/grant/0.1",
          "origin": "roleDerived"
        },
        "publishedAt": "2026-09-25T14:05:00Z"
      }
    ],
    "pendingChanges": 2
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a resource filter and a paging cursor. The response carries `registryConfigured`, and, across every namespace unless narrowed by `resource`: every entity, action, resource and publish time the VTC's mirror says it holds in the Trust Registry, plus each record's `context` — which framework version made it, its active window, and what implies it. None of this is a secret in the strict sense; a public reader of the Trust Registry can already see the published tuples one at a time. What this task adds, and what makes it a community-administrator-only read, is the join with `pendingChanges` and `registryConfigured`: whether the VTC's public assertions currently match its own records, and whether they are being kept up to date at all. A producer **MUST NOT** put anything beyond a resource identifier into `ext`.

### Correlation

Every entity and resource in `published` is already public in the Trust Registry itself, so this task adds no identifier a registry reader could not already find. What it adds is the VTC's own view of its publishing health — which records it believes are current, and how many it knows are stale — which the registry's own content does not carry and a verifier querying it cannot infer.

The **VTC** declares `identifierScope: public`, for the same reason as the rest of this family: the recipient is the community's own VTC, and a community administrator must recognise it as the same VTC on every projection read. The **community administrator** carries no such declaration.

### Retention

The request is kept only as long as the VTC keeps request logs. A VTC that audits administrative reads **MAY** record that the caller checked the projection, and **SHOULD NOT** record the response, whose content is already the VTC's own durable mirror and the public registry under their own retention.

### Consent/purpose

The purpose is to let a community administrator confirm the VTC's public assertions match what it actually governs, and to notice when publishing has stopped working (`registryConfigured: false`, or a `pendingChanges` count that never falls). A caller **MUST NOT** republish `context` or `pendingChanges` as if they were the registry's own attestation — they are the VTC's internal account of its own publishing, not a substitute for querying the registry directly.
