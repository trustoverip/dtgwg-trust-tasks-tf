---
slug: git-ns/bridge/job
version: "0.1"
title: "Git Namespaces — Bridge Job"
summary: "The VTC asks its bridge — the separate service holding the community's forge credentials — to do one convergent piece of forge work: project roles, create, bootstrap, archive or inspect a repository, or begin a namespace binding or account link."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - bridge
  - automation
parties:
  - role: VTC
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: bridge
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: "A job makes the bridge act on the forge with the community's credentials. The bridge must be able to tell its own VTC's jobs from anyone else's on every transport; nothing else authorises them."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Jobs are convergent, but a stale job replayed after later ones would push the forge back to an old desired state."
sideEffects:
  level: mutating
  rationale: "Changes repositories, roles and protection on the forge. Every change is convergent and is undone by a later job with a different desired state."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Jobs carry member DIDs joined to their forge accounts, which is personal data held only so the bridge can give those accounts roles."
retention:
  class: exchange
  rationale: "A job is kept until its result is acknowledged and its identifier long enough to recognise a repeat."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
  - code: git-ns/bridge/job:notCapable
    meaning: "This forge, or this namespace on it, cannot perform this kind of job — for example `createRepo` on a personal account."
    retryable: false
  - code: git-ns/bridge/job:jobIdReused
    meaning: "The bridge already holds a job with this `jobId` and different content."
    retryable: false
related:
  - git-ns/bridge/result
  - git-ns/bridge/event
  - git-ns/repo/create
  - git-ns/namespace/bind
  - git-ns/account/link
---

## Abstract

A VTC that governs forge namespaces does not talk to the forges itself. Each community runs a **bridge** next to its VTC: a separate service with its own DID, the only holder of the community's forge credentials — its own GitHub App key, or its Forgejo bot's token — and the home of the forge adapters that know how each forge does things. The VTC decides; the bridge carries it out and reports back.

This task is the VTC's half: one job, one piece of forge work in one namespace. Jobs carry desired state, never credentials, and every job is **convergent**: the bridge checks the forge and changes only what differs, so sending a job again is always safe. How it ended comes back as [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md); what the bridge sees happening on the forge comes back as [`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.1/spec.md).

The job vocabulary is forge-neutral. A forge that lacks a capability — a personal GitHub account, where no bot can create repositories — is handled by the VTC not sending that job, and by the bridge refusing it if sent.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the VTC this bridge was deployed for**. A bridge serves exactly one VTC and **MUST** refuse, with `permissionDenied`, a job from any other issuer, whatever its proof. It does no authorization of its own beyond that: every rule about who may hold which right was applied by the VTC before the job was sent, and a bridge that second-guessed it would be a second source of truth. The `proof` is what lets the bridge tell its VTC's jobs from anyone else's, on any transport.

## Definitions

**`jobId`** — the VTC's identifier for the job, unique across its jobs.

**`namespace`** — the namespace the job acts in; it selects the forge credentials the bridge uses.

**`kind`** — what to do. Each kind uses a fixed set of the optional members, and a bridge **MUST** refuse a job that lacks one its kind requires or carries one its kind does not use, with the framework's `malformedRequest`:

| `kind` | Requires | May carry | Does |
|---|---|---|---|
| `projectRoles` | `desiredRoles` | `repo` | converges forge roles on `repo`, or on the namespace itself when `repo` is absent |
| `createRepo` | `repo`, `spec` | `desiredRoles` | creates the repository, runs the bootstrap plan, projects roles |
| `bootstrap` | `repo` | `steps` | runs the bootstrap plan, or only `steps` of it |
| `archive` | `repo` | — | archives the repository |
| `inspect` | — | `repo` | compares the forge with the projection for `repo`, or sweeps the namespace |
| `beginBind` | `target` | — | starts the proof of control over `target` |
| `beginAccountLink` | `subject` | — | starts linking `subject`'s account on this forge |

**`desiredRoles`** — the complete set of people who should hold a forge role on the target, each with their linked account and their highest effective right there. The forge adapter maps each right to one of its roles (on GitHub, `own` → `admin`, `maintain` → `maintain`; on Forgejo, `maintain` → `write` plus the merge allow-list). People not listed lose any role the bridge manages; roles the bridge does not manage are reported as drift, not removed.

**`steps`** — names from the adapter's bootstrap plan. The forge-neutral names are `create`, `workflow`, `keyring`, `variables`, `requiredCheck`, `roles` and `archive`; an adapter **MAY** add its own.

