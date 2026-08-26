---
slug: vtc/join-requests/submit
version: "0.2"
title: VTC Join-Requests — Submit
summary: An applicant submits a request to join a Verifiable Trust Community, presenting the credentials the community's join policy requires.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - submit
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The document proof authenticates the applicant — its signer DID is the applicant DID. This replaces the transport-specific signature the pre-migration REST shape carried, and matches what DIDComm authcrypt provides intrinsically.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The applicant submits under their own authority, so a captured submission keeps re-presenting them to the community after they have stopped asking. Placing the document in a window is what lets the community stop treating it as a live request.
sideEffects:
  level: mutating
  rationale: "Creates a pending join request."
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: true
  rationale: "The applicant submits on their own behalf — the subject is the proof signer, so the applicant acts as themselves; there is no separate subject field. Nothing is disclosed back to the applicant, but the request carries `vp` into the community: a Verifiable Presentation of credentials about an identifiable party, whose claim set is whatever that community's join policy demands, plus an opaque applicant-supplied `extensions` bag the community stores verbatim."
retention:
  class: durable
  rationale: The community keeps the submitted presentation as the evidence its admission decision rested on — a decision it may have to account for to its members or to a regulator long after the fact, and which is unreconstructable if the presentation is discarded. That evidentiary value is exactly why a refused applicant does not get their claims back; see Security & Privacy → Retention, which states what this costs them.
errorCodes:
  - code: vtc/join-requests/submit:policyUnsatisfied
    meaning: The presentation did not satisfy the community's active join policy.
    retryable: false
  - code: vtc/join-requests/submit:presentationInvalid
    meaning: The Verifiable Presentation failed verification, or its holder did not match the proof signer.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Submit** Trust Task opens an application to join a community. The applicant presents a W3C Verifiable Presentation (`vp`) whose credentials satisfy the community's join policy, and optionally consents to trust-registry publication. On acceptance the community records a **pending** request and returns its `requestId`, which the applicant polls with [`vtc/join-requests/status`](../../status/0.1/).

### Changes from 0.1

`0.1` returned `status`, a `const: "pending"`. A submission has four outcomes, not one: the policy may admit outright, refuse outright, park the request for a human or quorum decision, or ask the applicant for more evidence. A constant can express one of them, so the other three had to be reported as `pending` or not reported at all — and an applicant told "pending" cannot tell whether to wait or to act.

`0.2` returns a `verdict` — `effect` plus its effect-dependent detail — from the new shared `vtc/_shared/0.1/ceremony` component. `pending` maps onto `refer`, which is the outcome it actually described: parked, waiting on the community.

The distinction that matters most is between `refer` and `requestMore`. Both mean "not decided", and they place the next action with different parties: `refer` waits on the community, `requestMore` waits on the applicant and names what it needs. Collapsing them is what makes a join flow feel like a black box.

The applicant identity is the **document proof's signer** — there is no `applicantDid` or `signature` field. This is the transport-agnostic form: over DIDComm the authcrypt sender is the signer, over REST/TSP the framework proof is, and the payload is identical on every transport.

## Conformance

Producer: supply `vp` (its holder MUST equal the proof signer); optionally `registryConsent` and `extensions`. Carry a proof.

Consumer: verify the proof and the presentation; if the VP fails verification or the holder mismatches the signer, return `presentationInvalid`; if it does not satisfy the active join policy, return `policyUnsatisfied`. Otherwise evaluate the join policy and return `{ requestId, verdict }`, where the verdict carries what the policy decided and the detail that decision implies.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is the **presentation in `vp`, whose holder MUST equal the envelope proof's signer**. That equality is the whole authorization: it establishes that the party asking to join is the party the presented credentials describe.

`exposure.actsAsSubject` is `true` because the request is made in the subject's own name. A consumer that accepted a presentation whose holder differed from the signer would be admitting one party on another's evidence, which is why the check is stated as an equality rather than as two independent verifications.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

### Data carried

`vp` is the payload, and it is the most personal member in this family. Its
schema calls it opaque because the *community's* join policy, not this
specification, decides what must be in it — so the sensitivity of a conforming
submission ranges from "proves control of a domain" to "proves a date of birth
off a government credential", and the wire form is identical in both cases. A
consumer cannot tell which it has been sent by looking at the type URI.

