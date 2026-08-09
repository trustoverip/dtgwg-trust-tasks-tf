---
slug: trust-ceremony-receipt
version: "0.1"
title: Trust Ceremony Receipt
summary: Evidence that one enactment of a Trust Ceremony completed — the steps it comprised, in order, attested by a recorder the definition names.
status: draft
targetFrameworkVersion: "0.4"
category: framework
keywords:
  - ceremony
  - receipt
  - evidence
  - framework
authors:
  - Glenn Gore (https://github.com/stormer78)
bearer: true
parties:
  - role: Recorder
    requirement: REQUIRED
    member: issuer
  - role: Verifier
    requirement: OPTIONAL
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A receipt exists to be retained and relied upon by parties who were not
    present at the exchange — the exact condition under which SPEC §4.7.1 makes a
    proof mandatory. Without one the recorder's attestation is unattributable,
    and an unattributable claim that a governance flow completed is not evidence
    of anything.
sideEffects:
  level: none
  rationale: "Reports on an enactment that has already happened; creates no state at the recipient."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Names every step of a flow, its type, and the VID that issued it — the shape
    of an interaction and who took part, though never the content of any step's
    payload. Where the steps of one ceremony reach genuinely disjoint audiences,
    that enumeration is itself a correlation; a definition that cannot accept it
    declares `enactmentPrivacy: blinded`.
errorCodes: []
related:
  - trust-task-error
  - trust-task-next-step
---

## Abstract

A **Trust Ceremony Receipt** is evidence that one *enactment* of a *Trust Ceremony* completed: which steps it comprised, in what order, and that nothing was dropped between them. It is issued by a **recorder** — a role the *ceremony definition* names — and is itself a *Trust Task document*, so one validation, signing and transport pipeline serves it as serves every other.

This specification is the registry publication of the receipt referenced by [SPEC.md §4.11](../../../SPEC.md#411-the-ceremony-member) and [§6.7](../../../SPEC.md#67-ceremony-namespace).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## What the recorder does and does not attest

The claim is deliberately narrow, and the narrowness is what makes a receipt worth having.

> The recorder attests **completeness and ordering**. It does **not** attest the content of any step.

The content of each step is attested by that step's own *issuer*, through that step's own `proof`, verifiable by anyone holding the document. The recorder mostly did not witness those steps and is not asked to vouch for them. A stronger claim would make the recorder a trusted third party, which is precisely what this framework is arranged to avoid.

What that buys, and what it does not:

| Attack | Prevented by |
|---|---|
| Forging a step | The step issuer's key — the recorder cannot mint one |
| Fabricating a whole enactment | The same |
| Omitting an **intermediate** step | The chain: its successor committed to its digest |
| Omitting the **trailing** steps (truncation) | The terminal marker — see below |
| Refusing to issue a receipt at all | **Nothing here.** An availability failure, not an integrity one |

A definition **SHOULD** name more than one recorder. Integrity rests on the terminal marker and the pinned definition rather than on who issued, so a second recorder costs nothing and removes the single point of availability. Receipts from different recorders for one enactment are directly comparable, because each is checked against the same pinned definition rather than against the other.

## Truncation, and why a terminal marker is required

A hash chain detects an omitted step through its *successor*. The trailing steps have none, so a chain alone lets a recorder present any valid prefix as a complete enactment — stopping the record just before the step that would have changed the outcome. `audit/verify` states the same property of the pattern this borrows from: *"a truncation to a valid prefix is indistinguishable from a quiet period."*

Two things close it, and both are required:

1. **A terminal step.** At least one enumerated step **MUST** have carried `ceremony.terminal` in its signed content. A recorder cannot mint that marker without the terminal step issuer's key, so a truncated receipt is visibly a prefix.
2. **Verifier-side completion.** The verifier evaluates the definition's completion rule itself. This works only because `definitionDigest` pins the rule — the two mechanisms are load-bearing on each other, and either alone is insufficient.

## Computing a step digest

A step digest **MUST** be computed as:

```
digestMultibase = multibase( multihash( H( JCS(document) ‖ salt ) ) )
```

where:

* `document` is the complete step *Trust Task document* **including its `proof`**. The digest names *the bytes a party received*, not a re-derivable signing input; excluding `proof` would let one issuer produce two equally valid documents sharing a digest.
* `JCS` is the [[RFC8785]] canonicalization, as used elsewhere in this registry.
* `salt` is the enactment salt, decoded from its multibase form.
* `H` is any hash function expressible in multihash; the multihash prefix declares which, so the algorithm is not fixed by this specification.
* The salt is a **suffix**, not a prefix. `H(salt ‖ message)` invites length-extension against the salt where `H` is a Merkle–Damgård construction; the suffix ordering does not.

The salt is per **enactment**, minted by whichever party opens it, at least 128 bits of entropy, and distributed to participants alongside the enactment identifier.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **recorder** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-ceremony-receipt/0.1`, with itself as `issuer`, and carry a `proof`.
2. Be named as a recorder by the definition identified in `payload.definition`.
3. Set `payload.definitionDigest` to the value every enumerated step carried in `ceremony.definitionDigest`, and refuse to issue a receipt where the steps disagree — steps pinning different digests are steps of different ceremonies.
4. Enumerate in `payload.steps` **every** step of the enactment it observed, each with the digest computed above.
5. Populate `payload.salt` wherever any step carried `ceremony.prev`.
6. Set `payload.complete` to its own determination, having evaluated the definition's completion rule.

A conforming **verifier** **MUST**:

1. Apply the [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) pipeline, including verifying the receipt's own `proof` against its `issuer`.
2. Resolve the definition and check that its digest equals `payload.definitionDigest`. A mismatch, or an unresolvable definition, means the receipt cannot be verified — **not** that it is invalid.
3. Confirm the recorder is one the definition names.
4. Evaluate the definition's completion rule over `payload.steps` **itself**, and **MUST NOT** rely on `payload.complete`.
5. Confirm at least one enumerated step carries `terminal`.
6. For every step document it holds, recompute the digest and reject the receipt on mismatch.
7. Verify each held step document's own `proof` against that step's `issuer`. The receipt does not vouch for step content and cannot be made to.

A verifier **MAY** hold none of the step documents. It then verifies the recorder's attestation and the shape of the enactment, and learns nothing about step content — which is the correct outcome, not a degraded one.

## Bearer

This is a **bearer specification** ([SPEC.md §4.8.3](../../../SPEC.md#483-bearer-specifications)): a receipt asserts that an enactment occurred, and that assertion is meaningful to any party able to verify the recorder's identity. Audience-binding it would defeat the point — a receipt addressed to one verifier is one that [§7.2](../../../SPEC.md#72-consumer-requirements) item 5 requires every *other* verifier to reject, and evidence that only its first recipient may act on is not evidence.

No member of the payload depends on the receiving party's identity, as [§4.8.3](../../../SPEC.md#483-bearer-specifications) requires of a bearer specification.

Bearer governs **audience**, not **distribution**. That any holder may rely on a receipt does not oblige anyone to hand it out; the recorder decides who receives one, and a ceremony whose participation must not be linkable declares `enactmentPrivacy: blinded` rather than relying on the receipt staying private.

## Security & Privacy

**A receipt is not an authorization.** It reports that a flow completed. It grants nothing, and [SPEC.md §4.11.4](../../../SPEC.md#4114-membership-is-a-claim-not-a-permission) applies to it as to the envelope member: a *consumer* **MUST NOT** grant authority on the strength of a receipt alone.

**An unresolvable definition is not a failure of the receipt.** A verifier that cannot fetch the definition cannot check completeness, and **MUST** report that it could not verify rather than that verification failed. The distinction matters where the definition was inlined for a closed deployment, or where the registry is simply unreachable.

**The salt is not a secret from receipt holders.** It defends against a party observing a document or a bare digest in transit — the threat `task-consent` names for its own salted digest. A holder of this receipt is by construction entitled to know the enactment happened and in what order, and can confirm a guessed low-entropy step payload once they have the salt. A ceremony that cannot accept this wants `enactmentPrivacy: blinded`.

**Enumeration is disclosure.** The step list names participants and the shape of an interaction. Where steps reach disjoint audiences, assembling them is itself a correlation the individual steps did not perform.

**A receipt says nothing about a nested ceremony's content.** Where a step was itself a ceremony, its evidence is that child's own receipt, verified on its own terms. A parent receipt that enumerated a child's internal steps would be attesting a flow its recorder did not observe.
