---
slug: vtc/join-requests/status
version: "0.1"
title: VTC Join-Requests — Status
summary: An applicant polls the state of their pending join request, learning what more is needed if deferred, and may ask for an approved request's credentials to be sent again.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - status
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
  rationale: The poller must prove they are the applicant that owns the request; the document proof's signer is checked against the request's applicant. This replaces the pre-migration REST signature.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A poll carrying `resendCredentials` asks the community to deliver credentials again. Without a date, a captured poll replayed later would trigger a delivery nobody asked for at that time; the window is what lets the consumer refuse it.
sideEffects:
  level: mutating
  rationale: "A plain poll reads the request's current state and persists nothing. With `resendCredentials`, it queues a re-delivery of credentials the community already issued for an approved request, and records that it did so for its rate limit. Nothing is issued, and the request's state is unchanged."
subjectPath: /requestId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The request carries at most `requestId`, an identifier the community minted, and often not even that — the id-less form is resolved from the proof signer. Nothing about the applicant travels inbound that the community did not already hold."
retention:
  class: transient
  rationale: A poll reads the request's current state, and a re-delivery request records only when it was last honoured, for the rate limit. The refusal members it returns (`code`, `reason`, `decidedAt`) are already durable on the community's own row; this task only reads them out, and does not extend their life.
errorCodes:
  - code: vtc/join-requests/status:notFound
    meaning: No join request with the supplied requestId exists, or it does not belong to the proof signer.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Status** Trust Task lets an applicant poll their own join request. It returns the current `status`; when `deferred`, `needs` names what the applicant must supply and `presentationDefinition` describes the additional evidence to present. When `rejected`, `code`, `reason` and `decidedAt` say why and when. The applicant is the proof signer, checked against the request's owner.

`requestId` is **optional**. An applicant whose first reply was lost never received an id, and a poll resolved from their own authenticated DID is the only form available to them — a refusal they cannot ask about is a refusal they cannot act on. A consumer that is given the id MUST prefer it over inferring the request from the caller.

An approved request is not finished until its membership credential reaches the applicant. On an `approved` response, `credentialsDelivered` says whether the community holds the applicant's acknowledgement that it arrived. An applicant still waiting may set `resendCredentials`, and the community answers in `credentialResend` whether it queued a re-delivery. A re-delivery sends again the credentials the community already issued for this request; it never issues new ones.

The three refusal members are the applicant's half of a rejection. `code` is stable and safe to branch on; `reason` carries the decider's words when there were any; `decidedAt` is when the decision was taken, not when this poll was produced — on an admin refusal the two diverge by however long the applicant takes to ask.

## Conformance

Producer: supply `requestId` when you hold one; omit it when you do not. Carry a proof; the signer MUST be the request's applicant.

Consumer: resolve the request — from `requestId` when supplied, otherwise from the proof signer's own DID — and confirm the proof signer owns it; if not (or it is absent), return `notFound` — the same code for both, so a poller cannot probe for requests it does not own. Return the current `status`, plus `needs`/`presentationDefinition` when deferred, and `code`/`reason`/`decidedAt` when rejected.

A consumer MUST NOT return the refusal members for any status other than `rejected`: they say a decision was taken, and emitting them beside a `pending` status would tell an applicant their request had been refused when it had not.

**Credential delivery.** On an `approved` response, a consumer SHOULD return `credentialsDelivered`: `true` once it holds the applicant's acknowledgement that the membership credential arrived, `false` otherwise. It MUST NOT return `credentialsDelivered` or `credentialResend` for any other status.

A producer MAY set `resendCredentials: true` when the response it last saw was `approved` with `credentialsDelivered: false`, and it has waited a reasonable interval without the credential arriving. A producer MUST NOT set it on every poll; it is a request for action, and the consumer rate-limits it.

A consumer that honours `resendCredentials`, for an `approved` request, answers `credentialResend`:

- `queued` — it has queued a re-delivery to the request's applicant. It MUST send the credentials it already issued for this request and MUST NOT issue new ones; re-issuing would mint a second credential for the same membership.
- `notNeeded` — it already holds the applicant's acknowledgement, so `credentialsDelivered` is `true`.
- `rateLimited` — it honoured a re-delivery for this request too recently. It SHOULD return `retryAfter`, the earliest time it will honour another.

A consumer MUST send a re-delivery only to the request's applicant, never to a party named in the payload. It SHOULD bound how often it honours the flag per request and how many times in total, and back off between them. An `approved` response carrying no `credentialResend` after `resendCredentials` was set means the consumer does not support re-delivery; the applicant then has to ask the community's administrators.

## Authorization

