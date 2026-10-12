---
slug: provision/vta/claim
version: "0.1"
title: "Provision — VTA Claim"
summary: "A wallet claims a VTA that a Farm reserved behind a vta-claim trigger link, with a fresh key that becomes the VTA's administrator; the Farm locks the reservation to that key and points the wallet at redeem for the hand-over."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [vta, farm, claim, provision, administrator, qr, trigger-link, wallet]
authors:
  - Geoff Turk (https://github.com/geoffturk)
parties:
  - role: Claimant (administrator key)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Farm
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request's proof is the whole point of the task: its issuer is the key the Farm makes an administrator of the VTA (VTI-LNK-113), so an unsigned claim would name an administrator nobody has shown control of. The next step the Farm answers with is a redirection, and the wallet acts on it only if it can authenticate the Farm as its origin; signing it with the Farm's assertionMethod key is what does that over a transport the wallet reached from a link.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The claim locks a reservation to a key, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. The handle lives at most 300 seconds (VTI-LNK-110), and an issue time lets the Farm refuse a stale claim without consulting the reservation.
sideEffects:
  level: mutating
  rationale: >-
    Moves the reservation from open to claimed in one compare-and-set and records the claimant's key as the administrator-to-be. Only one claim per handle can succeed (VTI-LNK-114). Nothing is written into the VTA by this task; that follows the person's confirmation of the claim check, and is reported by redeem.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The claim carries a handle under a fresh key. The next step names the redeem task and nothing else: no VTA DID, no account, nothing about the person who was shown the code.
retention:
  class: durable
  rationale: >-
    The claim is the record of how a VTA got its administrator: which key, when, under which handle. A Farm keeps it with the VTA's provisioning record, as it would any grant of administrative authority, whether or not the hand-over followed.
errorCodes:
  - code: provision/vta/claim:alreadyClaimed
    meaning: The handle has already been claimed by another key (VTI-LNK-114). Carries no details, so a second scanner learns nothing about the first.
    retryable: false
  - code: provision/vta:handleNotFound
    meaning: No reservation has this handle.
    retryable: false
  - code: provision/vta:handleExpired
    meaning: The handle's window (the link's `_exp`) has passed, or the reservation has otherwise ended.
    retryable: false
  - code: provision/vta:keyUnsupported
    meaning: The document's issuer is not an Ed25519 `did:key`.
    retryable: false
related:
  - provision/vta/redeem
  - auth/oob/claim
  - provision/integration
---

## Abstract

A **VTA Farm** sets up Verifiable Trust Agents for people who have none. It shows a `vta-claim` trigger link (the *Trigger Links* chapter of the Verifiable Trust Infrastructure specification, VTI-LNK-110 to VTI-LNK-117) on a page: as a QR code for a phone to scan, and as a link for a wallet on the same device to open. A wallet reads the link, resolves and verifies the Farm's DID, and once the person chooses to continue, generates a fresh Ed25519 key and sends this claim, signed by it. That key becomes the VTA's administrator. The first claim wins; every later claim for the same handle is refused.

The claim locks the reservation and nothing more. The Farm does not answer it with a `#response`, because the outcome is not yet known: the person still has to confirm, on the Farm's page, that the **claim check** shown there matches the one the wallet shows (VTI-LNK-115), and may instead say "Not mine", in which case the Farm discards the key. The Farm therefore answers a `trust-task-next-step` pointing at [`provision/vta/redeem`](../../redeem/0.1/spec.md), and the hand-over is that task's response. The family rules are in [`CONVENTIONS.md`](../../_shared/0.1/CONVENTIONS.md).

The same claim serves a VTA provisioned on demand, a device added to a running VTA, and a VTA handed out from a pre-provisioned pool. The Farm derives the operation from the reservation; the wallet sends the same document in every case.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The slug was proposed on trustoverip/dtgwg-vti-spec#58 as `vta/claim`. It lives under `provision/` because `vta/*` is the family of tasks a VTA itself consumes, and this one is consumed by a Farm.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **claimant** (the wallet):

