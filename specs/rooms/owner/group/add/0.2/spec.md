---
slug: rooms/owner/group/add
version: "0.2"
title: "Rooms Owner — Group — Add"
summary: "A room's owner asks their own agent to admit a member to the room's MLS group: add their KeyPackage, advance the room's epoch at its host, and hand back the Welcome to deliver."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords: [rooms, owner, group, mls, sealed, admit, welcome, commit]
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The recipient admits a party to every record the room seals from now on, and advances the room at its host in the owner's name. A request whose origin depended on the transport would let a compromised channel add a reader to a room nobody else can audit."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An admission advances the room's epoch. A replay must be placeable in time so it is not taken for a second admission."
sideEffects:
  level: mutating
  rationale: "The recipient advances its group, and makes an outbound request that advances the room's epoch at its host."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The recipient speaks to the host as its principal, presenting the owner's `admin` authority, and discloses to it a new epoch number, an epoch link and a commit — all opaque to a party without the group. The response carries a Welcome, which is sealed to the joiner's KeyPackage and opens nothing for anyone else; a repeated request (item 8) is answered with the same Welcome, which discloses nothing the first answer did not."
retention:
  class: durable
  rationale: "The advanced group state persists, as it must for the owner to keep sealing and opening the room. The Welcome of the last admission is kept, with its member and KeyPackageRef, only until the recipient's next advance of the room or 24 hours, whichever is sooner, so that a repeated request can be answered (item 8)."
errorCodes:
  - code: rooms/owner/group/add:noGroup
    meaning: "The recipient holds no group state for this room. Form one with rooms/owner/group/create, or join by a Welcome."
    retryable: false
  - code: rooms/owner/group/add:keyPackageInvalid
    meaning: "The KeyPackage did not validate, is for another room's ciphersuite, or its leaf does not identify `member`."
    retryable: false
  - code: rooms/owner/group/add:alreadyMember
    meaning: "`member` already has a leaf in the group, and the request is not a repeat of the admission that gave it to them (item 8): another KeyPackage, or an admission whose Welcome is no longer kept."
    retryable: false
  - code: rooms/owner/group/add:noAuthority
    meaning: "The recipient holds no authority credential conferring `admin` on this room for its caller, so it cannot advance the room's epoch at the host."
    retryable: false
  - code: rooms/owner/group/add:epochConflict
    meaning: "The host is at a different epoch than the recipient's group — another admin advanced the room. Catch up (rooms/epoch/commits) and try again. The host's epoch is carried in `details.hostEpoch`."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [hostEpoch]
      properties:
        hostEpoch:
          type: integer
          minimum: 1
  - code: rooms/owner/group/add:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer. The outcome is unknown — the host may have accepted the advance — so the recipient keeps the pending admission and the next request to it settles it (item 6). Repeating the identical request is safe: if the host accepted, the repeat is answered with the admission's Welcome (item 8)."
    retryable: true
  - code: rooms/owner/group/add:hostRefused
    meaning: "The host answered and declined the epoch advance. Its reason is carried in `details`. Nothing was changed."
    retryable: false
related:
  - rooms/owner/group/create
  - rooms/owner/group/remove
  - rooms/owner/group/renew
  - rooms/owner/invite
  - rooms/keys/key-package
  - rooms/keys/welcome
  - rooms/keys/commit
  - rooms/epoch/mint
  - rooms/epoch/commits
---

## Abstract

A room's owner asks their **key holder** to admit a member to the room's MLS group.

Admitting someone to a sealed room is three things that have to happen together: the group gains a leaf for them, the room's epoch advances at its host so records from now on are sealed to the new group, and the newcomer receives the Welcome that lets their own key holder join. The first and third are the key holder's — it holds the group — and the second needs the owner's `admin` authority, which the key holder also holds. So the key holder does all of it, for the same reason [`rooms/owner/register`](../../../register/0.1/spec.md) exists: the surfaces owners act from reach their own agent and no third party.

What comes back is the **Welcome**, for the caller to deliver to the newcomer's key holder with [`rooms/keys/welcome`](../../../../keys/welcome/0.1/spec.md). It is returned rather than delivered because on a `private` room it **MUST NOT** pass through the host, and the recipient has no other channel to a stranger's agent.

The newcomer must already hold an invitation from the room ([`rooms/owner/invite`](../../../invite/0.1/spec.md)): that is what authorized their KeyPackage, and their key holder will refuse the Welcome without it.

