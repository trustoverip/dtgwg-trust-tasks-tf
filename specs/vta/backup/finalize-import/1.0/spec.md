---
slug: vta/backup/finalize-import
version: "1.0"
title: "VTA Backup — Finalize Import"
summary: "Decrypt an uploaded bundle and replace this agent's state with it."
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
    This is the document that replaces an agent's identity, and it carries the password that unlocks the bundle it replaces it with. An unattributable request is an anonymous party substituting an agent's key material, and no later inspection of the agent can recover who did it — the state that would have recorded it is exactly what was overwritten. The response is REQUIRED because it is the receipt: the only surviving account of what the agent was replaced with, issued by the agent that no longer holds the state it describes.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed commit re-applies a bundle over whatever the agent has done since — silently discarding work the operator believed was saved, because the second application looks identical to the first. Bounding replay is the only thing standing between a retried request and an unnoticed rollback.
sideEffects:
  level: destructive
  rationale: >-
    On commit, replaces the agent's keys, access-control entries, trust contexts and audit trail with the bundle's. What the agent held beforehand is gone and is not recoverable from the agent — recovering it requires a separate export taken before this ran. The identity the agent presents to every counterparty may change as a result. The preview variant mutates nothing, but the level covers the worse of the two.
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: >-
    Inbound, `password` is the key-derivation input that unlocks a complete copy of an agent; the recipient uses it and MUST NOT retain it. Outbound the response is counts and a status — deliberately not an inventory, because a description of what was restored would let a party who guessed a password learn the bundle's contents without committing it.
retention:
  class: durable
  rationale: >-
    On commit, what the recipient receives becomes the agent's state — the most durable retention there is. The response is equally durable for the opposite reason: it is the only record that the replacement happened, and it survives in the operator's hands rather than the agent's, because the agent's own trail was replaced along with everything else.
errorCodes:
  - code: vta/backup/finalize-import:notFound
    meaning: >-
      The recipient holds no import bundle under this identifier that this producer may act on. Deliberately conflates "no such bundle", "not an import bundle", and "not yours" — see Correlation.
    retryable: false
  - code: vta/backup/finalize-import:noBytesUploaded
    meaning: >-
      The slot was opened but nothing has been written to it yet. Upload to the descriptor's transportUrl first.
    retryable: true
  - code: vta/backup/finalize-import:terminalState
    meaning: >-
      The bundle was already committed, aborted or expired. A second commit is refused rather than repeated — see Why commit is not idempotent.
    retryable: false
  - code: vta/backup/finalize-import:malformedBundle
    meaning: >-
      The uploaded bytes are not a bundle this recipient can read. Raised before decryption is attempted, so it says nothing about whether the password was right.
    retryable: false
  - code: vta/backup/finalize-import:decryptionFailed
    meaning: >-
      The bundle could not be decrypted or its authentication tag did not verify. Deliberately conflates a wrong password with tampered bytes — see Data carried.
    retryable: false
related:
  - vta/backup/initiate-import
  - vta/backup/abort
---

## Abstract

The **VTA Backup — Finalize Import** Trust Task supplies the password for a bundle already uploaded to an agent, and asks the agent to decrypt it and adopt what is inside.

This is the consequential end of the import family. Everything before it — opening a slot, writing bytes — is reversible by walking away; a slot that expires leaves the agent exactly as it was. This document is where that stops being true. On commit the agent's keys, access-control entries, trust contexts and audit trail are replaced with the bundle's, and what it held before is gone.

The task is therefore **two-phase**. With `confirm` false the recipient does everything except commit: it decrypts, validates, and reports what it found. With `confirm` true it commits. The preview exists because the alternative is asking an operator to authorize an irreversible replacement on the strength of a filename — and because the most common import mistake is not a wrong password, which fails loudly, but the right password on the wrong bundle, which succeeds.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **custody of the agent itself** — the same entitlement as export and import, and here the reason is most direct. This task does not act on a resource inside the agent; it replaces the agent. A recipient **MUST** refuse a producer holding anything less, and **MUST** additionally refuse a producer that is not the one that initiated the bundle, answering as `notFound` rather than as a refusal.

Two things are worth stating because implementers get them wrong in opposite directions.

First, **knowing the password is not authorization**. It is the ability to read the bundle, which is a different question from whether this producer may impose it on this agent. A recipient that accepts any correctly-passworded bundle has made possession of a backup file sufficient to take over an agent.

