---
slug: vta/webvh/dids/realign-keys
version: "1.0"
title: "VTA WebVH DIDs — Realign Keys"
summary: "An administrator brings a did:webvh's key records back into agreement with the verification methods its published document declares."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - verification method
  - key custody
  - repair
  - migration
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Realigning renames the records through which an agent's private keys are addressed. The recipient must attribute that to a specific administrator independently of the transport that carried it, exactly as the neighbouring rotation task does."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Replayed after a later rotation has renamed the same records, an old realign re-derives its plan from the document current at execution time and is therefore idempotent — but a document that cannot be placed in time cannot be absorbed by the duplicate window of SPEC §7.2 item 11 either, and a rename of key custody is worth being able to place."
sideEffects:
  level: mutating
  rationale: "Key records are renamed. No key material is created, destroyed or exported — the same private halves remain, reachable under the identifiers the DID document publishes for them."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/dids/realign-keys:notFound"
    meaning: "No such DID is held by this recipient, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webvh/dids/realign-keys:conflict"
    meaning: "A verification method is already held by a key this realignment does not move. Nothing was changed."
    retryable: false
related:
  - vta/webvh/dids/rotate-keys
  - vta/webvh/dids/create
  - keys/rename
---

## Abstract

**VTA WebVH DIDs — Realign Keys** asks an agent to rename its own key records so
that each one is addressed by the verification-method identifier its DID
document publishes for that key.

The two can disagree. An agent that names a record when it mints a key, rather
than reading the document it just published, will name it correctly only for
documents shaped the way it assumed — and a DID document may be supplied by the
caller or rendered from a template, which is free to identify its methods any
way it likes. The result is an agent holding a key that the document addresses
under a name the agent does not answer to.

This is a Trust Task rather than an API call because **the request carries no
names**. The producer identifies a DID; every identifier involved in the outcome
is derived by the recipient from that DID's own published document. A surface
that accepted the target names would be a surface for writing arbitrary
identifiers into an agent's key store, which is precisely what the neighbouring
`keys/rename` refuses to be.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming consumer **MUST** derive every target identifier from the DID
document it currently publishes for `did`. It **MUST NOT** accept an identifier
from the producer, and this payload gives it no way to supply one.

A consumer **MUST** match its key records to verification methods by **key
material** — the public half the method declares — and **MUST NOT** match them
by position in the document or by any pattern in the identifier. A record is
what it is because of the key it holds; its current name is the thing being
repaired and therefore cannot be an input to deciding what it should be.

A consumer **MUST NOT** apply a partial realignment. Where any planned rename
cannot be performed, it **MUST** perform none of them and respond
`vta/webvh/dids/realign-keys:conflict`.

A consumer **MUST** treat a verification method whose key it does not hold as a
reportable outcome rather than an error, and **MUST** name it in
`unmatched` — "there was nothing to move" and "that key is not here" are
different answers, and only the first means the DID is now consistent.

When `dryRun` is true a consumer **MUST NOT** write anything, and **MUST**
report the same plan it would have applied.

Realignment **MUST NOT** change the DID document. It renames records inside the
consumer; the document is the authority being conformed to, never the thing
being edited. A consumer that finds the document wrong **MUST** respond rather
than correct it — `vta/webvh/dids/update` is where a document changes.

## Authorization

Authority is **administrative control of the context the DID belongs to** — the
same entitlement `vta/webvh/dids/rotate-keys` requires, and for the same reason:
both decide how an identity's keys are addressed.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what
they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the
authorization.

The absence of caller-supplied identifiers is a structural part of this: an
administrator of one context cannot name a method belonging to a DID in another,
because they cannot name anything at all. Authority still has to be checked —
this narrows what a mistake can reach, not who may ask.

## Definitions

`did` — the DID whose key records are to be realigned. The subject of the task,
and the only identifier the producer chooses.

`dryRun` — when true, asks the recipient to compute and report the realignment
without performing it. Absent reads as false.