*This paragraph is non-normative.* The advance moves the room's profile too: the profile is stored per epoch, so a key holder that also serves [`rooms/keys/profile/write`](../../../../keys/profile/write/0.1/spec.md) reads it before advancing and re-writes it under the new epoch afterwards, which is what lets a member who joins now — or reads only from where they joined — see what the room is. See that specification's *Keeping the profile readable across epochs*.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`0.1` made the task safe across a lost reply in one direction only. Item 6 settles a pending
admission at the next request, so the room and the owner's group never disagree — but the
next request is, in the case that matters, **the same request again**: the caller lost the
answer and retries. Under `0.1` the recipient settled the pending admission, found `member`
already in the group, and refused the retry `alreadyMember`. The Welcome — the only thing the
caller needed, and the only way the newcomer can join — was never returned to anyone. The
member was in the group with no way in, and the owner's only remedy was to remove them and
start again. [`vtc/rooms/offers/accept`](../../../../../vtc/rooms/offers/accept/0.1/spec.md),
which relies on this task, already promised its own callers that retrying is safe; under
`0.1` it was not. The same loss happens when the recipient applied the admission and its
answer was lost on the way back, with no pending admission left to settle.

`0.2`:

- **Answers a repeat with the admission's Welcome** (item 8, *Repeating an admission*). The
  recipient keeps the Welcome of the last admission it applied for a room, with its member
  and the KeyPackage's `KeyPackageRef`, until its next advance of the room or 24 hours,
  whichever is sooner. A request for the same `member` with a KeyPackage of the same
  `KeyPackageRef` is answered with that admission's epoch, Welcome and tree, changing nothing.
- **Item 6 hands over to item 8** when the pending admission it applies is the one the request
  repeats.
- Narrows the meaning of `alreadyMember` to what is not a repeat. No error code is added; the
  request and response schemas are unchanged.

