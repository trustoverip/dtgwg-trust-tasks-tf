---
slug: rooms/owner/group/remove
version: "0.4"
title: "Rooms Owner — Group — Remove"
summary: "A room's owner asks their own agent to remove a member from a sealed room: on an `attributed` room withdraw every credential the room gave them, in force at its host at once, and on either sealed tier drop their leaf from the room's MLS group and advance the room's epoch."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords: [rooms, owner, group, mls, sealed, remove, revoke, revocation, epoch]
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
  rationale: "The recipient ends a party's access to the room — at its host at once, and to everything the room seals from now on — and signs as the room to do it. A request whose origin depended on the transport would let a compromised channel evict members from a room nobody else can audit."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A removal withdraws credentials by time and advances the room's epoch. A replay must be placeable in time so it is taken neither for a second advance nor for a withdrawal after a re-admission."
sideEffects:
  level: mutating
  rationale: "The recipient signs and keeps the room's next revocation list and advances its group, and makes outbound requests that put the list in force at each host and advance the room's epoch."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The recipient speaks to each host as its principal, presenting the owner's `admin` authority. On an `attributed` room it discloses the room's revocation list, which names `member` — a party that room's host has seen act. On either sealed tier it discloses to the room's host a new epoch number, an epoch link and a commit, all opaque to a party without the group; on a `private` room that is all it discloses, and which member left is not."
retention:
  class: durable
  rationale: "The advanced group state and the room's revocation list persist, as they must for the owner to keep sealing the room and to sign the next list."
errorCodes:
  - code: rooms/owner/group/remove:noGroup
    meaning: "The recipient holds no group state for this room."
    retryable: false
  - code: rooms/owner/group/remove:notMember
    meaning: "`member` has no leaf in the group."
    retryable: false
  - code: rooms/owner/group/remove:ownLeaf
    meaning: "`member` is the recipient's own leaf. A key holder does not remove itself; an owner leaving a room hands it on with rooms/owner/transfer."
    retryable: false
  - code: rooms/owner/group/remove:noSigningKey
    meaning: "The recipient holds no key by `signingKeyId`, or the key is not one its caller may name — or the request names none and the room is not `private`, so the withdrawal cannot be signed. Nothing changed."
    retryable: false
  - code: rooms/owner/group/remove:noAuthority
    meaning: "The recipient holds no authority credential conferring `admin` on this room for its caller, so it can neither deliver the withdrawal nor advance the room's epoch at the host."
    retryable: false
  - code: rooms/owner/group/remove:epochConflict
    meaning: "The host is at a different epoch than the recipient's group — another admin advanced the room. The withdrawal, if it was accepted, stays in force. Catch up (rooms/epoch/commits) and try again. The host's epoch is carried in `details.hostEpoch`."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [hostEpoch]
      properties:
        hostEpoch:
          type: integer
          minimum: 1
  - code: rooms/owner/group/remove:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer. The outcome is unknown, so the recipient keeps the pending removal and the next request to it settles it (item 8). `details.stage` says which step went unanswered; a `private` room's removal has no `revocation` step."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [stage]
      properties:
        stage:
          type: string
          enum: [revocation, advance]
  - code: rooms/owner/group/remove:earlierRemovalRefused
    meaning: "An earlier removal from this room was still pending at the recipient, and when it delivered that removal's revocation list to the host again (item 8) the host refused it. The earlier removal did not take effect: `details.member` is still in the group, and is withdrawn at the host only if `details.listKept` is true and the host ever accepted that list. Nothing of this request was done, and the earlier removal is no longer pending, so repeating this request proceeds; remove `details.member` again if that is still wanted. The host's own code is in `details.code`."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member, code, listKept]
      properties:
        member:
          type: string
          maxLength: 2048
        code:
          type: string
          maxLength: 128
        listKept:
          type: boolean
  - code: rooms/owner/group/remove:hostRefused
    meaning: "The host answered and declined. `details.stage` says which step: at `revocation` nothing changed; at `advance` the group is unchanged, and on an `attributed` room the withdrawal is in force at the host. The host's own code is in `details.code`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [stage]
      properties:
        stage:
          type: string
          enum: [revocation, advance]
        code:
          type: string
          maxLength: 128
