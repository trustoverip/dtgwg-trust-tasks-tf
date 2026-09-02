---
slug: vta/backup/initiate-import
version: "1.0"
title: "VTA Backup — Initiate Import"
summary: "Mint an upload slot for an encrypted bundle and return the descriptor that writes to it."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - restore
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
    The request opens a writable endpoint on the recipient and pre-commits what will be written to it, so an unattributable one is an anonymous party arranging to place bytes inside an agent. The response is REQUIRED for a different reason: it carries a bearer token for that endpoint, and an unsigned descriptor is one an intermediary can substitute, redirecting the operator's upload to a destination the agent never minted.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed initiate-import opens a second write slot from one authorization. Each slot is an address at which bytes may be placed inside the agent, so bounding replay is what keeps the number of open doors equal to the number of times a human asked for one.
sideEffects:
  level: mutating
  rationale: >-
    Reserves a bundle slot and mints a short-lived, write-capable transport endpoint. Nothing the agent holds is read, altered or destroyed — the uploaded bytes are staged and inert until finalize-import applies them, and a slot that is never finalized expires having changed nothing.
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response carries `transportToken`, a bearer credential that authorizes writing bytes into the recipient. It is classed secret for the same reason the export descriptor's token is, with the direction reversed: leaking an export token exposes the agent's state, and leaking an import token lets someone else fill the slot the operator opened. Inbound the request carries only a hash, a byte count and an algorithm name — descriptive metadata about bytes the recipient does not yet hold.
retention:
  class: exchange
  rationale: >-
    The slot, the token and any bytes uploaded into them live for the bundle's window and no longer: applied by finalize-import, aborted, or expired at `expiresAt`. Staged bytes that are never finalized are discarded unread.
errorCodes:
  - code: vta/backup/initiate-import:unsupportedAlgorithm
    meaning: >-
      The recipient does not implement the requested transport algorithm. The message names what it does implement.
    retryable: false
  - code: vta/backup/initiate-import:invalidDigest
    meaning: >-
      `expectedSha256` is not 64 lowercase hex characters, or `expectedSizeBytes` is not a positive count. Refused before a slot is opened.
    retryable: false
  - code: vta/backup/initiate-import:transportUnavailable
    meaning: >-
      The recipient has no address at which it can accept the bytes, so it cannot produce a descriptor. Not a fault in the request — see Transport preconditions.
    retryable: false
  - code: vta/backup/initiate-import:tooManyOpenBundles
    meaning: >-
      This operator already holds the maximum number of live bundles. Abort one or wait for expiry.
    retryable: true
related:
  - vta/backup/finalize-import
  - vta/backup/abort
---

## Abstract

The **VTA Backup — Initiate Import** Trust Task asks an agent to open a slot for an encrypted bundle the operator is about to upload, and to return the address at which those bytes should be written.

It is the mirror of [`initiate-export`](../../initiate-export/1.0/spec.md), and the asymmetry is worth naming: export mints a readable address, import mints a **writable** one. That makes this the task in the family with the smallest payload and the largest consequence, because what it opens is a door into the agent rather than a window onto it.

Nothing is applied here. The uploaded bytes are staged, inert, and remain so until [`finalize-import`](../../finalize-import/1.0/spec.md) supplies the password that decrypts them and commits the result. A slot that is never finalized expires having changed nothing about the agent — which is what lets an operator upload first and decide afterwards.

The digest and size are **pre-committed** in this request rather than reported after the upload. That is what makes the transfer verifiable: the recipient knows what it should have received before it receives anything, so bytes that differ are refused rather than reconciled.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **custody of the agent itself**, on the same terms as export and for a symmetric reason. An import does not write to a resource inside the agent that a capability could name; it stages the material from which the agent's entire state — including the key material every other authority derives from — will be replaced.

A recipient **MUST** refuse a producer holding anything less, and **MUST NOT** derive import authority from any accumulation of narrower write grants. A producer entitled to add an entry through this agent is not thereby entitled to supply the agent a new set of everything.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who asked and that the document is unaltered, and establishes nothing about entitlement.

This specification does not, and **MUST NOT**, declare that a human approval or a step-up is required ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). A consumer choosing to place one **SHOULD** be aware that this task is the weaker of the two places to do it: nothing is applied here, and the document that actually replaces the agent's state is `finalize-import`. An approval attached only to the slot leaves the commitment ungated.

## Definitions

**`expectedSha256`** — the SHA-256 of the bytes the producer is about to upload, as lowercase hex, committed before the upload begins. The recipient rejects a stream that hashes differently. It is a wire-integrity check and nothing more: it establishes that the bytes that arrived are the bytes that were sent, and says nothing about whether they are a bundle worth applying.

**`expectedSizeBytes`** — the byte count of that upload. Lets the recipient size the slot and refuse a truncated or oversized transfer without hashing it first.

**`algorithm`** — the transport mechanism the producer asks for, naming how the bytes move rather than how they are encrypted. `"stream"` — a single HTTPS transfer to the returned `transportUrl` — is the only value this version defines and is what an absent member means.

**`descriptor`** — the response's account of where to write, and until when. Its members are the same shape `initiate-export` returns, read in the opposite direction:

