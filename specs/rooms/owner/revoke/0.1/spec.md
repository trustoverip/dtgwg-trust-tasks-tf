---
slug: rooms/owner/revoke
version: "0.1"
title: "Rooms Owner — Revoke"
summary: "A room's owner asks their own agent to withdraw credentials the room issued: sign the room's next revocation list as the room, keep it, and deliver it to every host serving the room, which refuses them from then on."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords: [rooms, owner, revoke, revocation, removal, credential, withdraw]
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
  rationale: "The recipient signs as the ROOM, and what it signs withdraws parties' standing in it at every host. Nothing about this may rest on the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A withdrawal is consequential and is keyed on time: a subject entry withdraws what the room issued before it. A replay must be placeable in time so it is not taken for a second withdrawal after a re-admission."
sideEffects:
  level: mutating
  rationale: "Signs and durably keeps the room's next revocation list, and makes outbound requests that change what each host honours."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The recipient speaks to each host as its principal, presenting the owner's `admin` authority, and discloses to it the room's list: on `open` and `attributed` rooms the withdrawn credential identifiers and parties, on `private` rooms only times."
retention:
  class: durable
  rationale: "The recipient keeps the room's latest list for as long as it holds the room's key: the next list must carry every live entry of this one, and a host the room moves to must be given it."
errorCodes:
  - code: rooms/owner/revoke:noSigningKey
    meaning: "The recipient holds no key by that identifier, or the key is not one its caller may name."
    retryable: false
  - code: rooms/owner/revoke:noAuthority
    meaning: "The recipient holds no authority credential conferring `admin` on this room for its caller, so it cannot present one to a host."
    retryable: false
  - code: rooms/owner/revoke:entryNotPermitted
    meaning: "The room is `private`, and a withdrawal named a credential or a party. Only a `room` withdrawal can be applied there. Nothing was signed."
    retryable: false
  - code: rooms/owner/revoke:roomSubject
    meaning: "A `subject` withdrawal named the room itself. Nothing was signed."
    retryable: false
  - code: rooms/owner/revoke:noList
    meaning: "`withdraw` was empty and the recipient holds no list for the room, so there is nothing to deliver."
    retryable: false
  - code: rooms/owner/revoke:tierUnknown
    meaning: "A withdrawal named a credential or a party, and the recipient could not establish the room's tier from the first host. Nothing was signed."
    retryable: true
related:
  - rooms/revocations/put
  - rooms/revocations/get
  - rooms/owner/group/remove
  - rooms/owner/issue-authority
  - rooms/owner/issue-membership
  - rooms/owner/register
---

## Abstract

A room's owner asks their **key holder** to withdraw credentials the room issued — one credential, a party, or every credential issued until now — so that every host serving the room refuses them **immediately** rather than honouring them until they expire.

The key holder is the party that can do it. Withdrawing is a statement **by the room**, signed with the room's key, which the owner's key holder holds — the same entitlement that issues the room's grants ([`rooms/owner/issue-authority`](../../issue-authority/0.2/spec.md)). It builds the room's next [`RevocationList`](../../../_shared/0.1/room.schema.json) — the list it holds plus what is withdrawn now, at the next sequence — signs it as the room, keeps it, and delivers it to each named host with [`rooms/revocations/put`](../../../revocations/put/0.1/spec.md).

