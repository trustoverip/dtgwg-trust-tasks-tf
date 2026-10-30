---
slug: vta/webvh/dids/update
version: "1.1"
extends: "1.0"
title: WebVH DID Update
summary: A caller asks a Verifiable Trust Agent to publish a new entry in a did:webvh log whose update key the agent holds. The caller proposes the document, and may ask for the log's did:scid alias to be listed or removed; the agent decides, and signs.
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - did-webvh
  - did-management
  - update
  - key-rotation
  - did-scid
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
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An update appends to the DID's log, and appended entries cannot be withdrawn. An update that cannot be placed in time therefore writes an unretractable entry at a moment nobody chose.
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
  - code: vta/webvh/dids/update:notFound
    meaning: The agent holds no update key for this DID.
    retryable: false
  - code: vta/webvh/dids/update:versionConflict
    meaning: The DID's latest entry no longer matches `expectedVersionId` — someone else updated it since the caller read it. The caller SHOULD re-read and re-apply its edits.
    retryable: false
  - code: vta/webvh/dids/update:invalidDocument
    meaning: The document is not a valid DID document for this subject (for example, its `id` does not match `did`).
    retryable: false
  - code: vta/webvh/dids/update:scidAliasRequired
    meaning: "`scidAlsoKnownAs` is `false`, or `document` omits the alias, for an identity the agent presents in the did:scid form. That form exists only while the did:webvh document lists it (VTI-KEY-166)."
    retryable: false
  - code: vta/webvh/dids/update:noChange
    meaning: "The request would publish an entry identical in document and parameters to the latest one — for example `scidAlsoKnownAs: true` for a log that already lists the alias. Nothing is published, and the update key is not rotated."
    retryable: false
related:
  - task-consent/request
  - policy/evaluate
---

## Abstract

