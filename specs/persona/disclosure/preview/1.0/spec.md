---
slug: persona/disclosure/preview
version: "1.0"
title: Persona Disclosure — Preview
summary: Determine exactly what a disclosure would reveal, to whom, at what proof strength and through which renderer — signing nothing and sending nothing, so that no disclosure can occur without first producing the summary a human can be shown.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, disclosure, selective-disclosure, zero-knowledge, correlation]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Application
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A preview returns the holder's values and names a verifier. The agent must attribute the request to a key, and the audit trail must be able to say which application proposed a disclosure even when none followed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A preview is single-use and short-lived, and both properties depend on the agent being able to place the request in time.
sideEffects:
  level: mutating
  rationale: "Signs nothing and sends nothing, but records a short-lived previewId that present consumes. That state is what makes the two calls uncollapsible, so it cannot be classified as read-only."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/disclosure/preview:notBound
    meaning: The persona has no profile bound, so there is nothing to disclose. A normal condition rather than a fault — a persona need not have a profile.
    retryable: false
  - code: persona/disclosure/preview:rendererUnavailable
    meaning: The requested renderer is not offered by this maintainer. The available renderers are enumerable, so a producer discovers rather than guesses.
    retryable: false
  - code: persona/disclosure/preview:rendererCannotCarry
    meaning: >-
      The requested renderer cannot represent a claim in the disclosure — most
      commonly a predicate, which has no value to render. Failing here at format
      negotiation is deliberate; silently dropping the claim would produce a
      disclosure that verifies and says less than the holder approved.
    retryable: false
---

## Abstract

**Persona Disclosure — Preview** answers *what would this reveal, and to whom*,
and it signs and sends nothing.

It is the first of **two calls that cannot be collapsed**. The `previewId` it
returns is what
[`persona/disclosure/present`](/specs/persona/disclosure/present/1.0/spec.md)
consumes, so there is no path to a disclosure that did not first produce the
summary a human can be shown. The ordering is structural rather than a flag
someone can forget.

Three things the preview reports are worth stating up front.

**The proof rung per claim.** A credential-backed claim can be proven at several
strengths, and the difference between the strongest two and the weakest two is of
kind rather than degree: only a predicate or a derived proof avoids handing two
verifiers a join key. The preview says which applies, because *"you will prove
you are over the threshold without disclosing your date of birth, and this proof
cannot be linked to any other site you have used it at"* is a materially
different offer from *"this verifier will receive your entire licence"*.

**What the renderer drops.** Output formats differ in what they can carry, and
several cannot carry provenance at all. Lossiness is **declared, not
discovered**: a holder is owed *"this verifier will see your work number but not
that your employer attested it"* before deciding.

**What is anomalous for the stated purpose.** A preview listing fourteen fields
with fourteen equal weights is a notice-and-consent dialog, and notice-and-consent
is the pattern that trains people to click through. Ranking by what is out of
place for the purpose is what makes the line that gets read the line worth
reading.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/disclosure/preview/1.0`, populate
`contextId`, `personaDid` and `verifierDid`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Confine the caller to its own context.
2. Sign nothing and transmit nothing to the verifier.
3. Select, for each credential-backed claim, **the highest proof rung the credential's format supports**, and report it.
4. **MUST NOT** silently select a lower rung than a producer requested. A request that cannot be met at the requested rung is refused, because a silent privacy downgrade discloses material the holder believed was hidden.
5. Refuse with `persona/disclosure/preview:rendererCannotCarry` when the chosen renderer cannot represent a claim, rather than dropping it.
6. Mark any claim it could not re-derive as `stale`, and **MUST NOT** include a stale claim in the eventual disclosure.
7. Return a `previewId` that is **single-use** and short-lived.
8. Honour `requestedClaims` as a ceiling: when present, **MUST NOT** report a claim outside it.

## Authorization

**Context-scoped**, confined to the caller's own context.

This task and [`persona/binding/get`](/specs/persona/binding/get/1.0/spec.md) are
the only two an application inside a context may call against the holder's
identity, and this is the only one through which values reach it — after a human
decision. Being inside a context confers no privilege over identity data: an
application is a verifier, and takes the same path as a stranger's web page.

## Request

`verifierDid` is required rather than optional, because half of what a preview
says is *who is asking*. A preview that could not name its recipient would be a
list of fields rather than a decision.

## Response

The claims, their rungs, the renderer's declared lossiness, a correlation
assessment, and what is anomalous for the purpose.

A **predicate** claim carries no `value`, and that absence is the point. The
underlying attribute stays in the pool; the predicate is a disclosure-time
projection over it, which keeps the pool from filling with derived facts that are
all the same fact asked differently.

## Security & Privacy

### Data carried

The request carries identifiers and a purpose string. The **response carries the
holder's values** — the ones that would be disclosed — plus the verifier's
identity and the holder's correlation exposure.

A preview response is therefore as sensitive as the disclosure it describes,
minus the signature. A producer **MUST NOT** persist one, and **MUST NOT** treat
having obtained one as equivalent to the holder having approved it.

### Correlation

The correlation assessment is the response's most useful member and the one most
easily got backwards. **A credential presented whole correlates more than a
self-asserted value**, because the issuer's signature is identical at every
verifier; a derived proof correlates **less**, because it differs on every
presentation. Severity is a function of value *and* rung together, and a
maintainer that scored on provenance alone would push holders away from the safer
option.

The disclosure subject is **pairwise by default** — a per-relationship identifier
rather than the persona DID — so two verifiers cannot recognise the holder as one
party. The persona DID is the account; the subject is the face.

### Retention

A preview is transient by design: single-use and short-lived. A preview a holder
approved an hour ago is not evidence they approve it now, and one that could be
replayed would let a second disclosure ride an earlier decision.

A maintainer **SHOULD** record that a preview was requested even when no
disclosure followed, because a pattern of previews a holder declined is itself
something they may want to see.

### Consent/purpose

The purpose is decision support: to put in front of a human what a disclosure
would cost them, before it happens. Everything in the response exists to make one
decision better informed.

This specification describes what a preview reports and **does not** require that
a human be asked. Whether a producer prompts, and how, is the consumer's policy —
but the two-call split exists so that the option is always available, and a
producer that calls both in immediate succession has still produced the summary
it chose not to show.
