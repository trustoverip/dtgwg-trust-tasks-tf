---
slug: social-recovery/buddies/add
version: "0.1"
title: "Social Recovery — Add Buddy"
summary: "A device owner enrols a named contact as a custodial recovery buddy, so that person can later help the owner regain access to a device the owner has lost custody of."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - social-recovery
  - guardian
  - key-recovery
  - custodial-contact
  - device
parties:
  - role: device owner
    requirement: REQUIRED
    member: issuer
  - role: recovery maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Enrolling a buddy grants that person standing, if dormant, authority to
    participate in restoring access to the owner's device. An unattributed
    enrolment is one the owner could not later prove they did not make —
    exactly the failure mode that would let an attacker plant their own
    recovery buddy ahead of an intended device takeover.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed enrolment re-adds a buddy the owner has since removed,
    silently reversing a decision the owner deliberately made — the same
    concern `acl/grant`'s own `issuedAtRequirement` states for re-admitting a
    revoked subject, applied here to a guardian relationship instead of a
    role grant.
sideEffects:
  level: mutating
  rationale: >-
    Establishes a standing relationship the maintainer must honour at a
    future recovery event. Reversible via `social-recovery/buddies/remove`,
    but not inert in the interim: an enrolled buddy is live custodial
    authority from the moment this task completes, not from the moment a
    recovery is actually attempted.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    The request identifies the buddy, carries the relationship
    parameters (how many buddies are required to cooperate for a recovery
    to succeed, if the scheme is threshold-based), and carries the
    buddy's **public** Ed25519 key (`recoveryBuddyPublicKeyHex`,
    REQUIRED); the response echoes the identifier and the key record
    created. This task is deliberately NOT key-material-free — see Data
    carried — but no private key and no share of any secret is carried in
    either direction. See Correlation below for why this metadata is not
    therefore low-stakes.
errorCodes:
  - code: social-recovery/buddies/add:buddyAlreadyEnrolled
    meaning: >-
      This contact is already an enrolled recovery buddy for this device.
      Not an error in intent — the owner should query
      `social-recovery/buddies/list` first.
    retryable: false
  - code: social-recovery/buddies/add:selfEnrollment
    meaning: >-
      The named buddy identifier resolves to the owner's own identity. A
      recovery buddy must be a party distinct from the device being
      recovered, or the scheme provides no recovery path independent of the
      very access it exists to restore.
    retryable: false
related:
  - social-recovery/buddies/list
  - social-recovery/buddies/remove
  - social-recovery/status/get
---

## Abstract

The **Social Recovery — Add Buddy** Trust Task establishes a standing
custodial relationship: a device owner names another person who can later
help that owner regain access to a device if the owner loses custody of it
(lost, stolen, or destroyed hardware; a forgotten credential with no other
recovery path). It exists for the case a purely self-held backup cannot
cover — where the owner has lost *everything* they would need to restore
from a self-held bundle, and the only remaining path back in runs through
people who trust them and whom they trust in return.

🟠 GUESS, stated rather than assumed silently: this specification does not
take a position on the exact cryptographic reconstruction mechanism
(threshold secret-sharing among buddies, a simpler out-of-band
attestation-and-reissue scheme, or something else) because that is an
implementation detail the wire contract between owner and maintainer does
not need to fix. What this task fixes is the *relationship*: who is
enrolled, and that enrolling them is itself a consequential, auditable act.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)).
It targets framework version 0.6.0 and may change without a version bump
while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents a pattern a maintainer already implements and drives in
production tooling, written down so the shape stops being recoverable only
by reading an implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD
NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be
interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14)
when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy
[SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the
requirements stated here.

## Definitions

- **Recovery buddy** — a person the device owner has named as a custodial
  recovery contact. A buddy holds *dormant* authority: nothing it can
  exercise unilaterally, and nothing that activates until a recovery event
  is initiated (out of scope for this task — see `related`).
- **Enrolment key** — the buddy's Ed25519 public key, carried as
  `recoveryBuddyPublicKeyHex` (64 hex characters). It is what makes the
  enrolment verifiable later: a recovery event can check that the party
  cooperating is the party the owner named, rather than anyone who has
  since acquired the identifier. It is a PUBLIC key; no share of any
  secret is carried.
- **Scope** — the enrolment is owner-scoped, not device-scoped. The
  request carries no device identifier: the operation is gated to the
  owner of the identity being protected, and the enrolment applies to that
  owner's recovery path.

## Request

The device owner names the buddy and supplies the buddy's public key. Both
are REQUIRED — an enrolment without a key names a party nobody can later
verify. An optional `name` carries a display label for audit and UI. The
top-level schema is in [`payload.schema.json`](payload.schema.json).

- **`recoveryBuddyDid`** — REQUIRED, the `did:*` of the custodial party.
- **`recoveryBuddyPublicKeyHex`** — REQUIRED, the buddy's Ed25519 public
  key as 64 hex characters.
- **`name`** — OPTIONAL, a display label for the buddy.

### Enrolling a recovery buddy

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000401",
  "type": "https://trusttasks.org/spec/social-recovery/buddies/add/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:recovery-maintainer",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "recoveryBuddyDid": "did:example:sister",
    "recoveryBuddyPublicKeyHex": "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
    "name": "Sister"
  }
}
```

## Response

The maintainer confirms the buddy is enrolled, echoing the enrolled identifier
and the key record it created. The response shape below is normative prose:
`payload.schema.json` governs the REQUEST only and carries no response
sub-schema. Failures are `trust-task-error` documents.

### Buddy enrolled

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000402",
  "type": "https://trusttasks.org/spec/social-recovery/buddies/add/0.1#response",
  "issuer": "did:example:recovery-maintainer",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "recoveryBuddyDid": "did:example:sister",
    "keyId": "g-key-example:sister",
    "seq": 7,
    "bootstrap": true
  }
}
```

