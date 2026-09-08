---
slug: persona/claim-types/list
version: "1.0"
title: Persona Claim Types — List
summary: Serve the claim-type registry an agent resolves against, so a client reads the table rather than shipping a copy of it.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, privacy, correlation]
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
  rationale: The response describes the agent's vocabulary rather than the holder's data, but what a client learns here decides what it masks and what it gates, and the audit trail should be able to name who read it.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A capability read has no durable effect and no ordering hazard.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
errorCodes: []
---

## Abstract

**Persona Claim Types — List** serves the core claim-type registry
([CLAIM-TYPES.md](/specs/persona/_shared/0.1/CLAIM-TYPES.md)) that an agent
resolves against: the registered tokens, their per-axis defaults, the
unregistered floor, and the strictness ordering that makes resolution
computable.

It exists because **the alternative is a copy in every client**, and a copy is a
re-sync obligation on every change. Both of the first two implementations
vendored the table; between them they found four holes in it, and each fix
became a pull request against two repositories that do not own the data. A
client that can read the table learns tokens its build predates, and a
maintainer that tightens the registry protects clients that were not rebuilt.

`CLAIM-TYPES.md` §6 records this as an open question, to be answered "when the
first extension type ships, not before". The reasoning has not survived contact
with the copies: the cost being deferred is paid per change, not once.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/claim-types/list/1.0` with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** serve the registry it actually resolves
against. A table served here that differs from the one the agent applies is
worse than serving nothing: a client would mask and gate by one rule while the
agent disclosed by another, and nothing would report the disagreement.

A conforming maintainer **MUST** include `unregistered` and `strictness`, and
**MUST NOT** omit them on the grounds that they are constant. They are what let
a client compute [§4](/specs/persona/_shared/0.1/CLAIM-TYPES.md) rule 3 — take
the longest registered prefix and the floor, keep whichever is more protective —
without inventing an ordering. A client that hard-codes either cannot be
tightened by a maintainer that raises it, which is the whole benefit of serving
the table.

A conforming maintainer **MUST NOT** vary this response by caller. It describes
the agent's vocabulary and nothing about any holder; a per-caller table would
make the registry a channel for information about who is asking.

A conforming **consumer** **MUST** treat a `mask` style it does not recognise as
`full`, never as `none`. The style is deliberately a string rather than a closed
enumeration so a maintainer may serve one this version does not list, and the
failure mode of the unknown-value branch is the difference between showing a
value and hiding it.

A conforming consumer **MUST NOT** treat this response as authority over what
*may* be disclosed. It carries defaults; a holder's own override is recorded on
the attribute and always wins ([§4](/specs/persona/_shared/0.1/CLAIM-TYPES.md)
rule 1). A consumer that resolved from this table alone would show a holder
their own decisions overruled.

## Authorization

**Any authenticated caller**, scoped or unscoped.

Neither of the usual answers fits, and the reason is worth stating because the
sibling task got it wrong. The response is a constant: it carries nothing about
the holder, nothing about any context, and nothing about stored state at all.
Requiring an unscoped holder would refuse the application that needs it most —
one inside a context, deciding how to render a preview it has just been shown.
Requiring a context would refuse the holder's own tooling, which has no context
to name and is the other caller that must resolve these defaults.

The vocabulary is not a secret. Withholding it would not protect a holder; it
would only make a client guess, and every guess resolves *less* protectively
than the table does.

## Request

Empty but for extensions. The table is small and bounded, and a filter would let
a client ask about one token at a time — which is how a client ends up caching
a partial table and resolving a prefix it never fetched.

## Response

See the payload schema; every member carries its own rationale there.

Both exact tokens and family prefixes appear in `entries`, undistinguished. Which
one an entry is follows from the token being resolved — equal to it, or
descended from it — and marking them would invite a client to walk only one kind,
which is exactly the hole that let a gated family be escaped by inventing a
member.

## Security & Privacy

### Data carried

Neither document carries personal data. The request is empty; the response
describes a vocabulary.

### Correlation

Nothing here identifies the holder. A maintainer serving an unusual set of
extension types is distinguishable from one serving only the core set, which is
a property of the deployment rather than of the person — the same weak sense in
which a renderer set is.

### Retention

The registry is stable and **MAY** be cached by a producer, keyed on
`registryVersion`. A client **SHOULD** re-read when that value changes and
**SHOULD NOT** cache across agents: two agents may serve different extension
types, and a table cached from one applied to another would resolve tokens the
second has never heard of.

### Consent/purpose

The purpose is correct local resolution — masking, sensitivity and release
defaults — before a value is shown or released. The data is about the agent's
vocabulary, not the holder, and there is no secondary use to constrain.
