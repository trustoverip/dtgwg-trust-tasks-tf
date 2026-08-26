---
slug: audit/list
version: "0.1"
title: Audit — List
summary: Page through a maintainer's append-only audit log, newest first, with optional filters and an opaque continuation cursor.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - audit
  - list
  - pagination
  - oversight
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: auditor
    requirement: REQUIRED
    member: issuer
  - role: audit maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The response discloses the audit tail, which is confidential material the caller retains, so both halves of the exchange must be attributable and tamper-evident. On the request side a proof binds who asked for the tail; on the response side it lets the returned records be relied on as evidence by a party that was not present at the read. An audit log that cannot be attributed to the consumer that produced it is not audit evidence, and an unproven request leaves no record of who obtained it.
sideEffects:
  level: none
  rationale: "Reads the audit log; persists nothing."
subjectPath: /contextId
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Audit envelopes carry plaintext principal DIDs and full event payloads — the
    maintainer's operations record, and the tightest-gated read it offers.
    Inbound, the request is a query about people: `actor` is the DID of a
    principal whose activity the auditor wants isolated, and `contextId` narrows
    it to one part of that principal's life. Naming a person in order to ask what
    they did is personal data travelling toward the recipient, and it is retained
    for as long as the maintainer logs its own reads.
retention:
  class: durable
  rationale: >-
    The maintainer persists nothing from the request — the read is
    `sideEffects: none` — so the class is set by the surviving half. The auditor
    keeps the returned tail because it is evidence: entries are hash-chained
    (`prevHash`/`entryHash`) and the response is signed precisely so a party that
    was not present at the read can rely on it later. An auditor that discarded it
    would be unable to show what the log said at the moment it was read, which is
    the question a disputed record turns on once the maintainer's own copy is
    contested.
errorCodes:
  - code: audit/list:invalidCursor
    meaning: The supplied cursor failed to verify — malformed, or minted before an audit-key rotation. The consumer restarts from the first page rather than retrying.
    retryable: false
---

## Abstract

The **Audit — List** Trust Task pages through a maintainer's append-only audit log, returning [`AuditEnvelope`](../../_shared/0.1/audit.schema.json) entries **newest first**. It is the enumeration companion to [`audit/verify`](../../verify/0.1/): verify says whether the chain is intact, list shows what is in it. Optional filters (`from`/`to`, `action`, `actor`, `outcome`, `contextId`) narrow the result; an opaque `cursor` continues a previous page.

## Paging

Paging is by **opaque continuation cursor**, not offset. The response returns `cursor` when `truncated` is true; the consumer passes it back verbatim to fetch the next page. The cursor encodes the resume position — typically `(recordedAt, eventId)` of the oldest entry on the page just returned — so paging is stable under concurrent appends: a new entry written at the head does not shift the pages a consumer is walking, the way an offset would.

The cursor is opaque and SHOULD be signed by the maintainer, so it cannot be forged to skip entries or read a slice the caller could not otherwise reach. Filters are bound into the cursor's position; a consumer continuing a page MUST NOT also change the filters (start a fresh query instead). A cursor that fails to verify — malformed, or minted under an audit key that has since rotated — yields `invalidCursor`, and the consumer restarts from the first page.

## Conformance

Producer: optional filters; `cursor` to continue, `pageSize` to bound the page.

Consumer: verify the audit-read capability. Return matching entries newest-first as `AuditEnvelope`s, populating every field the maintainer tracks (only `eventId`/`recordedAt`/`action` are universal). Set `truncated` and return a `cursor` iff more entries match. Reject an unverifiable cursor with `invalidCursor`. A maintainer whose log is hash-chained SHOULD populate `prevHash`/`entryHash` so a consumer can spot-check links against `audit/verify`.

## Security & Privacy

### Data carried

This is the highest-sensitivity read a maintainer offers, and the classification
follows from what an [`AuditEnvelope`](../../_shared/0.1/audit.schema.json)
actually contains rather than from the fact that it is called audit.
`actor` and `target` are **plaintext principal DIDs** — who did it, and to whom.
`action`, `outcome`, `recordedAt` and `contextId` say what happened, how it
resolved, when, and in which part of the deployment. And `detail` is an open
object: `additionalProperties: true`, described in the shared schema as
event-specific payload "opaque to the framework". That last member is the one to
watch. It is the only place in this payload with no bound of any kind, its
contents are entirely maintainer-defined, and because the framework cannot see
inside it, this specification can neither enumerate what personal data it holds
nor promise that it holds none. `exposure.discloses` is `secret` rather than
`metadata` for exactly that reason: what an audit read exposes is its records,
not merely their shape. A context-scoped admin may manage their own context and
still **MUST NOT** be handed the whole-log tail; the audit-read gate is
community-wide.

The request side is smaller but not empty. `actor` and `contextId` name the
principal and the scope the auditor is interested in, and `pageSize` bounds a
page at 1000 entries. A producer **SHOULD** filter to the narrowest window that
answers its question — `from`/`to` and `contextId` cost nothing and shrink the
disclosure by orders of magnitude — because an unfiltered call is a request for
the maintainer's entire operations history and will be answered as one.

