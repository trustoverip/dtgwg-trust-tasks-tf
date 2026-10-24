---
slug: vtc/elections/tickets/issue
version: "0.1"
title: "VTC Elections — Tickets — Issue"
summary: "A party the election's roll admits asks the community for its ticket; the community checks eligibility, records the issuance against the signer and blind-signs a request it cannot open, so it never sees the ticket the voter's agent unblinds."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - ticket
  - blind-signature
  - anonymous-credential
  - pcs
parties:
  - role: eligible party (by its membership DID)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The community issues on the strength of who is asking: a party its roll admits, once per election. A proofless
    request would let anyone take another member's one ticket, and the community could never tell afterwards — the
    ticket it issued is one it deliberately cannot recognise.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    One ticket per party per election. Placing the request in time is what lets the community tell a resend of the
    request it answered from a new request, and answer the first with its first answer.
sideEffects:
  level: mutating
  rationale: >-
    Records that the signer took the election's ticket, which spends their one ticket for the election, and blind-signs
    their request under the community's voting key.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the election, the manifest digest, the voter's PCS identifier for this election and a blinded commitment. Out: a
    signature over a value the community cannot read. The community learns who took a ticket and when — never the
    ticket.
retention:
  class: durable
  rationale: >-
    The issuance record is what enforces one ticket per party, and its count is what certification holds the counted
    ballots to (VTI-VOT-072). It is also the one record that names who took part, so the community keeps the smallest
    record that does both, and no longer than the election needs it.
errorCodes:
  - code: vtc/elections/tickets/issue:electionUnknown
    meaning: "No registered election with this identifier exists for this community, or the signer may not see it (CONVENTIONS §7). Answered identically, so the task is no existence oracle."
    retryable: false
  - code: vtc/elections/tickets/issue:notOpen
    meaning: "Tickets are not being issued for the election: registration has not opened, voting has closed, the election is being closed or was cancelled, or the ballot box has not yet registered it (VTI-VOT-024)."
    retryable: false
  - code: vtc/elections/tickets/issue:manifestMismatch
    meaning: "`manifestDigest` is not the digest of the election's registered manifest. The requester read another manifest than the one the election runs under."
    retryable: false
  - code: vtc/elections/tickets/issue:notEligible
    meaning: "The election's roll rule does not admit the signer as the community's records stand now (VTI-VOT-021)."
    retryable: false
  - code: vtc/elections/tickets/issue:alreadyIssued
    meaning: "The community has already issued the signer a ticket for this election, in answer to a different request (VTI-VOT-022). A resend of the request it answered is answered with that answer, never with this code."
    retryable: false
  - code: vtc/elections/tickets/issue:requestInvalid
    meaning: "`request.label` is not `voter/<electionId>` for this election, or the blinded request does not verify under the community's voting parameters."
    retryable: false
related:
  - vtc/elections/show
  - vtc/elections/list
  - vtc/elections/cancel
  - vtc/elections/certify
  - vta/pcs/voter/register
  - vta/pcs/voter/accept
  - vtc/vetting/vetters/pcs-root
---

## Abstract

**VTC Elections — Tickets — Issue** is the only point at which a voter is named. The party asks with its membership DID; the community checks, against its own records, that the election's roll admits them and that they have not taken a ticket; it records the issuance against them; and what it signs is a commitment to a key it cannot see, built by the voter's VTA ([`vta/pcs/voter/register`](../../../../../vta/pcs/voter/register/0.1/spec.md)). The VTA unblinds the answer into the ticket ([`vta/pcs/voter/accept`](../../../../../vta/pcs/voter/accept/0.1/spec.md)) — a class credential under `voter/<electionId>` that the community has never seen, and which admits exactly one counted ballot at the ballot box without saying whose.

The request and answer mirror the blinded enrolment of hidden vetting ([`vtc/vetting/vetters/pcs-root`](../../../../vetting/vetters/pcs-root/0.1/spec.md)), under the community's separate **voting** deployment. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) signs with the DID it is a member by (or, for a party enfranchised by name, the DID `roll.listed` names), carries the `manifestDigest` it read from [`vtc/elections/show`](../../../show/0.1/spec.md), and sends the request its VTA built for this election and no other. A producer whose answer was lost **MUST** resend the same document — same `id`, same proof — rather than build a new request; a new request is refused `alreadyIssued`.

A conforming **community** (`recipient`):

