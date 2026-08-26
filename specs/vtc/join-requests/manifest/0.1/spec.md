---
slug: vtc/join-requests/manifest
version: "0.1"
title: VTC Join-Requests — Manifest
summary: Discover a community's join criteria — the presentation-definitions an applicant must satisfy — before submitting.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - discovery
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
  requirement: RECOMMENDED
  rationale: Join criteria are pre-membership discovery information; a proof is not required to read them, though it is recommended so a community can rate-limit or attribute discovery.
sideEffects:
  level: none
  rationale: "Reads the community's published join criteria; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
retention:
  class: transient
  rationale: The request has no members at all, so there is nothing for a community to keep from it beyond whatever attribution the optional proof carries. This is the one task in the family that leaves no trace of an applicant, which is precisely what makes it usable before deciding whether to apply.
---

## Abstract

The **VTC Join-Requests — Manifest** Trust Task returns a community's join criteria so a prospective applicant knows what to present. Each entry names a presentation-definition (and an optional human description) the applicant must satisfy; the applicant then submits via [`vtc/join-requests/submit`](../../submit/0.1/). This is pre-submit discovery — it precedes membership.

## Conformance

Producer: send with no parameters.

Consumer: return the `communityDid` and the list of `criteria`, each with an `id` and a `presentationDefinition`. A community MAY tailor the criteria it returns to the caller, but the default is its public join policy.

## Security & Privacy

### Data carried

The request payload has no members. That is the substance of this task, not an
omission: `manifest` is the only document in the join family a prospective
applicant can send without disclosing anything about themselves, and it is what
makes informed non-application possible. Someone can learn that a community
demands a government identity credential and decide not to apply, having handed
over nothing — which is a materially different position from discovering the same
fact by submitting and being told what was missing.

The response carries `communityDid` and `criteria`, each entry an `id`, an
optional human `description`, and a `presentationDefinition`. All of it describes
what the community asks of applicants. None of it is data about any member,
applicant, or natural person, which is why `exposure.discloses` is `metadata` and
why the framework recommends rather than requires a proof to read it.

The one thing a producer controls is *which* criteria come back: a community
**MAY** tailor them to the caller. Doing so trades this task's chief property for
targeting — a tailored manifest can only be tailored to a caller the community
recognises, which means a caller who has identified themselves. The default is
the community's public join policy, and communities **SHOULD** keep it that way
unless they have a reason that survives the paragraph below.

### Correlation

The community maintainer declares `identifierScope: public`, and this task is
where that scope is first exercised. Discovery only works if a stranger can name
the community: an applicant finds `communityDid` in a directory or a governance
document, asks it what to present, and later addresses the same identifier at
[`submit`](../../submit/0.2/spec.md). A pairwise community identifier would break
the chain at the first link — there would be no stable value for a directory to
publish or for two prospective applicants to agree named the same body. The cost
is accepted openly: a community DID is a fixed point any observer can name, and
its published criteria are enumerable by anyone who cares to ask.

The applicant declares `pairwise` because nothing here needs otherwise, and
because the recommended posture is stronger than the declaration: an unproofed
manifest read discloses no applicant identifier at all. A community that requires
a proof in order to rate-limit or attribute discovery converts an anonymous read
into an attributable one, and thereby learns *who is considering applying* — a
population strictly larger than its applicants and one that never chose to be
observed. That is a defensible trade against abuse; it is not a free one, and a
community making it **SHOULD** be deliberate about how long it keeps the record.

Tailored criteria compound this: varying the response by caller means the
community can distinguish its callers, and a caller cannot tell a tailored
manifest from the public one.

### Retention

Transient, and uniquely so in this family. There is nothing in the request to
retain. Where a community requires a proof, what it may retain is the fact that a
particular DID asked what the criteria were, at a particular time — attribution
metadata rather than application material, and it **SHOULD** be aged out on the
schedule that suits rate-limiting rather than the schedule that suits adjudication.

On the applicant's side the response is worth keeping for the length of the
decision it informs: a criteria set retained alongside a submission records what
the applicant was told to present, which is the only evidence that a later
`policyUnsatisfied` was measured against the criteria they saw.

### Consent/purpose

The purpose is pre-submission discovery, and it exists to serve data minimisation
directly. An applicant who knows the criteria presents against them; an applicant
who does not either over-presents to be safe or iterates through deferrals at
[`status`](../../status/0.1/spec.md), disclosing more at each attempt. Publishing
criteria in advance is therefore the mechanism by which the narrowest presentation
recommended at [`submit`](../../submit/0.2/spec.md) becomes achievable rather than
aspirational.

Nothing about a caller is collected here, so there is no reuse question about
applicant data. The reuse question that does arise is the community's own: a
deployment that requires a proof for rate-limiting has gathered a record of
interest, and using that record for anything other than rate-limiting — outreach,
targeting, or building a list of the curious — is a purpose the caller never
addressed. Whether to require the proof at all is a consumer policy decision, and
per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this
specification takes no position on it.
