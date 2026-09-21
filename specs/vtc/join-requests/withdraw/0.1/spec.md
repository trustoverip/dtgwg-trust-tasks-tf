---
slug: vtc/join-requests/withdraw
version: "0.1"
title: "VTC Join-Requests — Withdraw"
summary: "An applicant withdraws their own pending or deferred join request, closing it and releasing their ability to apply again."
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [vtc, join, requests, withdraw, a-term-a-searcher-would-use]
#   authors:
#     - Your Name (https://github.com/your-handle)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
    # identifierScope: pairwise   # framework 0.5.0, OPTIONAL: pairwise | public | any.
                                  # `public` says the counterparty must recognise a reusable
                                  # identifier, which forecloses pairwise identifiers for every
                                  # producer — the build warns unless the prose justifies it.
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The applicant ends a decision that other people are working on. Vetters may
    already have attested to them, and a maintainer may be mid-review; the
    community needs to be able to show who ended it and that the applicant
    cannot later deny doing so. A withdrawal attributable only to a transport
    session is not enough to answer "who closed this?" after the session is
    gone.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    An applicant may withdraw, then apply again — that is the point of the task.
    So a captured withdrawal replayed later does not repeat a harmless act: it
    closes the *next* request, which the applicant never withdrew. The two
    documents are identical apart from when they were issued, so `issuedAt` is
    the only thing that lets a consumer tell one from the other, and §7.2 item
    11 can only absorb the duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Ends an open request and releases the applicant to submit another. Nothing
    is erased: the record is retained under the community's own retention policy
    so the decision remains auditable, which is why this is not `destructive`.
    What it does end is work in progress — vetting attestations gathered against
    the withdrawn request no longer bear on anything, and a maintainer reviewing
    it finds it closed underneath them.
exposure:
  discloses: none
  actsAsSubject: true
  rationale: >-
    The applicant exercises their own authority over their own request: the
    subject of the task is the authenticated issuer. The response returns only
    the request's identifier and its new status, both of which the applicant
    already knows or supplied, so nothing about the community, its policy or
    its other applicants is disclosed by a withdrawal.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: durable
  rationale: >-
    The withdrawal is the record of how the request ended, and is only readable
    alongside the request it closed — so a consumer keeps it for as long as it
    keeps that request, and no longer. It is what shows the applicant ended
    this rather than the community, which is a question the applicant may
    themselves ask later; a consumer that discards it cannot answer.
errorCodes:
  - code: vtc/join-requests/withdraw:notFound
    meaning: >-
      The applicant has no open request, or the named `requestId` is not theirs.
      The two are deliberately answered the same way: distinguishing them would
      let a caller probe whether a given request id exists on this community.
    retryable: false
  - code: vtc/join-requests/withdraw:alreadyDecided
    meaning: >-
      The request has reached a terminal state — approved, rejected or already
      withdrawn — and there is nothing left to withdraw. Distinct from
      `notFound` because the applicant is entitled to know the outcome of their
      own request, and because retrying will never change it.
    retryable: false
related:
  - vtc/join-requests/submit
  - vtc/join-requests/status
  - vtc/members/self-remove
---

## Abstract

An applicant who has submitted a join request, and who no longer wants it decided, closes it themselves. The community records the request as withdrawn and stops treating the applicant as having one open, so they are free to apply again.

It is a Trust Task rather than an API call because the applicant is not a member and holds no session with the community — the whole point of a join request is that the relationship does not exist yet. The only thing that identifies them is the identifier they signed their submission with, so the withdrawal has to carry its own attributable proof for the same reason the submission did.

Without it, an applicant whose request is deferred pending evidence they cannot or will not produce has no way to close it: the community holds the request open, the applicant cannot submit another, and the only thing that resolves it is a retention sweep neither party controls.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is ownership: the applicant may withdraw **their own** request and no other. A consumer establishes that the `issuer` of this document is the applicant recorded on the request being withdrawn, and refuses otherwise.

That is the whole entitlement, and it is deliberately not membership, a capability or an invitation — an applicant has none of those, which is why they are applying. Nor is it transport identity: per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes who sent the document and that it is unaltered, never that they are entitled to the outcome. The entitlement is the match between the proven issuer and the applicant on the stored request.

A consumer that cannot make that match — because no open request exists for this issuer, or because the named `requestId` belongs to someone else — answers `notFound` for both, and the error code's entry says why the two are conflated.

This task is **not** open to any caller. A community maintainer who wants to end a request uses the maintainer-facing decision task instead; the two are separate because "the applicant changed their mind" and "the community said no" are different outcomes and a member reading the record later is entitled to tell them apart.

