---
slug: notifications/relay
version: "0.1"
title: "Notifications — Relay"
summary: "A group, community or network passes a notification it was addressed to on to one of the parties it stands for, carrying the originator's signed document unchanged so the final recipient can verify who originated it."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - relay
  - cascade
  - forward
  - election
parties:
  - role: Relayer — the addressee of the notification, passing it on
    requirement: REQUIRED
    member: issuer
  - role: Relay recipient — a party the relayer stands for
    requirement: REQUIRED
    member: recipient
  - role: Originator — the issuer of the innermost notifications/notify document
    requirement: REQUIRED
proofRequirement:
  request: REQUIRED
  response: OPTIONAL
  rationale: >-
    A relay may itself be relayed — a network to a community, the community to a group, the group
    to its member — and the transport tells each hop only who the previous hop was. Each relay's
    proof is what lets the final recipient check, layer by layer, that every relayer was the party
    the layer inside was addressed to (VTI-NTF-042). The receipt says only that the document
    arrived.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The recipient records the notification it carries, so it must be able to place the relay in a
    bounded window to absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    The recipient may hold a record of the innermost notification, as notifications/notify does.
    Nothing is granted, and a repeat changes nothing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the originator's notification, whole, and the relayers it passed through. Out: a receipt
    that is the same whatever the recipient did with it.
retention:
  class: durable
  rationale: >-
    A recipient's agent holds the relay whole, so that its principal's client can verify the
    originator's proof and every relay itself when it reads the notification (VTI-NTF-050).
maxDocumentBytes:
  request: 98304
  response: 4096
  rationale: >-
    At most four layers, each a notification or relay document of at most 16 KiB with its proof,
    plus this envelope.
errorCodes:
  - code: notifications/relay:chainInvalid
    meaning: >-
      A layer of the relay does not hold: a relay's or the notification's proof does not verify, a
      forwarded document is not addressed to the issuer of the relay carrying it, the innermost
      document is not a notification, the notification may not be relayed, or the relay carries
      more than four relays in all.
    retryable: false
related:
  - notifications/notify
  - vta/notifications/list
  - vta/notifications/show
---

## Abstract

The **Notifications — Relay** Trust Task is how a party that stands for others — a group, a
community, a network — passes on a [`notifications/notify`](../../notify/0.1/spec.md) document it
was addressed to. It carries the document it received **exactly as received, proof included**,
inside a relay signed by the relayer and addressed to one party the relayer stands for. The final
recipient therefore holds the originator's own signed notification and can verify it, as well as
who passed it on.

Whether to relay, to whom, when and by which means is the relayer's decision; this specification
defines only the properties one relay must have (VTI-NTF-040). A relay may itself be relayed: a
network relays to a community it lists, the community to one of its groups, the group to its
member. Each layer carries the one inside it whole.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the relayer) **MUST**:

1. Relay only a document it received and verified that is addressed to it — its `recipient` is
   the relayer's own DID — and carry that document in `forwarded` exactly as received, proof
   included, never altered, restated or re-signed (VTI-NTF-041).
2. **MUST NOT** relay a notification whose `relayable` is `false`, or whose kind may not be
   relayed, and **MUST NOT** relay a relay that already carries three relays (so that its own
   would make more than four).
3. Address each relay to one party and sign it as `issuer`.
4. **MUST NOT** tell the originator, or any earlier relayer, whom it relayed to, how many, or
   whether it relayed at all (VTI-NTF-043).

A conforming producer **SHOULD** apply its own acceptance policy before relaying, since its
recipients accept the relay on the relayer's standing, and **SHOULD** relay a later revision it
accepts — a withdrawal included — to every party it relayed an earlier revision to (VTI-NTF-044).

A conforming **consumer** (the relay's recipient, or its agent) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), including that it is
   the `recipient` and that `proof` verifies against `issuer`.
2. Verify every layer of `forwarded` in turn (VTI-NTF-042): that its `type` is
   `https://trusttasks.org/spec/notifications/notify/0.1` or
   `https://trusttasks.org/spec/notifications/relay/0.1`, and that it validates as a document of
   that type — without re-applying its acceptance window or `expiresAt`, which bound when it could
   be delivered to its own recipient, not how long it remains evidence; that its `recipient` is
   the `issuer` of the relay carrying it; that its `proof` verifies against its own `issuer`, over
   the document exactly as received; and that the innermost layer is a `notifications/notify`
   document whose `relayable` is `true`. It refuses a relay that fails any of these, or that
   carries more than four relays in all counting its own, with
   `notifications/relay:chainInvalid`, and holds nothing of it (VTI-NTF-032).
3. Decide whether to hold the innermost notification by its acceptance policy, counting its
   relationship with the relay's `issuer` — the relayer that delivered it — never with the
   originator (VTI-NTF-031).
4. Thereafter treat the innermost notification exactly as
   [`notifications/notify`](../../notify/0.1/spec.md) consumer items 3 to 5 require: one record
   per originator and `notificationId`, a receipt that says nothing of what it did, and no action
   beyond holding it unless the principal chooses.
5. Read the relay as the relayer's statement that it passed the originator's notification on,
   and never as the relayer's statement about the thing (VTI-NTF-045).

**Retry class** (VTI-OPS-060): the second, as for `notifications/notify`. The idempotency key is
the innermost notification's originator and `notificationId`, with `revision`: one notification
reaching a recipient by two relayers, or by a relayer and directly, is one record.

## Authorization

**The relayer's standing is being the notification's addressee; the recipient's acceptance policy
is the gate.** A relayer needs no authority from the originator to relay — a notification grants
nothing, and its `relayable` is the originator's statement that relaying is expected. What a
recipient checks is that each relayer was the party the layer inside was addressed to (consumer
item 2), which proves the relay was passed on by parties the notification was given to and not
by a party that obtained a copy. The proofs establish who signed each layer
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); none of them is authority over the
recipient, which holds the notification only if it has a relationship with the relayer that
delivered it (VTI-NTF-030, VTI-NTF-031).