**Every step is outbound.** The owner's agent may have no inbound address at all; nothing here needs one. A host never asks the owner anything: it is told, by a push it can verify, and holds what it was told.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noSigningKey` when it holds no key by `signingKeyId` that its caller may name, with `noAuthority` when it holds no `admin` authority for its caller to present, with `roomSubject` when a `subject` withdrawal names `roomId`, and with `noList` when `withdraw` is empty and it holds no list for the room — all before signing anything.
2. **MUST**, when `withdraw` names a credential or a party, establish the room's tier before signing — from [`rooms/info`](../../../info/0.1/spec.md) at the first of `hosts`, under its caller's `read` authority, unless it already knows it from registering the room — and refuse with `entryNotPermitted` on a `private` room, or with `tierUnknown` when it cannot establish it. A list signed with an entry a `private` host must refuse could never be superseded there, since every later list has to keep it.
3. **MUST**, when `withdraw` is not empty, build the next list: every entry of the list it holds except those whose `until` passed more than 24 hours ago — a margin, so that a host whose clock is behind the recipient's does not refuse the list as dropping an entry it still counts as live — followed by one entry per withdrawal with `revokedAt` set to the time it signs; `sequence` one greater than the list it holds, or `1` if it holds none. It **MUST** sign it as `roomId` with `signingKeyId`, as `RevocationList` states, and **MUST NOT** sign two lists at one sequence: building and signing a room's lists is serialized per room.
4. **MUST** retain the signed list durably **before** delivering it anywhere, and **MUST** keep it with the room's key for as long as it holds that key — backed up and restored with it. A recipient that loses its list cannot sign a later one that keeps the live entries; it **SHOULD** recover it with [`rooms/revocations/get`](../../../revocations/get/0.1/spec.md) from a host that holds it, accepting only a list the room signed at a sequence at or above its own.
5. **MUST** deliver the list it holds — the one it just signed, or with `withdraw` empty the one it already held — to **each** of `hosts` with [`rooms/revocations/put`](../../../revocations/put/0.1/spec.md), presenting its caller's `admin` authority minted as [`rooms/keys/present`](../../../keys/present/0.2/spec.md) would mint it, in a document addressed to that host whose proof covers its `recipient`. It **MUST** report each host's outcome in `deliveries` and **MUST NOT** report a host as `accepted` unless that host answered with the list's head.
6. **MUST** answer successfully once the list is signed and kept, whatever the deliveries' outcomes: the withdrawal is the room's from the moment it is signed, and an unreachable host is brought up to date by repeating the request with `withdraw` empty, which signs nothing new. A host that answers `stale` holds a list at a later sequence than the recipient's own, which this recipient did not sign; it **MUST** report it as `refused` with that code, and **SHOULD** surface it to its caller as a sign that the room's key is in more than one party's hands.

A recipient that implements this task and registers a room with a host ([`rooms/owner/register`](../../register/0.1/spec.md)) **MUST** deliver the list it holds for that room, if any, to that host before answering the registration. A room moving hosts otherwise arrives at its new host without its withdrawals, and honours there what it refused at the old one until somebody notices.

## Authorization

**The entitlement is control of the room's signing key**, exactly as for [`rooms/owner/issue-authority`](../../issue-authority/0.2/spec.md): this recipient signs as the room, and a caller who may name the room's key may withdraw in the room's name, as they may grant in it. Withdrawing is the inverse of granting a chain root, and is held by the same party for the same reason — a room's grants and its withdrawals are one accountable party's decisions.

Delivery additionally needs `admin` at each host, which the recipient presents for its caller as it does for an epoch advance.

**Who can withdraw what.** Only the room withdraws, because only the room's signature makes a list every host must honour. Another administrator of the room — a holder of `admin` who does not hold the room's key — can deliver a list the room signed ([`rooms/revocations/put`](../../../revocations/put/0.1/spec.md)) but cannot write one; they ask the owner. A member who delegated to an agent withdraws that delegation by the delegation's own short expiry, or by asking the room to withdraw it by `credentialId`. A delegator's own list was considered and not adopted: every delegator would become a second signer whose list each host must hold, sequence and keep from rolling back, and a host would hold state for parties the room never named. One signer and one sequence per room is what keeps rollback detection a comparison of two integers.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room. **`signingKeyId`** — the held key that signs as the room.

**`hosts`** — every host serving the room: its primary, its mirrors, and any host it is moving to.

**`withdraw`** — the withdrawals to add. A `credential` withdrawal names one credential by `id`. A `subject` withdrawal names a party, and withdraws every credential naming them as subject or issuer that was issued before now — their membership, their authority, and what they delegated. A `room` withdrawal withdraws every credential issued before now, from everyone; the room then re-issues to those who remain.

