---
slug: notifications/notify
version: "0.1"
title: "Notifications — Notify"
summary: "Tell the party a DID names — an individual, group, community or network — that a thing exists which it, or those it stands for, may take part in: where it is held, how to open it, whom it concerns. Signed by the originator, so it can be relayed and still verified."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - notify
  - inbox
  - announcement
  - election
parties:
  - role: Originator
    requirement: REQUIRED
    member: issuer
  - role: Addressee — an individual, group, community or network, by its DID
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: OPTIONAL
  rationale: >-
    A notification is relayed to parties that never had a session with its originator, and an
    authenticated transport tells each hop only who the previous hop was. The originator's proof is
    the only evidence that survives a relay, so it is what lets a final recipient tell who said the
    thing exists from who passed the message on (VTI-NTF-001). The receipt says only that the
    document arrived, which the transport already attributes to the recipient.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A recipient records the notification, so it must be able to place the document in a bounded
    window to absorb a repeat (SPEC §7.2 item 11); and a copy held by a mediator for longer than the
    recipient's acceptance window is re-issued under the same notificationId rather than accepted
    late (VTI-TRN-046).
sideEffects:
  level: mutating
  rationale: >-
    The recipient may hold a record of the notification, replacing one it holds for a lower
    revision. Nothing is granted, and a repeat of the same revision changes nothing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a reference — the DID holding the thing, its identifier there, the task that opens it — and
    the audience DID, never the thing's content. Out: a receipt that is the same whatever the
    recipient did with it.
retention:
  class: durable
  rationale: >-
    A recipient's agent holds an accepted notification until it lapses or is withdrawn, so that its
    principal can see what they were told they may take part in (VTI-NTF-050). The document is kept
    whole, because a relay of it, or a later read by the principal's client, must be able to verify
    the originator's proof.
maxDocumentBytes:
  request: 16384
  response: 4096
  rationale: >-
    A fixed-shape reference of at most a few DIDs and identifiers, plus the envelope and one proof.
errorCodes: []
related:
  - notifications/relay
  - vta/notifications/list
  - vta/notifications/show
  - vta/notifications/dismiss
  - vtc/elections/show
  - vtc/elections/list
---

## Abstract

The **Notifications — Notify** Trust Task tells the party a DID names — an individual, a group,
a community or a network — that a thing exists which it, or the parties it stands for, may act
on or take part in. It says where the thing is held, its identifier there, the Trust Task that
opens it, and the audience it concerns: *election `board-2027` at community C is open to the
members of party P; open it with `vtc/elections/show`*. It carries nothing of the thing itself.

The task is generic. What the thing is — an election, or anything else — is the notification's
`kind`, drawn from an extensible [registry](#kind-registry). An addressee that stands for others
decides for itself whether and how to pass a notification on; when it does, it uses
[`notifications/relay`](../../relay/0.1/spec.md), which carries this document unchanged so the
final recipient can still verify its originator. A member's agent holds the notifications it
accepts, and its principal's client reads them with
[`vta/notifications/list`](../../../vta/notifications/list/0.1/spec.md),
[`vta/notifications/show`](../../../vta/notifications/show/0.1/spec.md) and
[`vta/notifications/dismiss`](../../../vta/notifications/dismiss/0.1/spec.md).

The requirements this task implements are those of the *Notifications* chapter of the
[Verifiable Trust Infrastructure specification](https://trustoverip.github.io/dtgwg-vti-spec/)
(VTI-NTF-001 – VTI-NTF-063).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the originator) **MUST**:

1. Address the document to exactly one DID, as `recipient`, and only to a party the thing
   concerns or a party that stands for those it concerns (VTI-NTF-005).
2. Sign the document (`proof`) as `issuer`, whatever transport carries it (VTI-NTF-001).
3. Choose a `notificationId` unique among the notifications it issues, and carry the same
   `notificationId` and `revision` in every attempt at one delivery and in every new document it
   issues for it (VTI-NTF-021). A retry is a new document with a new `id` and the same
   notification; it is never the old document re-stamped.
4. Carry in `subject` only where the thing is held, its identifier there and the task that
   opens it, and carry nothing of the thing's content, nor who else was notified or how many
   (VTI-NTF-003).
5. State `relayable` explicitly, and set it `false` for a kind whose definition forbids relaying.
6. Change or withdraw a notification only by sending the same addressee a higher `revision`
   under the same `notificationId`; to withdraw it, with `status` `withdrawn` (VTI-NTF-024).
7. Treat the notification as delivered only on the evidence VTI-TRN-040 requires; a receipt
   (`#response`) is a correlated protocol reply. **MUST NOT** ask a relayer whom it relayed to.

A conforming **consumer** (the addressee, or its agent) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), including that it
   is the `recipient` and that `proof` verifies against `issuer`, and discard a document that
   fails (VTI-NTF-032).