related:
  - rooms/owner/revoke
  - rooms/revocations/put
  - rooms/owner/group/create
  - rooms/owner/group/add
  - rooms/owner/group/renew
  - rooms/owner/transfer
  - rooms/keys/commit
  - rooms/epoch/mint
  - rooms/epoch/commits
---

## Abstract

A room's owner asks their **key holder** to remove a member from a sealed room — `attributed` or `private` — and, where the room's host can be told whom to refuse, removal takes effect at that host **immediately**.

Removal is up to two acts. Taking the member's leaf out of the room's MLS group and advancing the epoch stops them opening what is written **next**. It does not stop the host honouring the membership and authority credentials the room gave them, which a host authorizes from and which last until their `validUntil`: until then they could still read every record and blob the host serves, fetch the commit relay and the key chain, and write into the room. So on an `attributed` room this task first **withdraws** their credentials — the room's next [`RevocationList`](../../../../_shared/0.1/room.schema.json), signed as the room and delivered with [`rooms/revocations/put`](../../../../revocations/put/0.1/spec.md) — and only then advances the group.

**Withdraw first, then advance.** In that order there is no instant at which the member is outside the group and still credentialed at the host. The reverse order leaves one: after the advance and before the withdrawal, a removed member who no longer holds the room's key can still present to the host, and could write records labelled with the new epoch that the host cannot tell from real ones. If the withdrawal lands and the advance does not, the member is refused at the host and holds keys only to the epoch the room was already at — and anything written under it is refused to them by the host that stores it.

**On a `private` room there is only the second act.** A `private` room's host verifies membership in zero knowledge and never learns which member acted, so it cannot be told to refuse one party without being told who that party was. This task therefore removes the member's leaf and advances the room, and withdraws nothing: the member's credentials are honoured at the host until their `validUntil`, and what they admit the member to is ciphertext they cannot open from the new epoch on. The response says so (`withdrawn: false`). An owner who needs more resets every credential the room has issued with [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) (a `room` entry) and re-admits those who remain.

**What removal still does not do, and a surface offering it must say so:** it does not recall anything the member already read or saved. On an `attributed` room it ends their access to everything the host serves, from the moment the host accepts the withdrawal, and to everything written from now on; on a `private` room it ends their access to everything written from now on.

### What changed from 0.3

`0.2` and `0.3` refused a `private` room with `privateRoom` and sent the owner to `0.1`, which removed the leaf and withdrew nothing. That made `0.1` the only way to remove a member of a `private` room from its group, and a version kept alive only for one tier is superseded all the same — when it is withdrawn, the capability goes with it. `0.4` takes that tier in: on a `private` room it does what `0.1` did, under the settlement rules of `0.3`, and says in its response that nothing was withdrawn.

- **A `private` room is removed from the group, not refused.** The `privateRoom` error code is gone. Item 2 establishes the tier as before; on a `private` room items 4, 6 and 9 do not apply, and no list naming `member` is built or sent to any host.
- **`signingKeyId` is optional in the schema**: a `private` room's removal signs nothing as the room. It is still required on an `attributed` room, which is refused with `noSigningKey` when the request names none.
- **The response carries `withdrawn`**, true when the member's credentials were withdrawn at the host and false on a `private` room, and carries `revocation` only when it is true. `revocation` is no longer required.
- Item 8 is `0.3`'s, unchanged, with one clarification: a pending removal that carries no list — a `private` room's, or one left by `0.1` — is settled by its step 3 alone.

Changing what a request yields and what a response requires is not editorial, so this is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). It does not declare `extends`, because `revocation` is no longer required and `privateRoom` is no longer declared. A recipient implementing `0.4` settles every pending removal by item 8 below, whichever version's request finds it.

