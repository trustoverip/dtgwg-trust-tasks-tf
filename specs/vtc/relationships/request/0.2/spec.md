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
    identifierScope: public
  - role: issuing member
    requirement: REQUIRED
    member: recipient
    identifierScope: public
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
  discloses: secret
  ingests: personal
  actsAsSubject: false
  rationale: >-
    The response carries `vrc` — the signed Verifiable Relationship Credential
    the issuing member just minted, which the response schema makes REQUIRED. The
    requester retains it and typically publishes it later, as Security & Privacy
    → Retention already states; a decline is an error
    document, so a successful response always releases the credential.
    Inbound the request carries `reason` — unbounded free prose the requester
    writes for a human to read, describing a relationship between two identified
    members and whatever history the requester thinks will persuade.
retention:
  class: durable
  rationale: >-
    The response delivers a signed credential the requester keeps and typically
    lodges with the community through `vtc/relationships/publish`, after which it
    is readable by anyone who can call `vtc/relationships/list`. A relationship
    credential is an assertion about two named parties that is meant to be relied
    on later — a requester who discarded it would lose the only artefact proving
    the issuing member ever vouched for them — and revocation, not deletion, is
    the mechanism for withdrawing it.
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
under [SPEC.md §5.2](/SPEC.md#52-compatibility-rules)'s `draft`
allowance. `0.1` remains published and unchanged; migrate with the
expand-then-contract sequence of
[§5.4](/SPEC.md#54-migrating-between-versions), and note that a
consumer at `0.2` **MUST** still accept `0.1` documents — which carry
`vrcSha256`, a member `0.2`'s schema rejects — so the two are distinguished by
the *Type URI*'s version, not by sniffing the payload.

## Conformance

A conforming **requesting member** (`issuer`):

1. Emits a document whose `type` is `https://trusttasks.org/spec/vtc/relationships/request/0.2`, addressed to the member it is asking.
2. **MAY** include a `reason` — a hint, not a term. The issuing member is under no obligation to honour it, and **MUST NOT** treat its absence as a defect.
3. **SHOULD** set `expiresAt` ([SPEC.md §4.2](/SPEC.md#42-top-level-members)) where the ask is time-bounded. The legacy exchange fixed this at 48 hours in the protocol; the framework carries it on the envelope, so the value is the producer's to choose and a *consumer* **MUST** honour it.

A conforming **issuing member** (`recipient`):

1. Applies the [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) pipeline.
2. On issuing, returns a `#response` carrying the signed `vrc`, whose `issuer` **MUST** be the issuing member and whose credential subject **MUST** be the requester.
3. On declining, returns a `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)) with `vtc/relationships/request:declined` and the reason in `message`.

### Declining is an error response, not a message type

The legacy exchange defined a third message type, `vrc/1.0/rejected`, carrying an optional `reason` and correlated by `thid`. The framework already has that: an *error response* is a Trust Task document of a framework-defined type, correlated by `threadId`, validated and signed by the same pipeline. Restating it as a task-specific message would mean a second refusal path for consumers to implement and a second place for the routing rules of [§8.1](/SPEC.md#81-the-trust-task-error-specification) to be got wrong.

So there is no `rejected` variant here. A decline is `vtc/relationships/request:declined`, and because [§8.2](/SPEC.md#82-error-payload) now carries `inResponseTo`, a retained decline names the request it answers — which the legacy `thid` correlation could not do for anyone outside the exchange.

## Relationship to `publish`

The two are separate exchanges and deliberately so. `request` obtains a VRC from its issuer; [`publish`](../../publish/0.2/spec.md) lodges a VRC with the community so it appears in `list` and `graph`. A requester that wants the credential visible performs both, in that order. Neither implies the other: a VRC may be held privately, and a member may publish a VRC obtained by other means.

## Security & Privacy

### Data carried

The request has exactly one substantive member, and it is free text. `reason` is
the requester's case for why the issuing member should vouch for them, written in
their own words, with no length bound in the schema and no structure imposed on
it. It is a hint and not a term — the issuing member is under no obligation to
honour it and **MUST NOT** treat its absence as a defect — but it is also the
only prose in this exchange, so it attracts whatever context the requester
believes will persuade: shared work, shared history, shared acquaintances. A
producer **SHOULD** keep it to what it is willing to have quoted back, and
**SHOULD NOT** name third parties in it, who are not party to this exchange and
have no way to know they were described in it.

The request being unsigned would be a social-engineering surface, which is why
`proof` is REQUIRED on it: the issuing member is being asked to make an
attributable statement about a relationship, and cannot weigh who is asking if
the ask is unattributable.

The response is the credential itself. `vrc` is a signed W3C Verifiable
Relationship Credential whose issuer is the issuing member and whose
`credentialSubject.id` names the requester — an assertion by one identified party
about another, released to the requester to keep.
`vrcDigestMultibase` is a digest over the RFC 8785 canonicalization of that
credential, and it is not neutral: it is a stable fingerprint by which the same
credential can be recognised wherever it later appears, which is the property
that lets [`publish`](../../publish/0.2/spec.md) be tied back to this exchange
without re-hashing.

A decline is a `trust-task-error` carrying
`vtc/relationships/request:declined`, with the human-readable reason in
`message`. That `message` is free text reaching the requester, and a consumer
**SHOULD NOT** put anything in it the requester could not already infer — in
particular, whether the issuing member holds relationships with third parties. A
decline that leaks the shape of the issuing member's other relationships is worse
than a silence, and the point of the standard error code is that it does not have
to.

### Correlation

Both parties declare `identifierScope: public`, and unlike elsewhere in this
registry that is not a concession — it is the task working as designed. A
relationship credential asserts that *this* member vouches for *that* one, and it
is worth something only to a third party who can recognise both DIDs as the ones
they see in the community directory, in a
[`list`](../../list/0.2/spec.md) response, and on the other credentials each
party holds. Pairwise identifiers would defeat the credential outright: a VRC
naming two identifiers nobody else can resolve asserts a relationship between two
parties nobody else can identify, which is a statement with no relying party.

So the correlation is the product. Every VRC issued through this task is one more
edge in a graph that [`list`](../../list/0.2/spec.md) and `graph` expose, and the
accumulation is far more revealing than any single credential: who a member is
connected to, how densely, and when each connection was formed. `vrcDigestMultibase`
makes individual edges trackable across stores, and the credential's own
`createdAt` timestamps the relationship's formation.

A member should therefore understand that requesting a VRC is requesting to be
placed in a public graph, and that the decision to publish is separate — see
*Relationship to `publish`* above. A credential obtained here and never published
is joinable only by the two parties that hold it; that is the privacy-preserving
path, and it remains available precisely because the two exchanges were kept
apart.

### Retention

Durable, and durable in the requester's hands rather than the issuer's. The
requester keeps the VRC and will typically lodge it with the community later, so
the response is relied upon well past the original exchange — the
[§4.7.1](/SPEC.md#471-when-to-include-a-proof) condition under which a proof is
mandatory, and why the response declares REQUIRED rather than inheriting a weaker
default.

The consequence is that this task's effect cannot be retracted by deleting
anything. Once released, the credential is a signed artefact the requester holds;
if it has been published it is also in the community's store and in the hands of
everyone who has listed it since. Withdrawal is
[`revoke`](../../revoke/0.1/spec.md) — a *new* signed statement that supersedes
the old one — not erasure, and a member who wants a relationship un-asserted
should expect the record of its having existed to remain.

The `reason` has a quieter retention story and no rule at all. It reaches a
human, may be recorded alongside the issuing member's decision, and is not
carried in the credential. Neither party has an obligation here, which is the
strongest argument for keeping it short.

### Consent/purpose

The purpose is attestation on request: a member asks to be vouched for, and an
identified member decides whether to say so in a form others can verify. The
consent that matters flows in an unusual direction for this registry — the
*subject* of the credential is the party asking for it, so the request is itself
the subject's agreement to be described.

That agreement is bounded by what the issuing member chooses to assert, and by
nothing else. `reason` is explicitly a hint rather than a term: a requester
**MUST NOT** read a granted VRC as having been issued on the terms they proposed,
and an issuing member is free to attest something narrower, broader, or otherwise
shaped than what was asked for. The credential's own contents are the agreement;
the request is not.

Nothing in this exchange authorises publication. Obtaining a credential and
lodging it with the community are separate acts by separate tasks, and a
consumer **MUST NOT** treat a successful `request` as consent to
[`publish`](../../publish/0.2/spec.md). Whether an issuing member consults a
human, applies a policy, or requires prior relationship evidence before agreeing
is a consumer decision on which this specification takes no position.