A request that `0.1` refused `alreadyMember` is answered here with a Welcome: a change to what a
conforming recipient returns, so it is a new version per
[SPEC §5.2](/SPEC.md#52-compatibility-rules) rather than an in-place edit to `0.1` — a `MINOR`
increment, as it is backwards-compatible for every caller that does not depend on the refusal.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noGroup` when it holds no group for `roomId`, and with `keyPackageInvalid` when the KeyPackage does not validate or its leaf credential does not identify `member`.
2. **MUST** build the commit adding `member`, and the epoch link that seals the outgoing epoch's storage key under the incoming one, **without** applying the commit to the group it holds. (A committer cannot process its own commit later, so what it keeps is the advanced group state alongside the one it holds, not only the commit.)
3. **MUST** retain that pending admission — the advanced state, the link, the commit and the Welcome — durably **before** sending anything to the host. A recipient holds at most one pending advance per room, whether an admission, a removal or a renewal, and settles any it finds (item 6) before building another.
4. **MUST** advance the room at `host` with [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md), carrying the new epoch, the link, the commit and a presentation of the caller's `admin` authority — minted as [`rooms/keys/present`](../../../../keys/present/0.2/spec.md) would mint it — in a document addressed to `host` whose proof covers its `recipient` ([SPEC §4.8.2](/SPEC.md#482-audience-binding)). The presentation is bound to the room, not to a host; the document's `recipient` is what binds the advance to `host`.
5. **MUST** apply the pending admission only once the host has accepted the advance, and **MUST** discard it, leaving the group unchanged, when the host **refuses** it. When the outcome is unknown — the host could not be reached, or did not answer — it **MUST** keep the pending admission for item 6, because the host may have accepted: discarding it then would leave the owner an epoch behind their own room.
6. **MUST**, when it finds a pending admission at a later request, ask the host the admission was sent to for the room's epoch — with [`rooms/info`](../../../../info/0.1/spec.md), under the caller's `read` authority — and apply the pending admission if the host is at its epoch, or discard it if the host is still at the epoch before it. A host anywhere else has moved on without it: discard it and refuse with `epochConflict`. This is what makes the task safe across a crash or a lost reply between the host's acceptance and the recipient's own write: a key holder cannot process its own commit from the wire, so the pending state is the only way back. A pending admission it applies becomes the room's retained admission (item 8), and when the request in hand repeats it, the recipient **MUST** answer as item 8 says rather than build another.
7. **MUST NOT** send the Welcome to the host.
8. **MUST** keep, with the group state, the last admission it applied for the room — `member`, the `KeyPackageRef` of its KeyPackage ([RFC 9420 §5.2](https://www.rfc-editor.org/rfc/rfc9420#section-5.2)), the epoch, the Welcome and the ratchet tree — until it builds its next advance of the room (item 3) or 24 hours after applying it, whichever comes first, and **MUST** discard it then. When a request names the same `roomId`, a `member` equal to the retained one as a string, and a KeyPackage whose `KeyPackageRef` equals the retained one, the recipient **MUST**, once item 1 and the checks of *Authorization* pass as for any request, answer with the retained admission's epoch, Welcome and tree, and **MUST NOT** advance the room, contact the host or change its group. Any other request for a `member` that already has a leaf **MUST** be refused `alreadyMember`. The recipient **MUST NOT** build a fresh Welcome for a repeat: it has no commit to build one from, and a second, different Welcome for one admission is not what the caller asked for.

## Authorization

**The entitlement is membership of the group and the room's `admin` authority**, both of which the recipient holds for its caller.

Membership of the group is necessary because only a member can add one; the recipient's own group state is the evidence. The `admin` authority is necessary because advancing a room's epoch is how a room changes who can read it, and [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md) restricts it to `admin` for exactly that reason. The recipient does not decide that its caller is an admin: it holds, or does not hold, an authority credential the room issued to them conferring it, and the host verifies the chain.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room whose group admits a member.

**`host`** — the room's host, as a DID: where its epoch advances.

**`member`** — the DID being admitted: the subject of the membership credential the room issued them, and the identity their leaf must carry.

**`keyPackage`** — the newcomer's MLS KeyPackage, base64url, as [`rooms/keys/key-package`](../../../../keys/key-package/0.1/spec.md) returned it to them.

**`reason`** — why, recorded with the epoch advance at the host where it records one.

**`epoch`** (response) — the room epoch after the admission.

**`welcome`**, **`ratchetTree`** (response) — for [`rooms/keys/welcome`](../../../../keys/welcome/0.1/spec.md), unchanged.

### Remote first

The order in items 2 to 5 is the point. A key holder that applied the commit first and then failed to advance the host would be at an epoch the room is not at: every record it sealed would be sealed to keys no other member holds, and nothing would say so until somebody failed to open one. Advancing the host first, and applying only what the host accepted, means the worst a failure leaves is an admission that did not happen.

### Repeating an admission

Remote first keeps the room and the owner's group in step across a failure; item 8 keeps the
**caller** in step. An admission's answer can be lost after the host accepted it — in the
host's reply, which item 6 recovers, or in the recipient's own reply to its caller. Either way
the caller's only safe move is to send the identical request again, and the newcomer's only
way in is the Welcome. So the recipient answers that repeat with the Welcome it already made.

**Re-serving the Welcome is safe.** A Welcome carries the group's secrets encrypted, under
HPKE, to the init key of the KeyPackage it admits, and is addressed by that KeyPackage's
`KeyPackageRef` ([RFC 9420 §12.4.3.1](https://www.rfc-editor.org/rfc/rfc9420#section-12.4.3.1)).
Only the holder of the init key's private half — the newcomer's key holder — can open it.
Sending the same bytes again discloses nothing the first answer did not, to the caller or to
any transport between them. And it cannot be replayed into a second join: the newcomer's key
holder discards the private half once it processes the Welcome, or when the KeyPackage
expires ([`rooms/keys/key-package`](../../../../keys/key-package/0.1/spec.md), *Retention*), so
a copy that arrives after either opens nothing.

**The match is exact, because anything looser admits a second party.** The repeat is
recognised by the `member` DID compared as a string — never normalised, never resolved —
and by the KeyPackage's `KeyPackageRef`, the hash MLS itself uses to name a KeyPackage. Equal
references mean byte-identical KeyPackages, which mean the same init key, so the Welcome
re-served opens for exactly the party the first one did. A request with another KeyPackage for
the same `member` is not a repeat: answering it with the retained Welcome would hand the
caller a Welcome it cannot use, and adding the new KeyPackage would give one member two leaves.
It is `alreadyMember`, as it was.

**The retention is short, because the Welcome is secret-bearing ciphertext.** It is kept for
the window in which a caller retries — until the recipient's next advance of the room, which
may itself remove the member, so that a request after it is no longer a retry of anything, or
24 hours, whichever is sooner. A caller that needs a Welcome after that has a member with a
leaf it can no longer join with, and the remedy is the owner's: remove them ([`rooms/owner/group/remove`](../../remove/0.1/spec.md))
and admit them again with a fresh KeyPackage.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Admitting a member whose key holder has minted a KeyPackage for the room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/group/add/0.2#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "keyPackage": "AAEAAgADAAQ"
  }
}
```

## Response

The **KeyHolder** returns the new epoch and the Welcome, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### The room advanced to epoch 2, with the Welcome to deliver

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/group/add/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "epoch": 2,
    "welcome": "AAEAAgADAAQ",
    "ratchetTree": "AAEAAgADAAQ"
  }
}
```

## Security & Privacy

### Data carried

A KeyPackage in (public: a signature key, an HPKE init key and the leaf credential); a Welcome and a ratchet tree out. The Welcome carries the group's secrets sealed to the newcomer's KeyPackage, so it opens nothing for anyone else, including the caller.

### Correlation

On a `private` room the host must not learn who joined. It sees an epoch advance, a link and a commit, none of which it can read; it never sees the Welcome (item 7), and the caller delivers that directly to the newcomer's key holder.

### Retention

The advanced group state is durable. The pending admission of item 3 is retained only until the host's answer is known — by its reply, or by item 6. The last applied admission's Welcome, tree, epoch, `member` and `KeyPackageRef` are retained only until the recipient's next advance of the room or 24 hours, whichever is sooner (item 8); the KeyPackage itself need not be kept.

### Consent/purpose

Admission is two-party: the owner asks here, and the newcomer consented by minting a KeyPackage against the room's invitation. Their key holder checks that invitation again when the Welcome arrives.
