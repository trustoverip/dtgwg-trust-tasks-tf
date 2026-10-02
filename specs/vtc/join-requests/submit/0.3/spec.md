---
slug: vtc/join-requests/submit
version: "0.3"
title: VTC Join-Requests — Submit
summary: An applicant submits a request to join a Verifiable Trust Community under one of its published join criteria, presenting what that criterion requires; the community decides it as the criterion's admission mode states.
status: draft
targetFrameworkVersion: "0.5.0"
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
  rationale: "The applicant submits on their own behalf — the subject is the proof signer, so the applicant acts as themselves; there is no separate subject field. Nothing is disclosed back to the applicant, but the request carries `vp` into the community: a Verifiable Presentation of credentials about an identifiable party, whose claim set is whatever that community's join policy demands, plus an opaque applicant-supplied `extensions` bag the community stores verbatim, plus `attributes` — self-asserted values about the applicant, such as a display name, answering the manifest's `requestedAttributes`."
retention:
  class: durable
  rationale: The community keeps the submitted presentation as the evidence its admission decision rested on — a decision it may have to account for to its members or to a regulator long after the fact, and which is unreconstructable if the presentation is discarded. That evidentiary value is exactly why a refused applicant does not get their claims back; see Security & Privacy → Retention, which states what this costs them.
errorCodes:
  - code: vtc/join-requests/submit:policyUnsatisfied
    meaning: The submission does not meet the criterion it is made under, and the community refuses it rather than asking for more.
    retryable: false
  - code: vtc/join-requests/submit:criterionUnknown
    meaning: "`criterion` names no criterion the community publishes, nor an earlier version still within its `requirementsGrace`. The applicant reads the manifest again and resubmits under a current criterion."
    retryable: false
  - code: vtc/join-requests/submit:notAccepting
    meaning: The community publishes no criteria, so it accepts no applications at present.
    retryable: false
  - code: vtc/join-requests/submit:presentationInvalid
    meaning: The Verifiable Presentation failed verification, or its holder did not match the proof signer.
    retryable: false
  - code: vtc/join-requests/submit:attributesMissing
    meaning: The manifest's `requestedAttributes` marks an attribute required and `attributes` does not answer it. The details name the missing types so the applicant can supply them and resubmit.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["types"]
      properties:
        types:
          type: array
          maxItems: 32
          items:
            type: string
  - code: vtc/join-requests/submit:attributesUnrequested
    meaning: An entry in `attributes` names a type the manifest does not request. Refused rather than stored, so a client that over-shares cannot leave an applicant's data with a community that never asked for it. The details name the types.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["types"]
      properties:
        types:
          type: array
          maxItems: 32
          items:
            type: string
---

## Abstract

The **VTC Join-Requests — Submit** Trust Task opens an application to join a community under one of the criteria it publishes ([`vtc/join-requests/manifest/0.3`](../../manifest/0.3/spec.md)). The applicant names the criterion by its `requirementsDigest`, presents a W3C Verifiable Presentation (`vp`) carrying what that criterion requires (which may be nothing), and optionally consents to trust-registry publication. The community decides the submission as the criterion's `admission` states: admitted, referred to an administrator, asked for more, or refused. On acceptance the community records a **pending** request and returns its `requestId`, which the applicant polls with [`vtc/join-requests/status`](../../status/0.1/).

### Changes from 0.2

0.2 decided a submission by the community's "active join policy", which the applicant could not see and the specification did not describe. Two applicants presenting the same thing to the same community could not tell, before applying, whether they would be admitted, queued, or refused.

0.3 decides a submission under **one criterion**: the one it names in the new `criterion` member by its `requirementsDigest`, or, when it names none, the first the community publishes that it meets. It states what deciding means: what meeting a criterion is, and what its `admission` obliges the community to do. Which criteria a community publishes, and so who it admits and how, stays its own policy. The outcomes `0.2` introduced (`allow`, `refer`, `requestMore`, `deny`) are unchanged.


## Conformance

Producer: supply `vp` (its holder MUST equal the proof signer); `criterion`, the `requirementsDigest` of the criterion it applies under, when it has chosen one; optionally `registryConsent` and `extensions`. Carry a proof. A criterion that requires no credential is met by a presentation carrying none.

