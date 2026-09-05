---
slug: persona/contact/put
version: "1.0"
title: Persona Contact — Put
summary: A holder's agent files what a peer disclosed as a new revision, never overwriting, and reports which claims changed — so a silently altered payment address becomes a visible event.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, contact, revision, phishing-resistance]
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
  rationale: A contact record is evidence of what a counterparty asserted and when. Attribution must survive the transport so a later dispute about what was disclosed has an answer that names the key that filed it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Revisions are ordered, and two filings applied out of order would make an older disclosure the current one — which is exactly the substitution the revision history exists to expose.
sideEffects:
  level: mutating
  rationale: "Appends a revision. Never overwrites, so no prior state is lost."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: The request carries another party's personal data — the claims they disclosed — into the holder's store. The response returns identifiers, a revision number, and the list of claim TYPES that changed; it does not echo values.
errorCodes:
  - code: persona/contact/put:documentTooLarge
    meaning: The document exceeds the maintainer's stated per-record cap. The cap and the actual size are both returned, so a producer can tell how far over it is rather than guessing.
    retryable: false
---

## Abstract

**Persona Contact — Put** files what a peer disclosed.

Two properties are not negotiable.

**Revisioned, never overwritten.** When a peer re-discloses, the previous
document is kept and the new one becomes current. An address book that silently
replaces a payment address is a phishing surface; one that reports *"this changed
four minutes ago, here is what it was"* is a defence. The `changedClaims` member
is what turns the history from an archive into that defence.

**A contact belongs to a relationship, not to the holder at large.** The same
peer met through two personas is two contacts. Collapsing them would correlate
the holder's own personas inside their own address book — the exact linkage the
rest of this family works to prevent, reintroduced at the one place nobody would
think to look.

The claim set a peer discloses is **structurally the same** as one a holder
composes: a profile and a contact card are one schema seen from two sides. That
is why one validator serves both.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/put/1.0`, populate `contextId`,
`subjectDid`, `knownByPersona` and `document`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Confine the caller to its own context.
2. Append a revision rather than replacing one, and assign `rev` itself — a recipient counts what it received, which is the only sequence it can vouch for. The publisher's `cardVersion` is advisory and **MUST NOT** be used to order revisions.
3. Compute `changedClaims` against the immediately previous revision.
4. Key the contact on `(contextId, subjectDid, knownByPersona)` and **MUST NOT** merge contacts that differ only in `knownByPersona`.
5. **MUST NOT** treat a claimed `credentialBacked` provenance as verified. It is a statement about what the publisher says backs a claim; verification is a separate act against the credential.

## Authorization

**Context-scoped**, confined to the caller's own context. Filing a contact writes
into the context the relationship lives in, and needs nothing from the pool.

## Request

`document` is the disclosed claim set. `credentialRefs` point into the credential
vault, where any credentials the peer presented belong — the contact holds
references, because two stores for credentials is the mistake this family's own
existence argues against.

`notes` is the holder's private annotation and is never disclosed to anyone,
including the contact it is about.

## Response

The contact identifier, the assigned revision, and — on an update — which claim
**types** changed.

`changedClaims` names types and not values. A producer that needs the old value
reads the prior revision, which is an explicit act.

## Security & Privacy

### Data carried

The request carries **another party's personal data**: the claims they chose to
disclose, and any private note the holder keeps about them. That note is the
holder's own words about a third party and is the most sensitive member here,
because the subject has no visibility of it and never consented to it.

A producer **MUST NOT** place secret material in `notes` or `ext`.

### Correlation

`subjectDid` is normally pairwise, so the record names *this relationship* rather
than the person across all of theirs. A maintainer **MUST NOT** index contacts by
subject DID across contexts or personas, because doing so would build exactly the
cross-persona linkage the pairwise identifier exists to prevent — and would build
it about the holder's counterparties, who did not choose it.

The revision history is itself a correlation surface: it records when a
counterparty changed what they present, which over time is a behavioural profile
of someone who is not the holder.

### Retention

The current revision is retained until the contact is deleted. **Superseded
revisions are reference-counted rather than reaped on a flat timer**: a revision
still pointed at by a disclosure record, a pinned entry or an unresolved diff is
evidence the holder can still be asked to account for, and a flat TTL would delete
it precisely when it mattered. Absent such a reference, a maintainer **SHOULD**
reap a superseded revision after a stated period and **MUST** state the period.

### Consent/purpose

The purpose is memory: the holder keeps what a counterparty told them, so that a
later interaction can be checked against an earlier one. The data was disclosed
to the holder deliberately by its subject, and retaining it is what makes the
disclosure useful.

The limit is that it was disclosed *to this persona*. Reusing a contact's data
under another of the holder's personas would present the counterparty with an
identity they never disclosed to, and a maintainer **MUST NOT** do so implicitly.
What further gate a producer applies is the consumer's policy and is deliberately
not specified here.