1. **MUST** generate `K_a`, a fresh Ed25519 key, only after the person has chosen to continue, and **MUST NOT** use it for another VTA or another exchange (VTI-LNK-113).
2. **MUST** follow VTI-LNK-054: set `issuer` to `K_a` as a `did:key`, `recipient` to the Farm's DID (the link's `_from`), a unique `id`, and `parentThreadId` to the handle (the link's `_id`), and sign with `K_a` for `authentication`.
3. **MUST** set `payload.handle` to the same handle as `parentThreadId`.
4. **MUST** take the endpoint from the Farm's verified DID document, never from the link: the `serviceEndpoint` of its `TrustTaskHTTPS` service is the Trust-Task base, composed into the request URL as [HTTPS binding 0.2 §6](../../../../../bindings/https/0.2/spec.md#6-endpoint-discovery) requires. The Farm is a first contact, so nothing is resolved before the person continues (VTI-LNK-111).
5. **MUST** verify the next step's proof against a key the Farm's DID lists under `assertionMethod`, and **MUST** check that its `issuer` equals the link's `_from` and that `expects` names `provision/vta/redeem/0.1`. Any failure ends the exchange on the claimant's side, and the wallet discards `K_a`.
6. **SHOULD** show the claim check computed from `K_a`'s `did:key` string (the first six characters of the upper-case base32 encoding of its SHA-256 hash, VTI-LNK-115) and tell the person to confirm on the Farm's page, then poll `redeem` for the outcome.
7. **MUST NOT** treat the VTA as its own before a `redeem` response names it.

A conforming **Farm**:

1. **MUST** refuse an issuer that is not an Ed25519 `did:key` with `provision/vta:keyUnsupported`.
2. **MUST** refuse a document whose `parentThreadId` is absent or not equal to `payload.handle` with `malformedRequest`. This compares two members of one document; it is not a check on `parentThreadId` alone.
3. **MUST** refuse a document whose `recipient` is not the Farm's own DID with `wrongRecipient`.
4. **MUST** move the reservation from `open` to `claimed` in one compare-and-set, only while the handle's window is open, recording `K_a` as the administrator-to-be. A handle nobody reserved is refused with `provision/vta:handleNotFound`; one already claimed with `provision/vta/claim:alreadyClaimed`; one whose window has passed or whose reservation has ended with `provision/vta:handleExpired`. None of these carries details (VTI-LNK-114).
5. **MUST** treat as a claim only a document that validates and whose proof verifies (VTI-LNK-114). A request that fails any check leaves the reservation unchanged.
6. **MUST** answer a successful claim with a `trust-task-next-step` whose `expects` names `https://trusttasks.org/spec/provision/vta/redeem/0.1` with the handle as `hint`, `continuation` `proceed`, and `inResponseTo` the claim, signed with the Farm's `assertionMethod` key. It **MUST NOT** answer a claim with a `#response` of this type, and **MUST NOT** put the VTA's DID, or anything about the person who was shown the code, in the next step.
7. **MUST** show the claim check on the page once the claim is recorded (VTI-LNK-115), and **MUST NOT** write `K_a` into the VTA before the person confirms. On "Not mine", or when the hold lapses, it **MUST** discard `K_a` and leave the VTA in the pool or as it was.
8. **MUST** keep the duplicate-execution record of [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) for at least the handle's window: a bit-for-bit resend of an accepted claim gets the same next step again, and a different document under the same `id` gets `idConflict`.

## Authorization

Possession of the handle entitles a party to *claim* the reservation, once, and to nothing more. Whoever claims first becomes the administrator-to-be; the proof establishes which key that is ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), and it is the same proof that VTI-LNK-114 makes the condition for treating a request as a claim at all.

What the handle is worth depends on where it was shown. A code for an empty VTA is shown to an anonymous visitor, and a wrong claimant locks an empty reservation the visitor can discard. A code that adds a device to a running VTA is shown only to a signed-in account that already controls that VTA (VTI-LNK-117), so the authority to show it is the account's, and the claim check is what ties the key that claimed to the person who was shown the code. Neither proof on the exchange is that authorization: the request's proof says which key claimed, the next step's proof says which Farm answered, and the comparison of the two claim checks is made by a person on the Farm's page. This specification describes when a Farm releases a VTA as VTI-LNK-115 and VTI-LNK-117 already do, and does not oblige a Farm to apply any gate beyond them.

## Definitions

- **Farm** — a service that provisions VTAs for people and hands them over. The `recipient` of the claim and the `_from` of the trigger link.
- **Reservation** — a VTA, or a slot for one, that the Farm has set aside behind one handle. Its states are in [`CONVENTIONS.md`](../../_shared/0.1/CONVENTIONS.md) §3.
- **`K_a`**, the **administrator key** — the claimant's fresh Ed25519 `did:key`. The `issuer` of the claim and of every `redeem`, and the identifier the Farm writes into the VTA as its administrator.
- **`handle`** — the reservation's handle from the trigger link's `_id`, also carried as `parentThreadId`.
- **Claim check** — the first six characters of the upper-case base32 encoding of the SHA-256 hash of `K_a`'s `did:key` string, computed by the wallet from its own key and by the Farm from the key that claimed, and compared by the person (VTI-LNK-115).

## Request

The claimant (`issuer`, its administrator key) claims the reservation at the Farm (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). It carries the handle and nothing else: the administrator is the envelope's `issuer`, so there is no payload member through which a different key could be named.