> **Live for identities created before key roles.** This version is how a pre-role did:webvh identity — one key in both `authentication` and `assertionMethod`, also holding the log's update authority — is managed until it is replaced. A key-role identity is managed with [`vta/dids/update/1.0`](../../../../dids/update/1.0/spec.md) and the rest of the method-neutral `vta/dids/*` family, and a VTA **MUST** refuse this version for a key-role identity with the framework's `unsupportedVersion`, because it cannot change one key role without the others (`vta/dids/_shared/0.1/CONVENTIONS.md` §3.1).

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
2. The task is classified `destructive`. SPEC [§7.3](/SPEC.md#73-specification-requirements)
   item 13 names rotation of a sole controlling key as authority-shifting, and the
   rotation happens whether or not the caller asked for it. The side effect is the
   dangerous one precisely because nobody asked for it.

## Listing the `did:scid` alias of an existing log

Every did:webvh log is also a `did:scid:vh:1:<scid>` identity: the same log
verifies it, and `?src=<source>` says where to fetch it (VTI-KEY-160, 161). The
two are distinct identities until the did:webvh document lists the did:scid DID
URL in `alsoKnownAs` (VTI-KEY-164). [`vta/webvh/dids/create/1.2`](../../create/1.2/spec.md)
lists it in the first entry by default; a log created before it, or created
without it, gets it from this task.

`scidAlsoKnownAs` asks for it by name, so a caller never builds the DID URL or
sends a whole document to change one entry. A consumer **MUST**:

1. Derive `<source>` from the did:webvh DID being updated: the part after
   `did:webvh:<scid>:`, with each `:` replaced by `/`, keeping its
   percent-encoding (a port stays `%3A<port>`). The DID URL is
   `did:scid:vh:1:<scid>?src=<source>`.
2. Start from `document` when it is supplied, and from the latest entry's
   document otherwise.
3. For `true`: add the DID URL after the `alsoKnownAs` entries already there,
   unless it is already listed. A did:scid entry naming this SCID with a
   different `src` — the log's location before a move — **MUST** be replaced
   by the current one, not kept beside it.
4. For `false`: remove every `alsoKnownAs` entry that is the did:scid DID of
   this SCID, with or without a `src`, and leave every other entry as it was.
   Refuse with `scidAliasRequired` when the agent presents this identity in the
   did:scid form (it was created with `didMethod` `scid`, or configured so
   since): removing the alias would end the identity it gives out. Apply the
   same refusal to a supplied `document` that drops the alias of such an
   identity, whether or not `scidAlsoKnownAs` is present.
5. Refuse with `noChange` when the resulting document and parameters equal the
   latest entry's, rather than publish an entry that changes nothing and
   rotates the update key.
6. Report `scidAlsoKnownAs` as the published document has it, and
   `scidDidUrl` always.

Listing or removing the alias changes the document, so it publishes an entry
and rotates the update key exactly as supplying `document` does, and the task
stays `destructive`. A consent surface **MUST** say so: "list the did:scid
alias" is a one-line change to `alsoKnownAs` and a rotation of the key that
controls the identity.

The alias links the forms from the version that lists it. A node **MUST NOT**
treat them as one subject for anything authorized against a version that did
not list it, and **MUST** stop treating them as one subject once a version
removes it (VTI-KEY-164).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/webvh/dids/update/1.1`, with the agent as `recipient`, carrying a verifiable `proof`.
2. Populate `payload.did` with the subject.
3. Populate `payload.expectedVersionId` with the versionId it based the edit on, **whenever a human or a concurrent writer could have changed the DID in between** — which is to say, whenever the edit was authored against something the caller read. See below.

A conforming **consumer** (the agent) **MUST**:

1. Verify the `proof` and that the issuer is authorized over `did`.
2. Reject a payload carrying members this schema does not define. The schema is closed, and it is closed for a reason (see *Closed payloads*).
3. Refuse the update when `expectedVersionId` is present and no longer matches the DID's latest entry → `versionConflict`.
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

The framework's `ext` slot ([SPEC §4.5.1](/SPEC.md#451-the-ext-extension-member))
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
`payload.scidAlsoKnownAs` (OPTIONAL) — list (`true`) or remove (`false`) the log's did:scid DID URL in `alsoKnownAs`. **Rotates the update key**, like `document`.
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Changes from 1.0

- `scidAlsoKnownAs` in the request, and `scidAlsoKnownAs` and `scidDidUrl` in
  the response.
- `scidAliasRequired` and `noChange` error codes.

Every 1.0 request is a valid 1.1 request with the same meaning, so 1.1
`extends` 1.0.

## Examples

### List the did:scid alias of an existing DID

```json
{
  "id": "urn:uuid:6a0f3c52-1d84-4b7e-9e21-5b8c7a4d2e19",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/1.1",
  "issuer": "did:key:z6MkCallerExample",
  "recipient": "did:key:z6MkAgentExample",
  "issuedAt": "2026-10-30T10:12:00Z",
  "payload": {
    "did": "did:webvh:QmSCIDExample:example.com:acme",
    "scidAlsoKnownAs": true,
    "expectedVersionId": "3-QmPriorEntryHashExample",
    "label": "list the did:scid alias"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-30T10:12:00Z",
    "verificationMethod": "did:key:z6MkCallerExample#z6MkCallerExample",
    "proofPurpose": "authentication",
    "proofValue": "z3Hn8QExampleProofValueForScidAliasRequest"
  }
}
```

The response reports the entry and both forms:

```json
{
  "did": "did:webvh:QmSCIDExample:example.com:acme",
  "newVersionId": "4-QmNewEntryHashExample",
  "updateKeysCount": 1,
  "preRotationKeyCount": 2,
  "scidAlsoKnownAs": true,
  "scidDidUrl": "did:scid:vh:1:QmSCIDExample?src=example.com/acme"
}
```

### Add a service endpoint to a DID the agent holds the key for

## Request

```json
{
  "id": "urn:uuid:2f7c1a90-4b6e-4d21-9a55-1c3e8b7d0f42",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/1.1",
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
    "proofPurpose": "authentication",
    "proofValue": "z4Xq7WExampleProofValueForWebvhUpdateRequest"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:8d1b6e34-7f92-4c05-b3a1-6e0d29c4f8b7",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/1.1#response",
  "issuer": "did:key:z6MkAgentExample",
  "recipient": "did:key:z6MkCallerExample",
  "issuedAt": "2026-07-14T10:12:04Z",
  "payload": {
    "did": "did:webvh:QmSCIDExample:example.com:acme",
    "newVersionId": "4-QmNewEntryHashExample",
    "updateKeysCount": 1,
    "preRotationKeyCount": 2,
    "scidAlsoKnownAs": false,
    "scidDidUrl": "did:scid:vh:1:QmSCIDExample?src=example.com/acme"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-07-14T10:12:04Z",
    "verificationMethod": "did:key:z6MkAgentExample#z6MkAgentExample",
    "proofPurpose": "authentication",
    "proofValue": "z9LmTpExampleProofValueForWebvhUpdateResponse"
  }
}
```
