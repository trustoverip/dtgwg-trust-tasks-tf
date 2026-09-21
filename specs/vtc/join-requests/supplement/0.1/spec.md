---
slug: vtc/join-requests/supplement
version: "0.1"
title: "VTC Join-Requests — Supplement"
summary: "An applicant answers a community's request for more evidence by re-presenting against their existing join request, rather than opening a second one."
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request carries the credentials the community's admission decision will
    be made on, against a request that already exists and that other people are
    working on. A consumer that cannot attribute the new evidence cannot tell a
    supplement from an attempt to substitute someone else's evidence into a
    stranger's application, and the applicant holds no session to be attributed
    by instead.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A supplement is replayable in a way a submission is not: the request it
    targets outlives it, so a captured supplement replayed later re-runs the
    policy against evidence the applicant has since replaced, and can overwrite
    a newer verdict with an older one. Two supplements to one request differ
    only in their payload and when they were issued, so `issuedAt` is what lets
    a consumer order them and apply §7.2 item 11 within a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the evidence on an open request and re-runs the community's
    admission policy against it, which may admit the applicant outright.
    Nothing is erased — the superseded presentation remains part of the
    request's history — so this is not `destructive`; what it changes is a
    decision in progress.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: true
  rationale: >-
    The applicant acts on their own behalf and on their own request: the
    subject is the proof signer. The request carries `vp` into the community —
    a Verifiable Presentation of credentials about an identifiable party, whose
    claim set is whatever the community asked for when it deferred. Nothing
    about the community's other applicants is disclosed back; the verdict the
    applicant receives concerns only their own request, and states the
    community's requirements only to the extent the deferral already did.
retention:
  class: durable
  rationale: >-
    The evidence an admission was granted on is the record that justifies the
    admission, and a community that discards it cannot later answer why it
    admitted someone. It is kept alongside the request for as long as the
    request is kept, and no longer.
errorCodes:
  - code: vtc/join-requests/supplement:notFound
    meaning: >-
      The applicant has no open request, or the named `requestId` is not
      theirs. Answered the same way for both, so that a caller cannot use this
      task to probe whether a given request id exists on this community.
    retryable: false
  - code: vtc/join-requests/supplement:notAwaitingEvidence
    meaning: >-
      The request is open, but the community has not asked this applicant for
      anything — it is queued for a decision the community owes. Retrying is
      futile until the community defers the request, which is why this is
      distinct from a transient failure.
    retryable: false
  - code: vtc/join-requests/supplement:alreadyDecided
    meaning: >-
      The request has reached a terminal state — approved, rejected or
      withdrawn — so there is no open decision left to supplement. Distinct
      from `notFound` because the applicant is entitled to know the outcome of
      their own request, and because retrying will never change it.
    retryable: false
related:
  - vtc/join-requests/submit
  - vtc/join-requests/status
  - vtc/join-requests/withdraw
---

## Abstract

A community that cannot decide a join request on what it was given defers it and says what more it needs. This task is how the applicant answers: they re-present against the request that already exists, the community re-runs its admission policy on the new evidence, and returns a fresh verdict.

It is a Trust Task rather than an API call for the same reason the submission was — the applicant is not a member and holds no session with the community, because the relationship they are asking for does not exist yet. The only thing that identifies them is the identifier they signed the submission with, so the new evidence has to arrive with its own attributable proof.

Without it, a deferral is a dead end dressed as a question. The community asks for more, and the applicant has nowhere to put it: the request stays open, the dedup rule on [`join-requests/submit`](../../submit/0.2/spec.md) refuses a second application, and the only ways out are to withdraw — discarding the vetting already gathered — or to wait for a retention sweep neither party controls.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is ownership: an applicant may supplement **their own** request and no other. A consumer establishes that the `issuer` of this document is the applicant recorded on the request being supplemented, and refuses otherwise.

That is the whole entitlement, and it is deliberately not membership, a capability or an invitation — an applicant holds none of those, which is why they are applying. Nor is it transport identity: per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes who sent the document and that it is unaltered, never that they are entitled to the outcome. The entitlement is the match between the proven issuer and the applicant on the stored request.

A consumer that cannot make that match — because no open request exists for this issuer, or because the named `requestId` belongs to someone else — answers `notFound` for both, and that error code's entry says why the two are conflated.

There is a second precondition, and it is not an authorization one: the request **MUST** be one the community has deferred. A request that is merely queued for a decision is waiting on the community, not on the applicant, and a consumer **MUST** answer `notAwaitingEvidence` rather than accept new evidence into it. The reason is that the alternative silently changes what a maintainer is looking at: a reviewer part-way through a request would find the evidence replaced underneath them, with the document they had been reading no longer the one they were asked to decide. An applicant who wants to change a request nobody asked them to change withdraws it and submits afresh, which is visible to everyone.

## Definitions

**`vp`** — the Verifiable Presentation the applicant offers in answer to the deferral, **REQUIRED**. It satisfies the `presentationDefinition` the community returned with its request for more, which the applicant reads from the deferral verdict or from a later [`join-requests/status`](../../status/0.1/spec.md) poll.

It **replaces** the presentation on the request rather than adding to it, and a consumer **MUST** evaluate its policy against this presentation alone. Merging it with what came before would produce a claim set that the applicant never presented and never signed as a whole, and which no single proof covers; a consumer cannot then say what the applicant actually asserted at the moment it admitted them. The practical consequence is that the applicant re-presents everything the community requires, not only the part that was missing, and a community that defers **SHOULD** describe the whole requirement in its `presentationDefinition` rather than only the shortfall.

