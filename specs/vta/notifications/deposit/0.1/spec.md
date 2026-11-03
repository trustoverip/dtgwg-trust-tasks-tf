---
slug: vta/notifications/deposit
version: "0.1"
title: "VTA Notifications — Deposit"
summary: "A client hands its VTA a notification or relay it received for a DID the VTA holds keys of — a persona DID whose mediator socket the client holds — exactly as received, and the VTA verifies and holds it exactly as if it had been delivered directly."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - inbox
  - vta
  - deposit
  - persona
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
  - role: Delivering party — the issuer of the deposited document
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Normally carried by an authenticated transport between a client and its own VTA, and the client is
    not the party whose word counts: the deposited document carries its own proofs — the
    originator's and every relayer's — and those are what the VTA verifies and its acceptance policy
    is decided on. The deposit's own proof would add only the client's word that it received the
    document, which no check relies on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task may add a record, so the VTA must be able to place the request in a bounded window to
    absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    The VTA may hold a record of the deposited notification, replacing one it holds for a lower
    revision, exactly as a direct delivery would. Nothing is granted, and a repeat changes nothing.
subjectPath: /contextId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context and a notification or relay document, whole — a reference to a thing and the
    parties that said and passed on that it exists, never the thing's content. Out: a receipt that is
    the same whatever the VTA did with it.
retention:
  class: durable
  rationale: >-
    An accepted notification is held, with the deposited document whole, until it lapses or is
    withdrawn, as one delivered directly is (VTI-NTF-050, VTI-NTF-052).
maxDocumentBytes:
  request: 106496
  response: 4096
  rationale: >-
    The largest document a deposit carries is a notifications/relay document of at most 96 KiB, plus
    this envelope and its proof.
errorCodes:
  - code: vta/notifications/deposit:notAddressedHere
    meaning: >-
      The deposited document's `recipient` is not a DID the VTA holds keys of in `contextId`. The VTA
      holds only what is addressed to a DID it acts as, in the context that DID belongs to
      (VTI-NTF-050, VTI-NTF-056).
    retryable: false
  - code: vta/notifications/deposit:documentInvalid
    meaning: >-
      The deposited document would not have been accepted had it been delivered to the VTA directly:
      its `type` is not notifications/notify/0.1 or notifications/relay/0.1, it does not validate as
      a document of that type, its proof does not verify against its `issuer`, it is outside its
      acceptance window or past its `expiresAt` at the time of the deposit, or — for a relay — a
      layer of it fails notifications/relay consumer item 2 (VTI-NTF-032, VTI-NTF-042). Nothing of it
      is held.
    retryable: false
related:
  - notifications/notify
  - notifications/relay
  - vta/notifications/list
  - vta/notifications/show
  - vta/notifications/dismiss
---

## Abstract

The **VTA Notifications — Deposit** Trust Task closes the gap between where a notification arrives
and where it is held. A [`notifications/notify`](../../../../notifications/notify/0.1/spec.md) or
[`notifications/relay`](../../../../notifications/relay/0.1/spec.md) document addressed to one of a
principal's persona DIDs is delivered over that DID's transport — and the mediator socket of a
persona DID is often held by the principal's client, not by the VTA. The VTA holds the DID's keys,
the trust context it belongs to, and the principal's acceptance policy; the client holds only the
socket. With this task the client hands the VTA the document **exactly as received, proof
included**, and the VTA treats it exactly as if it had been delivered to it directly (VTI-NTF-054 –
VTI-NTF-056): it verifies every layer, applies its acceptance policy to the party that delivered the
document — never to the depositing client — holds at most one record per originator and
`notificationId`, and answers with the same receipt whatever it did.

A deposited notification is then listed, read and dismissed like any other, with
[`vta/notifications/list`](../../list/0.1/spec.md), [`vta/notifications/show`](../../show/0.1/spec.md)
and [`vta/notifications/dismiss`](../../dismiss/0.1/spec.md); `show` returns the deposited document as
the document that delivered the revision held.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the client) **MUST**:

1. Deposit every `notifications/notify` and `notifications/relay` document it receives that is
   addressed to a DID whose keys its principal's VTA holds in a context the client has standing in,
   and deposit it with that VTA in that context (VTI-NTF-054). It **MUST NOT** apply an acceptance
   policy of its own in place of the VTA's: deciding not to deposit a document is deciding to
   decline it, which is the VTA's decision.
2. Carry the document in `document` exactly as received, proof included — never altered, restated,
   re-serialised from a typed form or re-signed — and **SHOULD** deposit it as soon as it receives
   it, since the VTA applies the document's acceptance window at the time of the deposit (consumer
   item 3).
3. Answer the document's sender itself, as the addressee, with the receipt
   [`notifications/notify`](../../../../notifications/notify/0.1/spec.md) or
   [`notifications/relay`](../../../../notifications/relay/0.1/spec.md) defines — `notificationId`
   and `revision`, the same whatever the VTA does with the deposit, and saying nothing of it
   (VTI-NTF-023). It **MAY** answer before the deposit completes, and **MUST NOT** answer the sender
   differently because of the deposit's outcome.
4. **MUST NOT**, in consequence of receiving the document, send anything else to its originator, a
   relayer or the thing's holder, or open the thing except at the principal's choice (VTI-NTF-053).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the deposit per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse with
   `permissionDenied` a caller without standing under [Authorization](#authorization), before it
   reads `document`.
2. Refuse with `vta/notifications/deposit:notAddressedHere` a `document` whose `recipient` is not a
   DID the VTA holds keys of in `contextId`, comparing identifiers as VTI-KEY-160 – VTI-KEY-169
   require for the form they take.
3. Validate `document` exactly as it would one delivered to that DID directly
   ([SPEC §7.2](/SPEC.md#72-consumer-requirements)), with two differences that follow from who carried
   it: the document's own `issuer`, not the depositing client, is the party the document is
   attributed to, established by its proof alone; and its acceptance window, `expiresAt` and the
   record of accepted document identifiers (VTI-OPS-024, VTI-OPS-026, VTI-OPS-027) are applied at the
   time of the deposit, with the VTA as the recipient of the DID it holds keys of (VTI-KEY-107). For a
   relay it verifies every layer as [`notifications/relay`](../../../../notifications/relay/0.1/spec.md)
   consumer item 2 requires. It refuses a document that fails any of these with
   `vta/notifications/deposit:documentInvalid`, and holds nothing of it (VTI-NTF-032).
4. Decide whether to hold the innermost notification by the same acceptance policy it applies to a
   direct delivery (VTI-NTF-030), counting its relationship with the party that delivered the
   document — the originator for a notification, the outermost relayer for a relay (VTI-NTF-031) —
   and **MUST NOT** count the depositing client, or its access control entry, as that relationship
   (VTI-NTF-055). It counts the deposit against the same bounds per originator and per delivering
   party as a direct delivery (VTI-NTF-033).
5. Thereafter treat the notification exactly as
   [`notifications/notify`](../../../../notifications/notify/0.1/spec.md) consumer items 3 to 5
   require: one record per originator and `notificationId` whichever way each revision arrived —
   directly, by a relay, or deposited — held in `contextId`, with `document` kept whole as the
   document that delivered the revision held.
6. Answer a document it verified with the receipt in [Response](#response), the same whether it held
   the notification, declined it, or already held that revision, and saying none of these
   (VTI-NTF-023).

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless. A repeat of
the same deposit is answered with the answer first given ([SPEC §7.2](/SPEC.md#72-consumer-requirements)
item 11); a second deposit of a document the VTA already accepted, directly or by deposit, is a
repeat of that document's identifier and is answered with the same receipt; and a different document
carrying the same notification is absorbed by consumer item 5.

**A repeated request** sets a state: depositing a notification already held at the same revision
succeeds, with the same receipt, and changes nothing.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/notifications/list`](../../list/0.1/spec.md#authorization): the principal, or a party the
principal authorized in that context. That standing lets a caller put a document **before** the
VTA's acceptance policy and nothing more: it is not a relationship the policy counts (consumer
item 4), so a client cannot make the VTA hold a notification from a party the principal has no
relationship with by depositing it. The deposited document's proofs establish who originated it and
who relayed it ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); none of them is authority
at the VTA.

## Definitions

- **`contextId`** — the trust context the addressed DID belongs to.
- **`document`** — the `notifications/notify` or `notifications/relay` document the client received,
  whole.
- **Depositing client** — the `issuer` of this task. **Delivering party** — the `issuer` of
  `document`: the originator, or the outermost relayer.
- **`notificationId`**, **`revision`** (response) — the innermost notification's, echoed.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### A client deposits a notification its persona DID received

The principal's persona `did:webvh:QmPersona:persona.example` has its mediator socket held by the
principal's wallet client. Community `did:web:community.example` notifies the persona of an election;
the wallet answers the community with the receipt and deposits the document with the VTA, which
holds the persona's keys in context `personal`.

```json
{
  "id": "urn:uuid:7a8b9c0d-1e2f-4a3b-8c4d-5e6f7a8b9c0d",
  "type": "https://trusttasks.org/spec/vta/notifications/deposit/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-01T09:00:05Z",
  "payload": {
    "contextId": "personal",
    "document": {
      "id": "urn:uuid:3c9d2b51-8e2f-4c9f-ab62-1d7f4e3e8b22",
      "type": "https://trusttasks.org/spec/notifications/notify/0.1",
      "issuer": "did:web:community.example",
      "recipient": "did:webvh:QmPersona:persona.example",
      "issuedAt": "2027-03-01T09:00:00Z",
      "expiresAt": "2027-03-01T10:00:00Z",
      "payload": {
        "notification": {
          "notificationId": "ntf-9Lm4q2Wx8nT5",
          "revision": 1,
          "kind": "election",
          "subject": {
            "heldBy": "did:web:community.example",
            "id": "board-2027",
            "openWith": "https://trusttasks.org/spec/vtc/elections/show/0.6"
          },
          "audience": "did:web:community.example",
          "status": "available",
          "relayable": false,
          "validFrom": "2027-03-08T00:00:00Z",
          "validUntil": "2027-03-15T00:00:00Z"
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2027-03-01T09:00:00Z",
        "verificationMethod": "did:web:community.example#key-1",
        "proofPurpose": "authentication",
        "proofValue": "z4Pn..."
      }
    }
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`: the innermost
notification's `notificationId` and `revision`, the same whatever it did with the deposit. A deposit
that fails a check is answered with `trust-task-error`.

### Receipt

```json
{
  "id": "urn:uuid:8b9c0d1e-2f3a-4b4c-9d5e-6f7a8b9c0d1e",
  "type": "https://trusttasks.org/spec/vta/notifications/deposit/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-01T09:00:06Z",
  "threadId": "urn:uuid:7a8b9c0d-1e2f-4a3b-8c4d-5e6f7a8b9c0d",
  "payload": {
    "notificationId": "ntf-9Lm4q2Wx8nT5",
    "revision": 1
  }
}
```

## Security & Privacy

### Data carried

A context and one notification or relay document, whole. The document carries a reference and an
audience and nothing of the thing's content, because no notification does (VTI-NTF-003). The
receipt carries nothing the originator did not send.

### Correlation

The deposit tells the VTA nothing it would not have learned had the document been delivered to it
directly. The receipt is the same whatever the VTA did, so even the principal's own client does not
learn from it whether the policy held the notification; it learns that by listing. Nothing goes from
the VTA to the originator, a relayer or the holder because of a deposit (VTI-NTF-053), and the
client's own receipt to the sender is the one the addressee owes it under VTI-NTF-023, independent of
the deposit.

### Retention

As for a direct delivery: an accepted notification is held with `document` whole until it lapses or
is withdrawn, then discarded after the period the VTA states (VTI-NTF-052). A refused or declined
deposit is not kept beyond what deduplication needs. The client **SHOULD NOT** keep the document once
the VTA has answered.

### Consent/purpose

The deposit exists so that a notification reaches the principal's held view and the principal's
stated acceptance policy, whichever device holds the socket it arrived on. It is not a way to widen
that policy: a client that wants the principal to hear from a party changes the policy, which is the
principal's choice, and never deposits its way around it. The client **MUST NOT** open the thing
because it received or deposited the notification (VTI-NTF-053).
