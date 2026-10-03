---
slug: vtc/operator/offline-write
version: "0.1"
title: VTC Operator — Offline Write (record type)
summary: A record type, never sent — it describes an access-control change an operator made offline on a community's host, so an acknowledge action and the audit trail have a Type URI to name it by.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - vtc
  - operator
  - offline
  - break-glass
  - emergency-bootstrap
  - audit
  - record
parties:
  - role: community maintainer recording the write
    requirement: REQUIRED
    member: issuer
  - role: administrator or auditor reading the record
    requirement: OPTIONAL
    member: recipient
  - role: operator with host access who ran the command
    requirement: REQUIRED
proofRequirement:
  requirement: OPTIONAL
  rationale: >-
    The record is never sent on its own, so it has no transport whose integrity a proof would replace. Where it matters
    it is carried as an `acknowledge` action's `payload`, whose integrity is the action's `payloadDigest` inside a
    response the community signs. A community that exports the record as a document of its own into an audit trail MAY
    sign it.
sideEffects:
  level: none
  rationale: >-
    A description of a change that already happened, made by a command on the community's host rather than by any Trust
    Task. Nothing executes a document of this type: a consumer that received one would have nothing to do, and the change
    it describes was never conditional on it.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The record carries which offline command ran, the DIDs whose access it changed, the host it ran on and when. No key
    material and no secret: the command's own inputs — an invitation code, a minted key — are never part of it.
retention:
  class: durable
  rationale: >-
    An offline write bypasses every approval the community enforces online. The record of it, and the acknowledgements
    administrators give against it, are what keep that bypass accountable; they are kept with the community's audit
    trail.
errorCodes: []
related:
  - vtc/admin/actions/acknowledge
  - vtc/admin/actions/show
  - vtc/admin/actions/list
  - vtc/admin/bootstrap
---

## Abstract

An operator who holds access to a Verifiable Trust Community's host can change
the community's access control without going through it: `vtc admin
emergency-bootstrap`, and the other offline commands, write to the community's
store directly while the service is stopped. These are break-glass paths, and
deliberately not Trust Tasks — they exist for when no administrator can act
through the community at all.

When the community next runs it records each such write as an action of category
`acknowledge`
([`vtc/admin/actions/_shared/0.2`](../../../admin/actions/_shared/0.2/action.schema.json))
and asks every remaining administrator to acknowledge it with
[`vtc/admin/actions/acknowledge/0.2`](../../../admin/actions/acknowledge/0.2/spec.md).
An action names the operation it concerns by Type URI, and an offline command
has none. **VTC Operator — Offline Write** is that Type URI: a record type whose
payload describes the write — which command, the DIDs it changed, the host, and
when — and which an action carries as its `payload`.

## A record type, not a request

No party sends a document of this type to another, and no consumer executes one.
The registry already has documents that are never sent *on their own* —
[`auth/step-up/approver/attest/0.1`](../../../../auth/step-up/approver/attest/0.1/spec.md)
and [`auth/signing-key/authorize/0.1`](../../../../auth/signing-key/authorize/0.1/spec.md)
are signed statements carried embedded inside the task they back. This
specification follows that pattern one step further: its payload is carried
embedded, as an `Action`'s `payload`, and the document form is used, if at all,
only for a community's own audit export. It declares `sideEffects: none` and no
response, and is published as a specification rather than as a shared schema
component because what an `Action` needs is a resolvable **Type URI** for its
`typeUri` member — the Type URI an approver's console resolves to learn what the
payload means and how to render it — and only a specification has one.

A consumer **MUST NOT** treat a document of this type as a request. One that
receives it over any transport **SHOULD** answer it with `trust-task-error`
`unsupportedType`, exactly as for a type it does not serve: there is nothing to
execute, and accepting it as though it did something would let a document claim
a change the community never made.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **community**, recording an offline write as an `acknowledge` action, **MUST**:

1. Set the action's `typeUri` to `https://trusttasks.org/spec/vtc/operator/offline-write/0.1` and its `payload` to a record valid against [`payload.schema.json`](payload.schema.json), and compute `payloadDigest` over that record as for any other payload.
2. List in `dids` every DID whose access the command changed, and set `at` to when the command wrote the change — taken from the host's own record of the write, not from when the community noticed it.
3. Record one action per offline command, never one for several; administrators acknowledge what one operator did at one time.
4. **MUST NOT** use this type for a change made through a Trust Task, including a break-glass operation the community executed itself. Such an action names that operation's own type.