1. **A resend is answered with its first answer** (CONVENTIONS §8.3). When the signer sends a document the community has already answered with a pre-credential — same signer, same `id`, same signed bytes — within its acceptance window, the community **MUST** answer with the pre-credential it recorded for it and **MUST NOT** sign again.
2. **The election is known.** An election the community holds no registered election under, or one the signer may not see under [`vtc/elections/show`](../../../show/0.1/spec.md) Conformance item 1, **MUST** be refused `electionUnknown`, identically. Because a caller without `vtc.elections.manage` sees an election only when its roll admits them, an ineligible caller without that capability is answered `electionUnknown`, not `notEligible`, and learns nothing about an election that is not theirs.
3. **Issuance is open** (VTI-VOT-024). The ballot box has registered the election; the clock is at or after `registrationOpensAt` (`registrationOpens` in a manifest pinned to `voting/_shared/0.1`) and before `votingClosesAt` (`votingCloses` in a manifest pinned to `voting/_shared/0.1`); the election is not cancelled and no close of it is in flight. Else `notOpen`.
4. **The manifest is the one read.** `manifestDigest` **MUST** equal the digest of the registered manifest. Else `manifestMismatch`.
5. **The signer is eligible** (VTI-VOT-021). The roll rule admits the proven signer as the community's records stand **now**, not when the election was created: under `members` and `membersAndListed`, the signer holds a live access-control entry that is a membership — not expired, not suspended, not `application` — and a member record that has not been removed; under `membersAndListed` and `listed`, a DID `roll.listed` names is admitted as itself. Else `notEligible`.
6. **No ticket is held** (VTI-VOT-022). The community holds no issuance record for this signer and election. Else `alreadyIssued`.
7. **The request is the election's.** `request.label` **MUST** be `voter/<electionId>` for this election, and the blinded request **MUST** verify under the community's voting parameters ([`VotingParameters`](../../../../../voting/_shared/0.1/voting.schema.json), VTI-VOT-020) for that label. Else `requestInvalid`.

Then, **one issuance at a time** for the election:

