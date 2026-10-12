---
slug: provision/vta/redeem
version: "0.1"
title: "Provision — VTA Redeem"
summary: "The claimant polls the Farm with the key that claimed a reservation and, once the person has confirmed the claim check and the key is in the VTA, receives the hand-over: the VTA's DID, what the claim did and where the VTA is reachable."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [vta, farm, redeem, provision, hand-over, long-poll, administrator, wallet]
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
    Only the key that claimed the reservation may learn its outcome or receive the hand-over, and the request's proof is what shows the caller holds it. The response's proof, by the Farm's assertionMethod key, is what lets the wallet treat the VTA DID as the Farm's statement that this key now administers that VTA, which it keeps for the life of the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Each poll is a freshly signed document, and an issue time lets the Farm refuse a replayed one. The response names a durable fact and is kept as evidence, so it needs to be placeable in time too.
sideEffects:
  level: none
  rationale: >-
    Reports the reservation's state and, once the key is in the VTA, the hand-over. It changes nothing: the state is moved by the person's confirmation on the Farm's page and by the Farm's own provisioning, never by this task.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a handle. The response names the VTA the claimant now administers, what the claim did and where the VTA is reachable: facts about a VTA that is now the claimant's, disclosed only to the key that claimed it.
retention:
  class: durable
  rationale: >-
    The response is the Farm's signed statement that this key administers this VTA. The wallet keeps it as its record of how it came to hold the VTA; the Farm keeps the fact that the hand-over happened with the VTA's provisioning record.
errorCodes:
  - code: provision/vta/redeem:pending
    meaning: The hand-over is not yet available. Returned after the Farm has held the call for up to 25 s. `details.state` says where the reservation is. The claimant retries with a freshly signed document.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [state]
      properties:
        state:
          type: string
          enum: [claimed, confirmed, provisioning]
        confirmBy:
          type: integer
          minimum: 0
  - code: provision/vta/redeem:rejected
    meaning: The person said the claim checks do not match. The key has been discarded and will not be written. Final.
    retryable: false
  - code: provision/vta:handleNotFound
    meaning: No reservation has this handle.
    retryable: false
  - code: provision/vta:handleExpired
    meaning: The hold passed before the person confirmed, or the reservation has otherwise ended. The key has been discarded.
    retryable: false
  - code: provision/vta:notClaimant
    meaning: The document is not signed by the key that claimed the reservation.
    retryable: false
  - code: provision/vta:keyUnsupported
    meaning: The document's issuer is not an Ed25519 `did:key`.
    retryable: false
related:
  - provision/vta/claim
  - auth/oob/redeem
---

## Abstract

After [`provision/vta/claim`](../../claim/0.1/spec.md) has locked a reservation to the wallet's administrator key `K_a`, the person compares the claim check on the Farm's page with the one in the wallet and confirms or refuses on the page (VTI-LNK-115). Nothing reaches the wallet from that page. This task is how the wallet learns what happened: it polls the Farm with `K_a`, and the Farm holds each poll for up to 25 seconds before answering `pending` with the reservation's state, `rejected`, or, once the key has been written into the VTA and the VTA answers, the **hand-over**: the VTA's DID, what the claim did, and where the VTA is reachable.

The hand-over is the terminal success of the whole exchange that the trigger link started, and the only document in the family that names the VTA as the claimant's. It is signed by the Farm and kept by the wallet. The family rules are in [`CONVENTIONS.md`](../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **claimant** (the wallet):

1. **MUST** sign each poll with `K_a`, as a new document with a fresh `id` and the claim's `threadId`, and **MUST** keep at most one poll open per reservation.
2. **MUST** set `payload.handle` to the handle it claimed, and `recipient` to the Farm's DID.
3. **MUST** send the poll to the same endpoint as the claim, taken from the Farm's verified DID document ([HTTPS binding 0.2 §6](../../../../../bindings/https/0.2/spec.md#6-endpoint-discovery)).
4. **MUST** verify the response's proof against a key the Farm's DID lists under `assertionMethod`, and **MUST** check that `handle` equals the one it claimed and that `issuer` equals the Farm's DID. Any failure ends the exchange on the claimant's side.
5. On `pending`, **SHOULD** show the person the state and, while it is `claimed`, remind them to confirm on the page; on `rejected` or `handleExpired`, **MUST** discard `K_a`.
6. **MUST NOT** treat the VTA as its own before this task's response names it.

