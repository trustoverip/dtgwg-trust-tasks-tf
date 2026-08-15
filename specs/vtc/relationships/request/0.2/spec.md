---
slug: vtc/relationships/request
version: "0.2"
title: VTC Relationships — Request
summary: A member asks another member to issue them a Verifiable Relationship Credential, and receives the signed VRC in the response.
status: draft
targetFrameworkVersion: "0.4"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - request
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: requesting member
    requirement: REQUIRED
    member: issuer
  - role: issuing member
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    On the request, because an unattributable ask to mint a relationship credential
    is a social-engineering surface — the issuing member decides on the strength of
    who is asking, and cannot weigh that if the ask is unsigned. On the response,
    because it delivers a signed VRC the requester will retain and later publish;
    the credential carries its own issuer signature, but the envelope proof is what
    attributes the delivery on a relayed path. Declared per variant rather than as
    one value so each states its own threat model, though both land on REQUIRED.
sideEffects:
  level: mutating
  rationale: "Successful execution mints a VRC at the issuing member; reversible via relationships/revoke."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/request:declined
    meaning: The issuing member declined to issue a VRC. Replaces the bespoke `vrc/1.0/rejected` message of the legacy exchange; the human-readable reason travels in the error payload's `message`.
    retryable: false
  - code: vtc/relationships/request:notMember
    meaning: The requester is not a member of a community the issuing member shares, so no relationship can be asserted.
    retryable: false
related:
  - vtc/relationships/publish
  - vtc/relationships/revoke
  - vtc/relationships/list
---

## Abstract

A member asks another member to issue them a **Verifiable Relationship Credential** — a credential in which the issuing member attests a relationship to the requester. The issuing member decides; the request carries at most a hint.

This replaces the legacy `https://firstperson.network/vrc/1.0/*` DIDComm exchange, completing a migration that `vtc/relationships/{publish,list,graph,revoke}` had already begun. Producers **SHOULD** build against this specification rather than the legacy types.

## Changes from 0.1

`0.1` carried the credential digest as **`vrcSha256`**, a bare lowercase-hex
SHA-256. `0.2` carries **`vrcDigestMultibase`**, the framework's
[`DigestMultibase`](../../../../_framework/0.3/framework.schema.json) — a
multibase-encoded multihash.

Three things were wrong with the old form, and the shared definition says so in
its own description. A bare hex string **hard-codes one algorithm into the wire
contract**, so moving off SHA-256 later would need a schema revision rather than
a different multihash prefix. It **names no base encoding**, so a verifier
infers base16 from context. And `0.1` **named no canonicalization** — the digest
was "SHA-256 of the VRC", which is not reproducible for a JSON document, so two
conforming implementations could compute different values for the same
credential and neither would be wrong. `0.2` states [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)
(JCS) explicitly.

This is a breaking change to the wire format, released as a `MINOR` increment
under [SPEC.md §5.2](../../../../../SPEC.md#52-compatibility-rules)'s `draft`
allowance. `0.1` remains published and unchanged; migrate with the
expand-then-contract sequence of
[§5.4](../../../../../SPEC.md#54-migrating-between-versions), and note that a
consumer at `0.2` **MUST** still accept `0.1` documents — which carry
`vrcSha256`, a member `0.2`'s schema rejects — so the two are distinguished by
the *Type URI*'s version, not by sniffing the payload.

## Conformance

A conforming **requesting member** (`issuer`):

1. Emits a document whose `type` is `https://trusttasks.org/spec/vtc/relationships/request/0.2`, addressed to the member it is asking.
2. **MAY** include a `reason` — a hint, not a term. The issuing member is under no obligation to honour it, and **MUST NOT** treat its absence as a defect.
3. **SHOULD** set `expiresAt` ([SPEC.md §4.2](../../../../../SPEC.md#42-top-level-members)) where the ask is time-bounded. The legacy exchange fixed this at 48 hours in the protocol; the framework carries it on the envelope, so the value is the producer's to choose and a *consumer* **MUST** honour it.

A conforming **issuing member** (`recipient`):

1. Applies the [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) pipeline.
2. On issuing, returns a `#response` carrying the signed `vrc`, whose `issuer` **MUST** be the issuing member and whose credential subject **MUST** be the requester.
3. On declining, returns a `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)) with `vtc/relationships/request:declined` and the reason in `message`.

### Declining is an error response, not a message type

The legacy exchange defined a third message type, `vrc/1.0/rejected`, carrying an optional `reason` and correlated by `thid`. The framework already has that: an *error response* is a Trust Task document of a framework-defined type, correlated by `threadId`, validated and signed by the same pipeline. Restating it as a task-specific message would mean a second refusal path for consumers to implement and a second place for the routing rules of [§8.1](../../../../../SPEC.md#81-the-trust-task-error-specification) to be got wrong.

So there is no `rejected` variant here. A decline is `vtc/relationships/request:declined`, and because [§8.2](../../../../../SPEC.md#82-error-payload) now carries `inResponseTo`, a retained decline names the request it answers — which the legacy `thid` correlation could not do for anyone outside the exchange.

## Relationship to `publish`

The two are separate exchanges and deliberately so. `request` obtains a VRC from its issuer; [`publish`](../../publish/0.2/spec.md) lodges a VRC with the community so it appears in `list` and `graph`. A requester that wants the credential visible performs both, in that order. Neither implies the other: a VRC may be held privately, and a member may publish a VRC obtained by other means.

## Security & Privacy

**An unsigned request is a social-engineering surface.** The issuing member is being asked to make an attributable statement about a relationship. `proof` is REQUIRED on the request so that decision rests on a verified identity rather than a claimed one.

**A decline leaks less than a silence.** `message` is free text and reaches the requester. A *consumer* **SHOULD NOT** put anything in it that the requester could not already infer — in particular, whether the issuing member holds relationships with third parties.

**The response is retained.** The requester keeps the VRC and will typically publish it later, so the response is relied upon well past the original exchange — the [§4.7.1](../../../../../SPEC.md#471-when-to-include-a-proof) condition under which a proof is mandatory, and why the response declares REQUIRED rather than inheriting a weaker default.