2. Decide whether to hold it by its stated acceptance policy (VTI-NTF-030). A VTA's policy, unless
   its principal chose otherwise, holds a notification only from a party with which the principal
   has a relationship: a community, group or network whose membership credential the principal
   holds; a party holding an access control entry at the VTA; or a party the principal chose to
   hear from.
3. Hold at most one record per `issuer` and `notificationId`: replace it with a higher `revision`,
   leave it for an equal or lower one (VTI-NTF-022), and discard a notification whose
   `validUntil` has passed.
4. Answer a document it verified with a receipt that echoes `notificationId` and `revision`, the
   same whether it held the notification, declined it or relayed it, and saying none of these
   (VTI-NTF-023). It **MAY** send no answer at all to an `issuer` it has no relationship with.
5. **MUST NOT** act on a notification of a kind it does not recognise beyond holding, listing and
   discarding it (VTI-NTF-012); **MUST NOT** treat any notification as evidence that it, or
   anyone, may take part in the thing (VTI-NTF-004); and **MUST NOT**, in consequence of
   receiving it, send anything to the originator or to `subject.heldBy`, or open the thing except
   at its principal's choice (VTI-NTF-053).

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless. The
idempotency key is the pair of `issuer` and `notificationId`, with `revision`; a repeat is
absorbed by consumer item 3 and answered with the same receipt (VTI-OPS-065).

**A repeated request** sets a state: a notification already held at the same revision
succeeds, with the same receipt, and changes nothing.

## Authorization

**None beyond identity, and the recipient's acceptance policy is the gate.** Any party may send a
notification to any DID, because a notification grants nothing: it is not a credential, an
invitation that admits, or evidence of eligibility, and whether a recipient may take part in the
thing is decided by `subject.heldBy` when the thing is opened, as if no notification had been sent
(VTI-NTF-004). The proof establishes who the originator is and that the notification is unaltered
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)) — which is what a recipient's
acceptance policy needs, since that policy is decided by the originator's identity — and is not
itself authority. What stands between an inbox and anyone with a DID is consumer item 2: a
recipient holds only notifications from parties it already has a relationship with, and bounds what
it spends on the rest (VTI-NTF-033).

## Definitions