## Security & Privacy

### Data carried

The request carries the buddy's identifier and the buddy's **public** key;
the response echoes the identifier and the key record it created. No
private key, and no share of any secret, is carried in either direction —
but note that this task is NOT key-material-free, and a reviewer should
read it as carrying a public key deliberately rather than assume the
enrolment is identifier-only. Whatever reconstruction mechanism a maintainer uses to actually
perform a recovery is out of scope for this enrolment task.

### Correlation

Recovery-buddy enrolment is itself a durable, sensitive social-graph fact,
independent of any secret it may protect: it discloses, to the recovery
maintainer, *who the owner trusts enough to hold recovery authority over
their digital access* — a narrower and often more intimate circle than a
general social or sharing graph (see the sibling `sharing` family's own
Correlation section for the parallel concern about disclosure-level
patterns). A maintainer **SHOULD** treat the list of enrolled buddies for a
given owner with the same care as the `sharing` family's disclosure matrix,
and **MUST NOT** expose one owner's buddy roster to another party, including
the buddies themselves seeing each other's identities, unless the owner's
own scheme requires buddies to coordinate directly (in which case that
requirement is a deliberate design property of the maintainer's chosen
mechanism, not a default this specification assumes).

### Retention

The enrolment is retained until explicitly removed via
`social-recovery/buddies/remove`; there is no implicit expiry. A
maintainer **SHOULD** support the owner reviewing the current roster at any
time via `social-recovery/buddies/list`, since a forgotten stale guardian
(an estranged former partner, for instance) is itself a security exposure
distinct from an active one.

### Consent/purpose

The enrolment is initiated by the device owner; this specification does not
address whether, or how, the *named buddy* is notified or asked to accept
the role before it takes effect — 🔴 ASSUMPTION: a conforming
implementation likely SHOULD notify the buddy, since a person unaware they
hold recovery authority cannot meaningfully exercise or decline it, but this
proposal does not invent a notification sub-protocol unverified against the
originating implementation. Descriptive only, in any case — per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification
**MUST NOT** declare that consent, approval or a step-up is required.

### Custody scope

A recovery maintainer typically serves more than one owner, and enrolling a
buddy grants a third party custodial recovery power over an identity, so the
maintainer **MUST** establish that the requester is the owner of the
identity being protected before acting. The request carries no device
identifier to scope against — authority comes from the proof on the
request itself — so the enumeration hazard here is not a device-id oracle
but the buddy roster: a refusal **MUST NOT** reveal whether some other
owner has already enrolled the same party, for the same
distinguishing-"not yours"-from-"doesn't exist" reason given in
`vault/credentials/archive`'s Custody scope section and restated in this
cluster's `sharing/connection/update` specification.
distinguishing-"not yours"-from-"doesn't exist" reason given in
`vault/credentials/archive`'s Custody scope section and restated in this
cluster's `sharing/connection/update` specification.

## Open Questions for Registry Review

🟡 NEEDS-REVIEW: The `selfEnrollment` error code declared in this spec's frontmatter is not emitted by any code path we could find in the reference implementation — nothing compares the proposed buddy's DID to the requesting owner's own DID, even though that owner identity is already resolved and in scope at the point of the check, so an owner naming themselves as their own recovery buddy is currently accepted rather than rejected. As with `deviceNotFound` in the companion `status/get` task, we are flagging the discrepancy between the documented contract and the observed behavior for reviewer attention rather than resolving it unilaterally.