A conforming **Farm**:

1. **MUST** refuse an issuer that is not an Ed25519 `did:key` with `provision/vta:keyUnsupported`, and an issuer that is not the key that claimed the reservation with `provision/vta:notClaimant`, with no details. A handle nobody reserved is refused with `provision/vta:handleNotFound`.
2. Before the hand-over is available, **SHOULD** hold the call for up to 25 s, with at most one open poll per reservation, and then answer `provision/vta/redeem:pending` with `details.state` (`claimed`, `confirmed` or `provisioning`) and, while the state is `claimed`, `details.confirmBy`, when the hold lapses.
3. For a reservation the person refused, **MUST** answer `provision/vta/redeem:rejected`; for one whose hold lapsed or that has otherwise ended, `provision/vta:handleExpired`. In both cases the key has already been discarded.
4. **MUST** answer with the hand-over only once `K_a` is written into the VTA as an administrator and the VTA answers, and **MUST** sign it with its `assertionMethod` key. `operation` is derived from the reservation, never from anything the claimant sent.
5. **MUST** keep answering the same hand-over to the same key for at least as long as it keeps the reservation's record, so a wallet that lost the response can fetch it again; this task changes no state.
6. **MUST NOT** put anything about the person who was shown the code in any reply.

## Authorization

Two things together: the person's confirmation on the Farm's page, which is what moves the reservation past `claimed` (VTI-LNK-115, mandatory for a running VTA under VTI-LNK-117), and `K_a`'s proof on this document, which shows the caller holds the key that claimed. Neither alone suffices: the key without the confirmation gets only `pending`, and the confirmation names no caller. The proof is identity, not authorization ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); what entitles the caller to the hand-over is that the key it holds is the one the person confirmed. The task itself exercises no authority: it reads the outcome of a decision made elsewhere.

## Definitions

- **Hand-over** — the response: the Farm's signed statement that `K_a` administers `vtaDid`, with `operation` and where the VTA is reachable.
- **`K_a`**, **`handle`**, **claim check** — as in [`provision/vta/claim`](../../claim/0.1/spec.md).
- **State** — the reservation's position in the state machine of [`CONVENTIONS.md`](../../_shared/0.1/CONVENTIONS.md) §3, as `details.state` reports it while pending.

## Request

