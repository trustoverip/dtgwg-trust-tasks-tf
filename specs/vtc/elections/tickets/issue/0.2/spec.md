---
slug: vtc/elections/tickets/issue
version: "0.2"
extends: "0.1"
title: "VTC Elections — Tickets — Issue"
summary: "A party the roll admits asks for its ticket, presenting under a credential roll its membership credential for the party the roll names; the community checks eligibility, records the issuance and blind-signs a request it cannot open, never seeing the ticket."
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
  - role: eligible party (by its membership DID, or under a credential roll the subject of its membership credential)
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
  ingests: personal
  actsAsSubject: false
  rationale: >-
    In: the election, the manifest digest, the voter's PCS identifier for this election and a blinded commitment, and,
    under a credential roll, a presentation of the requester's membership credential for the party the roll names. Out:
    a signature over a value the community cannot read. The community learns who took a ticket and when — and, under a
    credential roll, that they are a member of that party — never the ticket.
maxDocumentBytes:
  request: 98304
  rationale: >-
    Under a credential roll the request carries a presentation of one membership credential and its proofs, JSON- or
    base64-encoded, beside a blinded request of a few kilobytes; the envelope and its proof are headroom.
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
  - code: vtc/elections/tickets/issue:presentationRequired
    meaning: "The election's roll rule is `credential` and the request carries no `presentation`. Under a credential roll eligibility is proven only by presenting the membership credential the roll names (VTI-VOT-025)."
    retryable: false
  - code: vtc/elections/tickets/issue:membershipNotProven
    meaning: "The presentation does not prove a membership credential for the party `roll.memberOf` names, held by the signer, valid now (VTI-VOT-025 items 1 to 3). `details.reason` says which check failed."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [reason]
      properties:
        reason:
          type: string
          enum: [proof, binding, holder, issuer, authorization, validity]
  - code: vtc/elections/tickets/issue:membershipRevoked
    meaning: "The presented membership credential, or an authorization credential its issuer's authority rests on, is revoked or suspended as its status shows now (VTI-VOT-025 item 4, VTI-CRD-011)."
    retryable: false
  - code: vtc/elections/tickets/issue:rollUntrusted
    meaning: "`roll.memberOf` is no longer a party the community trusts for the roll (VTI-VOT-026): it names a group the community no longer holds, or a member party whose own membership of the community has since expired, been suspended, been removed or had its membership credential revoked. No ticket is issued under it."
    retryable: false
  - code: vtc/elections/tickets/issue:membershipIndeterminate
    meaning: "The community could not determine, now, whether the presented credential, or the membership credential it issued the party `roll.memberOf` names, is revoked: a status list or the trust registry its issuer publishes status through could not be reached or answered too long ago (VTI-CRD-012, VTI-REG-012). Nothing was recorded; the same document may be sent again."
    retryable: true
related:
  - vtc/elections/show
  - vtc/elections/list
  - vtc/elections/cancel
  - vtc/elections/certify
  - vta/pcs/voter/register
  - vta/pcs/voter/accept
  - vtc/vetting/vetters/pcs-root
  - vtc/elections/create
  - vault/credentials/present
---

## Abstract

**VTC Elections — Tickets — Issue** is the only point at which a voter is named. The party asks with its membership DID; the community checks, against its own records, that the election's roll admits them and that they have not taken a ticket; it records the issuance against them; and what it signs is a commitment to a key it cannot see, built by the voter's VTA ([`vta/pcs/voter/register`](../../../../../vta/pcs/voter/register/0.4/spec.md)). The VTA unblinds the answer into the ticket ([`vta/pcs/voter/accept`](../../../../../vta/pcs/voter/accept/0.1/spec.md)) — a class credential under `voter/<electionId>` that the community has never seen, and which admits exactly one counted ballot at the ballot box without saying whose.

Under a **credential roll** ([`Manifest`](../../../../../voting/_shared/0.4/voting.schema.json) `roll.rule: credential`), the community holds no list of who may vote: the roll is whoever holds a membership credential for the one DID `roll.memberOf` names — the community itself, one of its groups, or a party that is itself an active member of the community — another community, a group of one, or a network. The party proves it here, and only here, by presenting that credential with the request. The community checks it as it stands at that moment, so a party that joined after the election was called is admitted and one removed since is not (VTI-VOT-025).