- **`bundleId`** — the handle for this bundle for its entire lifecycle. Quoted to `finalize-import` and to `abort`.
- **`transportUrl`** — where to write the bytes.
- **`transportToken`** — a bearer credential for that write, minted fresh per bundle, presented in the `X-Backup-Token` header. Recipients **SHOULD** store only a hash of it.
- **`expectedSha256`**, **`expectedSizeBytes`** — echoed from the request, so the descriptor stands alone as the terms of the transfer.
- **`expiresAt`** — after which the slot closes, the token is refused, and any staged bytes are discarded.

**`completionHint`** — operator-facing text describing how to complete the upload. Advisory; a producer **MUST NOT** parse it or derive behaviour from it.

## Transport preconditions

A recipient can only return a `transportUrl` if it knows an address at which it is reachable. That is a property of the recipient's deployment, not of the request, and it is commonly absent: an agent that speaks only DIDComm or TSP has a perfectly good identity and no HTTPS address to publish.

Such a recipient **MUST** refuse with `transportUnavailable` rather than returning a descriptor that cannot be written to. The two failures need opposite responses — a malformed request is fixed by the producer, and this one can only be fixed by whoever configures the agent — so conflating them sends the operator looking in the wrong place.

## Request

The producer is the backup operator; the recipient is the agent that will receive the bundle. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Pre-committing a bundle before uploading it

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000007",
  "type": "https://trusttasks.org/spec/vta/backup/initiate-import/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T01:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "expectedSha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "expectedSizeBytes": 1048576
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### The upload slot for that bundle

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000008",
  "type": "https://trusttasks.org/spec/vta/backup/initiate-import/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T01:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "descriptor": {
      "bundleId": "9c858901-8a57-4791-81fe-4c455b099bc9",
      "algorithm": "stream",
      "transportUrl": "https://agent.example/backup/blob/9c858901-8a57-4791-81fe-4c455b099bc9",
      "transportToken": "dGhpcy1pcy1hLXdyaXRlLXNsb3QtYmVhcmVyLXRva2Vu",
      "expectedSha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "expectedSizeBytes": 1048576,
      "expiresAt": "2026-01-01T01:05:01Z"
    },
    "completionHint": "POST the bundle to the transportUrl with header X-Backup-Token, then send finalize-import."
  }
}
```

## Security & Privacy

### Data carried

The request carries **no secret**, and that is the deliberate difference from `initiate-export`. A digest, a byte count and an algorithm name describe bytes the recipient does not yet hold and reveal nothing about their contents. The password that decrypts the bundle is supplied to [`finalize-import`](../../finalize-import/1.0/spec.md), not here, so that it never travels alongside the slot that will hold the ciphertext.

The response carries **one secret**: `transportToken`. Leaking it does not expose the agent's state — nothing is readable through an import slot — but it lets a third party fill the slot the operator opened. The consequence of that is bounded by `expectedSha256`: bytes that hash differently are refused, so a substituted upload fails the transfer rather than being staged. The check is the containment, which is why pre-committing the digest is REQUIRED rather than advisory.

A recipient **MUST NOT** make an import slot readable. The endpoint accepts bytes; a `GET` against it that returned what was staged would let anyone holding the token read back a bundle the operator uploaded, which is the export risk reintroduced through the door meant to face the other way.

`completionHint` is free text. A recipient **MUST NOT** place a token, a password, or any other secret in it — it is written to be shown to a human and is routinely copied.

### Correlation

The recipient learns when this operator imports and how often, which describes an operational rhythm: imports cluster around migrations, recoveries and incidents in a way exports do not. That is intrinsic — an agent cannot accept a bundle without knowing it did.

`expectedSha256` is a stable identifier for the bundle's exact bytes. Two agents given the same bundle can, if they compare notes, establish that they received the same one; so can an intermediary that sees both requests. That is unavoidable for a value whose purpose is to be compared, and it is worth noting that the digest is of the *encrypted* bundle, so it identifies the copy rather than the contents.

`bundleId` joins this document to the later `finalize-import` or `abort`; `threadId` joins request to response. Both are intrinsic.

The producer's identifier **MUST** be stable across the bundle's lifecycle, because the recipient checks that whoever finalizes or aborts a bundle is the party that created it. A producer varying its identifier will find the bundle reported as not found, deliberately — see the same treatment in [`abort`](../../abort/1.0/spec.md).

### Retention

The slot, the token and any staged bytes are `exchange`-scoped: they live for the bundle's window and are discarded at `expiresAt` if nothing else ends them first. Recipients **SHOULD** keep that window short and **SHOULD** cap how many slots one operator may hold open, since each is a live writable endpoint.

Bytes staged against a slot that expires **MUST** be discarded unread. A recipient that retained them would be holding an operator's encrypted bundle indefinitely, having never been asked to apply it.

The recipient **SHOULD** record that an import was initiated, by whom and when, and **SHOULD** retain that beyond the bundle. An import that was opened and never finalized is a fact worth being able to state later, because it is what an interrupted or abandoned recovery looks like from the trail.

### Consent/purpose

The purpose is recovery: staging a previously exported bundle so the agent can be restored from it. The material a producer uploads is a complete agent, so a recipient **MUST NOT** treat a staged bundle as a source of data for anything other than the `finalize-import` that names it — in particular, it **MUST NOT** be inspected, indexed, or partially applied.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraph above states a purpose limitation, which is a different thing.
