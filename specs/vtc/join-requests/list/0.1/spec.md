---
slug: vtc/join-requests/list
version: "0.1"
title: VTC Join-Requests — List
summary: List a community's join requests, optionally filtered by status, newest paged.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only inventory of join requests. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the join-request registry; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The request carries only a `status` filter and paging controls — nothing about any applicant travels inbound. What comes back is the asymmetry: every matching `JoinRequest` in full, each with its applicant's `vp` and `vpClaims`."
retention:
  class: transient
  rationale: The community persists nothing from this request; it reads a page of the registry and answers. The response is a different matter — it is the one task in the family that puts many applicants' presentations into one caller's hands at once, which Security & Privacy → Retention treats as the risk it is.
errorCodes: []
---

## Abstract

The **VTC Join-Requests — List** Trust Task returns a community's join requests as [`JoinRequest`](../../../_shared/0.1/join-request.schema.json) entries, optionally filtered by `status`, paged by `cursor`/`limit`. The enumeration companion to [`vtc/join-requests/show`](../../show/0.1/).

## Conformance

Producer: optional `status` filter; `cursor`/`limit` to page.

Consumer: verify the community-admin capability. Return matching requests, clamping `limit` to 1..=200 and setting `nextCursor` when more remain.

## Security & Privacy

### Data carried

The request carries `status`, `cursor` and `limit` — a filter and two paging
controls, none of which say anything about any applicant. The response is where
this task earns its section: `items` is an array of full
[`JoinRequest`](../../../_shared/0.1/join-request.schema.json) records, each
carrying `applicantDid`, the submitted `vp`, its `vpClaims` projection, the
`policyDecision`, and any recorded `decision`.

That makes `list` categorically different from its read-one companion
[`show`](../../show/0.1/spec.md), and the difference is not one of degree. `show`
discloses one applicant's claims to a caller who named that applicant. `list`
discloses up to two hundred applicants' claims per page to a caller who named
none of them, and `nextCursor` walks the rest. A community's entire applicant
population — including everyone it refused — is reachable by iterating a call
whose request payload mentions no person at all.

Minimisation on this task is the *consumer's*, because the producer has nothing
to trim. Two levers exist and both are worth taking: clamp `limit` (the schema
already caps it at 200, which is a ceiling and not a recommendation), and answer
with the narrowest projection the caller's purpose needs. A console rendering a
queue of pending applications needs `id`, `applicantDid`, `submittedAt` and
`status`; it does not need `vp`, and returning it because the shared schema
permits it is a choice rather than an obligation.

### Correlation

Bulk enumeration is itself the correlation risk here, and it does not require
reading a single `vp`. `applicantDid` plus `submittedAt` across a full walk of
the registry yields the community's membership funnel: who applied, in what
order, how many were refused and when. Filtering on `status: "rejected"` returns
precisely the set of parties a community turned away — a list that exists nowhere
else and that no applicant consented to being enumerable in.

Read the `vp` bodies as well and the joins widen past this community. Credential
identifiers and issuer DIDs inside the presentations are stable across the
ecosystem, so a full listing can be aligned against another community's listing
to find the applicants common to both, regardless of whether either applicant
used a per-community DID.

The administrator declares `identifierScope: pairwise`: the identity a bulk read
is attributed to is meaningful within this community, and nothing in this task
asks anyone outside it to recognise that identity. Keeping it community-scoped
means an operator's enumeration pattern here is not joinable, by identifier, to
their activity elsewhere.

### Retention

The request is transient — a page is read and nothing is written. The durable
retention was incurred at [`submit`](../../submit/0.2/spec.md); this task is the
bulk interface onto it.

What deserves stating is what a single response becomes once it leaves the
community. One `list` call produces a document containing many people's
credential claims, and such documents are exactly the ones that end up cached in
a browser console, exported to a spreadsheet, or attached to a support thread —
copies over which the community's own disposal policy has no reach. A consumer
**SHOULD NOT** persist responses to this task, and where an operator needs a
durable working set it **SHOULD** be built from the projection its purpose
requires rather than from the whole page. Requests that must be kept for
audit **SHOULD** record the filter and the caller, not the returned bodies.

### Consent/purpose

The purpose is queue management: an administrator enumerates what is waiting on
the community so that it can be worked. That purpose is well served by
`status: "pending"`, and it is the reason the filter exists.

The other filter values are where purpose and capability part company. Nothing in
this specification bounds a listing to the requests a caller has business with,
and `status: "rejected"` in particular answers a question no applicant submitted
in order to have answered — they disclosed credentials to be considered for
membership, not to be enumerable afterwards as a refused party. A community that
routinely walks its rejected set is putting the material to a purpose beyond the
adjudication it was given for, and the payload will look identical either way.

Whether such a read requires a case reference, a second operator, or a narrower
capability than the community-admin gate is a consumer policy question; per
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification
describes the exposure and takes no position on the gate.