The applicant DID is deliberately **not** a payload member: it is the document
proof's signer. Collapsing the pre-migration REST `applicantDid` plus hex
`signature` into the framework proof removed a hand-rolled auth scheme in favour
of the one every conforming consumer already verifies, and it also removed a
field an applicant might have been tempted to populate with something other than
themselves. The VP is the *credential* evidence; the proof is *who submitted it*.

`registryConsent` is one boolean, and it is the member that governs the fate of
all the others — whether the applicant agreed to trust-registry publication.
`extensions` is an opaque applicant-supplied bag with no schema, stored verbatim
on the request row.

Minimisation is the applicant's, and it happens before the document is built: a
presentation is a selective disclosure, so a producer **SHOULD** present the
narrowest credential set and the fewest claims its reading of the
[manifest](../../manifest/0.1/spec.md) requires. There is no narrowing
afterwards. A producer **MUST NOT** move claims into `extensions` that it was
unwilling to put in `vp` — material there sits outside the presentation's
selective-disclosure machinery and is stored as plain JSON.

### Correlation

The community maintainer declares `identifierScope: public`, and it has to. An
applicant has to address the community it means to join, having found that DID in
a directory, a manifest, or a governance document published by someone else; a
pairwise community identifier would mean no two prospective applicants could
confirm they were applying to the same body, and the
[manifest](../../manifest/0.1/spec.md) that tells them what to present could
not be tied to the community that will judge it. The cost is the ordinary one: a
community DID is a fixed point every observer of the ecosystem can name.

The applicant declares `identifierScope: pairwise`, and nothing in this task
argues otherwise. The signer DID must be stable enough to poll
[`status`](../../status/0.1/spec.md) and to receive a membership credential,
which is a lifetime measured in this one relationship — not a reason to reuse the
identifier with a second community. An applicant that submits to several
communities under one DID hands every one of them a join key to the others'
records, and this specification neither needs nor rewards that.

What is unavoidable is inside `vp`. Credential identifiers, issuer DIDs, and any
non-selectively-disclosable claim travel with the presentation, so two communities
holding submissions from the same person can align them on credential material
even where the applicant used distinct DIDs. That is a property of the
presentation formats, not of this task, and it is the reason the pairwise
declaration above is a floor rather than a guarantee.

### Retention

Durable, and this is the hard edge of the family. The community records the
presentation on a [`JoinRequest`](../../../_shared/0.1/join-request.schema.json)
row — as `vp`, and again as `vpClaims`, a canonical projection extracted at
submission time so the policy engine need not re-parse the presentation. Deleting
one copy does not delete the claims.

That row survives the decision. A **refused** applicant is left in a position
worth stating plainly: the claims they disclosed to a community that would not
have them remain on that community's systems, readable by every administrator
through [`show`](../../show/0.1/spec.md) and enumerable in bulk through
[`list`](../../list/0.1/spec.md), for as long as the community keeps the row.
Nothing in this payload sets a lifetime, no member requests erasure, and no error
code refuses a submission on retention grounds — the applicant's only control is
the one they exercised before submitting, by choosing what to present.

The retention is not gratuitous. The presentation is the evidence the admission
decision rested on, and a community that discards it cannot later show why it
admitted or refused anyone. Implementers **SHOULD** publish the disposal policy
that this specification cannot supply, and **SHOULD** distinguish the retention a
refused application needs from the retention an admitted one does, because the
two are not the same question and the schema does not separate them.

### Consent/purpose

The purpose is admission: the applicant discloses credentials so that one
community can decide one application against its published join policy.
`registryConsent` is the only member in the payload that speaks to any use beyond
that, and it speaks to exactly one — whether the applicant agreed to trust-registry
publication. Its scope is therefore narrow by construction, and a consumer
**MUST NOT** read a `true` there as agreement to anything else it might do with
the presentation.

`extensions` and `ext` are applicant-authored and carry no consent signal at all.
A community that mines them, or that reuses `vpClaims` to seed a member directory,
a mailing list, or a shared registry, is putting the material to a purpose the
applicant addressed only insofar as `registryConsent` covers it.

Whether a human reviews an application, whether a second approver is required, and
what lawful basis a deployment relies on for holding the presentation are all
consumer policy questions. Per [SPEC §7.3](/SPEC.md#73-specification-requirements)
item 13 this specification takes no position on any of them; it describes what the
document moves and where it comes to rest.
