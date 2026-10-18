---
slug: rooms/owner/register
version: "0.2"
title: "Rooms Owner — Register"
summary: "A room's owner asks their own agent to register the room with a host, so a surface that can reach its agent and nothing else can finish creating a room — and is told whether the room's revocation list is in force there."
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
    The recipient makes outbound requests that create durable state at another party: the room's registration, and the room's revocation list put in force there. What changes is at the host; the recipient keeps only an outstanding delivery of that list, until the host has it.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The recipient speaks to the host as its principal, and what it discloses is the room's identifier, its tier and its owner — which is what invariant I1 requires a host to know at every tier, including `private` — and the room's revocation list, which on `open` and `attributed` rooms names withdrawn credentials and parties, and on `private` rooms only times.
retention:
  class: durable
  rationale: >-
    The recipient forwards and returns, except for one thing: when the room's revocation list did not reach the host, it keeps the room, the host and the list's sequence until it does, so it can go on delivering it. That record is gone once the host has the list or refuses it. Where the room lives is otherwise the caller's record to keep: the recipient holds key custody, not a hosting register, and a stored host would go stale the moment the room moved.
errorCodes:
  - code: rooms/owner/register:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer the registration. Reports the registration only, never the delivery of the room's revocation list, which is reported in a successful response's `revocation`."
    retryable: true
  - code: rooms/owner/register:hostRefused
    meaning: "The host answered and declined the registration — commonly its own creation policy. Its reason is carried in `details`. Reports the registration only, never the delivery of the room's revocation list, which is reported in a successful response's `revocation`."
    retryable: false
related:
  - rooms/create
  - rooms/owner/revoke
  - rooms/revocations/put
  - rooms/owner/invite
  - rooms/owner/issue-membership
  - rooms/keys/backfill
---

## Abstract

A room's owner asks their **key holder** to register the room with a host.

This is [`rooms/create`](../../../create/0.1/spec.md) performed by the agent rather than by the owner directly, and it exists for the same reason as [`rooms/keys/backfill`](../../../keys/backfill/0.1/spec.md): **the surfaces people create rooms from cannot reach a host.** A browser extension or a phone holds a channel to its own agent; a room host is a third party it has no session with.

Without it, creating a room from such a surface stops halfway. The room's identity gets minted — that is the agent's own work — and then the registration cannot be sent, leaving an owner holding a DID and a signing key for a room that exists nowhere. The identity is real and cannot be re-derived, so the half-finished state is not merely inconvenient; it is a thing the owner must now keep safe by hand.

**The order still matters and is unchanged.** The identity is minted first and a host is told about a room that already exists. A host that named the room would be a host the room could not leave.

### What changed from 0.1

A recipient that also serves [`rooms/owner/revoke`](../../revoke/0.1/spec.md) delivers the revocation list it holds for the room to the host before answering a registration (that specification, last paragraph of *Conformance*), so that a room moving hosts does not arrive without its withdrawals. `0.1` predates that requirement and declares no outcome for the one case it creates: **the room is registered, and its list did not reach the host.** A `0.1` recipient could only answer success, saying nothing about a host now serving the room while honouring every credential the room has withdrawn, or refuse with `hostUnreachable` or `hostRefused`, misreporting a registration that took effect — which a caller retries as a registration, and which a host may refuse as a conflict.

`0.2` answers success, because the registration succeeded, and adds `revocation` to the response: the list's head and whether it is `inForce`, `pending` or `refused` at the host. `hostUnreachable` and `hostRefused` now report the registration alone. The recipient **SHOULD** go on delivering a `pending` list on its own, and a caller **MUST** treat anything but `inForce` as a room served without its withdrawals (*The room's revocation list*, below). A new response member and new requirements are not editorial, so this is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules).

