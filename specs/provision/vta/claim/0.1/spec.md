---
slug: provision/vta/claim
version: "0.1"
title: "Provision — VTA Claim"
summary: "A wallet claims a VTA that a Farm reserved behind a vta-claim trigger link, with a fresh key that becomes the VTA's administrator, and gets back the Farm-signed VTA DID and what the claim did."
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
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request's proof is the whole point of the task: its issuer is the key the Farm makes an administrator of the VTA (VTI-LNK-113), so an unsigned claim would name an administrator nobody has shown control of. The response's proof, by the Farm's assertionMethod key, is what lets the wallet treat the VTA DID and the operation as the Farm's statement rather than as text from whoever answered at the endpoint.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The claim locks a reservation to a key, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. The handle lives at most 300 seconds (VTI-LNK-110), and an issue time lets the Farm refuse a stale claim without consulting the reservation.
sideEffects:
  level: mutating
  rationale: >-
    Moves the reservation from open to claimed in one compare-and-set and records the claimant's key as the VTA's administrator-to-be. Only one claim per handle can succeed (VTI-LNK-114). Writing the key into the VTA, or starting its provisioning, follows the claim check where the Farm applies one; both are recoverable while the reservation lasts.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The claim carries a handle under a fresh key. The response names the VTA's DID, what the claim did and how to follow it: facts about a VTA the claimant now administers, and nothing about anyone else.
retention:
  class: durable
  rationale: >-
    The claim is the record of how a VTA got its administrator: which key, when, under which handle. A Farm keeps it with the VTA's provisioning record, as it would any grant of administrative authority.
errorCodes:
  - code: provision/vta/claim:alreadyClaimed
    meaning: The handle has already been claimed by another key (VTI-LNK-114). Carries no details, so a second scanner learns nothing about the first.
    retryable: false
  - code: provision/vta/claim:handleNotFound
    meaning: No reservation has this handle.
    retryable: false
  - code: provision/vta/claim:handleExpired
    meaning: The handle's window (the link's `_exp`) has passed, or the reservation has otherwise ended.
    retryable: false
  - code: provision/vta/claim:keyUnsupported
    meaning: The document's issuer is not an Ed25519 `did:key`.
    retryable: false
related:
  - auth/oob/claim
  - provision/integration
---

## Abstract

A **VTA Farm** sets up Verifiable Trust Agents for people who have none. It shows a `vta-claim` trigger link (the *Trigger Links* chapter of the Verifiable Trust Infrastructure specification, VTI-LNK-110 to VTI-LNK-117) on a page: as a QR code for a phone to scan, and as a link for a wallet on the same device to open. A wallet reads the link, resolves and verifies the Farm's DID, and once the person chooses to continue, generates a fresh Ed25519 key and sends this claim, signed by it. That key becomes the VTA's administrator. The first claim wins; every later claim for the same handle is refused.

The Farm answers with the VTA's DID, what the claim did (`provisionVta` for a new VTA, `grantAcl` for a device added to a running one), and whether it is holding the VTA until the person confirms the **claim check** of VTI-LNK-115 on the page. The response carries no claim check of its own: the wallet computes it from the key it signed with, and the person compares the two.

The same task serves a VTA provisioned on demand, a device added to a running VTA, and a VTA handed out from a pre-provisioned pool. The Farm derives the operation from the reservation; the wallet sends the same document in every case.

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
5. **MUST** verify the response's proof against a key the Farm's DID lists under `assertionMethod`, and **MUST** check that `handle` equals the one it sent and that `issuer` equals the link's `_from`. Any failure ends the exchange on the claimant's side, and the wallet discards `K_a`.
6. **SHOULD** show the claim check computed from `K_a`'s `did:key` string (the first six characters of the upper-case base32 encoding of its SHA-256 hash, VTI-LNK-115), and, when `checkRequired` is `true`, tell the person to confirm on the Farm's page before `confirmBy`.
7. **MUST NOT** treat the VTA as its own until the Farm has released it: when `checkRequired` is `true`, that is after the person confirms on the page, which the wallet learns by following `progress` or by the VTA answering.

A conforming **Farm**:

1. **MUST** refuse an issuer that is not an Ed25519 `did:key` with `provision/vta/claim:keyUnsupported`.
2. **MUST** refuse a document whose `parentThreadId` is absent or not equal to `payload.handle` with `malformedRequest`. This compares two members of one document; it is not a check on `parentThreadId` alone.
3. **MUST** refuse a document whose `recipient` is not the Farm's own DID with `wrongRecipient`.
4. **MUST** move the reservation from open to claimed in one compare-and-set, only while the handle's window is open, recording `K_a` as the administrator-to-be. A handle nobody reserved is refused with `provision/vta/claim:handleNotFound`; one already claimed with `provision/vta/claim:alreadyClaimed`; one whose window has passed or whose reservation has ended with `provision/vta/claim:handleExpired`. None of these carries details (VTI-LNK-114).
5. **MUST** treat as a claim only a document that validates and whose proof verifies (VTI-LNK-114). A request that fails any check leaves the reservation unchanged.
6. **MUST** answer with the response below, signed by its `assertionMethod` key, and **MUST** set `operation` from the reservation, never from anything the claimant sent.
7. **MUST** keep the duplicate-execution record of [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) for at least the handle's window: a bit-for-bit resend of an accepted claim gets the first response again, and a different document under the same `id` gets `idConflict`.
8. **SHOULD** apply the claim check of VTI-LNK-115 before releasing the VTA, and **MUST** apply it for `grantAcl` (VTI-LNK-117). The response's `checkRequired` says which the Farm did. Where the Farm holds the VTA, it **MUST NOT** write `K_a` into the VTA, or hand the VTA over, before the person confirms, and **MUST** discard `K_a` and return the VTA to the pool if the person says the checks do not match or `confirmBy` passes.

## Authorization

Possession of the handle entitles a party to *claim* the reservation, once, and to nothing more. Whoever claims first becomes the administrator-to-be; the proof establishes which key that is ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), and it is the same proof that VTI-LNK-114 makes the condition for treating a request as a claim at all.

What the handle is worth depends on where it was shown. A code for an empty VTA is shown to an anonymous visitor, and a wrong claimant gains an empty agent and a reservation the visitor can discard. A code that adds a device to a running VTA is shown only to a signed-in account that already controls that VTA (VTI-LNK-117), so the authority to show it is the account's, and the claim check is what ties the key that claimed to the person who was shown the code. Neither proof on the exchange is that authorization: the request's proof says which key claimed, the response's proof says which Farm answered, and the comparison of the two claim checks is made by a person on the Farm's page. This specification describes that comparison as the Farm reports it (`checkRequired`) and does not oblige a Farm to apply it beyond what VTI-LNK-115 and VTI-LNK-117 already say.

## Definitions

- **Farm** — a service that provisions VTAs for people and hands them over. The `recipient` of the claim and the `_from` of the trigger link.
- **Reservation** — a VTA, or a slot for one, that the Farm has set aside behind one handle. Open until claimed, lapsed when the handle's window passes.
- **`K_a`**, the **administrator key** — the claimant's fresh Ed25519 `did:key`. The `issuer` of the claim, and the identifier the Farm writes into the VTA as its administrator.
- **`handle`** — the reservation's handle from the trigger link's `_id`, also carried as `parentThreadId`.
- **Claim check** — the first six characters of the upper-case base32 encoding of the SHA-256 hash of `K_a`'s `did:key` string, computed by the wallet from its own key and by the Farm from the key that claimed, and compared by the person (VTI-LNK-115).
- **Operation** — what the claim did at the Farm: `provisionVta` or `grantAcl`.

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

## Response

The Farm answers with the result of the claim, signed by its `assertionMethod` key. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

- `handle` — the handle the claim named.
- `vtaDid` — the DID of the VTA the claimant now administers, or will once the Farm releases it.
- `operation` — `provisionVta` when the claimant is the first administrator of a VTA being set up for this reservation; `grantAcl` when the claimant is added to a VTA that already has one. The Farm derives it from the reservation.
- `checkRequired` — `true` when the Farm holds the VTA until the person confirms the claim check on the page; always `true` for `grantAcl`.
- `confirmBy` — when that hold lapses. Present when `checkRequired` is `true`.
- `mediatorDid`, `didLogUrl` — where the VTA will be reachable and where its DID log is served. Conveniences; both are also derivable from the VTA's DID document once it resolves.
- `progress` — a URL the wallet can poll to follow the provisioning or the grant, and when it stops answering. The Farm's own interface; its shape is not part of this specification.