Vetting attestations travel **in** the presentation, and are therefore replaced with it. A community counts an applicant's vetting by reading the attestations out of the presentation it was handed, so a replacing presentation that omits them is a presentation with no vetting, and a policy will read it that way. An applicant re-carries their vetting attestations along with everything else, and a community that defers **MUST** name them in its `presentationDefinition` alongside the rest of what it requires.

What this does not cost is the vetters' work. A vetting attestation is a credential the applicant holds; re-presenting it is the applicant's act alone, and no vetter attests a second time because the applicant answered a question about some other credential.

What a consumer **MUST NOT** do is carry a superseded presentation's attestations forward into the new decision. They were offered in a presentation the applicant has replaced, and counting them would decide the request on evidence the applicant is no longer presenting — the same defect as merging the two presentations, arriving by a different route.

**`requestId`** — the request to supplement, **OPTIONAL**, for the same reason it is optional on [`join-requests/withdraw`](../../withdraw/0.1/spec.md) and [`join-requests/status`](../../status/0.1/spec.md): an applicant whose submit response was lost never received an id. When it is absent the consumer resolves the request from the authenticated applicant's own identifier; a consumer that is given one **MUST** prefer it over inferring the request from the caller.

**`extensions`** — an opaque bag the applicant may carry alongside the presentation, **OPTIONAL**, stored verbatim and not interpreted by this task. It replaces the bag on the request on the same terms as `vp`.

**`requestId` (response)** — the request that was re-decided, always returned even when the request omitted it, so an applicant who used the id-less form learns which request they answered.

**`verdict` (response)** — what the community decided, in the same shape [`join-requests/submit`](../../submit/0.2/spec.md) returns. Deliberately the same: a supplement has exactly the outcomes a submission has, and a client that can read one can read the other without a second code path.

## Request

The applicant produces this and sends it to the community maintainer. The top-level schema is in [`payload.schema.json`](payload.schema.json). `vp` is the only required member, so the smallest conforming request is a presentation and nothing else.

### The id-less form, which is the one an applicant who lost their submit response can use

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/join-requests/supplement/0.1#request",
  "issuer": "did:example:producer",
  "recipient": "did:example:recipient",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "vp": {
      "type": ["VerifiablePresentation"],
      "holder": "did:example:producer"
    }
  }
}
```

## Response

The community maintainer — the recipient of the request, now responding — returns this. The sub-schema is reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

`requestId` names the request that was re-decided, present even when the request omitted it. `verdict` carries the fresh decision, and **MAY** be any of the outcomes a submission can produce — including another request for more evidence, because the community is entitled to still not be satisfied, and including a refusal.

Failures use `trust-task-error` rather than a `#response` document: an applicant with no open request of their own receives `notFound`, one whose request is queued rather than deferred receives `notAwaitingEvidence`, and one whose request has already been approved, rejected or withdrawn receives `alreadyDecided`.

### The evidence satisfied the policy, and the applicant was admitted

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/join-requests/supplement/0.1#response",
  "issuer": "did:example:recipient",
  "recipient": "did:example:producer",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "requestId": "urn:uuid:00000000-0000-4000-8000-00000000a1b2",
    "verdict": {
      "effect": "allow",
      "with": {
        "role": "member"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a Verifiable Presentation of credentials about the applicant, which is personal data, and an opaque `extensions` bag the community stores verbatim. This is the same material a submission carries and it arrives for the same purpose: it is what the community's admission policy reads.

A producer **SHOULD** present only what the community's `presentationDefinition` asks for. A deferral names a shortfall, and the temptation is to answer it by presenting everything to hand in the hope that something satisfies; what that actually does is hand a community claims it never asked for and must now retain and justify holding.

Because the presentation replaces rather than accumulates, a producer **MUST NOT** rely on the community having forgotten a claim in a superseded presentation. Replacement governs what the policy evaluates; it is not an erasure mechanism, and a consumer retains the request's history.

### Correlation

The supplement is bound to an existing request and is signed by the same identifier that submitted it, so it links the new presentation to the old one and to everything already recorded against the request. That linkage is the point of the task — it is what lets the applicant keep their place in the queue rather than start again — but it does mean that an applicant using a pairwise identifier for this community keeps one consistent identifier across the exchange, and cannot answer a deferral pseudonymously.

The `threadId` **SHOULD** be the one the submission established, so that the whole admission — submission, deferral, supplement, decision — reads as one exchange rather than as unrelated documents that happen to share an identifier.

### Retention

What the consumer receives is kept as long as the request it belongs to, and no longer. The presentation an admission was granted on is the record that justifies the admission: a community that discards it can no longer say why it admitted a member, which is a question both a member and an auditor may later ask.

A superseded presentation is part of that record too. A consumer **MAY** retain it for the same period, and a consumer that does **SHOULD** treat it with the minimisation obligations the current one carries — it is the same personal data, and being superseded does not make it less sensitive.

### Consent/purpose

The applicant sends this to be admitted to this community, and that is the only purpose the material it carries may be used for. A consumer **MUST NOT** use the credentials in `vp` for any purpose beyond deciding this request, and in particular **MUST NOT** evaluate them against another community's policy or retain them for a future application the applicant has not made.

A deferral asks a question; it does not oblige the applicant to answer it. An applicant who decides the community is asking for more than they are willing to give withdraws instead, via [`join-requests/withdraw`](../../withdraw/0.1/spec.md), and that remains available at every point a supplement is.
