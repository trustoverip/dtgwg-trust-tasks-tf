---
slug: persona/attribute/list
version: "1.0"
title: Persona Attribute — List
summary: A holder enumerates their own attribute pool, narrowed by vocabulary prefix and paginated, with values withheld unless explicitly requested so a picker can render the pool without decrypting every fact in it.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - persona
  - attribute
  - pagination
  - data-minimisation
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
  rationale: The response enumerates the holder's identity data, so the agent must be able to attribute the request to a key rather than to the session it arrived on. A read that discloses the whole pool deserves the same attribution as the write that filled it.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A list has no durable effect and no ordering hazard; replaying one returns the pool as it stands, which is what a fresh request would have returned.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/attribute/list:cursorInvalid
    meaning: The supplied `cursor` is unrecognised, expired, or was issued against a different filter. A producer restarts the enumeration rather than attempting to repair the token.
    retryable: false
---

## Abstract

**Persona Attribute — List** enumerates the holder's attribute pool.

Two choices in it are worth the reader's attention, and both are about giving
away less by default.

**Values are withheld unless asked for.** The common case — rendering a picker
so a holder can choose what goes into a profile — needs `type` and `label`, not
plaintext. Making `includeValues` opt-in means the expensive, sensitive path is
the one a producer has to ask for, rather than the one it gets by forgetting.

**Stale attributes are returned, not hidden.** A credential-backed fact whose
backing has been revoked or has expired comes back carrying `stale` and a reason.
Omitting it would present a pool that looks smaller than it is, and the holder
would not learn that a claim they believe they can present has stopped being
presentable. A holder can suppress them with `includeStale: false`; the default
is the one that tells them the truth.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/attribute/list/1.0`, with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming producer **MUST NOT** construct or parse a `cursor`, and **MUST
NOT** infer that an enumeration is complete from a short page: only an absent
`nextCursor` means the end.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped** — see [Authorization](#authorization).
2. Return attributes in a stable order across the pages of one enumeration, so that a record is neither skipped nor repeated as the caller pages.
3. Omit `value` from every returned attribute unless `includeValues` is true.
4. Re-derive credential-backed values before returning them, and mark those it cannot as `stale` with a `staleReason` rather than returning a cached value whose backing has gone.
5. Emit `persona/attribute/list:cursorInvalid` for a token it cannot honour, rather than silently restarting the enumeration — a caller that believes it is continuing and is in fact restarting will process records twice.

## Authorization

**Holder-authorized and unscoped**, for the reasons given in
[`persona/attribute/put`](/specs/persona/attribute/put/1.0/spec.md#authorization).
This task is the one that would make an authorization defect maximally
expensive: it enumerates the pool in bulk. A caller holding a context-scoped
session **MUST** be refused whatever its role, and an administrator scoped to one
context **MUST** be as powerless here as an application in that context.

## Request

All members are optional; an empty payload enumerates the pool from the start.

`typePrefix` narrows over the vocabulary. Because tokens are dotted with the most
general segment first, `phone` selects every phone number and `name` every name,
which is the access pattern a builder has. The comparison is over bytes and the
maintainer attaches no further meaning to it.

## Response

Attributes and, when more remain, a `nextCursor`.

A page may be shorter than `limit` for reasons of the maintainer's own. The only
exhaustion signal is the absence of `nextCursor`.

## Security & Privacy

### Data carried

The request carries a filter and a pagination token — no personal data. The
**response** is the sensitive half: it enumerates the holder's identity, and with
`includeValues` it carries plaintext facts in bulk.

That asymmetry is why the default withholds values. The smallest response that
answers the common question — *what facts do I have to choose from* — is metadata
only. A producer that requests values is choosing to move them, and **SHOULD**
request them only for the attributes it is about to render or compose with.

### Correlation

The response is the holder's pool, so a party that obtains one holds the material
to correlate that holder anywhere those values are presented. This is the
strongest reason the task is holder-authorized and unscoped: the correlation risk
is not created by presenting the response to a verifier, it is created by the
response existing outside the holder's own tooling.

Within the document itself there is nothing to join on beyond transport metadata.
Successive pages of one enumeration share a `threadId` and a cursor lineage, so
an intermediary can tell that several requests are one enumeration; it learns the
size of the pool from that, and nothing about its contents.

### Retention

A response is a point-in-time view with no evidentiary value and **SHOULD NOT**
be retained. A producer that caches it for a picker **SHOULD** hold it in memory
for the composition it serves and discard it after, because a cached pool on disk
is the same disclosure as the pool, without the encryption the maintainer applied.

Maintainer-side, a cursor is retained only as long as the enumeration it serves.

### Consent/purpose

The purpose is composition and inspection: a holder looks at their own facts in
order to choose among them, or to see what has gone stale. The data is not
collected here at all — it is returned to the party it already belongs to.

Because the response is the pool, a producer **SHOULD** treat obtaining it as
equivalent to obtaining every value in it, and scope its request accordingly.
What gate a producer places in front of that is the consumer's policy and is
deliberately not specified here.
