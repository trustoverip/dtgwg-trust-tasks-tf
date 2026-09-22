---
slug: persona/profile/usage
version: "1.0"
title: Persona Profile — Usage
summary: A holder asks where one of their faces is worn now — every context and persona wearing it, with the face's reach beside the answer.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - profile
  - binding
  - audit
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
  requirement: OPTIONAL
  rationale: A read of the holder's own arrangement, returned to the holder. Attribution adds nothing an authenticated session does not already give.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: Read-only; replay returns the current state and changes nothing.
sideEffects:
  level: none
  rationale: Reads bindings. Writes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Returns which personas in which contexts wear a face — the linkage between the holder's identities, returned to the holder alone.
errorCodes: []
---

## Abstract

**Persona Profile — Usage** answers "where am I wearing this face?".

A face composed once may be worn in many contexts, by different personas. That
is the point of a pool face, and also the thing a holder loses track of. This
task returns every place it is worn now, with the `until` of each, and the
face's `reach` beside them so an allow-list and what it allows can be read
together.

It is a view of the present. What a face has done over time — composed, worn,
taken off, what it told whom — is persona/profile/timeline.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/usage/1.0` and populate
`payload.profileId`.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Treat a `profileId` that names no face — in the pool, or in `contextId` when given — as not found.
3. Return every binding that wears the face now, and none whose `until` has passed.

## Authorization

**Holder-authorized and unscoped.** The answer spans contexts: it is the map of
which of the holder's personas are one face, which no context may read.

## Request

`contextId` names the context of a context-local face.

## Response

`usage` lists the bindings; `reach` the face's allow-list.

## Security & Privacy

### Data carried

Context identifiers and persona DIDs — the linkage between the holder's
identities. No value.

### Correlation

This is the correlation map itself, which is why it is holder-only. A
maintainer **MUST NOT** return it to a context-scoped caller in any form.

### Retention

Nothing is written.

### Consent/purpose

The holder auditing their own arrangement.
