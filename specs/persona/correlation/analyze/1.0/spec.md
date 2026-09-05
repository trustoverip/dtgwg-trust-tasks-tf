---
slug: persona/correlation/analyze
version: "1.0"
title: Persona Correlation — Analyze
summary: Report where the holder's identities link to one another, why, and what can be done about each link — including the remedy a holder would not otherwise think of.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, privacy, correlation]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The response maps the holder's personas to one another and to the verifiers that have seen them, which is the linkage the rest of the family works to prevent anyone else from assembling.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: An analysis has no durable effect; a replay returns the assessment as it stands.
sideEffects:
  level: none
  rationale: Reads only. A supplied `candidate` is analysed and MUST NOT be stored.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: A `candidate` carries a personal value the holder is considering, so it may be analysed before it is written. The response returns the holder's own profile, context and verifier identifiers — the linkage map — and no values.
errorCodes: []
---

## Abstract

**Persona Correlation — Analyze** is the findings task behind the counts the
write tasks return.

Multiple personas exist to be unlinkable, and a composition tool is a machine for
accidentally linking them: the same value in two profiles correlates the personas
presenting them, permanently, for anyone who sees both. The holder will not notice
while composing, which is why this is a first-class output rather than a lint.

Three things distinguish it from a warning.

**It analyses candidates.** A value the holder is *considering* can be assessed
before it is written, which is the difference between a guard and a report.

**It returns identifiers, not counts.** The write tasks deliberately return only
counts, because a count is enough to warn and identifiers would disclose the
holder's other compositions to whatever tool made the write. This task is
holder-authorized and its purpose is to let the holder *act*, which they cannot do
on a number.

**It names remedies, including the non-obvious one.** `reissueCredentialToThisDid`
matters more than it looks: without it, a holder told *this links your personas*
has no action available but to abandon the attribute, and the honest fix — a
credential re-issued against the persona actually using it — stays invisible
unless the analysis names it.

One correction the analysis must encode: **a credential presented whole correlates
more than a self-asserted value**, because the issuer signature is identical at
every verifier, while a derived proof correlates less because it differs every
time. Severity is a function of value and proof rung together. An analysis that
scored on provenance alone would push holders away from the safer option.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/correlation/analyze/1.0` with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** reject the document unless the caller is
**holder-authorized and unscoped**; **MUST NOT** store a supplied `candidate`;
**MUST** compute matches over a keyed hash rather than a plaintext index; and
**MUST** score severity from the value and the proof rung together rather than
from provenance alone.

A conforming maintainer **MUST NOT** treat a per-profile override as reducing
severity. Changing a displayed value does nothing about the credential beneath it,
and an analysis that scored on the displayed value would report a false all-clear
— which is worse than reporting nothing, because the holder would act on it.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role. The response is the linkage map between the holder's
identities, which is precisely the artifact the family exists to keep from being
assembled by anyone else.

## Request

See the payload schema; every member carries its own rationale there.

## Response

See the payload schema.

## Security & Privacy

### Data carried

A `candidate` carries a personal value the holder has not yet written; it is
analysed and **MUST NOT** be stored. The response carries the holder's own profile,
context, persona and verifier identifiers — no values.

### Correlation

This task computes correlation and therefore concentrates it: its response is the
map that says which of the holder's identities share what and who has seen it.

The computation itself is designed not to enlarge the risk it measures. Matching
is over a **keyed hash** of the value, so the maintainer can answer *does this
value appear elsewhere* without holding a plaintext index of the holder's personal
data. Exact match is the only comparison correlation requires; prefix and fuzzy
search are outside the family by construction, and that trade is deliberate.

### Retention

An analysis is a point-in-time assessment with no evidentiary value and **SHOULD
NOT** be retained. Candidates are never stored.

### Consent/purpose

The purpose is informed choice at the moment of composition: to tell a holder what
a value would link, before they commit to it, and what they could do instead.

A maintainer **MUST NOT** refuse a write on the strength of an analysis. The
holder decides; a store that substituted its judgment would be answering a
question only the holder can answer. What a producer does with a `high` severity
is the consumer's policy and is deliberately not specified here.