The request and answer mirror the blinded enrolment of hidden vetting ([`vtc/vetting/vetters/pcs-root`](../../../../vetting/vetters/pcs-root/0.1/spec.md)), under the community's separate **voting** deployment. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

The request gains an optional **`presentation`**: the requester's Verifiable Presentation of their membership credential for the party a `credential` roll names (`voting/_shared/0.4`, VTI-VOT-021, VTI-VOT-025). It is required when the election's roll rule is `credential`, and refused under any other rule. Conformance item 5 says how the community checks it, item 6 whom the one ticket is held against under such a roll (VTI-VOT-027), and items 10 and 11 what of it the community keeps. Five error codes are appended — `presentationRequired`, `membershipNotProven`, `membershipRevoked`, `rollUntrusted` and `membershipIndeterminate` — and the request may now be up to 96 KiB.

Every `0.1` request is a `0.2` request with the same meaning, so `0.2` declares `extends: "0.1"`. Released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules). `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) signs with the DID it is a member by (or, for a party enfranchised by name, the DID `roll.listed` names, or, under a credential roll, the DID that is the subject of the membership credential it presents), carries the `manifestDigest` it read from [`vtc/elections/show`](../../../show/0.4/spec.md), and sends the request its VTA built for this election and no other. Under a credential roll it carries `presentation`, made by the credential's subject and bound to this document: a Data Integrity presentation carries `nonce` equal to this document's `id` and `domain` equal to the community's DID as members of the presentation its holder proof signs, and a proof that also states `challenge` or `domain` states the same values; an SD-JWT presentation's key-binding JWT carries `nonce` equal to this document's `id` and `aud` equal to the community's DID. It presents one credential, and discloses nothing from it that the check below does not read. Under any other rule it carries no `presentation`. A producer whose answer was lost **MUST** resend the same document — same `id`, same proof — rather than build a new request; a new request is refused `alreadyIssued`.

A conforming **community** (`recipient`):

1. **A resend is answered with its first answer** (CONVENTIONS §8.3). When the signer sends a document the community has already answered with a pre-credential — same signer, same `id`, same signed bytes — within its acceptance window, the community **MUST** answer with the pre-credential it recorded for it and **MUST NOT** sign again.
2. **The election is known.** An election the community holds no registered election under, or one the signer may not see under [`vtc/elections/show`](../../../show/0.4/spec.md) Conformance item 1, **MUST** be refused `electionUnknown`, identically. Because a caller without `vtc.elections.manage` sees an election only when its roll admits them, an ineligible caller without that capability is answered `electionUnknown`, not `notEligible`, and learns nothing about an election that is not theirs.
3. **Issuance is open** (VTI-VOT-024). The ballot box has registered the election; the clock is at or after `registrationOpensAt` (`registrationOpens` in a manifest pinned to `voting/_shared/0.1`) and before `votingClosesAt` (`votingCloses` in a manifest pinned to `voting/_shared/0.1`); the election is not cancelled and no close of it is in flight. Else `notOpen`.
4. **The manifest is the one read.** `manifestDigest` **MUST** equal the digest of the registered manifest. Else `manifestMismatch`.
5. **The signer is eligible** (VTI-VOT-021). The roll rule admits the proven signer as the community's records stand **now**, not when the election was created: under `members` and `membersAndListed`, the signer holds a live access-control entry that is a membership — not expired, not suspended, not `application` — and a member record that has not been removed; under `membersAndListed` and `listed`, a DID `roll.listed` names is admitted as itself. Else `notEligible`. A request carrying `presentation` under any of these three rules **MUST** be refused `malformedRequest`, and its presentation discarded unread. Under **`credential`**, the community establishes eligibility from the presentation, as it stands **now** (VTI-VOT-025), in this order:
   1. **A presentation is carried.** Else `presentationRequired`.
   2. **`roll.memberOf` is still trusted** (VTI-VOT-026, VTI-VOT-025 item 6): it is the community's own DID, the DID of a group the community still holds, or the DID of a party whose own membership of the community is active **now** — in the community's records not expired, not suspended and not removed, and the membership credential the community issued it not revoked by its status. A membership that has lapsed or been revoked since the election was created stops the roll yielding tickets. Else `rollUntrusted`; where the status of the party's own membership credential cannot be determined, `membershipIndeterminate`.
   3. **The presentation proves the membership.** Its holder proof verifies and commits to this document's `id` and the community's DID as the producer's paragraph requires (`binding`); its holder is the proven signer (`holder`); it carries a credential whose subject is the signer (`holder`), whose proof verifies (`proof`), whose issuer is `roll.memberOf` (`issuer`) — for one of the community's own groups, also under the community's live authorization credential to the group's DID (VTI-DIR-083, VTI-DIR-086; `authorization`); for a member party, the party's own membership checked under item 2 is what the issuer's standing rests on — and whose validity period includes now (`validity`). Else `membershipNotProven`, with `details.reason` the first failed check as named here. The issuer's key is the one its DID document gives the `attestation` role (VTI-CRD-002).
   4. **Nothing is revoked.** The credential's status — its status list, or the trust registry its issuer publishes status through — and that of any authorization credential its issuer's authority rests on, read now, shows it neither revoked nor suspended (VTI-CRD-011, VTI-MEM-030). A credential with no status mechanism — a group membership credential, whose expiry is its revocation (VTI-DIR-085) — is checked by its validity period under item 3, and the group's authorization credential by its status here. Else `membershipRevoked`; where a status cannot be determined, `membershipIndeterminate` (VTI-CRD-012), never valid and never revoked.
   5. **The community's own records agree.** Where `roll.memberOf` is the community's own DID, the signer is an active member as under `members`; where it is one of the community's groups, the signer is also a member of that group in the community's records. A credential not yet expired does not outlast a removal the community has recorded. Else `notEligible`.
6. **No ticket is held** (VTI-VOT-022). The community holds no issuance record for this party and election. Under `members`, `membersAndListed` and `listed` the party is the signer. Under `credential` it is the credential's subject, never `roll.memberOf` (VTI-VOT-027): where `roll.memberOf` is the community or one of its groups, the member the subject identifies in the community's records, so that one ticket follows them through a rotation of their membership DID (VTI-MEM-041); where it is a member party, the subject's DID. Else `alreadyIssued`.
7. **The request is the election's.** `request.label` **MUST** be `voter/<electionId>` for this election, and the blinded request **MUST** verify under the community's voting parameters ([`VotingParameters`](../../../../../voting/_shared/0.1/voting.schema.json), VTI-VOT-020) for that label. Else `requestInvalid`.

Then, **one issuance at a time** for the election:

8. **Issue sequentially.** The community **MUST** make items 6 to 10 one critical section per election, so that no two issuances of the election interleave. Two concurrent requests from one signer would otherwise both pass item 6, and the count of tickets issued — which certification holds the board to (VTI-VOT-072) — would not be the count of tickets that exist.
9. **Sign blind.** The community blind-signs the request under its voting key, for the label. It never sees, and **MUST NOT** ask for, the ticket that results.
10. **Record before answering.** The community **MUST** durably record the issuance against the party item 6 names — the election, that party, the time, and what item 1 needs to answer a resend (the document's identifier and digest, and the pre-credential) — **before** it answers. Under a credential roll it **MUST NOT** keep the presentation, the credential or any claim of it once it has decided (VTI-CRD-042); the party item 6 names is all the record needs. A crash after signing and before recording answers nothing and records nothing, so no ticket escapes the count; a crash after recording is recovered by the producer's resend under item 1. The community **SHOULD** discard the stored pre-credential at `votingClosesAt`, when no resend can be useful.
11. **Audit** (VTI-VOT-100). The community **MUST** audit the issuance: the signer, the election and the time, and under a credential roll the credential's issuer — never the presentation or the credential. It **MUST NOT** put `request.id`, the blinded request or the pre-credential in the audit record, or anything else derived from them.

## Authorization

The entitlement is **standing on the election's roll** — the community's active membership, or being named in `roll.listed`, read from the community's own records at the moment of issuance, or, under a credential roll, holding a live membership credential for the party `roll.memberOf` names, proven by the presentation at that moment — together with **not having taken this election's ticket**. Both are authorization, not validation: the roll rule decides who may vote (VTI-VOT-021), and the issuance record holds each of them to one ticket, which is what makes each counted tag one voter (VTI-VOT-022). Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 the proof establishes who is asking and nothing about their standing; it is the key the community looks the standing up by. Under a credential roll the presentation is the evidence of the standing, and the document's proof is what binds it to the party asking: a presentation captured from one request cannot be carried in another, because it commits to that request's `id` and its holder must be that request's signer. A ticket confers no authority, so eligibility under a roll naming one of the community's groups is not a decision of the community's access control (VTI-DIR-087).

No administrative capability issues a ticket to anyone else, and none is needed to take one's own: an administrator takes a ticket under the same roll as every other party. Nobody asks for a ticket on another's behalf.

## Definitions

- **`electionId`** (REQUIRED) — the election.
- **`manifestDigest`** (REQUIRED) — the digest of the manifest the requester read.
- **`request.label`** — the ticket class label, `voter/<electionId>` (CONVENTIONS §3.2).
- **`request.id`** — the voter's PCS identifier for this election, derived from a voter key allocated for this election alone (VTI-VOT-023), so it links nothing across elections.
- **`request.request`** — the suite's blinded root request (`RootRequest`), as [`vtc/vetting/vetters/pcs-root`](../../../../vetting/vetters/pcs-root/0.1/spec.md) carries one.
- **`preCredential`** (response) — what the community signs. Only the holder of the request's blinding state can unblind it, and the ticket it unblinds into is not derivable from this document.
- **`presentation`** — under a credential roll, the requester's Verifiable Presentation of its membership credential for the party `roll.memberOf` names, bound to this document; a W3C Verifiable Presentation, or a compact SD-JWT presentation with its key-binding JWT. Absent under every other rule.
- **Issuance record** — the community's record that a party took an election's ticket: the signer, or under a credential roll the party Conformance item 6 names.

## Request

The member's client sends the request its VTA built. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member takes their ticket

```json
{
  "id": "urn:uuid:4d5e6f70-8192-43a4-a565-d6e7f8091a2b",
  "type": "https://trusttasks.org/spec/vtc/elections/tickets/issue/0.2#request",
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

### A group member takes their ticket under a credential roll

The roll is `{ "rule": "credential", "memberOf": "did:webvh:QmGroupEng:community.example:groups:g7" }`. The member presents the group membership credential the group's DID issued them, bound to this document. The presentation is abbreviated.

```json
{
  "id": "urn:uuid:9e0f1a2b-3c4d-45e6-8f70-8192a3b4c5d6",
  "type": "https://trusttasks.org/spec/vtc/elections/tickets/issue/0.2#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-02T08:10:00Z",
  "threadId": "urn:uuid:9e0f1a2b-3c4d-45e6-8f70-8192a3b4c5d6",
  "payload": {
    "electionId": "eng-offsite-2026",
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "request": {
      "label": "voter/eng-offsite-2026",
      "id": "z5jokfsiZx1sk1mJyQnBnR519B9mcQhw9a8LrF9VQdAjExwoLzcdVdZ2aXaH14a3mdF",
      "request": {
        "encoding": "z6P4jNSt4qHKYhSBGSWgr6VcQGMsjmfYGUBdYWNtS7xbpGsTuiY1PDfoce4yTApuqUm",
        "proof": "zSzFTCny3qaXpSHTsTE877ffQfSuF9T5zo53iuZkwx9NbLYSq3GeLZpRda286SBhqwJg"
      }
    },
    "presentation": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiablePresentation"],
      "holder": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "nonce": "urn:uuid:9e0f1a2b-3c4d-45e6-8f70-8192a3b4c5d6",
      "domain": "did:webvh:QmCommunity:community.example",
      "verifiableCredential": [
        {
          "@context": ["https://www.w3.org/ns/credentials/v2"],
          "type": ["VerifiableCredential", "MembershipCredential"],
          "issuer": "did:webvh:QmGroupEng:community.example:groups:g7",
          "validFrom": "2026-10-25T00:00:00Z",
          "validUntil": "2026-11-24T00:00:00Z",
          "credentialSubject": { "id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK" },
          "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "proofValue": "z5vExampleCredentialProof" }
        }
      ],
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "proofPurpose": "authentication",
        "challenge": "urn:uuid:9e0f1a2b-3c4d-45e6-8f70-8192a3b4c5d6",
        "domain": "did:webvh:QmCommunity:community.example",
        "proofValue": "z2pExamplePresentationProof"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-11-02T08:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3qExampleProofValue"
  }
}
```

## Response

The community answers with the blind pre-credential — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Every refusal above is a `trust-task-error`, never a `#response` document.

### The blind pre-credential

```json
{
  "id": "urn:uuid:5e6f7081-92a3-44b5-b676-e7f8091a2b3c",
  "type": "https://trusttasks.org/spec/vtc/elections/tickets/issue/0.2#response",
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

### Refused: the credential was revoked

```json
{
  "id": "urn:uuid:a1b2c3d4-e5f6-4789-8a9b-0c1d2e3f4a5b",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T08:10:01Z",
  "threadId": "urn:uuid:9e0f1a2b-3c4d-45e6-8f70-8192a3b4c5d6",
  "payload": {
    "code": "vtc/elections/tickets/issue:membershipRevoked",
    "retryable": false
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

Timing is the remaining channel. A ballot box sees when casts arrive, and if a cast followed its ticket's issuance closely, a community and a ballot box comparing notes could pair them. A member's VTA **SHOULD NOT** build a cast in the exchange that accepted the ticket, and a client **SHOULD** let a random interval pass before casting (CONVENTIONS §6.3, VTI-VOT-033); taking the ticket early in the registration period, well before voting opens, removes the channel altogether. The `disclosure` a member reads before asking ([`vtc/elections/show`](../../../show/0.4/spec.md)) says whether the community's operator also runs the ballot box, which is when that pairing needs no collusion at all (VTI-VOT-090).

Under a credential roll the community learns one more fact, from the presentation: that the requester is a member of the party `roll.memberOf` names. For the community's own roll that is nothing new. For one of its groups it is what the community's records already hold. For a member party it is a fact about a person the community may not otherwise know at all, learnt from the one exchange in which it already learns who is asking, and never paired with the ticket. A presentation of a predicate credential system proof that the requester holds such a credential, under a pseudonym scoped to the election, would let the community enforce the roll and one ticket per party without learning which member asked; that is not specified here. Checking the credential's status is a query to its issuer's status list, which identifies neither the credential nor its subject (VTI-CRD-013, VTI-CRD-014); a group membership credential needs no status query at all (VTI-DIR-085).

The requester is declared `identifierScope: pairwise`: the community must recognise the requester as a member, by whatever DID the member holds the community by, and nothing here needs anyone else to.

### Retention

The presentation is **transient**: the community discards it, and every claim it carries, once it has decided (Conformance item 10, VTI-CRD-042). The issuance record is kept while it enforces one ticket per party and while certification needs its count. The community **SHOULD** discard the stored pre-credential at `votingClosesAt`, and **SHOULD** reduce its per-signer issuance records to their count once the election is certified or cancelled: the count is all certification needs, and the per-signer record is the one thing on the community's side that says who took part. The audit row records the issuance and nothing PCS-derived (Conformance item 11, VTI-VOT-100), so that neither it nor any other record pairs a voter's identifier with a tag, a cast or a ballot (CONVENTIONS §6.4).

### Consent/purpose

The membership DID and the issuance are collected to enforce the roll and one ticket per party, and to bound the count at certification, and for nothing else. Under a credential roll the presentation is collected to decide this one issuance; the member was shown, before claiming, whose credential counts and that claiming discloses their membership of that party ([`vtc/elections/show`](../../../show/0.4/spec.md), VTI-VOT-090). The community **MUST NOT** use it, or the membership it shows, for any other purpose — admission, recognition, the directory — or disclose it to anyone (VTI-CRD-043).

Under a credential roll the community cannot say how many parties the roll admits, so it states no roll size and reports turnout as counted ballots against the tickets it issued (VTI-VOT-028). In particular the community **MUST NOT** use its issuance records to measure or publish who took part in an election beyond the count, and the design is such that it cannot learn from them how anyone voted.