- **`notification`** — the notification ([`Notification`](../../_shared/0.1/notification.schema.json)):
  - **`notificationId`** — the originator's identifier for it, unique among the notifications
    that originator issues, and the same across retries, re-issued documents and relays.
  - **`revision`** — starts at 1; a higher one replaces a lower one.
  - **`kind`** — what the thing is: a registered name from the [kind registry](#kind-registry),
    or `<reverse-DNS domain>:<name>` under a domain the kind's definer controls.
  - **`subject`** — **`heldBy`**, the DID holding the thing; **`id`**, its identifier there;
    **`openWith`**, the Type URI of the task that opens it.
  - **`audience`** — the DID whose holders or members the thing concerns.
  - **`status`** — `available`, or `withdrawn`.
  - **`relayable`** — whether the addressee may relay it.
  - **`validFrom`**, **`validUntil`** — optionally, the period in which the thing can be acted on.
    The notification lapses at `validUntil`, or as its kind's definition says.
- **Originator** — the `issuer`. **Addressee** — the `recipient`.

## Kind registry

A kind is registered by adding a row to this table in a pull request to this registry, naming
the definition that states, for the kind (VTI-NTF-011): what the thing is; the task that opens
it; what `audience` means; what a notification of the kind may carry beyond the members above —
nothing, unless the definition says otherwise; whether it may be relayed; and when it lapses. A
kind defined elsewhere uses a namespaced name (`org.example:meetingInvite`) and needs no row.

| Kind | Thing | `subject.heldBy` / `subject.id` | `subject.openWith` | `audience` | Relayable | Lapses | Defined by |
|---|---|---|---|---|---|---|---|
| `election` | an election, poll or survey of a community | the community's DID / the election's `electionId` | [`vtc/elections/show`](../../../vtc/elections/show/0.5/spec.md) | the DID the election's credential roll names (`roll.memberOf`), or the community's own DID under any other roll rule | yes | at `validUntil`, the close of voting | VTI-NTF-060 – VTI-NTF-063 |

**The `election` kind.** A community **SHOULD** send one to the party a credential roll names
when that party is itself a member of the community, once the election is registered with its
ballot box, and to no other party; and **MUST** withdraw it if the election is cancelled before
voting closes (VTI-NTF-061). Such an election is never in the community's
[`vtc/elections/list`](../../../vtc/elections/list/0.5/spec.md), not even its identifier
(VTI-VOT-015), so this notification — sent to the party, which alone holds the roster, and relayed
by it to its members — is how those members learn of it. It carries `validFrom` and `validUntil`
as the voting period, and nothing else of the election: no title, questions, options or roll
size. A member's agent shows a notified election as notified — community, identifier, audience,
period — until the member opens it with `vtc/elections/show`, presenting their membership
credential for the audience only by their own choice (VTI-VOT-016, VTI-NTF-063).

## Request

The originator sends the addressee the top-level schema in [`payload.schema.json`](payload.schema.json).

### A community tells a member party of an election put to its members

Community `did:web:community.example` holds election `board-2027`, whose credential roll names
`did:web:partner.example`, a community that is itself its member. It tells the partner, and lets
it relay.

```json
{
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
```

### The election is cancelled: revision 2 withdraws it

```json
{
  "id": "urn:uuid:8a0d33b2-11c4-4f5e-8d0a-6b2c9e41f7d0",
  "type": "https://trusttasks.org/spec/notifications/notify/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:web:partner.example",
  "issuedAt": "2027-03-10T14:30:00Z",
  "payload": {
    "notification": {
      "notificationId": "ntf-7Qk2p9Xv3mR8",
      "revision": 2,
      "kind": "election",
      "subject": {
        "heldBy": "did:web:community.example",
        "id": "board-2027",
        "openWith": "https://trusttasks.org/spec/vtc/elections/show/0.5"
      },
      "audience": "did:web:partner.example",
      "status": "withdrawn",
      "relayable": true
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-10T14:30:00Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3Hq..."
  }
}
```

## Response

The addressee answers with the sub-schema reachable via `$anchor: "response"`: `notificationId`
and `revision`, echoed. The receipt is identical whether the addressee held the notification,
declined it or relayed it. A document that fails validation is answered with `trust-task-error`,
not a `#response`.

### Receipt

```json
{
  "id": "urn:uuid:c41e9f07-5b2a-4a8d-a3f6-2e7d90b1c512",
  "type": "https://trusttasks.org/spec/notifications/notify/0.1#response",
  "issuer": "did:web:partner.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2027-03-01T09:00:02Z",
  "threadId": "urn:uuid:2f8c1a40-7d1e-4b8e-9a51-0c6f3e2d7a11",
  "payload": {
    "notificationId": "ntf-7Qk2p9Xv3mR8",
    "revision": 1
  }
}
```

## Security & Privacy

### Data carried

A reference and an audience: the DID holding the thing, the thing's identifier there, the task
that opens it, and the DID whose holders or members it concerns, with an optional period. The
schema admits nothing else — no title, description or free text — so a notification relayed to
every member of a network before any of them has shown they may see the thing tells them only
where to look (VTI-NTF-003). An originator **MUST NOT** put the thing's content, or who else was
notified, in `ext`, which is part of its signed surface and travels as far as the notification.
`subject.id` and `notificationId` are chosen by the originator and **SHOULD NOT** be descriptive,
for the same reason. The receipt carries nothing the originator did not send.

### Correlation

The notification names the originator, the holder and the audience, so a recipient learns that
the holder holds something for the audience — for an election, that the community holds an
election and that the audience is its member. A relayer that passes it on tells each party it
reaches the same, which is the relayer's decision (VTI-NTF-040). `notificationId` is the same in
every copy, by design: it is what lets one notification arriving by two paths be one record. An
originator learns from receipts only that the addressee received the notification, never what it
did with it (VTI-NTF-023), and receives no receipts from a relay's recipients (VTI-NTF-043), so it
cannot assemble the audience's roster from its deliveries.

### Retention

A recipient keeps an accepted notification, whole and with its proof, until it lapses or is
withdrawn, and then **SHOULD** discard it after a period it states (VTI-NTF-052). A declined one
is not kept beyond what deduplication needs. The originator keeps its own record of what it sent
and to whom, which is its business and no more than it already knew.

### Consent/purpose

The notification exists to tell a party that it may take part in something. A recipient
**MUST NOT** use it to infer, or to show anyone, that the recipient is eligible, and its agent
**MUST NOT** open the thing — which, for an election under a credential roll, shows the holder the
reader's membership — except at the principal's choice (VTI-NTF-053, VTI-VOT-016).