8. **Issue sequentially.** The community **MUST** make items 6 to 10 one critical section per election, so that no two issuances of the election interleave. Two concurrent requests from one signer would otherwise both pass item 6, and the count of tickets issued — which certification holds the board to (VTI-VOT-072) — would not be the count of tickets that exist.
9. **Sign blind.** The community blind-signs the request under its voting key, for the label. It never sees, and **MUST NOT** ask for, the ticket that results.
10. **Record before answering.** The community **MUST** durably record the issuance against the signer — the election, the signer, the time, and what item 1 needs to answer a resend (the document's identifier and digest, and the pre-credential) — **before** it answers. A crash after signing and before recording answers nothing and records nothing, so no ticket escapes the count; a crash after recording is recovered by the producer's resend under item 1. The community **SHOULD** discard the stored pre-credential at `votingClosesAt`, when no resend can be useful.
11. **Audit** (VTI-VOT-100). The community **MUST** audit the issuance: the signer, the election and the time. It **MUST NOT** put `request.id`, the blinded request or the pre-credential in the audit record, or anything else derived from them.

## Authorization

The entitlement is **standing on the election's roll** — the community's active membership, or being named in `roll.listed` — read from the community's own records at the moment of issuance, together with **not having taken this election's ticket**. Both are authorization, not validation: the roll rule decides who may vote (VTI-VOT-021), and the issuance record holds each of them to one ticket, which is what makes each counted tag one voter (VTI-VOT-022). Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 the proof establishes who is asking and nothing about their standing; it is the key the community looks the standing up by.

No administrative capability issues a ticket to anyone else, and none is needed to take one's own: an administrator takes a ticket under the same roll as every other party. Nobody asks for a ticket on another's behalf.

## Definitions

- **`electionId`** (REQUIRED) — the election.
- **`manifestDigest`** (REQUIRED) — the digest of the manifest the requester read.
- **`request.label`** — the ticket class label, `voter/<electionId>` (CONVENTIONS §3.2).
- **`request.id`** — the voter's PCS identifier for this election, derived from a voter key allocated for this election alone (VTI-VOT-023), so it links nothing across elections.
- **`request.request`** — the suite's blinded root request (`RootRequest`), as [`vtc/vetting/vetters/pcs-root`](../../../../vetting/vetters/pcs-root/0.1/spec.md) carries one.
- **`preCredential`** (response) — what the community signs. Only the holder of the request's blinding state can unblind it, and the ticket it unblinds into is not derivable from this document.
- **Issuance record** — the community's record that a signer took an election's ticket.

## Request

The member's client sends the request its VTA built. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member takes their ticket

```json
{
  "id": "urn:uuid:4d5e6f70-8192-43a4-a565-d6e7f8091a2b",
  "type": "https://trusttasks.org/spec/vtc/elections/tickets/issue/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-02T08:05:00Z",
  "threadId": "urn:uuid:4d5e6f70-8192-43a4-a565-d6e7f8091a2b",
  "payload": {
    "electionId": "board-2026",
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "request": {
      "label": "voter/board-2026",
      "id": "z5jokfsiZx1sk1mJyQnBnR519B9mcQhw9a8LrF9VQdAjExwoLzcdVdZ2aXaH14a3mdF",
      "request": {
        "encoding": "z6P4jNSt4qHKYhSBGSWgr6VcQGMsjmfYGUBdYWNtS7xbpGsTuiY1PDfoce4yTApuqUm",
        "proof": "zSzFTCny3qaXpSHTsTE877ffQfSuF9T5zo53iuZkwx9NbLYSq3GeLZpRda286SBhqwJg"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-11-02T08:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4oJRBExampleProofValue"
  }
}
```

## Response

The community answers with the blind pre-credential — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Every refusal above is a `trust-task-error`, never a `#response` document.

### The blind pre-credential

```json
{
  "id": "urn:uuid:5e6f7081-92a3-44b5-b676-e7f8091a2b3c",
  "type": "https://trusttasks.org/spec/vtc/elections/tickets/issue/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T08:05:01Z",
  "threadId": "urn:uuid:4d5e6f70-8192-43a4-a565-d6e7f8091a2b",
  "payload": {
    "electionId": "board-2026",
    "label": "voter/board-2026",
    "preCredential": "z3Xef1JY2x1s2vY6yKWfLu8Ap2vKuoZGwaQLZ2tjq8QkEZbGeZyaNgtznZ8B7N1yf14"
  }
}
```

### Refused: a ticket was already taken

```json
{
  "id": "urn:uuid:6f708192-a3b4-45c6-8787-f8091a2b3c4d",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:708192a3-b4c5-46d7-9898-091a2b3c4d5e",
  "payload": {
    "code": "vtc/elections/tickets/issue:alreadyIssued",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries the election, the manifest digest, the voter's PCS identifier for this election and a blinded commitment; the response carries a signature over a value the community cannot read. Nothing here is a ballot or concerns one: the community never handles a cast (VTI-VOT-001). A producer **MUST NOT** put into `ext` anything about how it intends to vote.

### Correlation

The community learns exactly who took a ticket for which election, and when. That is the price of a roll the community can enforce, and it is paid here once. What the community cannot do is join this record to a cast: the ticket is blind, the cast's attestation and tag are unlinkable to the identifier and commitment recorded here (CONVENTIONS §3), the cast is signed by a key used for nothing else (VTI-VOT-003), and board entries carry no time of arrival (CONVENTIONS §6.1). `request.id` is derived from a key allocated for this election alone (VTI-VOT-023), so it does not link a member's participation across elections.

Timing is the remaining channel. A ballot box sees when casts arrive, and if a cast followed its ticket's issuance closely, a community and a ballot box comparing notes could pair them. A member's VTA **SHOULD NOT** build a cast in the exchange that accepted the ticket, and a client **SHOULD** let a random interval pass before casting (CONVENTIONS §6.3, VTI-VOT-033); taking the ticket early in the registration period, well before voting opens, removes the channel altogether. The `disclosure` a member reads before asking ([`vtc/elections/show`](../../../show/0.1/spec.md)) says whether the community's operator also runs the ballot box, which is when that pairing needs no collusion at all (VTI-VOT-090).

The requester is declared `identifierScope: pairwise`: the community must recognise the requester as a member, by whatever DID the member holds the community by, and nothing here needs anyone else to.

### Retention

The issuance record is kept while it enforces one ticket per party and while certification needs its count. The community **SHOULD** discard the stored pre-credential at `votingClosesAt`, and **SHOULD** reduce its per-signer issuance records to their count once the election is certified or cancelled: the count is all certification needs, and the per-signer record is the one thing on the community's side that says who took part. The audit row records the issuance and nothing PCS-derived (Conformance item 11, VTI-VOT-100), so that neither it nor any other record pairs a voter's identifier with a tag, a cast or a ballot (CONVENTIONS §6.4).

### Consent/purpose

The membership DID and the issuance are collected to enforce the roll and one ticket per party, and to bound the count at certification, and for nothing else. In particular the community **MUST NOT** use its issuance records to measure or publish who took part in an election beyond the count, and the design is such that it cannot learn from them how anyone voted.
