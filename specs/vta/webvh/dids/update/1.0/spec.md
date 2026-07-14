---
slug: vta/webvh/dids/update
version: "1.0"
title: WebVH DID Update
summary: A caller asks a Verifiable Trust Agent to publish a new entry in a did:webvh log whose update key the agent holds. The caller proposes the document; the agent decides, and signs.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - did-webvh
  - did-management
  - update
  - key-rotation
  - delegated-execution
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: Verifiable Trust Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The agent signs a log entry that is thereafter part of a public, append-only identity history, using a key the caller does not hold and cannot be given. It must be able to attribute the request to a party authorized over the subject, and a bearer token proves only who opened the channel.
sideEffects:
  level: destructive
  rationale: "Supplying `document` rotates the DID's update key: the key that could authorize changes before this entry cannot afterwards. That is rotation of a sole controlling key, and so authority-shifting. The published log entry is also permanent — a subsequent entry can supersede it, but nothing can unpublish it."
consequences:
  - "Any change to the document rotates this DID's update key. The current update key stops being able to authorize further changes."
  - "Refreshes the pre-rotation commitments that will authorize the next rotation."
  - "Appends to a public, append-only log. The new state resolves immediately and the entry cannot be withdrawn."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/webvh/dids/update:not_found
    meaning: The agent holds no update key for this DID.
    retryable: false
  - code: vta/webvh/dids/update:version_conflict
    meaning: The DID's latest entry no longer matches `expectedVersionId` — someone else updated it since the caller read it. The caller SHOULD re-read and re-apply its edits.
    retryable: false
  - code: vta/webvh/dids/update:invalid_document
    meaning: The document is not a valid DID document for this subject (for example, its `id` does not match `did`).
    retryable: false
related:
  - task-consent/request
  - policy/evaluate
---

## Abstract

The **WebVH DID Update** Trust Task is how a caller who does **not** hold a
`did:webvh` update key asks the agent that does to publish a change.

This is the whole point of it. The caller — a web page, a CLI, another agent —
proposes a document. It cannot sign a log entry, it is never given the key, and
nothing it sends carries authority. The agent validates the request, decides
whether its owner's policy permits it, signs, and publishes.

## The consequence that is not in the payload

Supplying `document` **rotates the DID's update key** and refreshes its
pre-rotation commitments.

That is not stated anywhere in this payload, and it cannot be: it is a property of
the handler's semantics, not of the document's shape. A caller who added one
service endpoint has also, unavoidably, replaced the key that controls the
identity.

Two things follow, and both are normative:

1. A consent surface asking a human to approve this task **MUST** render effects
   the executing agent computed — by dry-running the handler it is about to
   invoke — and **MUST NOT** derive them from this payload. A diff of the document
   shows a one-line endpoint addition and silently hides a key rotation, and every
   signature over the resulting approval still verifies.
2. The task is classified `destructive`. SPEC [§7.3](../../../../../../SPEC.md#73-specification-requirements)
   item 13 names rotation of a sole controlling key as authority-shifting, and the
   rotation happens whether or not the caller asked for it. The side effect is the
   dangerous one precisely because nobody asked for it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/webvh/dids/update/1.0`, with the agent as `recipient`, carrying a verifiable `proof`.
2. Populate `payload.did` with the subject.
3. Populate `payload.expectedVersionId` with the versionId it based the edit on, **whenever a human or a concurrent writer could have changed the DID in between** — which is to say, whenever the edit was authored against something the caller read. See below.

A conforming **consumer** (the agent) **MUST**:

1. Verify the `proof` and that the issuer is authorized over `did`.
2. Reject a payload carrying members this schema does not define. The schema is closed, and it is closed for a reason (see *Closed payloads*).
3. Refuse the update when `expectedVersionId` is present and no longer matches the DID's latest entry → `version_conflict`.
4. Derive the task's side-effect class from the handler it is about to invoke — **not** from this specification's declaration, which is advisory. See SPEC §7.3 item 13.

## `expectedVersionId`, and why it is not a formality

Without it, a `get → edit → save` cycle silently overwrites a concurrent edit
with a chain that is structurally valid, verifies perfectly, and is based on a
stale read. The lost update leaves no trace, because there is nothing wrong with
the log it produces.

Where the update is gated on human approval the window is minutes wide, so this
is a routine race rather than an exotic one.

It is OPTIONAL in the schema because a scripted caller with no concurrent writers
has nothing to protect against. It is not optional for anything a person looked
at.

## Closed payloads

`additionalProperties` is `false`, and a consumer **MUST** enforce it.

An unrecognised member is not harmless. A member the consumer silently ignores is
a member the caller believes it sent — and if that member was a safety
precondition, the caller's own source reads as though the danger were handled
while nothing is handling it. A silently-ignored precondition is worse than an
absent one, because an absent one is visible.

The framework's `ext` slot ([SPEC §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member))
remains the sanctioned place for anything not defined here.

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.document` (OPTIONAL) — the new DID document. **Rotates the update key.**
`payload.preRotationCount` (OPTIONAL) — commitments to publish; `0` disables.
`payload.witnesses` (OPTIONAL) — new witness configuration.
`payload.watchers` (OPTIONAL) — new watcher URLs; `[]` removes them.
`payload.ttl` (OPTIONAL) — new TTL, seconds.
`payload.label` (OPTIONAL) — operator-facing audit label.
`payload.expectedVersionId` (OPTIONAL) — optimistic-concurrency precondition.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Add a service endpoint to a DID the agent holds the key for

## Request

```json
{
  "id": "urn:uuid:2f7c1a90-4b6e-4d21-9a55-1c3e8b7d0f42",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/1.0",
  "issuer": "did:key:z6MkCallerExample",
  "recipient": "did:key:z6MkAgentExample",
  "issuedAt": "2026-07-14T10:12:00Z",
  "payload": {
    "did": "did:webvh:QmSCIDExample:example.com:acme",
    "document": {
      "@context": ["https://www.w3.org/ns/did/v1"],
      "id": "did:webvh:QmSCIDExample:example.com:acme",
      "service": [
        {
          "id": "#files",
          "type": "FileStore",
          "serviceEndpoint": "https://files.example.com/acme"
        }
      ]
    },
    "expectedVersionId": "3-QmPriorEntryHashExample",
    "label": "add file store"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-14T10:12:00Z",
    "verificationMethod": "did:key:z6MkCallerExample#z6MkCallerExample",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Xq7WExampleProofValueForWebvhUpdateRequest"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:8d1b6e34-7f92-4c05-b3a1-6e0d29c4f8b7",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/1.0#response",
  "issuer": "did:key:z6MkAgentExample",
  "recipient": "did:key:z6MkCallerExample",
  "issuedAt": "2026-07-14T10:12:04Z",
  "payload": {
    "did": "did:webvh:QmSCIDExample:example.com:acme",
    "newVersionId": "4-QmNewEntryHashExample",
    "updateKeysCount": 1,
    "preRotationKeyCount": 2
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-14T10:12:04Z",
    "verificationMethod": "did:key:z6MkAgentExample#z6MkAgentExample",
    "proofPurpose": "assertionMethod",
    "proofValue": "z9LmTpExampleProofValueForWebvhUpdateResponse"
  }
}
```
