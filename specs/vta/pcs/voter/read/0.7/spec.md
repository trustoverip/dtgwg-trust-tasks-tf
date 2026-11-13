---
slug: vta/pcs/voter/read
version: "0.7"
title: "VTA PCS — Voter Read"
summary: "A voter's client asks its VTA for an elections list, show or tombstone read signed as the voter's membership DID — a show presenting a membership credential the member chose — so it reads as the member without the member's key."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - election
  - vta
  - persona
  - presentation
parties:
  - role: Client driving the PCS engine
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: PCS engine (the VTA)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA sign as the voter's membership DID. Where the transport does not
    already authenticate the caller, the proof is the only thing between that signature and
    anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer is a signature as the member; placing the request in time bounds how long a
    captured request could be replayed to obtain one.
sideEffects:
  level: none
  rationale: >-
    The engine builds and signs one request document. Nothing it holds changes, and the read it
    signs changes nothing at the community either.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA signs as the voter's membership DID. In: which of three reads, its payload and, for a
    show, optionally the vault handle of one membership credential. Out: that read as a signed
    document, carrying where asked a presentation of that credential to the one community — no
    key, and nothing the community would not show the member anyway.
retention:
  class: transient
  rationale: >-
    The engine keeps nothing; the signed document is the client's to send.
errorCodes:
  - code: vta/pcs/voter/read:memberNotHeld
    meaning: "`member` names a DID the engine holds no `authentication` key of, or one the caller may not act as."
    retryable: false
  - code: vta/pcs/voter/read:membershipCredentialUnusable
    meaning: "`membershipCredential` names no credential in the vault the caller may present as `member`, or one whose subject is not `member`, or whose validity period has ended."
    retryable: false
  - code: vta/pcs/voter/read:membershipCredentialMismatch
    meaning: "The request names `memberOf`, and the credential `membershipCredential` names was not issued by that DID. The engine presents no membership unrelated to the election the member means to read (VTI-VOT-016)."
    retryable: false
related:
  - vtc/elections/list
  - vtc/elections/show
  - vtc/elections/tombstones/show
  - vta/pcs/voter/receipt
  - vta/pcs/voter/register
  - vault/credentials/present
---

## Abstract

The **VTA PCS — Voter Read** Trust Task signs one of the reads a voter needs — the two before taking
a ticket, [`vtc/elections/list`](../../../../../vtc/elections/list/0.5/spec.md) and
[`vtc/elections/show`](../../../../../vtc/elections/show/0.6/spec.md), and, after an election
opened on a community board whose board was deleted,
[`vtc/elections/tombstones/show`](../../../../../vtc/elections/tombstones/show/0.1/spec.md), which
carries the board heads the community kept so the voter can still check their receipt
([`vta/pcs/voter/receipt`](../../receipt/0.1/spec.md), VTI-VOT-053, VTI-BRD-009) — as the voter's
membership DID. A community answers those reads only to a party its roll admits, so the
request must be signed by the member; the member's key usually lives in their VTA, and this
task lets a client that holds none read as the member, exactly as
[`vta/pcs/voter/register`](../../register/0.5/spec.md)'s `ticketDocument` lets it ask for the
ticket. An election whose credential roll names a party that is itself a member of the
community is shown only to a reader who presents their membership of that party in the read
([`vtc/elections/show`](../../../../../vtc/elections/show/0.6/spec.md), VTI-VOT-015); for such a read the client names the
credential, and the engine presents it from its vault, as `vta/pcs/voter/register` does for the
ticket. The family's shared rules are in
[`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.6

- `read` gains a third value, **`tombstone`**: a [`vtc/elections/tombstones/show/0.1`](../../../../../vtc/elections/tombstones/show/0.1/spec.md) request. Once a community deletes the board an election was opened on, `vtc/elections/show` no longer finds the election; the community keeps a tombstone with the board heads it kept (VTI-BRD-008, VTI-BRD-009), which it shows only to an active member, so the read has to be signed as the member like the other two. The response's `document.type` enum gains `https://trusttasks.org/spec/vtc/elections/tombstones/show/0.1`.
- `membershipCredential` stays a `show`'s alone: a tombstone read carries no presentation.

A new `read` value and a new `document.type` value change the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.6` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the client) **MUST** send the community the returned `document`
unchanged, and **MUST NOT** ask for a read it does not mean to send. It names
`membershipCredential` only for a `show` of an election whose `credential` roll names a member
party, which the community does not otherwise show the member, and only the credential the member
chose to present to that community for that election, after being told that reading it shows the
community their membership of that party (VTI-VOT-016, VTI-VOT-090) — never to learn which
elections exist or concern the member. With it, it **SHOULD** name `memberOf`: the DID it was told
the election's roll names, from the party or the link that told it of the election. Having sent
the read, it **SHOULD** compare the shown manifest's `roll.memberOf` with that DID, and tell the
member where they differ. Answered `indeterminate`, it **MAY** send the same document again, after
any `retryAfter` and with backoff, before it expires: the community kept nothing of it. After a lost answer it asks for a new read rather than
resending the old one: the community accepts a presentation in a document once
([`vtc/elections/show/0.6`](../../../../../vtc/elections/show/0.6/spec.md) Conformance item 8).

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `malformedRequest` a `payload` that does not validate against the request schema of the task `read` names, and one that carries `presentation`: only the engine builds a presentation, bound to the document it signs.
4. Refuse with `vta/pcs/voter/read:memberNotHeld` unless it holds an `authentication` key of `member` that the caller may act as, under the same rule by which it acts as a persona elsewhere.
5. Where the request names `membershipCredential`, refuse with `vta/pcs/voter/read:membershipCredentialUnusable` unless the vault holds under it a credential the caller may present as `member`, whose subject is `member` and whose validity period has not ended.
6. Where the request names `memberOf`, refuse with `vta/pcs/voter/read:membershipCredentialMismatch` unless that credential's issuer is `memberOf`, compared as DIDs, character for character. A credential issued by any other party — another member of `memberOf`, a party `memberOf` recognises, or a group of `memberOf` — is a mismatch: the community admits a read on a credential `roll.memberOf` itself issued, and nothing else ([`vtc/elections/show/0.6`](../../../../../vtc/elections/show/0.6/spec.md) Conformance item 8.4).
7. Build the request document — a fresh random `id`, the type `https://trusttasks.org/spec/vtc/elections/list/0.5`, `https://trusttasks.org/spec/vtc/elections/show/0.6` or `https://trusttasks.org/spec/vtc/elections/tombstones/show/0.1` as `read` is `list`, `show` or `tombstone`, `issuer` `member`, `recipient` `community`, an `issuedAt` of now and an `expiresAt` no more than an hour later, and exactly `payload`, to which, where the request names `membershipCredential`, it adds `presentation`: a presentation of that one credential, made as `member`, with `nonce` the document's `id` and `domain` (or, for an SD-JWT, `aud`) `community`, disclosing nothing the community's check does not read ([`vtc/elections/show/0.6`](../../../../../vtc/elections/show/0.6/spec.md) Conformance item 8.2) — and sign it with that key for `authentication`.
8. Sign nothing else: no other type, no other payload, and present only that credential, only to `community`, only bound to that document. Neither `member` nor `membershipCredential` confers any other signature or presentation. `read` is closed; a later read is a later version of this task.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)), expressed as the capability **`pcsVoter`**, and standing to act as `member`. It is not `sign`, and a VTA **MUST NOT** require `sign` for this task: the engine signs only these three reads, as `vta/pcs/voter/register` signs only the ticket request. Presenting `membershipCredential` in a `show` is likewise part of this task: a presentation of one credential the member holds, to the one community and bound to the one document item 7 fixes, needing no standing beyond that and the standing to present the credential from the vault that [`vault/credentials/present`](../../../../../vault/credentials/present/0.1/spec.md) would need.