Second, **authority to import is not conferred by the bundle**. The bundle names a source agent and may contain access-control entries granting broad authority; none of that bears on whether the request is admissible. A recipient **MUST** evaluate the producer's entitlement against its own current state, before decrypting — otherwise the material being authorized is the material doing the authorizing.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who asked and that the document is unaltered, never that they are entitled.

This specification does not, and **MUST NOT**, declare that a human approval or a step-up is required ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). Where a consumer chooses to place one, this task — and specifically the `confirm: true` variant — is where it belongs, because it is the only document in the family that changes the agent. An approval attached to `initiate-import` gates opening a slot and leaves the commitment ungated.

## Why commit is not idempotent

[`abort`](../../abort/1.0/spec.md) is idempotent and this task deliberately is not. A second `finalize-import` against a committed bundle is refused with `terminalState`.

The asymmetry is about what a repeat would mean. Repeating an abort re-destroys something already destroyed: no new outcome, so making it harmless removes a race. Repeating a commit re-applies a snapshot over whatever the agent has done since — and by then the agent has usually been running, holding new keys and new grants. A retry that silently succeeded would roll the agent back to a state the operator believed they had moved on from, and the two commits would be indistinguishable in the result.

So a bundle commits **once**. An operator who genuinely wants to re-apply the same bytes uploads them again, through a new slot, and gets a new document saying so.

## Definitions

**`bundleId`** — the handle returned in the descriptor from [`initiate-import`](../../initiate-import/1.0/spec.md). Opaque: a producer quotes what it was given and **MUST NOT** derive, guess or enumerate one.

**`password`** — the secret from which the recipient derives the bundle's decryption key. It is supplied here rather than at `initiate-import` so that the key never travels alongside the slot holding the ciphertext, and so that it is not sitting in the recipient for the duration of an upload.