*This paragraph is non-normative.* The advance moves the room's profile too: the profile is stored per epoch, so a key holder that also serves [`rooms/keys/profile/write`](../../../../keys/profile/write/0.1/spec.md) reads it before advancing and re-writes it under the new epoch afterwards, which is what lets a member who joins now — or reads only from where they joined — see what the room is. See that specification's *Keeping the profile readable across epochs*.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noGroup` when it holds no group for `roomId`, with `notMember` when `member` has no leaf in it, with `ownLeaf` when `member` is the recipient's own leaf, with `noSigningKey` when `signingKeyId` is given and names no key it holds that its caller may name, and with `noAuthority` when it holds no `admin` authority for its caller — all before anything else.
2. **MUST** establish the room's tier — from [`rooms/info`](../../../../info/0.3/spec.md) at `host` under its caller's `read` authority, unless it already knows it. On a room that is not `private` it **MUST** refuse with `noSigningKey`, changing nothing, when the request names no `signingKeyId`.
3. **MUST** build the commit removing `member`'s leaf, and the epoch link that seals the outgoing epoch's storage key under the incoming one, **without** applying the commit to the group it holds.
4. **MUST**, on a room that is not `private`, build and sign the room's next revocation list as [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) item 3 does, adding one `subject` entry for `member` with `revokedAt` the time it signs and `until` from the request. On a `private` room it **MUST NOT** build, sign or deliver any list for this removal, and ignores `until` and `mirrors`.
5. **MUST** retain the pending removal — the advanced state, the link, the commit and, on a room that is not `private`, the signed list — durably **before** sending anything to any host. A recipient holds at most one pending advance per room, whether an admission, a removal or a renewal, and settles any it finds (item 8) before building another.
6. **MUST**, on a room that is not `private`, deliver the list to `host` with [`rooms/revocations/put`](../../../../revocations/put/0.1/spec.md) **before** advancing the room, and **MUST NOT** advance it until `host` has accepted the list. When `host` refuses the list it **MUST** discard the whole pending removal — list included, since no host has accepted it — and refuse with `hostRefused` (`stage: revocation`).
7. **MUST** then advance the room at `host` with [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md), carrying the new epoch, the link, the commit and a presentation of the caller's `admin` authority — minted as [`rooms/keys/present`](../../../../keys/present/0.2/spec.md) would mint it — in a document addressed to `host` whose proof covers its `recipient`. It **MUST** apply the pending advance only once the host has accepted it, and **MUST** discard the pending advance, leaving the group unchanged, when the host **refuses** it, refusing with `hostRefused` (`stage: advance`). A list is **not** discarded then: it is in force at the host and is the room's, and the member stays withdrawn there. When the outcome is unknown it **MUST** keep the pending removal for item 8 and refuse with `hostUnreachable` (`stage: advance`).
8. **MUST**, when it finds a pending removal at a later request — of this task or of any other that settles the room's pending advance (item 5) — settle it in order. A pending removal that carries no list — a `private` room's, or one left by `0.1` — is settled by step 3 alone.
   1. **Confirm the withdrawal.** If the host has not confirmed the list, deliver it to the host again; a repeat is answered as accepted, and the list is confirmed once the host answers with its head. A host that does not answer leaves the removal pending, and the request is refused with `hostUnreachable` (`stage: revocation`) — at another task, with that task's own `hostUnreachable`, at `stage: revocation` where that task's version declares a `stage` ([`rooms/owner/group/add`](../../add/0.3/spec.md) `0.3`, [`rooms/owner/group/renew`](../../renew/0.2/spec.md) `0.2`).
   2. **When the host refuses the repeat**, what may be discarded depends on what the refusal shows, because the first delivery's outcome is unknown:
      - `listInvalid`, `entryNotPermitted` or `sequenceConflict` refuse the list itself, and would have refused it the first time, so no host has accepted it and no mirror was sent it (item 9). The recipient **MUST** discard the whole pending removal as item 6 does, list included — but **MUST** discard the list only while it is still the list it holds: a later list signed on it carries its withdrawal, and is kept. On `sequenceConflict` it **SHOULD** surface the refusal as a sign that the room's key is in more than one party's hands, as [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) item 6 does for `stale`.
      - `stale` says the host holds a later list. When the list the recipient holds is later than the pending one — it has signed one since, which keeps every live entry of this one ([`rooms/owner/revoke`](../../../revoke/0.1/spec.md) item 3) — it **MUST** deliver that list to the host instead, and the withdrawal is confirmed once the host answers with that list's head; the removal then continues at step 3. When that delivery is not answered, the removal stays pending, as in step 1.
      - Any other refusal — `stale` when the recipient holds no later list, which means the host holds a list this recipient did not sign and is surfaced as [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) item 6 says; a refusal of the delivery rather than the list, such as `permissionDenied`; or a refusal of the later list of the case above — does not show that the host never accepted the list. The recipient **MUST NOT** discard the list then: it **MUST** discard the pending advance, leaving the group unchanged, and keep the list as the room's, to be carried by every later list it signs and delivered by [`rooms/owner/revoke`](../../../revoke/0.1/spec.md).

      Whenever step 2 discards the pending removal or its advance, the recipient **MUST** refuse the request at which it settled — at this task, with `earlierRemovalRefused`, carrying the earlier removal's `member`, the host's code and whether the list was kept (`listKept`); at another task, with that task's own `earlierRemovalRefused` where its version declares one ([`rooms/owner/group/add`](../../add/0.3/spec.md) `0.3`, [`rooms/owner/group/renew`](../../renew/0.2/spec.md) `0.2`), and otherwise with its own `hostRefused`, carrying the same three members in `details` — and **MUST NOT** go on to that request's own work. The owner was told the earlier removal was pending; it must not vanish inside a later request that succeeds, leaving a member they believe removed still in the group.
   3. **Settle the advance.** Ask the host for the room's epoch with [`rooms/info`](../../../../info/0.3/spec.md) and apply the pending advance if the host is at its epoch, or discard it if the host is still at the epoch before it; a host anywhere else has moved on without it: discard it and refuse with `epochConflict`. Discarding the advance never discards a confirmed list, which is the room's once any host has accepted it — the member stays withdrawn, and a repeated removal advances the group.
9. **MUST**, on a room that is not `private`, once `host` has accepted the list, deliver it to each of `mirrors` and name in `mirrorsPending` every mirror that did not answer with its head. The removal succeeds without them; a mirror not yet holding the list is reported, not waited for.
10. **MUST** answer with `withdrawn: true`, `revocation` and `revokedAt` when it withdrew the member at the host, and with `withdrawn: false` and none of `revocation`, `revokedAt` or `mirrorsPending` on a `private` room.

A host receives the withdrawal and the advance as two independent requests and treats them so ([`rooms/revocations/put`](../../../../revocations/put/0.1/spec.md), *Enforcement*): each takes effect when it is accepted, in whichever order they arrive. Item 6 fixes the order this recipient sends them in; nothing at the host depends on it.

A conforming **producer** — the owner's surface:

1. **MUST NOT** represent a response with `withdrawn: false` as withdrawing the member's credentials, and **SHOULD** say, when it reports one, that the member's credentials stay valid at the host until they expire and that [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) with a `room` entry resets every credential the room has issued.

## Authorization

**The entitlement is membership of the group and the room's `admin` authority — and, on an `attributed` room, control of the room's signing key** — all of which the recipient holds for its caller. Signing the withdrawal as the room is the entitlement of [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) and of [`rooms/owner/issue-authority`](../../../issue-authority/0.2/spec.md); the host verifies the `admin` chain on both the delivery and the epoch advance. The recipient does not decide that its caller is an admin.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room whose group loses a member.

**`host`** — the room's host, as a DID: where the withdrawal, if any, is delivered first and the epoch advances.

**`mirrors`** — any other host serving the room. On a room that is not `private`, each must hold the withdrawal too. Unused on a `private` room.

**`member`** — the DID whose leaf is removed and, on a room that is not `private`, whose credentials are withdrawn — every credential naming them as subject or issuer that the room's chain gave them before `revokedAt`.

**`signingKeyId`** — the held key that signs the revocation list as the room. Required unless the room is `private`.

**`until`** — when the withdrawal may be dropped from the room's list: the latest `validUntil` of anything the room issued `member`. Unused on a `private` room.

**`reason`** — why, carried to the host's epoch advance.

**`withdrawn`** (response) — whether the member's credentials were withdrawn at `host`: true on an `attributed` room, false on a `private` one.

**`revocation`**, **`revokedAt`**, **`epoch`**, **`mirrorsPending`** (response) — the list in force at `host`, the withdrawal's time, the room epoch after the removal, and any mirror not yet holding the list. Only `epoch` is present on a `private` room.

### Re-admitting someone removed

On a room that is not `private`, the withdrawal reaches what the room issued `member` **before** `revokedAt`, and nothing after. Admitting them again is admitting them as anybody is admitted — fresh credentials ([`rooms/owner/issue-membership`](../../../issue-membership/0.1/spec.md), [`rooms/owner/issue-authority`](../../../issue-authority/0.2/spec.md)) and a new leaf ([`rooms/owner/group/add`](../../add/0.3/spec.md)) — and the withdrawal does not have to be undone. What they delegated before removal stays withdrawn. On a `private` room nothing was withdrawn, and re-admission is a new leaf alone.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Removing a member of an attributed room who has left the deal

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.4#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T09:29:57Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "signingKeyId": "room-northwind-signing",
    "until": "2027-01-01T00:00:00Z",
    "reason": "Left the deal team"
  }
}
```