<!-- scaffold note retired: this task is *consequential* ([SPEC §3](/SPEC.md#3-terminology)), because it declares `sideEffects.level: mutating`, `exposure.actsAsSubject: true`. [SPEC §7.3 item 15](/SPEC.md#73-specification-requirements) requires it to describe the class of authorization evidence a consumer needs.

Name the **authority**, not the pipeline step. "The consumer verifies the proof, then executes" describes a check; it never says what entitles the producer to the outcome. Write the entitlement in one sentence — ownership of the resource, a held capability, membership of the exchange named in `parties`, an accepted prior proposal, possession of a token — and say which conformance rule enforces it.

Then distinguish it from identity and proof validation: per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying a VID, `issuer`, `recipient`, transport identity or `proof` establishes *who* and *unaltered*, never *authorized*. If this task is open to any caller, say so explicitly — that is a legitimate design and item 15 asks for it to be stated rather than inferred from silence.

Keep it descriptive: a specification **MUST NOT** declare that consent, human approval or a step-up is required. That policy belongs to the consumer. -->

## Definitions

**`requestId`** — the request to withdraw, chosen by the applicant from what the community returned when they submitted, or from a status poll. **OPTIONAL**, for the same reason it is optional on [`join-requests/status`](../../status/0.1/spec.md): an applicant whose submit response was lost never received an id, and would otherwise have no way to reach their own request. When it is absent the consumer resolves the request from the authenticated applicant's own identifier; a consumer that is given one **MUST** prefer it over inferring the request from the caller.

**`reason`** — free text the applicant may supply, **OPTIONAL**, carried into the community's record of the withdrawal. It is for the humans who will read that record — a vetter who attested to this applicant, or a maintainer who was mid-review — and a consumer **MUST NOT** interpret it or branch on its content.

**`requestId` (response)** — the request that was withdrawn, always returned even when the request was omitted, so an applicant who used the id-less form learns which request they closed.

**`status` (response)** — the request's state after the withdrawal. Always `withdrawn`; it is returned rather than implied so the response is self-describing to a reader who has only the document.

## Request

The applicant produces this and sends it to the community maintainer. The top-level schema is in [`payload.schema.json`](payload.schema.json); every member is optional, so the smallest conforming request carries an empty `payload`.

### The id-less form, which is the one an applicant who lost their submit response can use

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/join-requests/withdraw/0.1#request",
  "issuer": "did:example:producer",
  "recipient": "did:example:recipient",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "reason": "I no longer wish to join."
  }
}
```

## Response

The community maintainer — the recipient of the request, now responding — returns this. The sub-schema is reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

`requestId` names the request that was closed, and is present even when the request omitted it, so an applicant using the id-less form learns which one they withdrew. `status` is always `withdrawn`, returned rather than implied so a reader holding only this document can interpret it.

Failures use `trust-task-error` rather than a `#response` document: an applicant with no open request receives `notFound`, and one whose request has already been approved, rejected or withdrawn receives `alreadyDecided`.

### The response names the request even though the request did not

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/join-requests/withdraw/0.1#response",
  "issuer": "did:example:recipient",
  "recipient": "did:example:producer",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "requestId": "urn:uuid:00000000-0000-4000-8000-00000000a1b2",
    "status": "withdrawn"
  }
}
```

## Security & Privacy

### Data carried

The request carries a request identifier the community issued and, optionally, free text from the applicant. Neither is personal data the community does not already hold: the identifier is the community's own, and the applicant's identifier is already on the request being withdrawn.

`reason` is the one free-form member, and it is the one to be careful with. A producer **MUST NOT** put credentials, secrets, or personal data about anyone other than the applicant into it — a vetter's identity, or an account of why a third party declined to attest, ends up in a record retained for audit and readable by every maintainer. "I no longer wish to join" is the expected shape.

The smallest payload that answers the task is empty: the authenticated issuer identifies the applicant, and at most one request per applicant is open. Everything here is an addition for cases the empty form cannot serve — `requestId` for a consumer that allows more than one, `reason` for the humans reading the record.

### Correlation

The applicant's identifier appears on both the submission and the withdrawal, so the community can join them — unavoidable, and the point: the entitlement *is* that match.

`requestId` is a community-scoped identifier, so it links the two documents for anyone who sees both, including an intermediary. A producer who does not want that link visible in transit can use the id-less form, which carries no community-issued value at all; the consumer still resolves the request, but from the authenticated identifier rather than from anything on the wire.

What an applicant cannot vary is the identifier itself. It is the same one that submitted, necessarily, and a producer using a pairwise identifier per community keeps the correlation inside that one relationship rather than across communities.

### Retention

A consumer keeps the withdrawal for as long as it keeps the request it closed, and for the same reason: together they are the record of how the request ended. A withdrawn request that is pruned while its withdrawal is kept, or the reverse, leaves an account that cannot be read.

The document is evidentiary — it is what shows the applicant ended this, not the community — so a consumer that deletes it loses the ability to answer that question later, including to the applicant. That argues for keeping it to the end of the community's retention window for join requests, and not beyond: once the request itself is gone, the withdrawal documents nothing.

### Consent/purpose

The applicant supplies this to end their own application. A consumer uses it to close the request and to record who closed it — and, where the community keeps one, to release the applicant's ability to submit another.

Reusing it for anything else is outside the purpose it was given for. In particular, a withdrawal is not evidence about the applicant: that someone applied and changed their mind says nothing about their suitability, and carrying it into a future decision — a policy that scores repeat applicants, a shared list of people who withdrew — uses the record against the person who created it.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) this is descriptive: a specification **MUST NOT** declare that consent, approval or a step-up is required, and none of the above does.