**`confirm`** — whether to commit. `false` selects preview: decrypt, validate, report, mutate nothing. `true` commits. Absent means `true`, which is stated in prose rather than as a schema default deliberately — see [Defaulting](#defaulting).

**`status`** — `"preview"` or `"committed"`, the recipient's account of which it did. Reported rather than inferred from the request, so that a response is a complete record on its own: a document saying `committed` is evidence the agent was replaced, and one saying `preview` is evidence it was not.

**`sourceDid`** — the identifier of the agent the bundle was exported from. The single most useful thing a preview returns, because the common import error is a valid bundle from the wrong agent, and this is what shows it.

**`keyCount`**, **`aclCount`**, **`contextCount`**, **`auditCount`**, **`importedSecretCount`** — how many of each the bundle holds. Counts, not contents: enough to recognise a bundle as plausibly the right one, and not enough to learn what is in it. `auditCount` is zero for a bundle exported without its trail.

**`message`** — operator-facing text elaborating on the outcome. Advisory; a producer **MUST NOT** parse it or derive behaviour from it.

## Defaulting

`confirm` absent means `true`. The rule is written here rather than as a JSON Schema `default`, and the reason is a practical one worth recording: code generators materialise `default` into the type they emit, so an absent member arrives at a handler indistinguishable from one a producer explicitly set. For an ordinary member that is harmless. For the member that separates a rehearsal from an irreversible replacement, it means a recipient cannot tell whether the producer chose to commit or simply omitted a field.

Recipients **MUST** treat an absent `confirm` as `true`, so that the wire behaviour matches what deployed producers already do. Producers **SHOULD** send it explicitly in both directions, and any producer offering a preview **MUST** send `false` rather than relying on an omission meaning anything.

## Request

The producer is the operator that initiated the bundle; the recipient is the agent that will adopt it. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Previewing before committing

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000009",
  "type": "https://trusttasks.org/spec/vta/backup/finalize-import/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T01:02:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "bundleId": "9c858901-8a57-4791-81fe-4c455b099bc9",
    "password": "correct horse battery staple",
    "confirm": false
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### What the preview found

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000a",
  "type": "https://trusttasks.org/spec/vta/backup/finalize-import/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T01:02:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "bundleId": "9c858901-8a57-4791-81fe-4c455b099bc9",
    "status": "preview",
    "sourceDid": "did:example:agent",
    "keyCount": 12,
    "aclCount": 34,
    "contextCount": 3,
    "auditCount": 0,
    "importedSecretCount": 12,
    "message": "Bundle is readable. Nothing has been changed."
  }
}
```

## Security & Privacy

### Data carried

The request carries **the secret that opens a complete copy of an agent**. A recipient **MUST NOT** log `password`, echo it in a response, include it in an audit entry, or persist it in any form; it is consumed deriving the key and discarded. A producer **MUST NOT** place it anywhere the envelope's confidentiality does not cover.

As with [`initiate-export`](../../initiate-export/1.0/spec.md), the password's real exposure is decided by *where it is typed*, which is why the member is annotated `writeOnly`. A field in a shared or scriptable environment — a browser form reachable by autofill, a password manager, a co-resident extension or screen capture; a shell history; a CI variable — exposes it to everything with reach into that environment, and nothing in this protocol recovers from that. Producers **SHOULD** collect it somewhere with a smaller reach and **SHOULD NOT** offer to remember it.

`decryptionFailed` deliberately does not distinguish a wrong password from tampered bytes. Separating them would turn this task into a password oracle against a bundle an attacker holds, testing candidates one request at a time and learning which are wrong. Recipients **SHOULD** additionally rate-limit failures, since the digest check that protects the *upload* says nothing about how many times the *password* may be guessed.

The response is **counts, not contents**. A recipient **MUST NOT** return key identifiers, ACL subjects, context names or audit entries from the bundle. Preview runs on a bundle whose password the producer has just demonstrated they know, but before the recipient has committed to anything — so an inventory here would be a read of the bundle's contents dressed as a rehearsal, available to anyone who obtained a bundle and its password without any entitlement to impose it.

`sourceDid` is the one identifier that is returned, and it is returned because the alternative is worse: without it, an operator cannot tell a right bundle from a wrong one before replacing an agent with it. It names the exporting agent, which the producer necessarily already knows.

### Correlation

A committed import joins two agents. `sourceDid` states that this agent now holds what that one held, and where the two identifiers differ the response is a durable link between them — a fact about a migration that did not exist before. This is intrinsic to the operation and is the reason `sourceDid` belongs in a record rather than only on a screen.

`bundleId` joins this document to the `initiate-import` that opened the slot; `threadId` joins request to response. Both are intrinsic.

The producer's identifier **MUST** be the same one that initiated the bundle, because the ownership check compares against it.

Three conditions are answered as `notFound`: no bundle with that id, a bundle of the export kind with that id, and a bundle owned by a different operator. Conflating them keeps an opaque handle from becoming an oracle over the agent's backup activity.

The sharpest correlation risk is one the protocol cannot fix and implementers should know about: after a commit, the agent's audit trail is the **bundle's**, not its own. Everything the agent did between its last export and this import is absent from it, including this import. That is why the response is the record — see [Retention](#retention).

### Retention

On commit, what the recipient receives becomes the agent's state. That is `durable` in the strongest sense available, and it displaces what was there: a recipient **MUST NOT** silently merge a bundle into existing state, because a partial adoption produces an agent that is neither what it was nor what the bundle described, and nothing in the response would say so.

The recipient **SHOULD** record that the import happened, by whom and when — and **MUST NOT** assume that record survives its own commit. A trail written into state that is then replaced is gone. Recipients **SHOULD** write the entry to a sink outside the imported state, or after the commit, or both.

This is why the response matters more here than anywhere else in the family: it is issued by the agent, about the agent, and held by the **operator**, which is the one place the account of the replacement is not itself replaced. Producers **SHOULD** retain it.

The `password` is not retained at all, at any class.

### Consent/purpose

The purpose is recovery — restoring an agent from a bundle previously exported from it or from an agent it is replacing. The operation's shape makes two abuses easy and neither is legitimate here.

A producer **MUST NOT** use an import to *acquire* an identity: applying a bundle exported from an agent one does not own makes this agent present that agent's keys to every counterparty, which is impersonation carried out through a recovery mechanism. A recipient **MUST NOT** treat a bundle's own contents as evidence that this is acceptable — see [Authorization](#authorization).

A producer **SHOULD NOT** use preview as a general inspection tool. It is a rehearsal for a commit that is intended, not a way to read bundles; the counts are deliberately thin so that using it that way yields little, and a recipient **SHOULD** record previews as well as commits so that the pattern is visible if it happens anyway.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraphs above state purpose limitations, which is a different thing.
