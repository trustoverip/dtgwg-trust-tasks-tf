---
slug: vta/backup/initiate-export
version: "1.0"
title: "VTA Backup — Initiate Export"
summary: "Mint an encrypted full-state export bundle and return the descriptor that fetches it."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - password
parties:
  - role: backup operator
    requirement: REQUIRED
    member: issuer
  - role: verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request carries the password that protects the agent's entire state, and the recipient stakes the whole export on the caller being entitled to it — an unattributable request is one there is no way to answer for afterwards. The response is REQUIRED for a different reason: it carries a bearer capability, and an unsigned descriptor is one an intermediary can substitute, pointing the operator's download at bytes the agent never minted.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed initiate-export mints a second bundle, and therefore a second live download address for the same state, from one authorization. Bounding replay is what keeps the number of outstanding copies equal to the number of times a human asked for one.
sideEffects:
  level: mutating
  rationale: >-
    Serializes the agent's state into an encrypted bundle, stages the bytes, and mints a short-lived transport slot. Nothing existing is altered or destroyed — but a new copy of everything the agent holds now exists at a fetchable address, which is a change in the agent's exposure even though it is not a change in its data.
exposure:
  discloses: secret
  ingests: secret
  actsAsSubject: false
  rationale: >-
    Both directions carry secret material and they are different secrets. Inbound, `password` is the key-derivation input protecting the whole export; the recipient uses it and MUST NOT retain it. Outbound, `transportToken` is a bearer credential for the byte endpoint, and the bundle it opens contains the agent's key material — so the descriptor is, in effect, the export.
retention:
  class: exchange
  rationale: >-
    The staged bytes and the token live for the bundle's slot and no longer: fetched and acknowledged, aborted, or expired at `expiresAt`. The password is shorter still — it is consumed deriving the key and never belongs at rest.
errorCodes:
  - code: vta/backup/initiate-export:transportUnavailable
    meaning: >-
      The recipient has no address at which it can publish the bytes, so it cannot produce a descriptor. Not a fault in the request — see Transport preconditions.
    retryable: false
  - code: vta/backup/initiate-export:weakPassword
    meaning: >-
      The password is shorter than the recipient's floor. Refused before any state is serialized.
    retryable: false
  - code: vta/backup/initiate-export:unsupportedAlgorithm
    meaning: >-
      The recipient does not implement the requested transport algorithm. The message names what it does implement.
    retryable: false
  - code: vta/backup/initiate-export:tooManyOpenBundles
    meaning: >-
      This operator already holds the maximum number of live bundles. Abort one or wait for expiry.
    retryable: true
related:
  - vta/backup/complete-export
  - vta/backup/abort
---

## Abstract

The **VTA Backup — Initiate Export** Trust Task asks an agent to serialize everything it holds — keys, access-control entries, trust contexts and, optionally, its audit trail — into a single password-encrypted bundle, and to return the address at which those bytes can be fetched.

It is a Trust Task and not an API call because of what the bundle *is*. A verifiable trust agent's value rests on the exclusivity of its key material; an export is the one operation that deliberately produces a second copy of it. The document that asks for one has to be attributable, placeable in time, and reviewable afterwards, and that is precisely the set of properties a Trust Task envelope provides and a bare HTTP request does not.

