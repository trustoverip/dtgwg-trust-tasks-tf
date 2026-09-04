---
slug: rooms/keys/key-package
version: "0.1"
title: Rooms Keys — Key-Package
summary: "An invited party asks the agent that will hold its room keys to mint an MLS KeyPackage, so the room's owner has something to add."
status: draft
targetFrameworkVersion: "0.5"
category: ai-agents
keywords:
  - room
  - mls
  - keypackage
  - membership
  - unlinkability
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The mint retains a private key on the recipient's behalf; a request whose origin depended on the transport would let a compromised channel fill a key-holder with unusable key material."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed request mints a second KeyPackage and retains a second private half, which is storage a caller never asked for."
sideEffects:
  level: mutating
  rationale: "Mints an MLS KeyPackage and retains its private half against a Welcome that may never arrive."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The KeyPackage carries a signature key, an HPKE init key and the leaf's credential. It opens nothing; what it discloses is that this party is preparing to join something."
retention:
  class: durable
  rationale: "The private half must outlive the request — it is what processes the Welcome — but only until the KeyPackage is used or expires."
errorCodes:
  - code: rooms/keys/key-package:notInvited
    meaning: "The recipient requires an invitation to mint for this room and holds none."
    retryable: false
related:
  - rooms/keys/welcome
  - rooms/keys/commit
---

## Abstract

The **Rooms Keys — Key-Package** Trust Task mints the MLS KeyPackage a room's owner needs in
order to add a member to the group.

It is the first of the three steps that put a group into a key-holding agent — mint, be
[welcomed](../../welcome/0.1/spec.md), then keep applying [commits](../../commit/0.1/spec.md)
— and the only one the joining side initiates.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Behaviour

### One KeyPackage per room, never reused across rooms

A KeyPackage is a stable, public identifier for whoever presents it. The same one offered to
two rooms tells anyone who sees both that one party is in both — which is precisely the
linkage a `private` room exists to deny, arriving through the door rather than the wall.

So `roomId` is required and a recipient **MUST** mint per room. Reuse is not an optimisation
worth having: a KeyPackage is small, minting is cheap, and correlation is forever.

### The private half is retained, so the mint is not free

The recipient keeps the private key that will process the Welcome. A caller that mints and
never joins has left key material behind, and a caller that mints repeatedly has left a pile
of it.

Two consequences, both **SHOULD** rather than **MUST** because the right bound is
deployment-specific: a recipient should bound how long it retains an unused KeyPackage —
`expiresAt` says what it chose — and should require an invitation before minting at all,
answering `notInvited` otherwise. A key-holder that mints for any room on any request is a
key-holder anyone can fill.

### Single use

MLS consumes a KeyPackage on add. A recipient **MUST NOT** offer the same one twice; a second
request mints a second package.

## Security & Privacy

**Nothing here opens anything.** A KeyPackage is public by construction — signature key, HPKE
init key, credential. It is safe to hand to the owner, and safe for the owner's transport to
see. What it is not safe to do is *reuse*, which is a privacy property rather than a
confidentiality one and so is easy to get wrong while everything appears to work.

**Minting is not joining.** A party that mints has consented to nothing. Consent is the
invitation, and it becomes load-bearing at the Welcome. An implementation that treated a mint
as acceptance would have collapsed the two-party act the room family is built on into a
one-party one.

### Data carried

A room identifier and, where required, an invitation credential. The response carries a
public KeyPackage and an expiry.

### Correlation

A KeyPackage is a correlator by nature: it is a stable value that identifies its holder to
anyone who sees it twice. Per-room minting confines that to one room, which is the most this
task can do — the rest is the owner's transport, and on a `private` room that is why the
Welcome path avoids the host.

### Retention

The private half is retained until the KeyPackage is used or expires, and **MUST** be
discarded at whichever comes first. Retaining it past expiry keeps a key alive for a join
that will never happen.