A conforming **renderer** shows the record's members through the action's summary fields like any other payload, and **SHOULD** show `command` by a name an administrator recognises as an offline, unapproved path.

## Authorization

This record confers nothing and is never authorization for anything. The
authority it describes is **access to the community's host** — the operator's
ability to run a command against the store — which is outside the community's
own access control by construction; that is why the write needs acknowledging at
all. Acknowledging the record ([`vtc/admin/actions/acknowledge/0.2`](../../../admin/actions/acknowledge/0.2/spec.md))
records that an administrator saw it; it does not ratify the change, and reversing
the change is an ordinary administrative operation through the community.

## Definitions

- **Offline write** — a change to a community's access control made by a command run against its store on its host, not through a Trust Task.
- **`command`** — which offline command ran; a closed set.
- **`dids`** — the DIDs whose access it changed.
- **`host`** — where it ran.
- **`at`** — when it wrote the change.

## Request

There is no request in the ordinary sense: nothing is sent. The example below is
the record in document form, as a community might sign it into an exported audit
trail for an auditor. As an action's `payload`, only the `payload` member
appears.

### An emergency bootstrap, as exported to an auditor

```json
{
  "id": "urn:uuid:5d7f9b1c-3e2a-4c86-9f0b-7a1c3e5d9b42",
  "type": "https://trusttasks.org/spec/vtc/operator/offline-write/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:web:auditor.example",
  "issuedAt": "2026-10-04T12:00:00Z",
  "payload": {
    "command": "emergencyBootstrap",
    "dids": ["did:web:erin.example"],
    "host": "vtc-01.community.example",
    "at": "2026-10-02T06:12:44Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-04T12:00:00Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4bPHwJm2oCiY2XM5fmadEARvFygfZLHNf2mZvDGQPp1eMr4qg5snhSGGGy6oryxyybi3XyFQ47FqYaXhAeaqzS9Y"
  }
}
```

## Named by an acknowledge action

The same record, as the `payload` of the action administrators are asked to
acknowledge (abridged to the members that concern it; see
[`vtc/admin/actions/acknowledge/0.2`](../../../admin/actions/acknowledge/0.2/spec.md)
for the whole action):

```json
{
  "category": "acknowledge",
  "kind": "operator.offlineWrite",
  "typeUri": "https://trusttasks.org/spec/vtc/operator/offline-write/0.1",
  "payload": {
    "command": "emergencyBootstrap",
    "dids": ["did:web:erin.example"],
    "host": "vtc-01.community.example",
    "at": "2026-10-02T06:12:44Z"
  },
  "payloadDigest": "zQmbay7SyLLJyRieYo81yjde3YuF19D9h9kCNGwGBDxE5z1"
}
```

## Security & Privacy

### Data carried

Which offline command ran, the DIDs whose access it changed, the host it ran on
and when. The record **MUST NOT** carry the command's secret inputs or outputs —
an invitation's claim code, a minted key's seed — and a producer **MUST NOT** put
anything in `ext` about parties other than those in `dids`. `host` names a
machine, not a person; attribution to the operator, where the community has one,
is the action's `requester` and is not repeated here.

### Correlation

The record joins the named DIDs to a moment of break-glass access on a named
host, which is its purpose: administrators must be able to see who was given or
lost access outside the approval path. `host` is stable across records and lets
a reader join every offline write made on one machine — also intended, since
repeated offline writes on one host are what an administrator should notice.
The record discloses nothing about the community's other members.

### Retention

The record is kept with the community's audit trail, for as long as that trail
is kept, together with the acknowledgements given against it. It is the only
evidence that a change to access control was made outside the community's own
controls; deleting it would make that change indistinguishable from one the
community approved.

### Consent/purpose

The record exists to make an unapproved change visible to, and verifiably seen
by, the community's administrators and auditors. It is not authority for the
change, consent to it, or approval of anything that follows from it, and must
not be reused as any of those. Whether a community records offline writes for
acknowledgement, and which commands it counts, is the community's own policy;
this specification describes the record it uses when it does.