**`list`**, **`head`**, **`deliveries`** (response) — the list the recipient now holds, its head, and what each host did with it.

### Re-admitting a party

A `subject` withdrawal is keyed on time, so it never has to be undone. Re-admitting a party is issuing them fresh credentials ([`rooms/owner/issue-membership`](../../issue-membership/0.1/spec.md), [`rooms/owner/issue-authority`](../../issue-authority/0.2/spec.md)): their `validFrom` is after the withdrawal, which therefore does not reach them. What they delegated before the withdrawal stays withdrawn, because it names them as issuer and predates it.

### Removal from the group, and the order

On a sealed room, removing a member is two acts — taking their leaf out of the group, and withdrawing their credentials — and [`rooms/owner/group/remove`](../../group/remove/0.2/spec.md) at `0.2` does both, withdrawing **first**. This task is the withdrawal alone: for an `open` room, which has no group; for an agent credential, which has no leaf; and for re-delivering the list.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdrawing a party from an open room hosted with a mirror

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/revoke/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T09:29:57Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "signingKeyId": "room-northwind-signing",
    "hosts": [
      "did:webvh:example.com:northwind-community",
      "did:web:mirror.example"
    ],
    "withdraw": [
      {
        "kind": "subject",
        "subject": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "until": "2027-01-01T00:00:00Z"
      }
    ],
    "reason": "Left the deal team"
  }
}
```

## Response

The **KeyHolder** returns the list it holds and each host's outcome, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Signed, kept, and in force at one host; the mirror did not answer

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/revoke/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-10-16T09:30:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "list": {
      "roomId": "did:webvh:example.com:rooms:northwind",
      "sequence": 1,
      "issuedAt": "2026-10-16T09:29:58Z",
      "entries": [
        {
          "kind": "subject",
          "subject": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
          "revokedAt": "2026-10-16T09:29:58Z",
          "until": "2027-01-01T00:00:00Z"
        }
      ],
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:example.com:rooms:northwind#key-0",
        "proofPurpose": "assertionMethod",
        "created": "2026-10-16T09:29:58Z",
        "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKcuiBNPvFaEb5PNoq9"
      }
    },
    "head": {
      "sequence": 1,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "deliveries": [
      {
        "host": "did:webvh:example.com:northwind-community",
        "outcome": "accepted",
        "head": {
          "sequence": 1,
          "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
        },
        "inForceAt": "2026-10-16T09:30:00Z"
      },
      {
        "host": "did:web:mirror.example",
        "outcome": "unreachable"
      }
    ]
  }
}
```

## Security & Privacy

**The withdrawal binds the hosts that hold it.** A success says the room signed the list. It says a host enforces it only for a delivery reported `accepted`. A surface **SHOULD** show an `unreachable` or `refused` host to its caller as a host still honouring what the room withdrew, and offer to deliver again.

**Rollback and forks.** The recipient signs one list per sequence and keeps it before delivering it, so no host can be handed two lists at one sequence by this recipient, and a host refuses a lower one. A host answering `stale` holds a list this recipient never signed at a higher sequence — the room's key is signing elsewhere — which is the one condition this task treats as a security signal rather than a delivery failure.

### Data carried

A room, a key, a set of hosts and the withdrawals; a signed list out. The `reason` stays with the owner and never enters the list.

### Correlation

Each host learns what the list says. On `open` and `attributed` rooms that is which parties and credentials were withdrawn and when — parties each host has seen act. On `private` it is only that every credential was reset. The list is delivered to the named hosts and to nobody else; nothing is published.

### Retention

The recipient keeps the room's latest list with the room's key, and drops entries whose `until` has passed when it next signs. It keeps no history of earlier lists: the latest carries every live withdrawal.

### Consent/purpose

Withdrawing is the owner's act, and the party withdrawn is not asked — which is the point, and why it needs the room's key.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
