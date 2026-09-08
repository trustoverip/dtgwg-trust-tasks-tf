---
slug: rooms/keys/backfill
version: "0.1"
title: "Rooms Keys — Backfill"
summary: "A member asks their own agent to fetch the room's epoch key chain from its host and keep it, so history written before they joined becomes readable."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request makes the recipient act outward in its principal's name — presenting their room credentials to a party this request names. One whose origin depended on the transport would let a compromised channel aim a member's credentials at a host of somebody else's choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed backfill is harmless — rungs are idempotent by epoch — but an undated one cannot be reasoned about when a member is re-establishing a key holder from more than one source."
sideEffects:
  level: mutating
  rationale: >-
    The recipient makes an outbound request and retains what comes back. Both halves matter: the fetch is a network act performed in the principal's name, and the rungs are kept for the life of the room.
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` is the whole point: the recipient presents its principal's membership and authority to the named host, as them. What comes back is wrapped key material, so `ingests` is `secret`. The response discloses only how far back the recipient can now read.
retention:
  class: durable
  rationale: >-
    The rungs are kept for the life of the room. Discarding them means the recipient can no longer open anything written before its principal joined, and running this again is the only repair.
errorCodes:
  - code: rooms/keys/backfill:notAMember
    meaning: "The recipient holds no group state for this room, so there is nothing for the rungs to extend and no credentials to present."
    retryable: false
  - code: rooms/keys/backfill:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer."
    retryable: true
  - code: rooms/keys/backfill:hostRefused
    meaning: "The host answered and declined. Its own reason is carried in `details`; the commonest is that this host does not serve the named room."
    retryable: false
related:
  - rooms/epoch/chain
  - rooms/keys/chain
  - rooms/keys/present
  - rooms/keys/open
---

## Abstract

A member asks their **key holder** — the agent that holds their room keys — to fetch the room's epoch key chain from its host and keep it, so that records written before they joined become readable.

This is [`rooms/epoch/chain`](../../../epoch/chain/0.1/spec.md) and [`rooms/keys/chain`](../../chain/0.1/spec.md) performed as one act by the party that can perform both. It exists because **the member frequently cannot make the middle call at all**: fetching from a host needs a channel to that host, and the surfaces a person actually uses — a browser extension, a phone — hold a channel to their own agent and to nothing else.

The alternative, which this replaces, is a member who mints a presentation at their agent, carries it to the host themselves, carries the answer back, and hands it to the same agent. Three hops through the party least able to make the middle one, to move key material that never needed to leave the agent that already holds the rest of it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal**, the same as every other custody task in this family. A key holder holds one member's room keys; this asks it to use them and to extend what they reach.

It is deliberately *not* gated on a room credential. The recipient already holds everything the room issued to its principal — that is what makes it their key holder — so requiring one here would be asking the caller to present, to the party that minted the presentation, a credential it is about to present on their behalf.

A conforming consumer **MUST** accept this only from the party whose keys it holds, and **MUST** refuse it for a room it holds no group state for.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

### What the recipient presents, and to whom

The recipient **MUST** present a `read` grant scoped to this room, bound to `host` as its audience — the same presentation [`rooms/keys/present`](../../present/0.1/spec.md) would produce for that pair. It **MUST NOT** present a grant conferring more than `read`: reading the room and reading the parts of it written earlier are the same act, and a backfill that presented `write` would hand a host authority the operation never needed.

**The audience binding is what makes a caller-named host safe.** A presentation minted for one host is not accepted by another, so a caller who names a host of their choosing obtains a presentation usable only there — and they are already a member of the room, so they could have obtained the same thing directly. What the caller gains is a network request they could not otherwise make; what they do not gain is any standing they did not have.

A consumer **SHOULD** nonetheless bound this: it is an outbound request to a caller-supplied party, and an unbounded one is a request-forgery primitive pointed at whatever the agent's network can reach. Resolving `host` as a DID and speaking only the transports its document advertises is the bound the rest of this framework already provides — an agent that would connect to an arbitrary URL here has widened its own attack surface, not this task's.

## Definitions

**`roomId`** — the room to backfill. The recipient **MUST** already hold group state for it.

**`host`** — the host to fetch from, as a DID.

**`fromEpoch`**, **`limit`** — the window to ask the host for. Both are passed through to [`rooms/epoch/chain`](../../../epoch/chain/0.1/spec.md).

**`earliestReadableEpoch`**, **`fetched`**, **`stored`** (response) — see below; the three are not restatements of each other.

### Why the host is named rather than looked up

Nothing maps a room to its host, and inventing that mapping would add a lifecycle to get wrong. A room is **portable** — a host authorizes from credentials the room issued, never from its own records, so re-pointing a room moves it with nothing reissued — which means any remembered host is a value that goes stale silently.

The recipient holds key *custody*, which is a different fact from hosting and one it has no view of. Naming the host is also the honest shape: a member learned it from whoever invited them.

### Three numbers, because there are three ways this ends

`fetched` is what the host served. `stored` is how many of those were new. `earliestReadableEpoch` is how far back the recipient can now derive a key, **having walked what it holds**.

They come apart, and each combination means something different:

- `fetched: 0` — the host has nothing below what the recipient already reads. Either the room's history begins there, or it was severed before this member joined; from here the two are indistinguishable, and neither is a failure.
- `fetched > 0`, reach unmoved — the served rungs sit below a gap. A rung extends reach only if every rung above it is present, so these are **early rather than wrong**: they become useful the moment the gap is filled. A consumer **MUST NOT** discard them.
- `stored: 0`, `fetched > 0` — everything served was already held. That is what a retry looks like, and it is a success.

A consumer that reported only `fetched` would tell a member "12 rungs stored" over a room that still cannot open a word of its history.

## Request

A **Member** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member who joined at epoch 7 asks for everything below it

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/backfill/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "fromEpoch": 6
  }
}
```

## Response

The **KeyHolder** responds, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The whole history is now reachable

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/backfill/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "earliestReadableEpoch": 1,
    "fetched": 5,
    "stored": 5
  }
}
```

## Security & Privacy

### Data carried

The request carries two identifiers. What crosses the network *because of it* is the more interesting half: the recipient presents its principal's membership and authority to the named host, and receives wrapped key material back.

A rung is inert to anyone holding no epoch key, so the fetch itself is safe to observe. The presentation is not: it names a member of the room to the host it is shown to, which on a `private` room is the fact that tier exists to withhold. **That disclosure is inherent in reading a private room at all** — a member who reads must present — and this task makes it no worse. It does make it easier to do repeatedly, so a consumer **SHOULD NOT** turn this into a poll.

A producer **MUST NOT** put record content, member identifiers, or anything descriptive in `ext`.

### Correlation

To an observer of the recipient's outbound traffic, this is a member of that room contacting that host. To the host, it is a member reading — which is what the presentation already said.

The shape is informative: this is sent about **once**, on joining or after restoring a key holder, so its presence says a member has just arrived or just recovered.

### Retention

The rungs are kept for the life of the room. The presentation the recipient minted for the fetch is single-purpose and **SHOULD NOT** be retained past the request it was made for — it is bound to one audience and one action, so keeping it buys nothing and widens what a compromise of the recipient yields.

### Consent/purpose

The chain is fetched so that a member's agent can read the room its principal is a member of. It confers nothing the member did not already hold: they hold a `read` grant the room issued, and this exercises it.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