`moved` — the renames that were performed, or in a dry run would be. Each names
the identifier the record had (`from`), the verification-method identifier it
takes (`to`), and the public key (`publicKey`) that identified it, so the
producer can check a rename against the published document rather than trusting
the report.

`alreadyAligned` — verification-method identifiers whose record was already
correct.

`unmatched` — verification-method identifiers the recipient holds no key for.
Not a failure: a DID may publish a method whose private half lives elsewhere
entirely.

`nextFragmentId` — where the recipient will allocate the next identifier it
mints for this DID. Reported because a realignment establishes what the document
already occupies, and an allocator that has not learned this may reissue a
published identifier under new key material.

## Request

The producer is an administrator of the DID's context; the recipient is the
agent holding the keys. The payload is described by the top-level schema in
[`payload.schema.json`](payload.schema.json).

### Realigning a DID, having first asked what would change

```json
{
  "id": "urn:uuid:8f14e45f-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/realign-keys/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-09-14T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:rooms:northwind"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-14T11:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

The recipient of the request responds, reporting what it did — or, for a dry
run, what it would do. The payload is described by the sub-schema reachable via
`$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A failure
is a `trust-task-error` document carrying one of the codes declared above, not a
`#response`.

`dryRun` is echoed so that a report is self-describing. A plan and a receipt are
the same shape, and a reader who has to remember which one they asked for will
eventually be wrong about it.

### Two records that swapped names

```json
{
  "id": "urn:uuid:8f14e45f-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/realign-keys/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-09-14T11:00:01Z",
  "threadId": "urn:uuid:8f14e45f-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:rooms:northwind",
    "moved": [
      {
        "from": "did:webvh:QmScidAbCdEfGh:example.com:rooms:northwind#key-0",
        "to": "did:webvh:QmScidAbCdEfGh:example.com:rooms:northwind#key-1",
        "publicKey": "z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2"
      },
      {
        "from": "did:webvh:QmScidAbCdEfGh:example.com:rooms:northwind#key-1",
        "to": "did:webvh:QmScidAbCdEfGh:example.com:rooms:northwind#key-2",
        "publicKey": "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc"
      }
    ],
    "alreadyAligned": [],
    "unmatched": [],
    "nextFragmentId": 3,
    "dryRun": false
  }
}
```

Note what this example is: the first record's destination is the second
record's current name. Renames of this family interleave, and in the general
case form a cycle, so a consumer cannot satisfy this task by requiring each
destination to be free before it moves anything.

## Security & Privacy

### Data carried

The request carries one DID and a boolean. There is nothing else to carry: the
recipient derives every identifier it uses from its own published document, and
an implementation that accepted more would be offering a way to write chosen
identifiers into a key store.

The response carries verification-method identifiers and the **public** halves
of the keys behind them. Both are already published in the DID document, which
is world-readable by construction; the response restates them so the producer
can verify a rename against that document instead of taking the recipient's word
for it. No private key material appears in either direction, and a conforming
consumer **MUST NOT** place any in `ext`.

The smallest payload that answers the task is the DID alone. `dryRun` earns its
place by being the difference between asking and doing.

### Correlation

The DID is a public identifier and appears in both documents; nothing here
reveals a relationship that resolving the DID would not. The response discloses
which of the document's published methods this particular agent holds keys for,
which is a fact about custody rather than about the document: an observer learns
that this recipient is one of the parties holding key material for that DID.
Where that matters, it is an argument about who may issue this task, not about
what the response may say.

### Retention

The response is a maintenance receipt. A producer that keeps one holds a record
of which identifiers were in use before the change, which is the evidence for
what happened and worth keeping for as long as the DID's own history is. The
recipient needs to retain nothing beyond its own audit trail.

### Consent/purpose

The purpose is repair: making an agent's key custody answer to the identifiers
its DID document publishes. The response's contents describe that repair and
are not a general inventory of the agent's keys — a producer **MUST NOT** treat
a sequence of realignments as a way to enumerate key custody across an agent's
DIDs.