The bulk bytes deliberately do **not** travel in the envelope. This task is the control plane: it mints a bundle and hands back a [descriptor](#definitions); the operator fetches the bytes out of band and then acknowledges with [`complete-export`](../../complete-export/1.0/spec.md), or discards the bundle with [`abort`](../../abort/1.0/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **custody of the agent itself**. An export is not scoped to a resource inside the agent that a capability could name — it reproduces the whole agent, including the key material every other authority in the system is ultimately derived from. So the entitlement a producer needs is the one that admits of no narrower statement: the producer is an owner or operator of this agent, at the level that could equally destroy it.

A recipient **MUST** refuse a producer holding anything less. In particular, a producer entitled to *read* a resource through this agent is not thereby entitled to export it: the export bypasses every per-resource control by construction, so deriving export authority from any accumulation of narrower grants defeats them all.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes *who asked* and *that the document is unaltered*. None of it establishes entitlement, and a recipient that treats a valid signature from a known operator as sufficient has checked identity and called it authorization.

This specification does not, and **MUST NOT**, declare that a human approval or a step-up is required before an export ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). That is consumer policy. It is worth being explicit that the policy has a limit, though, so that implementers do not over-read what one can achieve: an approval ceremony can establish that a human is present and intends the export, and cannot establish anything about the `password` member, which has already been chosen and transmitted by the time any approval is displayed. See [Data carried](#data-carried).

## Definitions

**`password`** — the secret from which the recipient derives the bundle's encryption key. Chosen by the producer, never by the recipient, and never recoverable from the recipient afterwards: a bundle whose password is lost is indistinguishable from random bytes. Recipients declare a minimum length and refuse anything shorter with `weakPassword`.

**`includeAudit`** — whether the agent's audit trail is serialized into the bundle alongside its operational state. Separable because the trail answers a different question from the state and has a different sensitivity: it records who did what, which is a history of the operator's own actions and of every counterparty the agent has dealt with. Absent means the trail is excluded.

**`algorithm`** — the transport mechanism the producer asks for, naming how the bytes will move rather than how they are encrypted. `"stream"` — a single HTTPS transfer against the returned `transportUrl` — is the only value this version defines. The member exists so that a recipient offering more (a pre-signed object-store URL, a chunked transfer) can be asked for it rather than having the choice inferred. Absent means `"stream"`.

**`descriptor`** — the response's account of where the bytes are and what they should be. Its members:

- **`bundleId`** — the handle for this bundle for its entire lifecycle. The producer quotes it to `complete-export` and to `abort`.
- **`transportUrl`** — where to fetch the bytes.
- **`transportToken`** — a bearer credential for that fetch, minted fresh per bundle, presented in the `X-Backup-Token` header. Recipients **SHOULD** store only a hash of it, so that a compromise of the recipient's own storage does not yield a usable token, and **SHOULD** accept it once.
- **`expectedSha256`** — the hash of the byte stream, as lowercase hex. A wire-integrity check independent of the encrypted envelope's own authentication tag, so that a truncated or substituted transfer is detected before the password is ever applied to it.
- **`expectedSizeBytes`** — the total byte count, letting a producer detect a truncated transfer even without hashing.
- **`expiresAt`** — after which the bytes are collected and the token refused. Short by design.

**`completionHint`** — operator-facing text describing how to complete the download. Advisory, and safe to ignore; a producer **MUST NOT** parse it or derive behaviour from it.

## Transport preconditions

A recipient can only return a `transportUrl` if it knows an address at which it is reachable. That is a property of the recipient's deployment, not of the request, and it is commonly absent: an agent that speaks only DIDComm or TSP has a perfectly good identity and no HTTPS address to publish — the arrangement much of this ecosystem is built to support.

Such a recipient **MUST** refuse with `transportUnavailable` rather than returning a descriptor whose URL cannot be fetched. The distinction matters to the producer, because the two failures need opposite responses: a malformed request is fixed by the producer, and this one can only be fixed by whoever configures the agent.

A recipient **MAY** offer an `algorithm` that requires no address of its own — an export written to an operator-supplied destination, for instance. This version defines no such algorithm, and a recipient offering only `"stream"` has no way to satisfy the task without one.

## Request

The producer is the backup operator; the recipient is the agent being exported. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A full export including the audit trail

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/backup/initiate-export/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "password": "correct horse battery staple",
    "includeAudit": true
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### The descriptor for the export above

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/backup/initiate-export/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "descriptor": {
      "bundleId": "3f2504e0-4f89-41d3-9a0c-0305e82c3301",
      "algorithm": "stream",
      "transportUrl": "https://agent.example/backup/blob/3f2504e0-4f89-41d3-9a0c-0305e82c3301",
      "transportToken": "dGhpcy1pcy1hLW9uZS1zaG90LWJlYXJlci10b2tlbg",
      "expectedSha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "expectedSizeBytes": 1048576,
      "expiresAt": "2026-01-01T00:05:01Z"
    },
    "completionHint": "GET the transportUrl with header X-Backup-Token, then send complete-export."
  }
}
```

## Security & Privacy

### Data carried

The request carries **one secret going in**: `password`. This is unusual and worth stating plainly, because most of this task family's risk is read as being about what comes back. The password is the only thing standing between the exported bundle and whoever ends up holding it, and it is chosen by the producer at the moment of asking.

A recipient **MUST NOT** log it, echo it in a response, include it in an audit entry, or persist it in any form. It is consumed deriving the key and discarded. A producer, correspondingly, **MUST NOT** place it anywhere the envelope's confidentiality does not cover.

This has a direct consequence for producer *implementations*, and it is the reason the member is annotated `writeOnly`: the password's exposure is decided by where it is typed. A field in a shared or scriptable environment — a browser form reachable by autofill, a password manager, a co-resident extension or screen capture; a shell history; a CI variable — exposes it to everything with reach into that environment, and no property of this protocol recovers from that. Producers **SHOULD** collect it somewhere with a smaller reach, and **SHOULD NOT** offer to remember it.

The response carries **two secrets coming out**, and they compound: `transportToken` opens `transportUrl`, and the bytes behind it are the agent's entire state. Anyone holding both holds the export, subject only to the password. A producer **MUST** treat the descriptor with the sensitivity of the bundle itself, and **MUST NOT** write it to a shared log or pass it through an intermediary that does not need it.

`includeAudit` widens what the bundle contains beyond the agent's own state to a record of its dealings, including counterparties who were never party to this export and cannot object to it. A producer **SHOULD** set it only when the trail is part of what is being preserved.

`completionHint` is free text. A recipient **MUST NOT** place a token, a password, or any other secret in it — it is written to be shown to a human and is routinely copied into places the descriptor should never go.

### Correlation

The recipient learns when this operator exports and how often, which over time describes an operational rhythm — before migrations, before risky changes, after incidents. That is unavoidable: an agent cannot export itself without knowing it did.

`bundleId` joins this document to the later `complete-export` or `abort` in the same lifecycle, and `threadId` joins the request to its response. Both are intrinsic to the exchange.

The producer's identifier must be **stable across the bundle's lifecycle**, because the recipient checks that whoever acknowledges or aborts a bundle is the party that created it. A producer varying its identifier between the two documents will find the bundle reported as not found — deliberately, since reporting it as forbidden would confirm to a stranger that a bundle with that id exists. That check is what makes the identifier reused here, and it does not extend beyond this exchange: nothing requires the same identifier for the next export.

`transportUrl` is fetched over a separate connection that carries no Trust Task envelope, so it is observable to the network as an ordinary HTTPS transfer of known size to a known agent. An observer who sees it learns that an export of that size happened, without learning anything of its content.

### Retention

The staged bytes and the token are `exchange`-scoped: they exist for one bundle's slot and are collected at `expiresAt` if nothing else ends them first. Recipients **SHOULD** keep that slot short — long enough for a download, not for a forgotten bundle to linger as a fetchable copy of the agent — and **SHOULD** cap the number a single operator may hold open, since each is another live address.

The recipient **SHOULD** record that an export was initiated, by whom and when, and **SHOULD** retain that record beyond the bundle. "A copy of this agent was made, on this date, at this operator's request" is the one fact about an export that stays relevant after the bundle is gone, and it is what a later investigation into a leaked copy has to start from.

The `password` is not retained at all, at any class.

### Consent/purpose

The purpose is continuity: preserving an agent against loss, migration or corruption. A bundle produced for that purpose is a complete, offline, indefinitely-lived copy of the agent, which makes reuse both easy and unobservable — nothing in the bundle reports having been opened.

A producer **SHOULD NOT** use an export to move data somewhere a narrower task would have supplied it, and **MUST NOT** use one to obtain material the agent's own access controls would refuse it. That is the specific abuse this task's shape invites: exporting the whole agent is often the path of least resistance to one thing inside it, and it silently discards every control that would have applied.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraph above states a purpose limitation, which is a different thing.