## Definitions

- **`forwarded`** — the document the relayer received, whole: a `notifications/notify` document,
  or a `notifications/relay` document carrying the next layer in.
- **Relayer** — the `issuer`. **Originator** — the `issuer` of the innermost document.
- **Layer** — one document in the chain, from this relay inward to the notification.

## Request

The relayer sends one party it stands for the top-level schema in
[`payload.schema.json`](payload.schema.json).

### A member party relays an election notification to one of its members

`did:web:partner.example` received the [example notification](../../notify/0.1/spec.md#a-community-tells-a-member-party-of-an-election-put-to-its-members)
from `did:web:community.example` and relays it to its member `did:key:z6MkMember`.

```json
{
  "id": "urn:uuid:5e7b2c91-0a3f-4d68-b1e4-93c0f8a27d65",
  "type": "https://trusttasks.org/spec/notifications/relay/0.1",
  "issuer": "did:web:partner.example",
  "recipient": "did:key:z6MkMember",
  "issuedAt": "2027-03-01T11:20:00Z",
  "payload": {
    "forwarded": {
      "id": "urn:uuid:2f8c1a40-7d1e-4b8e-9a51-0c6f3e2d7a11",
      "type": "https://trusttasks.org/spec/notifications/notify/0.1",
      "issuer": "did:web:community.example",
      "recipient": "did:web:partner.example",
      "issuedAt": "2027-03-01T09:00:00Z",
      "expiresAt": "2027-03-01T10:00:00Z",
      "payload": {
        "notification": {
          "notificationId": "ntf-7Qk2p9Xv3mR8",
          "revision": 1,
          "kind": "election",
          "subject": {
            "heldBy": "did:web:community.example",
            "id": "board-2027",
            "openWith": "https://trusttasks.org/spec/vtc/elections/show/0.5"
          },
          "audience": "did:web:partner.example",
          "status": "available",
          "relayable": true,
          "validFrom": "2027-03-08T00:00:00Z",
          "validUntil": "2027-03-15T00:00:00Z"
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2027-03-01T09:00:00Z",
        "verificationMethod": "did:web:community.example#key-1",
        "proofPurpose": "assertionMethod",
        "proofValue": "z5Lk..."
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T11:20:00Z",
    "verificationMethod": "did:web:partner.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Rw..."
  }
}
```

The forwarded document's `expiresAt` has passed by the time the member receives it, and that is
expected: the acceptance window applies to the relay, the outer document, and the forwarded one is
evidence of what the originator said, verified by its proof. Its period is `validUntil`.

## Response

The recipient answers with the sub-schema reachable via `$anchor: "response"`: the innermost
notification's `notificationId` and `revision`, the same whatever the recipient did with it. A
relay that fails a layer is answered with `trust-task-error` carrying
`notifications/relay:chainInvalid`.

### Receipt

```json
{
  "id": "urn:uuid:d7f30a12-6c4e-48b1-9e25-1a3b5c7d9e0f",
  "type": "https://trusttasks.org/spec/notifications/relay/0.1#response",
  "issuer": "did:key:z6MkMember",
  "recipient": "did:web:partner.example",
  "issuedAt": "2027-03-01T11:20:01Z",
  "threadId": "urn:uuid:5e7b2c91-0a3f-4d68-b1e4-93c0f8a27d65",
  "payload": {
    "notificationId": "ntf-7Qk2p9Xv3mR8",
    "revision": 1
  }
}
```

## Security & Privacy

### Data carried

The originator's notification, whole, and each relayer's signed layer around it. A relayer adds
nothing of its own — the payload admits only `forwarded` and `ext` — and **MUST NOT** put in `ext`
anything about the thing, the originator, or whom else it relayed to. A relayer that wants to say
something about the thing sends a notification of its own, as its originator.

### Correlation

A relay tells its recipient that the relayer received the notification and chose to pass it on,
and every layer names the party it was addressed to, so a member learns the chain from the
originator to itself — for an election put to a member party, that the community holds the
election and that the party is the community's member. That is the relayer's disclosure to make,
and the reason relaying is never required (VTI-NTF-040). In the other direction, receipts go only
to the relayer, which reports nothing to the originator (VTI-NTF-043): the originator cannot learn
the audience's roster by relaying through it.

### Retention

As for [`notifications/notify`](../../notify/0.1/spec.md#retention): a recipient keeps the relay
whole until the innermost notification lapses or is withdrawn, so that every layer stays
verifiable, then discards it after a period it states.

### Consent/purpose

The relay serves the same purpose as the notification it carries, and no other. A relayer decides
whom to pass a notification to from its own records, and **MUST NOT** use the act of relaying to
learn anything of its recipients — whether they opened the thing, or took part in it — beyond the
receipt.