### A new VTA, held for the claim check

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e02",
  "type": "https://trusttasks.org/spec/provision/vta/claim/0.1#response",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-12T09:30:11Z",
  "payload": {
    "handle": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
    "vtaDid": "did:webvh:QmYyWfX2gJmZk9zNqV8pL3tRbH4sKc6dEa7uF1nG5oP2wQ:dids.firstperson.dev:alice-vta",
    "operation": "provisionVta",
    "checkRequired": true,
    "confirmBy": 1791626860,
    "mediatorDid": "did:webvh:QmTr5sUv2wXy8zAb3cDe6fGh9iJk1lMn4oPq7rSt0uVw3x:dids.firstperson.dev:platform-mediator",
    "didLogUrl": "https://dids.firstperson.dev/alice-vta/did.jsonl",
    "progress": {
      "url": "https://vtafarm-api.firstperson.dev/api/v1/mobile-connections/progress/4f9c2e7a1b8d3c6e5f0a9b2c",
      "expiresAt": 1791630400
    }
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

### A device added to a running VTA

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e12",
  "type": "https://trusttasks.org/spec/provision/vta/claim/0.1#response",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e11",
  "parentThreadId": "Zp8kQ2mN5vXr3tLw7bHd1A",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7hJMwvyWBzda",
  "issuedAt": "2026-10-12T14:02:33Z",
  "payload": {
    "handle": "Zp8kQ2mN5vXr3tLw7bHd1A",
    "vtaDid": "did:webvh:QmYyWfX2gJmZk9zNqV8pL3tRbH4sKc6dEa7uF1nG5oP2wQ:dids.firstperson.dev:alice-vta",
    "operation": "grantAcl",
    "checkRequired": true,
    "confirmBy": 1791643200,
    "mediatorDid": "did:webvh:QmTr5sUv2wXy8zAb3cDe6fGh9iJk1lMn4oPq7rSt0uVw3x:dids.firstperson.dev:platform-mediator"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth#key-1",
    "created": "2026-10-12T14:02:33Z",
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

The claim carries the handle under a fresh key, and nothing else: no name, no account, no device. The response carries the VTA's DID, what the claim did, whether the Farm is holding the VTA and until when, and where to follow progress. All of it is about a VTA the claimant is about to administer, and none of it is about anyone else. The `progress.url` is the Farm's own interface and **MUST NOT** carry userinfo; where it carries a capability in its path, that capability is for reading progress and nothing more. A Farm **MUST NOT** put anything about the person who was shown the code in the response, and a wallet **MUST NOT** put anything but the handle in the request. `ext` is available on both documents for ecosystem use under the framework's namespacing rules, and this specification defines no semantics for anything in it.

The documents are small and fixed in shape, so no `maxDocumentBytes` is declared. A consumer's acceptance window for the request is the handle's own: a claim carrying an `issuedAt` more than a few seconds before the code was made, or after its `_exp`, was never acceptable, and VTI-LNK-110 caps that window at 300 seconds.

### Correlation

`K_a` is fresh per claim, so claims cannot be joined to each other or to a person through the key, and the claimant declares `identifierScope: pairwise` for that reason. The key does become the VTA's administrator, which is a durable public fact in the VTA's own ACL and DID document; the point of a fresh key is that it correlates to that one VTA and to nothing before it.

The handle appears both in the payload and as `parentThreadId`, so the Farm can join the claim to the reservation it opened, which is the point. It expires within five minutes and is never reused.

The Farm's `identifierScope: public` is a real constraint. The wallet resolves the Farm's DID from the link, verifies its document, takes the endpoint from it and checks the response's proof against a key it lists: every step needs the Farm to present the same identifier to everyone, and VTI-LNK-118 to VTI-LNK-124 additionally require that identifier to qualify under the contact rule. A pairwise Farm DID would leave the wallet nothing to verify against. The cost is that every VTA a Farm hands out is handed out by a known Farm, which is accepted: a Farm is an operator, not a person.

### Retention

The Farm keeps the claim, with the key that made it, as the record of how the VTA got its administrator, for as long as it keeps the VTA's provisioning record. The duplicate-execution record of the claim's `id` is kept for at least the handle's window, so a resend is absorbed rather than executed again. A claim that was refused, or whose claim check the person rejected, leaves nothing behind but a log line; VTI-LNK-115 says the Farm discards the identifier in that case, and this specification expects the same of `K_a` on the wallet's side. The wallet keeps `K_a` for the life of the VTA: it is the administrator key.

### Consent/purpose

The handle is sent for one purpose: to lock a reservation to the key of the person who chose to continue, so that the VTA is handed to that key and no other. The response exists so the wallet can show the person which VTA they are about to administer and what to do next. Neither document is collected for anything else, and the Farm has no use for the claim once the VTA is handed over except as the record described above. Which gate a Farm puts in front of releasing a VTA is the Farm's policy, reported through `checkRequired`; this specification does not set it.

## References

- Verifiable Trust Infrastructure, chapter *Trigger Links* (`VTI-LNK-*`): the `vta-claim` flow (VTI-LNK-110 to VTI-LNK-117), the first request (VTI-LNK-054) and the contact rule (VTI-LNK-118 to VTI-LNK-124). Under development at [trustoverip/dtgwg-vti-spec#58](https://github.com/trustoverip/dtgwg-vti-spec/pull/58).
- [Trust Task HTTPS binding 0.2](../../../../../bindings/https/0.2/spec.md): the `TrustTaskHTTPS` service and the Trust-Task base.
- [`auth/oob/claim`](../../../../auth/oob/claim/0.1/spec.md): the same first-request shape for `sign-in`, which this task mirrors.