## Definitions

- **`contextId`** — the context the voter belongs to.
- **`community`** — the community's DID.
- **`member`** — the voter's membership DID at the community.
- **`read`** — `list`, `show` or `tombstone`.
- **`payload`** — that read's request payload, never carrying `presentation`.
- **`membershipCredential`** — with `read` `show` only: the vault handle of the membership credential the engine presents in the signed read.
- **`memberOf`** — with `membershipCredential` only: the DID the presented credential must be issued by, which the member was told the election's roll names.
- **`document`** (response) — the signed read.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02001",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/read/0.7#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "member": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222",
    "read": "show",
    "payload": { "electionId": "board-2026" }
  }
}
```

### Reading an election put to a member party's members

The member chose to present their membership credential from a partner community — itself a member of this one — to read the election that partner's members were asked. The engine adds the presentation to the signed `show`.

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02003",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/read/0.7#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:05:00Z",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "member": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222",
    "read": "show",
    "payload": { "electionId": "partner-charter-2026" },
    "membershipCredential": "vc-7f3a",
    "memberOf": "did:webvh:QmPartner:partner.example"
  }
}
```

### Reading what is kept of an election after its board was deleted

The voter checks their receipt after the board the election was opened on was deleted; the community shows the tombstone, with the heads it kept, only to an active member.

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02004",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/read/0.7#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "member": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222",
    "read": "tombstone",
    "payload": { "electionId": "board-2026" }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02002",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/read/0.7#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a02001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/vtc/elections/show/0.6",
      "issuer": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222",
      "recipient": "did:webvh:QmCommunity:vtc.example.org",
      "issuedAt": "2026-11-02T09:00:01Z",
      "expiresAt": "2026-11-02T10:00:01Z",
      "payload": { "electionId": "board-2026" },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkmMemberPersonaExampleKey22222222222222222222#z6MkmMemberPersonaExampleKey22222222222222222222",
        "created": "2026-11-02T09:00:01Z",
        "proofPurpose": "authentication",
        "proofValue": "z4ExampleMemberProofValue"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request names the member and which of the community's election reads it signs and,
for a `show`, possibly a vault handle; the response is that read, signed, carrying where asked a
presentation of that one credential. No key, ticket or ballot crosses.

### Correlation

The signed read is the member's own request to their own community, which the community
already attributes to them. Nothing in it links to a ticket or a cast: those are made under a
key per election that never signs as the member. A read carrying a presentation shows the
community the member's membership of the party the credential is from, before the member has
seen the election; that is why the engine presents only the credential the client names, and the
client names one only by the member's choice and for one election (VTI-VOT-016).

### Retention

Nothing is kept by the engine.

### Consent/purpose

The member's VTA signs as the member only to read the community's elections, and presents a
credential only in a `show` and only the one the client names; a client holding `pcsVoter` can
make it sign or present nothing else.