A recipient implementing `0.2` **SHOULD** continue to serve `0.1`. Answering a `0.1` request whose list did not reach the host, it **SHOULD** refuse with `hostUnreachable` or `hostRefused` as the delivery failed, with a `message` that says the room **is** registered and names the remedy below: a misreported failure sends the caller back to the host, which answers it, while a success that says nothing leaves the caller no reason to look. That is the least harmful answer `0.1` can give, not a good one; a caller that can, sends `0.2`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal.** This asks an agent to speak outward for the party it acts for, which is the standing relationship every task in this family rests on. A recipient MAY narrow it further — requiring the caller to hold the grant it already requires for signing as a room, say — and refuses a caller who lacks it with the framework's standard `permissionDenied`.

Note what it deliberately does **not** check. It does not verify that the caller controls the room's DID, because the recipient is not a DID resolver and `rooms/create` does not ask it to be one: a host authorizes room operations from credentials the room issued, and a room registered by a party who cannot issue in its name is a room that admits nobody. The failure is real, immediate, and at the point of use.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

**The recipient speaks as the owner, and so signs as the owner.** [`rooms/create`](../../../create/0.1/spec.md) names the Owner as the document's issuer, and a host that runs a creation policy decides it against that party — a community admits rooms for its own members, and checks that the party asking is the `ownerDid` it names. So a conforming recipient:

1. **MUST** send the `rooms/create` document with `ownerDid` as its `issuer`, signed with a key of `ownerDid` that the recipient holds, and **MUST NOT** sign it as itself. A document issued by the recipient while naming another party as owner is one no host can tell from somebody registering rooms in a stranger's name, and a host that checks is right to refuse it.
2. **MUST** refuse, with the framework's standard `permissionDenied` and before contacting the host, when it holds no signing key for `ownerDid` that its caller may use. Absent `ownerDid` means the caller, so a caller whose own key the recipient does not hold names an `ownerDid` it does.

The owner is therefore normally a DID the recipient minted for its principal — the identity they are known by at the host, such as their membership of a community. That is the same identity they then present to the room.

**The host decides whether to store it, and that decision is not this one.** A host runs its own policy over who may create a room on it and at which tier; a registration this recipient forwards happily may still be refused there, and `rooms/owner/register:hostRefused` carries the host's own reason rather than restating it.

## Definitions

**`roomId`** — the room being registered. Its identity exists already.

**`host`** — the host to register with, as a DID.

**`visibility`**, **`retentionPolicy`**, **`retentionDays`** — passed through to [`rooms/create`](../../../create/0.1/spec.md), whose descriptions of each govern. `retentionPolicy` is immutable once set, so omitting it takes the host's default for the life of the room.

**`ownerDid`** — the accountable party, and the `issuer` of the `rooms/create` document the recipient sends. Absent means the caller. The recipient must hold a signing key for it.

**`host`**, **`epoch`** (response) — what the recipient reached, and where the room now stands.

**`revocation`** (response) — the room's revocation list as the recipient holds it, and what the host did with it. Absent when the recipient holds no list for the room.

### The room's revocation list

A host authorizes from credentials the room issued and, for each room it serves, refuses the ones the room's [revocation list](../../../revocations/put/0.1/spec.md) withdraws — once it holds that list. A host told about a room and not given its list therefore honours, from the moment it serves the room, every credential the room has withdrawn: a removed member reads and writes there again. That is the case this section exists for, and why the recipient may not let it pass quietly.

A conforming recipient that holds a revocation list for `roomId`:

1. **MUST** deliver it to `host` with [`rooms/revocations/put`](../../../revocations/put/0.1/spec.md), as [`rooms/owner/revoke`](../../revoke/0.1/spec.md) item 5 delivers it, **after** the host has accepted the registration — a host accepts a list only for a room it holds — and **before** answering.
2. **MUST** answer with a `#response` once the host has accepted the registration, whatever became of the delivery, and **MUST NOT** answer `hostUnreachable` or `hostRefused` for the delivery. The registration took effect; reporting it as a failure would send the caller to register again, and leave it believing no host holds a room that one is serving.
3. **MUST** include `revocation` in that response, carrying the list's `head` and its `status` at the host: `inForce` when the host answered with that head; `refused` when the host refused the list, with the host's `code`; and `pending` otherwise — the host did not answer, or answered with a head that is not this list's. It **MUST NOT** report `inForce` unless the host answered with the list's head.
4. **SHOULD**, when the list is `pending`, go on delivering it to `host` on its own, with a capped back-off, until the host answers with its head or refuses it — delivering the latest list it holds at each attempt, since each keeps every live entry of the one before. It keeps for this only the room, the host and the sequence, and only until then.
5. **MUST NOT**, when the list is `refused`, deliver it again on its own: a refusal of the list (`listInvalid`, `entryNotPermitted`, `notAppendOnly`, `sequenceConflict`) or of the caller's authority (`permissionDenied`) is not changed by repeating it. On `stale` — the host holds a later list this recipient did not sign — it **SHOULD** surface the refusal as a sign that the room's key is in more than one party's hands, as [`rooms/owner/revoke`](../../revoke/0.1/spec.md) item 6 does.

A recipient that holds no list for the room delivers nothing and omits `revocation`: the room has withdrawn nothing, and there is nothing a host could honour by mistake.

A **caller** — the Owner, or a surface acting for it — **MUST** treat a response whose `revocation.status` is not `inForce` as a room its host is serving **without its withdrawals**, and **SHOULD** say so to the person who asked rather than present the room as ready. It **MAY** bring the host up to date at once with [`rooms/owner/revoke`](../../revoke/0.1/spec.md), naming that host and no `withdraw`, which delivers the list the recipient holds and reports what the host did with it. It **MUST NOT** register the room again to repair it: the registration is done, and a second one is answered by the host as whatever its policy makes of a room it already holds.

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
  "type": "https://trusttasks.org/spec/rooms/owner/register/0.2#request",
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

The **KeyHolder** returns what the host said — about the registration, and about the room's revocation list — using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Registered, and the room's revocation list in force there

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/register/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "epoch": 1,
    "revocation": {
      "head": {
        "sequence": 2,
        "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
      },
      "status": "inForce"
    }
  }
}
```

### Registered at a new host, which did not answer the list

The room moved here from another host. It is registered, and the host is serving it **without** the withdrawals its list carries; the recipient goes on delivering the list, and a surface tells the owner the room is not yet safe to use there.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/owner/register/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:31Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.org:rooms-mirror",
    "epoch": 4,
    "revocation": {
      "head": {
        "sequence": 2,
        "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
      },
      "status": "pending"
    }
  }
}
```

## Security & Privacy

### Data carried

The room's identifier, its tier, and the party accountable for it — and, when the recipient holds one, the room's revocation list, which the host must have to refuse what the room withdrew. On `open` and `attributed` rooms the list names withdrawn credentials and parties the host would see act; on a `private` room it carries only times. **The owner is disclosed at every tier, including `private`, and that is invariant I1 rather than a leak**: a host storing content it cannot read still needs a party it can reach about quota, abuse, and the reclamation notice the lifecycle obliges it to send.

A producer **MUST NOT** put a description of the room, its expected membership, or its purpose in `ext`. A host stores what it is given, and the tiers exist so that what it is given is as little as the arrangement can manage.

### Correlation

The host learns that this owner created a room. It was always going to: registration is the moment a host is told, and there is no version of this that keeps it from the party being asked to store it.

To an observer of the recipient's outbound traffic, this is its principal registering something with that host — a signal that a room was created, and with whom, but not what it is for or who will be in it.

### Retention

The recipient stores nothing about the registration — only, while the room's revocation list is `pending` at the host, the room, the host and the list's sequence, dropped once the host has the list or refuses it. That is deliberate and it has a cost worth naming: **the owner is the only party who knows where their room is registered.** A recipient that kept a hosting record would hold a value that goes stale the moment the room moves — a room is portable precisely because a host's records do not define it — and a stale answer here sends the next operation to a party that no longer serves the room.

### Consent/purpose

The room is registered so that it can be stored and served. It confers nothing on the recipient and nothing on the host beyond the storage being asked for: authority over the room stays with the credentials the room itself issues.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
