---
slug: persona/attribute/purge-version
version: "1.0"
title: Persona Attribute — Purge Version
summary: A holder permanently removes earlier versions of an attribute that their agent kept because a profile pins them — the explicit override on retention, for a value they want gone even where a face still asks for it.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - persona
  - attribute
  - retention
  - erasure
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
  rationale: A purge destroys holder data that is still presented somewhere. Attribution must survive the transport so an audit record names the key that removed it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A purge races an attribute edit — the edit is what creates a retained version — and only an issue time lets an agent order the two.
sideEffects:
  level: destructive
  rationale: "Removes retained versions for good. Every profile pinning one presents that entry as stale afterwards; the current value and the attribute itself are untouched."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/attribute/purge-version:currentVersion
    meaning: A named version is the attribute's current one. The current value is removed with persona/attribute/delete, never here — a purge that could take the live value would make "tidy up old names" able to erase the present one.
    retryable: false
---

## Abstract

**Persona Attribute — Purge Version** permanently removes earlier versions of one
attribute.

A maintainer may keep a replaced value, and only for one reason: a profile pins
it (`{ref, pinVersion}`), because a counterparty verified that value and must
keep being shown it. After a name change the old name is therefore still held for
the bank that has not yet been told. Retention bounded by reference is the right
default — nothing is kept that nothing asks for — but it leaves one case
uncovered: the holder who wants the old value **gone**, even though a face still
pins it. A deadname is the case that makes this necessary rather than tidy.

This task is that override. It removes the named retained versions, or all of
them, and reports every profile that pinned one. Those profiles do not silently
fall back to the current value — a pin exists precisely so a counterparty is
**not** shown a value the holder did not choose for them — they present the
entry as stale, which discloses nothing for it, until the holder repins or edits
them.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/attribute/purge-version/1.0`, populate
`payload.attributeId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming producer **SHOULD** show the holder which profiles pin the versions
it is about to purge — `retainedVersions[].pinnedBy` on the attribute — before
sending, since those profiles will present less afterwards.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Refuse with `persona/attribute/purge-version:currentVersion` when a named version is the attribute's current one, and purge nothing.
3. Remove every named retained version (every retained version when `versions` is omitted) such that no copy of its value remains recoverable from the maintainer's store.
4. Report each profile that pinned a removed version in `stalePins`, and thereafter resolve those entries as stale rather than serving any other version in their place.
5. Treat a version that is not held as nothing to do: return it absent from `purged`, and succeed.

## Authorization

**Holder-authorized and unscoped**, as elsewhere in the family. A context-scoped
caller **MUST** be refused whatever its role.

## Request

`attributeId` names the attribute. `versions` names the retained versions to
remove; omitted, every retained version goes.

## Response

`purged` lists what was actually removed. `stalePins` names the profiles whose
entries now present nothing — a holder is owed the list of what their one action
changed.

## Security & Privacy

### Data carried

Identifiers and version numbers only. A maintainer **MUST NOT** echo a purged
value in the response or in an error: returning what was just destroyed would put
it into logs and transcripts that outlive the record.

### Correlation

Nothing here reaches a third party. The profiles named in `stalePins` are the
holder's own, returned to the holder.

A purge does not recall anything. A verifier that was shown a purged value keeps
what it was shown; persona/disclosure/history continues to say so.

### Retention

The purged versions are destroyed, together with any index entry derived from
their values — a keyed-hash correlation entry for a value no longer held would
report a linkage that no longer exists.

### Consent/purpose

The purpose is erasure the holder chooses over retention the maintainer would
otherwise keep. It is deliberately separate from persona/attribute/put: keeping
an old value while a counterparty needs it is the correct default for a name
change, and removing it anyway is a decision the holder should make on its own
rather than as a side effect of an edit.