**`next`** — for the two `begin*` kinds: where the VTC sends the person, a device-flow code where the forge has one, and when the chance lapses.

## Request

The VTC sends the job to its bridge. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming bridge:

1. Refuses a namespace it does not serve with `git-ns:unknownNamespace`.
2. Refuses a kind this forge or namespace cannot perform — `createRepo` on a personal account, `projectRoles` where it has no role management — with `git-ns/bridge/job:notCapable`.
3. Refuses a `jobId` it already holds with different content with `git-ns/bridge/job:jobIdReused`.
4. Otherwise records the job durably before answering `accepted: true`, and later sends exactly one [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md) for it. For a `jobId` it has already finished, answers `accepted: false`, does not run it again, and sends its result again: that is how a VTC that lost a result recovers it.
5. For `beginBind` and `beginAccountLink`, returns `next` with a single-use nonce bound to the job, and reports completion as a `bindCompleted` or `accountLinked` event, then its result. An attempt nobody completes ends in a `failed` result with error code `expired`.
6. Uses credentials scoped as narrowly as the forge allows — on GitHub, one installation token per job, limited to the job's repository where the API permits — and discards them after.

### Creating a repository

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c01",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.1",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZA3K7F",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "createRepo",
    "repo": "github.com/acme/gadgets",
    "spec": {
      "visibility": "public",
      "description": "Small tools for widgets"
    },
    "desiredRoles": [
      {
        "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "account": {
          "forge": "github.com",
          "id": "9120045",
          "login": "bob-builds"
        },
        "right": "git.repo.own"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zk5f5NdC3fyVGTjHt3wAfoua65baH7EMsYGngmCMHUp8sV5gfHLq5eCqkQi6wZn3HAfnsE8Rzu9XAaaK4nxDHRy"
  }
}
```

### Beginning an account link

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c03",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.1",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c03",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZB0P2C",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "beginAccountLink",
    "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zsMQG7URHk2DRiFd9kuB5WojM61NFSJj7eLqEjFLvuU4LTcw2FSegdznkN8yDuRtdrVo1twjjPb4Ndfzh3sTqBV"
  }
}
```

### Re-running one bootstrap step after drift

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c05",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.1",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c05",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZC9D4H",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "bootstrap",
    "repo": "github.com/acme/widgets",
    "steps": [
      "requiredCheck"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zfNBCVyyxc4YJ4LBc3MC4LswfMaAVgwC7Pv5mW2ZAnXCCP84g3KpfQX3UjZL63ovm2257Fbk78GdZNt2rZTCDPF"
  }
}
```

## Response

The bridge, now responding, says whether it took the job, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Accepted

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c02",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.1#response",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c01",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "jobId": "job_01J8ZA3K7F",
    "accepted": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zB6QrBqXENi973GjjCeVjVVUUDempgtCdj4GY4ztNUpL2ZvJv97ZVmZo66EkCYFZbu1YdaWPJnTj42jJ57qx6cv"
  }
}
```

### Accepted, with where to send the member

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c04",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.1#response",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6c03",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "jobId": "job_01J8ZB0P2C",
    "accepted": true,
    "next": {
      "url": "https://github.com/login/device",
      "userCode": "WDJB-MJHT",
      "expiresAt": "2026-09-23T10:15:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6xNLK38BZKGr3S86pfLF3E6y77dxH2i8Kv7mpPG7uVLBBDGgEac6CRC8T9UrHP9L8CSpUmp3yc3GETRsV2dB2N"
  }
}
```

## Security & Privacy

### Data carried

Jobs carry DIDs joined to forge accounts (`desiredRoles`), repository names and descriptions. That join is personal data, and it is why the bridge exists as a separate service: it holds the forge credentials and the account mappings it is sent, and nothing else of the community's. Jobs never carry credentials; the bridge **MUST NOT** accept one that tries (there is no member for it, and `additionalProperties` refuses it).

### Correlation

The VTC declares `identifierScope: public`, because it is the DID repositories name. The bridge declares `pairwise`: only its VTC needs to recognise it, and each community runs its own. Forge-side, every action the bridge takes is attributed to the community's app or bot, not to the member it acts for.

### Retention

A bridge keeps a job until its result is acknowledged, and its `jobId` and result long enough to answer a repeated job with `accepted: false` — at least as long as the VTC may retry. It **SHOULD NOT** keep `desiredRoles` beyond that: the VTC is the source of truth and sends the full set every time.

### Consent/purpose

The purpose is to make the forge match what the VTC decided. A bridge **MUST NOT** use what jobs carry — forge accounts, DIDs, repository names — for anything else.