Redaction is the one erasure affordance the schema builds in. `actor` and
`target` are typed `["string", "null"]` so a right-to-be-forgotten override can
null the plaintext in place, and entries redacted that way are returned with
`actor: null`. That this is expressible at all is a consequence of where the
hash-chain commitment sits: `entryHash` is taken over the entry's *immutable*
content, so nulling a redactable principal does not break the chain that
[`audit/verify`](../../verify/0.1/spec.md) walks. Maintainers that keep a keyed
correlation hash of a redacted actor carry it in `ext` and **MUST NOT** expose it
as a queryable plaintext, which would reconstitute the identifier the redaction
removed.

### Correlation

The filters are a correlation instrument, not merely a convenience. `actor`
matches on the plaintext DID, so a single call assembles everything one principal
did across every action type in a context — a per-person dossier drawn from a
log that was written per-event. Two properties bound that. Redacted entries do
not match an `actor` filter, so erasure genuinely removes a principal from this
query surface rather than merely hiding them. And there is a deliberate
asymmetry: `target` is returned on every entry but is **not** a filter. An
auditor can enumerate what a principal *did*; enumerating everything that was
*done to* them requires walking the log, which is a materially more expensive and
more visible operation. Implementers adding a `target` filter in a later version
should understand they are removing that friction on purpose.

The cursor deserves its own care. It encodes a reachable position — typically
`(recordedAt, eventId)` — with the filters bound into it, and it is signed, which
makes it a bearer handle to that slice of the log rather than an inert offset.
A maintainer **SHOULD** bind it to the calling principal and key so a leaked
cursor is not a cross-principal read, and at minimum it **MUST NOT** be forgeable
into a position the filters did not authorize.

Reading the log is itself an auditable event. A maintainer **SHOULD** record
`audit/list` calls, at a sampled rate if volume demands, so the oversight surface
is not the one blind spot in its own trail — and implementers should notice that
doing so writes the auditor's DID, and the `actor` they were asking about, into
the same log the next auditor will read.

The **audit maintainer** declares `identifierScope: public`. An audit log is only
evidence if independent parties can agree on which log they read: a returned tail
is attributed to the maintainer that signed it, and a regulator, a second auditor
and a disputing member must each be able to recognise that identifier as the same
community's maintainer, months apart and without a prior relationship. A pairwise
identifier would give each auditor a different name for the same log and break
exactly that agreement — the response would remain verifiable and stop being
comparable. The **auditor** carries no such declaration: this task takes no
position on whether an auditor's identifier is reused elsewhere, and a deployment
that wants oversight reads to be unlinkable to the auditor's other activity is
conforming.

### Retention

Nothing is retained from the request; the read is `sideEffects: none` and the
maintainer's own log is append-only regardless of who reads it. The retention
question is about the auditor, and the answer is durable: the returned entries are
evidence, which is the whole reason the response carries a REQUIRED `proof`. What
the auditor holds is a signed, attributable snapshot of what the log said at the
moment it was read, and that is worth more than a re-read later precisely when the
maintainer's own copy is what is in dispute.

The uncomfortable consequence is that an audit read manufactures a second copy of
the log outside the maintainer's control, and every erasure affordance described
above stops at the maintainer's boundary. A principal whose `actor` DID is nulled
by a right-to-be-forgotten override is removed from the maintainer's plaintext and
not from any page an auditor pulled last quarter. Consumers **SHOULD** hold
retrieved tails for a stated, bounded period tied to the oversight purpose that
justified the read, **SHOULD** apply the maintainer's redactions to their own
copies when they learn of them, and **SHOULD NOT** treat a retrieved page as a
general-purpose dataset. The `cursor`, being a bearer handle, **SHOULD** be
discarded once a walk completes rather than retained with the results.

### Consent/purpose

The purpose is oversight: establishing what a maintainer did, on whose authority,
and with what outcome, so that governance is checkable rather than asserted. The
basis is the audit-read capability, which is community-wide by design — an
oversight surface that only ever showed a reader their own context would not be
one — and the REQUIRED `proof` on the request is what makes each exercise of that
capability attributable to a named auditor rather than to a session.

The limit is on reuse, and it is the limit that matters most here, because
technically the data is already lawfully held. The principals recorded in these
entries were not asked; they appear because they acted, and the log exists so
their actions can be reviewed. Using the same records for anything else — building
a behavioural profile of a member, feeding a scoring model, or answering a
question the oversight mandate does not cover — is outside the purpose that
justified the disclosure, however routine the query looks. Per
[SPEC.md §7.3](/SPEC.md#73-specification-requirements) item 13 this specification
records what the read is *for* and declines to say what gate a deployment must put
in front of it; that choice, including whether an actor-filtered query about a
named individual warrants more authority than an unfiltered page, belongs to the
consumer.
