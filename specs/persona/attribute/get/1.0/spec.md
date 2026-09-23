---
slug: persona/attribute/get
version: "1.0"
title: Persona Attribute — Get
summary: A holder reads one attribute they already hold, by identifier, optionally at a retained earlier version — so revealing a single value does not mean enumerating every attribute that shares its vocabulary prefix.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - attribute
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
  rationale: The response returns the holder's own identity data, and with `includeValue` it returns plaintext. A read of one fact deserves the same attribution as the write that made it — the agent must be able to name the key that asked, not the session it arrived on.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A get has no durable effect and no ordering hazard; replaying one returns the attribute as it stands, which is what a fresh request would have returned.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/attribute/get:notFound
    meaning: No attribute exists at the given identifier.
    retryable: false
  - code: persona/attribute/get:versionPurged
    meaning: The attribute exists, but the `version` asked for is no longer held — the holder purged it (`persona/attribute/purge-version`). Distinct from `notFound`, because the attribute is still there and its current version is readable.
    retryable: false
---

## Abstract

**Persona Attribute — Get** reads one attribute the holder already holds.

The family had no narrow read, and the shape of the workaround is the argument
for this task. A client wanting to reveal one value called
[`persona/attribute/list`](../../list/1.0/spec.md) with a `typePrefix` and
`includeValues`, then filtered the result to the identifier it already had. So
revealing one email address decrypted and returned *every* email address the
holder has, across the wire, into a client that wanted one of them — and the
audit trail recorded a listing of the pool rather than a decision about one
fact.

Naming the attribute is both cheaper and less exposing, and it makes the audit
row say what actually happened.

## Status of this Document

This specification is at `draft` status: its schema and prose may change.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/attribute/get/1.0`, populate `attributeId`,
and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer**:

1. **MUST** refuse a caller that is not authorized over the holder's pool, on
   the same test every other holder-scoped task in this family applies. The pool
   sits above every trust context, and naming a single attribute does not narrow
   who may read it.
2. **MUST** emit `persona/attribute/get:notFound` for an unknown identifier
   rather than an empty success. A caller that cannot tell "absent" from "empty"
   treats a typo as an attribute that says nothing.
3. **MUST** withhold `value` unless `includeValue` is true, and **MUST** withhold
   it for an attribute resolving to `sensitivity: high` unless `includeSensitive`
   is *also* true. The metadata is still returned in both cases — a withheld
   value is not a missing attribute.
4. **MUST** return the attribute as it stands when `version` is absent, and the
   named retained version when it is present. A `version` that the holder has
   purged is `persona/attribute/get:versionPurged`, never a silent fall back to
   the current one: a caller asking what it said in March must not be handed
   April's answer as though it were the same fact.
5. **MUST NOT** treat this task as a cheaper listing. One identifier, one
   attribute; a maintainer that accepted a list of them would recreate the
   enumeration this task exists to avoid, one call later.

## Request

See the payload schema; every member carries its own rationale there.

## Response

The attribute record, and the versions of it the maintainer still holds.

## Security & Privacy

### Data carried

The request carries an identifier. The response carries one attribute's
metadata, and its plaintext only when asked for — twice, for a sensitive one.

A producer **MUST NOT** place secret material in `ext`.

### Correlation

A get discloses nothing to anyone but the caller, and the caller is the holder's
own authority. It performs no correlation analysis and **MUST NOT** report one:
[`persona/correlation/analyze`](../../../correlation/analyze/1.1/spec.md) is
where "how linkable would this make me" is answered, and answering it here
would put a second, thinner opinion in a response whose job is to return a
record.

`retainedVersions` names versions, never their values. A caller that wants an
earlier value asks for it, which is a second decision and a second audit row.

### Retention

Reads only; retains nothing. What it can *see* is bounded by what the holder has
retained — [`persona/attribute/purge-version`](../../purge-version/1.0/spec.md) is
the holder's override, and a purged version is gone here too.

### Consent/purpose

No consent ceremony. This is the holder reading their own record, not a
disclosure: nothing leaves the holder's own authority, and a task that asked for
consent to read one's own attribute would teach the holder to dismiss the
prompt that matters.

A maintainer **MAY** gate `includeValue` on a re-authentication where the
attribute's `release` says so, on the same footing as any other path that puts
plaintext in a response.
