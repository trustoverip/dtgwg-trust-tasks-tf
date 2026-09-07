---
slug: rooms/epoch/mint
version: "0.1"
title: Rooms Epoch — Mint
summary: "A room's owner advances its key epoch, which is how a member is removed: the new key is distributed only to those who remain."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - epoch
  - rekey
  - removal
  - membership
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "An epoch advance changes what every member can read next; its authorization must be verifiable independently of transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An epoch advance is the mechanism of removal, and a replayed one could roll a room to a superseded membership."
sideEffects:
  level: mutating
  rationale: "Advances the room's key epoch."
subjectPath: /roomId
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The host learns the new epoch number and nothing else. Key distribution happens between the owner and the remaining members, out of the host's sight."
retention:
  class: durable
  rationale: "The epoch is part of the room's durable state and determines which records a given key can open."
errorCodes:
  - code: rooms/epoch/mint:notAuthorized
    meaning: "The presentation does not confer `admin` at this room's scope."
    retryable: false
  - code: rooms/epoch/mint:nonSequential
    meaning: "The epoch is not exactly one greater than the current one."
    retryable: false
  - code: rooms/epoch/mint:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
related:
  - rooms/create
  - rooms/records/put
---

## Abstract

The **Rooms Epoch — Mint** Trust Task advances a room's key epoch. **This is how removal
works**: the owner mints a new epoch and distributes the new key only to the members who
remain, so nothing written afterwards is reachable by the party removed.

Removal is **forward-only, and implementations should say so**. A removed member keeps
whatever they could already read — they held the plaintext. What they lose is everything
after. A member who believes removal retracts history is wrong in a way that matters, and
an interface that lets them believe it has mis-stated the guarantee.

**Only a holder of `admin` may mint an epoch.** If any key-holder could, any member could
evict any other by minting one and declining to seal the new key to them — silently, and
with no server-side check possible on a room whose membership the host cannot see. Binding
this to `admin`, which the room confers, is what makes the restriction enforceable by a
host that knows nothing about the membership.

### The epoch link, and what an advance costs without one

Minting is the only moment at which one party holds both the outgoing epoch's key and the
incoming one, so it is the only moment at which the bridge between them can be made. That
bridge is the optional `link`: the outgoing key, sealed under the incoming one.

**A room that advances without producing one loses its past.** A record is sealed under the
epoch current when it was written, and a key schedule offers no way to derive an earlier
epoch's key from a later one — the property this whole task depends on for removal to mean
anything. Run forward without a link and the first advance makes every record already in
the room unopenable by *everyone*, the writer included. That is not forward secrecy; it is
a library deleting itself.

The rungs accumulate into the room's **epoch key chain**, which a member fetches with
[`rooms/epoch/chain`](../../chain/0.1/spec.md) and walks backwards. Backwards only: a
member holding an earlier key still derives nothing later, so removal stays exactly as
forward-only as the paragraphs above describe.

A host stores rungs and cannot read them — the key that opens one is a storage key no host
ever holds. What it **MUST** do is refuse to be made a laundry for someone else's key
material: a `link` whose `epoch` is not the `epoch` being minted **MUST** be rejected, and
a rung a host already holds for an epoch **MUST NOT** be replaced. A second rung is either
a replay or a re-pointing of the room's history at key material of somebody else's
choosing, and the members who already walked the original would never see the difference.

Omitting `link` is a choice, not an oversight, and rooms are entitled to make it: a room
that is a message stream rather than a library wants exactly the behaviour the paragraph
above calls losing its past. It is optional in the schema for the additional reason that
this member was added to an already-published version, where requiring it would break every
conforming producer.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the owner, or a holder of `admin`) **MUST** present a chain
conferring `admin` at the room's scope, and **MUST** set `epoch` to exactly one greater
than the current epoch.

A conforming **consumer** (the host) **MUST** reject a non-sequential epoch, **MUST** apply
the chain rules of [`rooms/records/put`](../../../records/put/0.1/spec.md), and **MUST NOT**
accept a record sealed under a superseded epoch once the new one is in force. A host
**never learns the key** — only the number.

Where `link` is present, a conforming consumer **MUST** reject the request unless
`link.epoch` equals `epoch`, and **MUST NOT** replace a link it already holds for that
epoch. Where it is absent, a host **MUST** advance the epoch anyway: an absent link is a
room's stated choice, and a host that refused would be enforcing a policy it was never
told.

## Security & Privacy

**A host verifies chains; it does not keep a roster.** Authorization is decided entirely by
credentials the room issued. A host that consults state of its own has made the room
unmovable and has made itself part of the membership.

**Chain verification is the security of this family.** Anyone can mint a well-formed
authority credential naming any scope and any actions; what makes it worthless is that its
chain does not reach the room. A host that verifies only the credential it was handed
accepts a self-issued grant of arbitrary authority. Every link is verified, and any link
that widens actions, scope, or validity beyond its parent invalidates the chain.

**Chain depth is a denial-of-service surface.** Verification is linear in length and runs on
every operation, hence the maximum of 8. The known uses need 2 to 3.

**Parents are never dereferenced.** A producer presents every link. Resolving one over the
network would make verification depend on availability, turn an identifier into a request
the host can be induced to make against an address the *producer* chooses, and signal
credential use to whoever hosts that identifier. Identifiers in a chain are identifiers, not
locators.

**Credential pooling.** Where membership and authority are presented with the subject
withheld, a host that does not require proof that both describe the same subject lets two
parties combine one's membership with the other's authority and present as a single party
holding both.

**What a host learns anyway.** Sealing content does not hide activity. A host observes room
identifiers, owners, record counts, sizes, epochs, and timing, and — unless traffic is
routed — the network origin of whoever is acting. A room whose adversary can correlate on
those wants a different host, not a different visibility.

### Data carried

A room identifier and an epoch number. **No key material.** Distribution of the new key
happens between the owner and the remaining members, out of the host's sight.

### Correlation

A host sees that a room rekeyed and when. On `attributed` rooms it may infer a membership
change from the timing; on `private` rooms it learns only that the number advanced.

### Retention

The epoch is durable room state and determines which records a given key can open.

### Consent/purpose

Minting is restricted to holders of `admin`, so that removal is an act of the room's
accountable party rather than of any key-holder.