### Removing a member of a private room

No signing key is named: a `private` room's removal signs nothing as the room.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.4#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T10:14:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board-papers",
    "host": "did:webvh:example.com:northwind-community",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "reason": "Term ended"
  }
}
```

## Response

The **KeyHolder** returns whether the member was withdrawn at the host, the list in force when they were, and the room's new epoch, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Withdrawn at the host, and the room advanced to epoch 4 without them

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.4#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-10-16T09:30:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "withdrawn": true,
    "revocation": {
      "sequence": 2,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "revokedAt": "2026-10-16T09:29:58Z",
    "epoch": 4
  }
}
```

### A private room advanced to epoch 7 without them; nothing withdrawn

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.4#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-10-16T10:14:05Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board-papers",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "withdrawn": false,
    "epoch": 7
  }
}
```

## Security & Privacy

**Why a `private` room's removal withdraws nothing.** A host serving a `private` room verifies membership in zero knowledge and never learns which member acted, so an entry naming the member is one it could not apply — and one that would tell it who had been in the room. The only withdrawal a `private` host can apply is a reset of every credential issued until now (a `room` entry, [`rooms/owner/revoke`](../../../revoke/0.1/spec.md)), after which the room re-issues to those who remain. That is too large an effect to perform implicitly as part of removing one member, so this task removes the leaf alone, says so in `withdrawn`, and leaves the reset to the owner. Until the member's credentials expire they can still fetch what the host serves — ciphertext of the epochs they held keys to, which they could already read, and of later ones, which they cannot.

`0.2` and `0.3` refused a `private` room instead, so that no surface would report a removal there as a withdrawal. `withdrawn` and the producer requirement carry that protection now, without making the owner use a second version to do the part that is possible.

**Mirrors.** A mirror that has not received the withdrawal honours the member's credentials still. The removal is not held back for it — a mirror that is down would otherwise prevent anyone being removed — but it is reported, and a surface **SHOULD** show it as a host still serving the removed member.

### Data carried

A room, a host, its mirrors, a DID and a key name in; whether the member was withdrawn, a list head, a time and an epoch out. On an `attributed` room the list names `member` to every host it reaches. The commit and link go to the room's host, opaque to it.

### Correlation

On an `attributed` room the host learns from the list that `member` was withdrawn and when; it sees an epoch advance besides. On a `private` room no list is built (item 4), so nothing naming a member reaches a host that is meant not to know who its members are: the host sees an epoch advance, a link and a commit, none of which it can read.

### Retention

The advanced group state and the room's revocation list are durable. The pending removal of item 5 is retained only until each step's outcome is known — by the host's reply, or by item 8. A list item 8 keeps after discarding the advance is the room's list, kept as [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) item 4 keeps every list.

### Consent/purpose

Removal is the owner's act alone; the member is not asked. That is the point of it, and why it needs the room's `admin` authority and, to withdraw, the room's key.