The claimant (`issuer`, its administrator key) polls the Farm (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet polls its claim

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e21",
  "type": "https://trusttasks.org/spec/provision/vta/redeem/0.1",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "issuedAt": "2026-10-12T09:30:40Z",
  "payload": {
    "handle": "q3Vn7Zk2Xo9Rt1LwPb4HdA"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-12T09:30:40Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

Once the key is in the VTA and the VTA answers, the Farm answers with the hand-over. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Every other outcome, including "not yet", is a `trust-task-error`.

- `handle` — the handle the claim named.
- `vtaDid` — the DID of the VTA the claimant now administers.
- `operation` — `provisionVta` when the claimant is the first administrator of a VTA provisioned for this reservation; `grantAcl` when the claimant was added to a VTA that already had one.
- `mediatorDid`, `didLogUrl` — where the VTA is reachable and where its DID log is served. Conveniences; both are also derivable from the VTA's DID document.

### The VTA is handed over

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e22",
  "type": "https://trusttasks.org/spec/provision/vta/redeem/0.1#response",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-12T09:34:02Z",
  "payload": {
    "handle": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
    "vtaDid": "did:webvh:QmYyWfX2gJmZk9zNqV8pL3tRbH4sKc6dEa7uF1nG5oP2wQ:dids.firstperson.dev:alice-vta",
    "operation": "provisionVta",
    "mediatorDid": "did:webvh:QmTr5sUv2wXy8zAb3cDe6fGh9iJk1lMn4oPq7rSt0uVw3x:dids.firstperson.dev:platform-mediator",
    "didLogUrl": "https://dids.firstperson.dev/alice-vta/did.jsonl"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth#key-1",
    "created": "2026-10-12T09:34:02Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Claimed, waiting for the person to confirm

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e23",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-12T09:31:05Z",
  "payload": {
    "code": "provision/vta/redeem:pending",
    "message": "Confirm on the page that the six characters match.",
    "retryable": true,
    "details": { "state": "claimed", "confirmBy": 1791626860 }
  }
}
```

### The person said "Not mine"

```json
{
  "id": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e24",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:7c2d4e6f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "parentThreadId": "q3Vn7Zk2Xo9Rt1LwPb4HdA",
  "issuer": "did:webvh:QmXw3fYz8kLm2nPq5rSt7uVa9bCd1eFg4hJk6lMn8oPq2r:dids.firstperson.dev:vtafarm-auth",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-12T09:31:30Z",
  "payload": {
    "code": "provision/vta/redeem:rejected",
    "message": "The person at the page said this claim was not theirs.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries the handle under the administrator key, and nothing else. A pending reply carries the reservation's state and, while a confirmation is awaited, when the hold lapses; a rejection carries nothing. The hand-over carries the VTA's DID, what the claim did and where the VTA is reachable: facts about a VTA that is now the claimant's, and nothing about the person who was shown the code or about any other VTA. A Farm **MUST NOT** put a credential, a token or a session in any reply; the key the wallet already holds is what administers the VTA. `ext` is available on both documents for ecosystem use under the framework's namespacing rules, and this specification defines no semantics for anything in it.

The documents are small and fixed in shape, so no `maxDocumentBytes` is declared. A Farm accepts a poll whose `issuedAt` is within a few seconds of its own clock; each poll is freshly signed, so there is no reason to accept an older one.

### Correlation

Every poll is signed by `K_a`, so the Farm can join the polls to the claim, which is the point, and nobody else can: `K_a` is fresh per claim and the claimant declares `identifierScope: pairwise`. The handle joins the polls to the reservation for the Farm and expires with it. The hand-over joins `K_a` to `vtaDid` permanently, which is the fact the whole exchange exists to establish and which the VTA's own ACL and DID document make public anyway.

The Farm's `identifierScope: public` is the same constraint as on the claim: the wallet verifies the hand-over against a key the Farm's published DID document lists, and that needs the Farm to present one identifier to everyone. The timing of polls says when a person was standing at a Farm's page; a Farm that logs polls **SHOULD** log the handle, not the key.

### Retention

The wallet keeps the hand-over for the life of the VTA as its record of how it came to hold it. The Farm keeps the fact of the hand-over with the VTA's provisioning record, and keeps answering the same response to the same key for as long as it keeps that record. Pending replies and rejections are kept by neither side beyond the exchange; a rejection means the key is already gone from the Farm and should go from the wallet.

### Consent/purpose

The poll exists for one purpose: to tell the wallet whether the person confirmed the claim and, when they did, to deliver the VTA. The hand-over is a statement to the holder of the key and is used for nothing else by the Farm. Whether and how a Farm gates the release of a VTA is its own policy within VTI-LNK-115 and VTI-LNK-117, decided on its page; this task reports the result and does not set the policy.

## References

- [`provision/vta/claim/0.1`](../../claim/0.1/spec.md): the claim this task follows, and its `trust-task-next-step` reply.
- Verifiable Trust Infrastructure, chapter *Trigger Links* (`VTI-LNK-*`): VTI-LNK-114, VTI-LNK-115 and VTI-LNK-117 on the claim and the claim check. Under development at [trustoverip/dtgwg-vti-spec#58](https://github.com/trustoverip/dtgwg-vti-spec/pull/58).
- [`auth/oob/redeem`](../../../../auth/oob/redeem/0.1/spec.md): the long-poll pattern this task mirrors.