The authority this task assumes is **ownership of the request**: being the applicant who submitted it. The `proof` does not grant it; it identifies the signer so the consumer can compare them against the request's applicant, and an unknown request and somebody else's are both `notFound`. That ownership is all a poll needs to read its own request's state. It is also all `resendCredentials` needs, because the re-delivery sends the applicant credentials the community already decided to issue them, to them alone: it confers nothing they do not already hold a decision for.

## Security & Privacy

### Data carried

Almost nothing goes out and something quite specific comes back. The request is
at most `requestId` and the `resendCredentials` flag; in the id-less form it is
empty apart from the flag, and the community resolves the request from the proof
signer's own DID. Nothing about the applicant travels
inbound that the community did not mint or already hold.

The response is the applicant's half of a decision. `status` is a five-value
enum. On a refusal, `code` is a stable branchable value, `reason` is free prose
written by whoever decided, and `decidedAt` is when — not when the poll was
answered, so the gap between the two is visible to the applicant. On a deferral,
`needs` names what to supply and `presentationDefinition` describes the evidence
to present.

`reason` is the member to watch, because it is the one place in this task where a
human's words reach a party outside the community. It originates at
[`decide`](../../decide/0.1/spec.md) and is relayed here verbatim. A producer of
that reason **SHOULD NOT** write anything into it that reveals a third party — a
referee who objected, another applicant whose case set a precedent, an internal
deliberation — because this task is the channel by which it leaves the community.
`code` exists precisely so a client can behave correctly on a refusal without
`reason` needing to carry the detail.

The consumer rule that the refusal members appear only when `status` is
`rejected` is a data-minimisation rule as well as a correctness one: emitting them
alongside a `pending` status would tell an applicant their request had been
refused when it had not.

### Correlation

The community maintainer declares `identifierScope: public`. An applicant polls
the community it applied to, addressing it by the same DID it found before
submitting and the same one the
[`manifest`](../../manifest/0.1/spec.md) published; a pairwise community
identifier would leave a poller unable to confirm that the party answering is the
party that took their evidence. The applicant declares `pairwise` for the reason
[`submit`](../../submit/0.2/spec.md) gives — the identifier needs to persist for
the life of this application and no further.

The id-less poll deserves a note, because it looks like a shortcut and is really
a binding. Resolving the request from the signer's DID means the community
maintains a mapping from applicant DID to open request, so an applicant who
polls under the same DID they submitted under is correlatable across the whole
lifecycle by construction — which is what makes the recovery case work at all,
and is the reason it is not a privacy loss so much as a restatement of the
pairwise scope.

The collapse of *unknown request* and *not your request* into one `notFound` is
what keeps this from being an oracle: a poller cannot probe for the existence of
other applicants' requests, because both answers look the same. `decidedAt` is
still a real signal — repeated polling reconstructs how long a community sat on
a decision — but it is a signal about the applicant's own request.

### Retention

Transient. A plain poll reads current state and writes nothing. A honoured
`resendCredentials` records when it was honoured, for the rate limit, beside the
request it concerns. The values a poll returns were already durable on the
community's row from
[`submit`](../../submit/0.2/spec.md) and [`decide`](../../decide/0.1/spec.md).

The asymmetry worth naming is that the applicant is the one party in this family
who ends up holding a durable record they can act on. The refusal — its `code`,
its `reason`, its `decidedAt` — is delivered here and is the applicant's own
copy of a decision the community will otherwise retain unilaterally. A client
**SHOULD** keep it: it is the only artefact that lets an applicant later
demonstrate what they were told and when.

### Consent/purpose

The purpose is narrow and mutual: the applicant learns the state of their own
application so they know whether to wait, to act, or to give up. That is the
whole of it, and the ownership check is the mechanism — the proof signer must own
the request, or the answer is `notFound`.

`needs` and `presentationDefinition` are a second, deliberate purpose: telling an
applicant precisely which evidence is missing so they can present that and no
more. This is a privacy feature and should be read as one — a community that
answers `deferred` without naming what it needs pushes the applicant toward
over-disclosing on the next attempt. It is also, unavoidably, a channel that
tells the applicant which specific claim their presentation failed on. That is
information about the community's policy, disclosed to the party who has the
strongest legitimate claim to it, and a community uncomfortable with that
**MAY** answer at the granularity of `code` alone.

`resendCredentials` is a third, narrow purpose: getting the applicant credentials
the community already decided to issue them, when the first delivery did not
arrive. It cannot widen the decision, because it sends only what was issued, and
only to the applicant. A replayed poll could trigger a re-delivery, which is why
this task requires `issuedAt`; a consumer SHOULD accept a poll only within a few
minutes of its `issuedAt`.

Whether an applicant is notified rather than made to poll, and how long a
community leaves a decision undelivered, are consumer policy questions on which
this specification takes no position.