Consumer: verify the proof and the presentation; if the VP fails verification or the holder mismatches the signer, return `presentationInvalid`. Then decide the submission as below, and return `{ requestId, verdict }`, where the verdict carries the decision and the detail it implies.

### Deciding a submission

The community's published criteria are its join rules; this section says how a submission is decided against them and adds no rule of its own.

**Which criterion governs.** The criterion `criterion` names, at the version that digest names while that version is within its `requirementsGrace`, and the current version otherwise (vtc/join-requests/manifest/0.3 item 5). With `criterion` absent, the first criterion, in the order the community publishes them, that the submission meets; and when it meets none, the first criterion published. The order is the community's, so where several criteria are met, which one governs — automatic or reviewed — is its choice. A consumer **MUST** refuse with `notAccepting` when it publishes no criteria, and with `criterionUnknown` when `criterion` names none of them.

**Meeting it.** A submission meets the governing criterion when every requirement the criterion states is met, and only then:

- *credentials* — the presentation carries credentials satisfying its `presentationDefinition`, each verifying (signature, validity window, revocation) and issued by a party its `credentialIssuers` admits;
- *vetting* — the statements presented meet its `vetting` object, counted as vtc/join-requests/manifest/0.3 defines;
- *invitation* — when `invitationRequired`, the presentation carries a valid, unconsumed invitation this community issued to the applicant.

A criterion that states none of these is met by every submission whose proof and presentation verify. Something presented that the governing criterion does not ask for — an invitation, a credential — does not help meet it.

**Deciding.** A consumer:

1. **MUST NOT** admit a submission that does not meet its governing criterion. It answers `requestMore`, naming what is missing, where the applicant can supply it; otherwise `deny`, or `policyUnsatisfied`.
2. For a submission meeting a criterion whose `admission` is `review`, **MUST** answer `refer` and **MUST NOT** admit on the submission. The applicant is admitted only if an administrator approves the request ([`vtc/join-requests/approve`](../../approve/0.1/spec.md)).
3. For a submission meeting a criterion whose `admission` is `automatic`, **MUST** answer `allow` and admit the applicant, unless it refuses or refers on grounds outside its criteria — an applicant it has excluded, for instance. A consumer **MAY** do that, and **MUST** record the ground it applied.
4. **MUST** record which criterion, and which version of it, governed the decision.

What a community admits on is therefore exactly what it publishes: a criterion's `admission` is a commitment, `review` never admits by itself, and nothing outside a criterion admits anyone.

**Requested attributes.** Producer: answer the manifest's `requestedAttributes` in `attributes`, one entry per attribute given, and nothing it does not ask for. The values are self-asserted; the proof binds them to the applicant and attests nothing further.

Consumer: refuse with `attributesMissing`, naming the types, when a required requested attribute is not answered, and with `attributesUnrequested`, naming the types, when an entry names a type the manifest does not request. **MUST** store the answers with the request and show them to its reviewers as the applicant's own statement — never as verified — and **MUST NOT** let a join policy treat one as attested evidence. Their retention is the request's: they are deleted with it.

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
[manifest](../../manifest/0.3/spec.md) requires. There is no narrowing
afterwards. A producer **MUST NOT** move claims into `extensions` that it was
unwilling to put in `vp` — material there sits outside the presentation's
selective-disclosure machinery and is stored as plain JSON.

`attributes` carries plain values the applicant states about themselves —
self-asserted by construction, stored as given, and bound to the applicant only
by the document proof. It holds exactly what the manifest's
`requestedAttributes` asks for: a consumer refuses a type it did not request
(`attributesUnrequested`) rather than keep it, because a value accepted "just in
case" is personal data held with no stated purpose. A producer **SHOULD** send
these through the applicant's own disclosure path so that the applicant's record
of what they told whom includes them, and **MUST NOT** move a value into
`attributes` that the applicant would only share inside a credential.

### Correlation

The community maintainer declares `identifierScope: public`, and it has to. An
applicant has to address the community it means to join, having found that DID in
a directory, a manifest, or a governance document published by someone else; a
pairwise community identifier would mean no two prospective applicants could
confirm they were applying to the same body, and the
[manifest](../../manifest/0.3/spec.md) that tells them what to present could
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
