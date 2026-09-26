---
slug: git-ns/right/break-glass-notice
version: "0.1"
title: "Git Namespaces — Break-Glass Notice"
summary: "The VTC tells each administrator of a namespace, unprompted and at once, that a member broke the glass on an elevated git right — who, what, where and why — and later that the break-glass was ratified or revoked."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - break-glass
  - notice
  - separation-of-duties
parties:
  - role: VTC
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: administrator
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: "The notice is what an administrator acts on — revoking a peer's right — and what they may later show others. Without a proof it evidences nothing beyond the transport that carried it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A notice replayed after the break-glass was resolved would put an alert back in front of administrators who had already dealt with it. The recipient places it in time, and against `at`, to tell."
sideEffects:
  level: none
  rationale: "Reports an event already carried out at the VTC; the notice changes nothing at the recipient."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Carries a right record with the break-glass justification — free text a member authored — and, for ratification and revocation, the actor's statement."
retention:
  class: durable
  rationale: "A recipient keeps the notice as its record of being told; the VTC keeps the audit event it reports."
errorCodes: []
related:
  - git-ns/right/break-glass
  - git-ns/right/ratify
  - git-ns/right/revoke
  - vtc/members/removal-notice
---

## Abstract

A [break-glass](../../break-glass/0.1/spec.md) is safe only if nobody can miss it. This task is how the VTC makes sure of that outside its own consoles: it pushes a signed notice to every administrator of the namespace — every community administrator and every live `git.ns.admin` of it — the moment a member breaks the glass, and again when another administrator ratifies or revokes it, so that every alert an administrator was shown is also cleared.

Like [`vtc/members/removal-notice`](../../../../vtc/members/removal-notice/0.1/spec.md), it answers nothing: the recipient did not ask, is not waiting, and may be offline.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential for the recipient, and declared for clarity. The notice presupposes that its `issuer` is the VTC governing the namespace; a recipient **MUST** verify the `proof` against that VTC's DID and **MUST NOT** act on a notice from anyone else. A notice is information, never authority: a recipient that wants to act on it — ratify, revoke — sends the task for that act, which the VTC checks on its own terms.

## Producer requirements

A conforming VTC:

1. Sends a notice of event `breakGlass` after the break-glass is durable ([`git-ns/right/break-glass`](../../break-glass/0.1/spec.md) step 10), `ratified` after a ratification is durable ([`git-ns/right/ratify`](../../ratify/0.1/spec.md) step 6), and `revoked` after the revocation of any record carrying `breakGlass` is durable ([`git-ns/right/revoke`](../../revoke/0.3/spec.md) step 6) — never on deciding one.
2. Sends one to every administrator of the namespace other than `by`, as they stand when the event takes effect, each as its own document with that administrator as `recipient`. It **MUST NOT** narrow that audience for any reason, community policy included.
3. Sends it over the VTC's own authenticated messaging channel to the recipient, queued for guaranteed delivery with a window long enough for an administrator who is away; a VTC that can wake a recipient's device **SHOULD** do so.
4. Sets `at` to when the event took effect, and `record` to the record as it stands after the event (for `revoked`, as it stood when revoked).
5. Records in its audit history each notice it could not queue. A failure to notify **MUST NOT** undo the event.

## Consumer requirements

A recipient that shows administrators alerts **SHOULD** show a `breakGlass` notice at once and prominently, with the justification, and **SHOULD** clear it on the `ratified` or `revoked` notice for the same record — the same `record.subject`, `record.right`, `record.resource` and `record.breakGlass.at`. A recipient **MUST NOT** render `justification` or `statement` as markup.

### Carol broke the glass on `widgets`

```json
{
  "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e21",
  "type": "https://trusttasks.org/spec/git-ns/right/break-glass-notice/0.1",
  "threadId": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e21",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-25T02:10:32Z",
  "payload": {
    "event": "breakGlass",
    "namespace": "ns_01J8Z6Q4M2",
    "record": {
      "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
      "right": "git.repo.own",
      "resource": "github.com/acme/widgets",
      "grantedBy": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
      "grantedAt": "2026-09-25T02:10:31Z",
      "breakGlass": {
        "by": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "at": "2026-09-25T02:10:31Z",
        "justification": "CVE-2026-4411 fix must ship tonight; both owners (Alice, Bob) unreachable since 22:00, paged twice. Will ask Dana to ratify in the morning."
      }
    },
    "by": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "at": "2026-09-25T02:10:31Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-25T02:10:32Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6Rn3Qw9Kb2Tx8Lm4Hc1Np5Fz7Ya3Js6Vg9Ue1Ti8Ob2Qn4Mk6Cr9Pw1El7Gh3Sy5Ad2Bv6Xf9Lj3Tq1Rz8Kn4M"
  }
}
```

## Security & Privacy

### Data carried

The notice carries one right record, its break-glass justification and, for ratification and revocation, the actor's statement. Each recipient is an administrator of the namespace, entitled to all of it through [`git-ns/view`](../../../view/0.4/spec.md). A VTC **MUST NOT** send it to anyone else.

### Why everyone, every time

A break-glass is justified by nobody else being available. The notice is how the VTC tests that claim: whoever *was* available hears of it at once and can act. That is also why policy cannot narrow the audience, and why the `ratified` and `revoked` notices go to the same people: an administrator who was shown an alert should see it resolved rather than wonder.

### Correlation

The VTC declares `identifierScope: public`, as the authority whose records these are. Each recipient declares `pairwise`.

### Retention

A recipient keeps the notice for as long as it keeps other administrative records; the VTC's audit history is the authoritative record either way.

### Consent/purpose

The purpose is to make a break-glass visible to the people who can ratify or revoke it. The VTC and the recipient **MUST NOT** use a notice for anything else.
