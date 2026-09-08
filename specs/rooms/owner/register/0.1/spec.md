---
slug: rooms/owner/register
version: "0.1"
title: "Rooms Owner — Register"
summary: "A room's owner asks their own agent to register the room with a host, so a surface that can reach its agent and nothing else can finish creating a room."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request makes the recipient act outward in its principal's name, announcing a room and naming an accountable party for it. One whose integrity depended on the transport would let a compromised channel register rooms, at hosts of its choosing, against an owner who never asked.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Registration is not idempotent at every host — a second one may be a conflict rather than a no-op — so a replay must be placeable in time."
sideEffects:
  level: mutating
  rationale: >-
    The recipient makes an outbound request that creates durable state at another party. It stores nothing itself; what changes is at the host.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The recipient speaks to the host as its principal, and what it discloses is the room's identifier, its tier and its owner — which is what invariant I1 requires a host to know at every tier, including `private`.
retention:
  class: transient
  rationale: >-
    The recipient forwards and returns. Where the room now lives is the caller's record to keep: the recipient holds key custody, not a hosting register, and a stored host would go stale the moment the room moved.
errorCodes:
  - code: rooms/owner/register:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer."
    retryable: true
  - code: rooms/owner/register:hostRefused
    meaning: "The host answered and declined — commonly its own creation policy. Its reason is carried in `details`."
    retryable: false
related:
  - rooms/create
  - rooms/owner/invite
  - rooms/owner/issue-membership
  - rooms/keys/backfill
---

## Abstract

A room's owner asks their **key holder** to register the room with a host.

This is [`rooms/create`](../../../create/0.1/spec.md) performed by the agent rather than by the owner directly, and it exists for the same reason as [`rooms/keys/backfill`](../../../keys/backfill/0.1/spec.md): **the surfaces people create rooms from cannot reach a host.** A browser extension or a phone holds a channel to its own agent; a room host is a third party it has no session with.

Without it, creating a room from such a surface stops halfway. The room's identity gets minted — that is the agent's own work — and then the registration cannot be sent, leaving an owner holding a DID and a signing key for a room that exists nowhere. The identity is real and cannot be re-derived, so the half-finished state is not merely inconvenient; it is a thing the owner must now keep safe by hand.

**The order still matters and is unchanged.** The identity is minted first and a host is told about a room that already exists. A host that named the room would be a host the room could not leave.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal.** This asks an agent to speak outward for the party it acts for, which is the standing relationship every task in this family rests on.

Note what it deliberately does **not** check. It does not verify that the caller controls the room's DID, because the recipient is not a DID resolver and `rooms/create` does not ask it to be one: a host authorizes room operations from credentials the room issued, and a room registered by a party who cannot issue in its name is a room that admits nobody. The failure is real, immediate, and at the point of use.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

**The host decides whether to store it, and that decision is not this one.** A host runs its own policy over who may create a room on it and at which tier; a registration this recipient forwards happily may still be refused there, and `rooms/owner/register:hostRefused` carries the host's own reason rather than restating it.

## Definitions

**`roomId`** — the room being registered. Its identity exists already.

**`host`** — the host to register with, as a DID.

**`visibility`**, **`retentionPolicy`**, **`retentionDays`** — passed through to [`rooms/create`](../../../create/0.1/spec.md), whose descriptions of each govern. `retentionPolicy` is immutable once set, so omitting it takes the host's default for the life of the room.

**`ownerDid`** — the accountable party. Absent means the caller.

**`host`**, **`epoch`** (response) — what the recipient reached, and where the room now stands.

### Why the host is echoed back

A caller records where its room lives, and the value worth recording is the one the recipient actually reached rather than the one the caller asked for. They are normally the same. Where they are not — a host that redirected, a deployment that resolves one DID to another — a caller that stored its own request would hold a host it never spoke to.

### Registering twice is a real thing to do

A room may be registered with more than one host: a mirror serves reads while its primary takes writes. So a second registration is not necessarily a mistake, and this task does not treat it as one. Whether a *particular* host accepts a room it already holds is that host's decision, reported as `hostRefused` with its own reason.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A private room, registered with the community that will store it

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/register/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "visibility": "private",
    "retentionDays": 90
  }
}
```

## Response

The **KeyHolder** returns what the host said, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/register/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "epoch": 1
  }
}
```

## Security & Privacy

### Data carried

The room's identifier, its tier, and the party accountable for it. **The owner is disclosed at every tier, including `private`, and that is invariant I1 rather than a leak**: a host storing content it cannot read still needs a party it can reach about quota, abuse, and the reclamation notice the lifecycle obliges it to send.

A producer **MUST NOT** put a description of the room, its expected membership, or its purpose in `ext`. A host stores what it is given, and the tiers exist so that what it is given is as little as the arrangement can manage.

### Correlation

The host learns that this owner created a room. It was always going to: registration is the moment a host is told, and there is no version of this that keeps it from the party being asked to store it.

To an observer of the recipient's outbound traffic, this is its principal registering something with that host — a signal that a room was created, and with whom, but not what it is for or who will be in it.

### Retention

The recipient stores nothing. That is deliberate and it has a cost worth naming: **the owner is the only party who knows where their room is registered.** A recipient that kept a hosting record would hold a value that goes stale the moment the room moves — a room is portable precisely because a host's records do not define it — and a stale answer here sends the next operation to a party that no longer serves the room.

### Consent/purpose

The room is registered so that it can be stored and served. It confers nothing on the recipient and nothing on the host beyond the storage being asked for: authority over the room stays with the credentials the room itself issues.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