### A wallet claims a scanned VTA reservation

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "type": "https://trusttasks.org/spec/provision/vta/claim/0.1",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "issuedAt": "2026-10-12T09:30:10Z",
  "payload": {
    "handle": "q3Vn7Zk2Xo9Rt1LwPb4HdA"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-12T09:30:10Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## What the Farm answers

This specification defines no `#response`. A successful claim is answered with a `trust-task-next-step` ([SPEC §4.12](/SPEC.md#412-document-lifecycle): the claim is understood and accepted, and the exchange stays open) that names [`provision/vta/redeem/0.1`](../../redeem/0.1/spec.md) as the continuation. The hand-over, with the VTA's DID, is the response to that task, and exists only once the person has confirmed the claim check and the key has been written. Failures use `trust-task-error`.

### The claim is recorded; confirm on the page

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e02",
  "type": "https://trusttasks.org/spec/trust-task-next-step/0.1",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-12T09:30:11Z",
  "payload": {
    "expects": [
      {
        "typeUri": "https://trusttasks.org/spec/provision/vta/redeem/0.1",
        "reason": "The VTA is handed over once the person confirms the claim check on the Farm's page.",
        "hint": { "handle": "q3Vn7Zk2Xo9Rt1LwPb4HdA" }
      }
    ],
    "continuation": "proceed",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/provision/vta/claim/0.1",
      "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01"
    },
    "message": "Compare the six characters on the page with the ones in your wallet, then confirm on the page."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth#key-1",
    "created": "2026-10-12T09:30:11Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Someone else claimed it first

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-12T09:30:11Z",
  "payload": {
    "code": "provision/vta/claim:alreadyClaimed",
    "message": "This code was already used by another device.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The claim carries the handle under a fresh key, and nothing else: no name, no account, no device. The next step carries the type of the task to send next, the handle as a hint, and a sentence for the person; it carries nothing about the VTA and nothing about the person who was shown the code, so a reply to a claim on a stolen code teaches the thief nothing. A wallet **MUST NOT** put anything but the handle in the request. `ext` is available on the request for ecosystem use under the framework's namespacing rules, and this specification defines no semantics for anything in it.

The documents are small and fixed in shape, so no `maxDocumentBytes` is declared. A consumer's acceptance window for the request is the handle's own: a claim carrying an `issuedAt` more than a few seconds before the code was made, or after its `_exp`, was never acceptable, and VTI-LNK-110 caps that window at 300 seconds.

### Correlation

`K_a` is fresh per claim, so claims cannot be joined to each other or to a person through the key, and the claimant declares `identifierScope: pairwise` for that reason. The key does become the VTA's administrator, which is a durable public fact in the VTA's own ACL and DID document; the point of a fresh key is that it correlates to that one VTA and to nothing before it.

The handle appears both in the payload and as `parentThreadId`, so the Farm can join the claim to the reservation it opened, which is the point. It expires within five minutes and is never reused.

The Farm's `identifierScope: public` is a real constraint. The wallet resolves the Farm's DID from the link, verifies its document, takes the endpoint from it and checks the next step's proof against a key it lists: every step needs the Farm to present the same identifier to everyone, and VTI-LNK-118 to VTI-LNK-124 additionally require that identifier to qualify under the contact rule. A pairwise Farm DID would leave the wallet nothing to verify against. The cost is that every VTA a Farm hands out is handed out by a known Farm, which is accepted: a Farm is an operator, not a person.

### Retention

The Farm keeps the claim, with the key that made it, as the record of how the VTA got its administrator, for as long as it keeps the VTA's provisioning record. The duplicate-execution record of the claim's `id` is kept for at least the handle's window, so a resend is absorbed rather than executed again. A claim that was refused, or whose claim check the person rejected, leaves nothing behind but a log line; VTI-LNK-115 says the Farm discards the identifier in that case, and this specification expects the same of `K_a` on the wallet's side. Once the hand-over has happened, the wallet keeps `K_a` for the life of the VTA: it is the administrator key.

### Consent/purpose

The handle is sent for one purpose: to lock a reservation to the key of the person who chose to continue, so that the VTA can be handed to that key and no other. The next step exists so the wallet knows what to do next and the person knows to confirm on the page. Neither document is collected for anything else. Which gate a Farm puts in front of releasing a VTA is the Farm's policy within VTI-LNK-115 and VTI-LNK-117; this specification does not set it.

## References

- Verifiable Trust Infrastructure, chapter *Trigger Links* (`VTI-LNK-*`): the `vta-claim` flow (VTI-LNK-110 to VTI-LNK-117), the first request (VTI-LNK-054) and the contact rule (VTI-LNK-118 to VTI-LNK-124). Under development at [trustoverip/dtgwg-vti-spec#58](https://github.com/trustoverip/dtgwg-vti-spec/pull/58).
- [Trust Task HTTPS binding 0.2](../../../../../bindings/https/0.2/spec.md): the `TrustTaskHTTPS` service and the Trust-Task base.
- [`trust-task-next-step/0.1`](../../../../trust-task-next-step/0.1/spec.md): the reply a successful claim receives.
- [`auth/oob/claim`](../../../../auth/oob/claim/0.1/spec.md): the same first-request shape for `sign-in`, which this task mirrors.
