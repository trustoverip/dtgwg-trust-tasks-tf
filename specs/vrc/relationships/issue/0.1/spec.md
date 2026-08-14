---
slug: vrc/relationships/issue
version: "0.1"
title: VRC Relationships — Issue
summary: Delivers one party's signed Verifiable Relationship Credential to the other within an accepted relationship exchange, and returns a delivery receipt. Performed once in each direction — the exchange is mutual.
status: draft
targetFrameworkVersion: "0.4"
category: credentials
keywords:
  - vrc
  - relationship
  - issuance
  - delivery
authors:
  - Alberto L (https://github.com/albertoleon7794)
parties:
  - role: issuing party
    requirement: REQUIRED
    member: issuer
  - role: receiving party
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: OPTIONAL
  rationale: >-
    On the request, because it delivers a credential the receiving party
    retains and later presents — the SPEC.md §4.7.1 retained-and-relied-upon
    condition; the credential carries its own issuer signature, but the
    envelope proof is what attributes the delivery itself on a relayed path.
    On the response, OPTIONAL: the receipt is consumed inside the exchange by
    the connected peer, and the transport's authcrypt authenticates it.
sideEffects:
  level: mutating
  rationale: "The receiving party stores a credential. Not compensatable by this exchange; revocation is the issuer's own act."
exposure:
  discloses: none
  actsAsSubject: false
  rationale: >-
    The delivery carries a credential naming the two relationship DIDs, which
    the receiving party already holds from the accepted proposal, to the very
    party the credential is about. Nothing reaches anyone who did not already
    have it, which is why this is `none` where vrc/relationships/propose — the
    document that first discloses a relationship DID to its counterparty — is
    `metadata`.
errorCodes:
  - code: vrc/relationships/issue:notAccepted
    meaning: The receiving party refuses the delivery — the credential does not match the accepted proposal (wrong parties, wrong relationship DIDs) or arrives outside an accepted exchange.
    retryable: false
related:
  - vrc/relationships/propose
  - vtc/relationships/publish
---

## Abstract

Within an accepted relationship exchange
([`vrc/relationships/propose`](../../propose/0.1/spec.md)), each party issues
the other a **Verifiable Relationship Credential** naming the pairwise
relationship DIDs the proposal established. This specification carries one
such delivery; a conforming exchange performs it **twice, once in each
direction** — the relationship is mutual, and neither delivery is a response
to the other.

The delivery idiom is the DTG one — the signed credential in the payload with
a receipt in the `#response` — following `vtc/members/vmc` and
`vtc/join-requests/accept`: the receipt names what was received and does not
echo the credential back.

## Both deliveries share one thread

The relationship exchange runs on a single `threadId` (the `propose`
document's `id`), and it carries five documents: the proposal, its acceptance,
and **two** `issue` requests with **two** receipts. A `#response` carries no
`inResponseTo` — [SPEC.md §8.2](../../../../../SPEC.md#82-error-payload) added
that member to the *error* response only — so a party holding a receipt cannot
learn from the framework's envelope which of its deliveries the receipt
answers.

Naming the stored artifact is therefore not decoration. The receipt's
`vrcDigestMultibase` is **REQUIRED** and is what correlates it, alongside the
`issuer`/`recipient` pair that gives the direction. A receiving party
**MUST** compute that digest over the credential it stored rather than copy
the value the request carried: a copied digest attests nothing about what was
stored, and would correlate a receipt to a delivery it did not actually
receive.

The alternative — a separate thread per direction, each naming the relationship
exchange via `parentThreadId` — was rejected because it would make one mutual
exchange look like two unrelated ones to every consumer that groups by thread,
to solve a problem one required member solves.

## Conformance

A conforming **issuing party** (`issuer`):

1. Emits a document whose `type` is `https://trusttasks.org/spec/vrc/relationships/issue/0.1`, on the relationship exchange's thread (`threadId` = the `propose` document's `id`).
2. Carries in `vrc` a signed credential whose issuer is the issuing party's relationship DID and whose credential subject names the receiving party's relationship DID — the values the proposal exchanged.
3. **SHOULD** set `vrcDigestMultibase` over the [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonicalization of that credential, so the delivery can be tied to later references to it without re-hashing.
4. On receiving the receipt, **SHOULD** compare its `vrcDigestMultibase` against the credential it delivered, and **MUST NOT** treat a receipt whose digest matches no delivery of its own as acknowledging one.

A conforming **receiving party** (`recipient`):

1. Applies the [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) pipeline.
2. Verifies the credential's own proof and its party bindings against the accepted proposal **before** storing it or returning a receipt.
3. On acceptance, returns the `#response` receipt carrying `vrcDigestMultibase`, **computed over the credential as stored**. On refusal, returns a `trust-task-error` with `vrc/relationships/issue:notAccepted`.

## Witnessing

Where the parties agreed a witnessed exchange — `witnessed: true` on both the
proposal and its acceptance — each party's witness session
([`witness/session`](../../../../witness/session/0.1/spec.md)) completes before
the deliveries: what a witness attests includes that the parties exchanged
credentials under its observation, and a delivery that precedes the session is
outside that attestation. Nothing in this document references a session — the
nesting lives on the session's side, via `parentThreadId`.

## Security & Privacy

**Two proofs, two jobs.** The credential's inner proof makes the *credential*
verifiable indefinitely; the envelope proof (REQUIRED) attributes the
*delivery*. Conflating them — accepting an unproofed envelope because the
credential inside verifies — attributes storage-changing action to a document
nobody signed.

**The receipt asserts receipt, not validity.** A `#response` here means the
receiving party accepted and stored the delivery, and names which delivery by
its digest. It is not an endorsement a third party may rely on; the credential
stands on its own proof.

**A receipt is not evidence outside this exchange.** `proof` is OPTIONAL on the
response because the receipt is consumed by the connected peer under transport
authentication. A party that needs a retained, third-party-checkable record
that the exchange completed has one already — the witness session's
`submit#response`
([`witness/session/submit`](../../../../witness/session/submit/0.1/spec.md)),
which declares `proof` REQUIRED for exactly that reason. Retaining an unproofed
receipt and presenting it to a verifier who was not party to the exchange
proves nothing, and this specification does not support that use.
