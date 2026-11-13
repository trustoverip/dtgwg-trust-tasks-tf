---
slug: vtc/elections/show
version: "0.6"
extends: "0.5"
title: "VTC Elections — Show"
summary: "A member or an administrator reads one election whole — manifest and digest, state, voting parameters, disclosure, board checkpoints and certified result; a member of a party a credential roll names reads it by presenting that membership."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - poll
  - registrar
  - disclosure
  - certification
  - presentation
parties:
  - role: member or administrator, or a member of the party a credential roll names
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Whether the election exists for the caller, and whether the answer carries the count of tickets issued, depend on who
    is asking. The community can establish that from a verified proof or from a transport-authenticated sender; a proof
    makes the read attributable on every transport. A request carrying `presentation` is considered only with a verified
    proof by the presentation's holder (Conformance item 8).
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing; a bounded `issuedAt` keeps a captured request from being replayed indefinitely. A request
    carrying `presentation` is considered only with an `issuedAt` inside the community's acceptance window, which bounds
    how long the community must remember that the presentation was used (Conformance item 8).
sideEffects:
  level: none
  rationale: >-
    Reads one election's record. Persists nothing but, for a request carrying a presentation, that its document was
    used, for the acceptance window.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Discloses one election's manifest, the community's voting verification key, the community's disclosure about how the
    election is run, the ballot box's signed heads, the certified result once there is one, and — to an administrator
    only — how many tickets were issued. Nothing names a voter. In, where the reader presents one: their membership credential
    for the party a credential roll names, checked and discarded.
maxDocumentBytes:
  request: 98304
  rationale: >-
    A request may carry a presentation of one membership credential and its proofs, JSON- or base64-encoded, as
    vtc/elections/tickets/issue/0.3 does; the envelope and its proof are headroom.
retention:
  class: transient
  rationale: >-
    A view of a record the community keeps; the caller refetches as the election moves through its states. A presentation
    is discarded once the community has decided.
errorCodes:
  - code: vtc/elections/show:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it — including a reader whose presentation does not pass. These are answered identically (CONVENTIONS §7). A presentation whose check could not be completed is answered with the framework's `indeterminate` instead, before the election is looked up and identically whatever `electionId` names (Conformance item 8)."
    retryable: false
related:
  - vtc/elections/list
  - vtc/elections/create
  - vtc/elections/certify
  - vtc/elections/tickets/issue
  - voting/election/show
  - voting/board/head
  - voting/board/consistency
  - voting/result/show
  - vta/pcs/voter/register
  - vta/pcs/voter/read
  - vault/credentials/present
---

## Abstract

**VTC Elections — Show** reads one election of a community whole. It is what a member's client opens before requesting a ticket: the manifest says what is asked and the manifest digest is what the ticket request names; the **disclosure** says who holds and reads the board, whether the community also runs the ballot box, and that the system cannot protect a voter from being made to vote a certain way (VTI-VOT-090); the **voting parameters** are the key the ticket will be issued under, which the member's VTA needs ([`vta/pcs/voter/register`](../../../../vta/pcs/voter/register/0.5/spec.md)). After voting it is where the community's **checkpoints** of the board and its **certified result** are read. An election whose credential roll names a party that is itself a member of the community — another community, a group of one, a network — concerns people the community holds no record of; such a reader reads it by presenting, in the request, their membership credential for that party (VTI-VOT-015).

It is the read-one half of a split pair with [`vtc/elections/list`](../../list/0.5/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.5

A presentation's outage is answered as an outage, and gives nothing away (VTI-VOT-018). In `0.5` a presentation whose status could not be determined — a status list or a trust registry that could not be reached — was answered `electionUnknown`, and the presentation was checked only after the election had been looked up and found to be a member-party election. A reader who was entitled to the election was then told that it did not exist, and could not tell that trying again might succeed. Here the community verifies `presentation` before it looks up the election and without reference to it, making the checks [`vtc/elections/tickets/issue/0.3`](../../tickets/issue/0.3/spec.md) Conformance item 2 makes; where one cannot be completed it answers the framework's standard `indeterminate` (SPEC §8.3, retryable), identically whether the election exists, whether it is a member-party election, and whether the reader could see it. The checks made after the lookup read only the community's own records. Conformance item 8 is rewritten to say so; Security & Privacy gains *Side channels*.

The checks a presentation must pass are those of `vtc/elections/tickets/issue/0.3`, which, unlike `0.2`, also answers an unproven claimant of a member-party election `electionUnknown` (VTI-VOT-017), so that neither task says such an election exists.

The request and the response are unchanged, and the task declares no new error code. `0.6` declares `extends: "0.5"`. Released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules). `0.5` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `electionUnknown` both for an `electionId` it holds no election under and for one the caller may not see, identically in code, `details` and message (VTI-VOT-015). Who may see an election is as [`vtc/elections/list`](../../list/0.5/spec.md) Conformance items 1 and 2 define — a caller holding `vtc.elections.manage` (VTI-VOT-080) sees every election; any other caller sees one whose roll admits them now, once it is registered, or, under a `credential` roll, one whose roll could admit them by the community's records, as that item says — with one addition a list never makes: a reader whose `presentation` passes item 8 also sees a registered election whose `credential` roll names a member party. An election whose `credential` roll names a group the community no longer holds is seen only by a caller holding `vtc.elections.manage`, with or without a presentation (VTI-VOT-029).
2. **MUST** return the manifest exactly as registered, and `manifestDigest` computed over it.
3. **MUST** always carry `disclosure` and `parameters` (VTI-VOT-090, VTI-VOT-020), generated from the manifest and the community's configuration — never written by hand — so that a member has read both before any ticket is requested. `disclosure.sameOperator` **MUST** be true whenever the party operating the community's VTC also operates the ballot box. Under a `credential` roll, `disclosure.rollCredential` **MUST** be present, naming `roll.memberOf` and what it is to the community — `thisCommunity`, `communityGroup` or `memberParty` — and, under any other rule, **MUST** be absent (VTI-VOT-090). `memberOfName` **MUST** be absent where the caller may not know the group exists (VTI-DIR-070).
4. **MUST** carry `ticketsIssued` to a caller holding `vtc.elections.manage`, and **MUST NOT** carry it to any other caller.
5. **MUST** carry each checkpoint's `document` exactly as the ballot box sent it, proof included, and **MUST NOT** carry a checkpoint whose document did not verify against the ballot box's DID when it was read. A community **SHOULD** keep a checkpoint when voting opens, at intervals while voting is open, and when voting closes; certification checks the board against every one ([`vtc/elections/certify`](../../certify/0.1/spec.md), VTI-VOT-071).
6. **MUST** carry `electionKey` once the trustees of a sealed or tallied-encrypted election have formed it, and not before. The community forms it itself: it reads the board ([`voting/board/read`](../../../../voting/board/read/0.2/spec.md), open to it on any board) through its `electionKey` entry, replays the ceremony from the `trusteeStep` entries, each verified as CONVENTIONS §9.6 requires, and **MUST NOT** carry a key unless the entry states the very key that replay forms (CONVENTIONS §9.6 item 5). A key read any other way — from [`voting/trustees/status`](../../../../voting/trustees/status/0.1/spec.md), which carries no trustee's proof — **MUST NOT** be carried. With it, the community **MUST** carry `ceremony`: the `trusteeStep` requests through that `electionKey` entry, each exactly as received. A voter's VTA forms the key itself from them and encrypts under no other (CONVENTIONS §9.6), so a community cannot hand one member a key it holds. A community **SHOULD** read it as soon as voting opens. It is how a voter on a private board, who cannot read the board, encrypts a ballot (CONVENTIONS §9.6).
7. **MUST** carry `certified` once, and only once, [`vtc/elections/certify`](../../certify/0.1/spec.md) has recorded a certification. A result the ballot box published but the community has not certified is read from the ballot box ([`voting/result/show`](../../../../voting/result/show/0.1/spec.md)), never here.
8. **MUST** decide on `presentation` as follows (VTI-VOT-015, VTI-VOT-018):
   1. **Only bound and fresh.** It considers `presentation` only in a document whose proof verifies by an `authentication` key of its `issuer`, whose `issuedAt` is within the community's acceptance window, and whose `id` it has not already accepted a presentation in within that window (VTI-CRD-022). A presentation in any other document is treated as absent, and is not verified.
   2. **Verified first, on its own.** It **MUST** verify a presentation it considers before it looks up the election `electionId` names, and without reference to that election — not to whether one exists under that identifier, to its roll, or to the caller's standing toward it — making the checks [`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md) Conformance item 2 makes, with this document in place of the ticket request. Where their outcome is indeterminate — the issuer's DID document, a status list or a trust registry could not be obtained, or answered too long ago to be relied on (VTI-CRD-012) — it **MUST** answer the framework's `indeterminate` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) with `retryable` true, and stop. That answer is the same for an election that exists and one that does not, for a member-party election and any other, and for a caller who could see the election and one who could not; it **MUST NOT** be `electionUnknown`, and its `details`, where present, say only the kind of reference that could not be obtained.
   3. **Used only where it is needed.** The outcome is used only for a registered election whose `credential` roll names a **member party** — `roll.memberOf` neither the community's own DID nor the DID of one of its groups, as the community's records stood when the election was created — and which the caller may not see under item 1 without it. For any other election, and for an `electionId` it holds no election under, the community answers as item 1 would without the presentation. A group the community has deleted since is still one of its groups here, never a member party: its election is answered as item 1 says, and a presentation of a membership credential the group issued shows it to nobody (VTI-VOT-029). It **MUST NOT** refuse a request for carrying a presentation, or for carrying one that failed: a refusal would say what the election's roll is.
   4. **The checks a ticket claim makes.** It shows a member-party election on a presentation only where item 8.2's outcome is passed and the presentation also passes, as things stand now, the checks [`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md) Conformance items 6.2 to 6.4 make under `credential` — that `roll.memberOf`'s own membership of the community is still active, decided from the community's own records; that the credential's issuer is `roll.memberOf`; and that nothing item 8.2 read is revoked or suspended (VTI-VOT-025). These read only the community's own records, so they cannot be indeterminate.
   5. **One answer for every failure.** A check that fails is answered `electionUnknown`, identically to an election that does not exist; the community **MUST NOT** say which check failed, or that one was made.
   6. **Nothing kept.** Once it has decided it **MUST NOT** keep the presentation, the credential or any claim of it (VTI-CRD-042), and **MUST NOT** use the membership it shows for anything but this read (VTI-CRD-043). What it keeps is the `id` of a document whose presentation it accepted, for the acceptance window, so that item 8.1 can refuse its reuse; an `indeterminate` answer keeps nothing. A read admitted by a presentation is not standing to take a ticket: [`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md) checks its own presentation.

A conforming **producer** — a member's client about to request a ticket — **SHOULD** show the member `disclosure` before requesting one, and **MUST** request the ticket under the `manifestDigest` read here. Where the manifest asks a `text` question and its `textReleaseThreshold` is below 10, a client that shows the member the disclosure **MUST** show with it that the threshold is below 10 and that a written answer may identify its author (VTI-VOT-065, VTI-VOT-090). Under a `credential` roll, a client that shows the member the disclosure **MUST** show with it whose membership credential admits them (`rollCredential`) and that claiming a ticket shows the community their membership of that party (VTI-VOT-090), and **MUST NOT** claim a ticket with a credential the member did not choose to present.

A producer reading an election whose `credential` roll names a member party, which the community's records do not otherwise admit it to, carries `presentation`, made by the credential's subject, who signs this document: a Data Integrity presentation carries `nonce` equal to this document's `id` and `domain` equal to the community's DID as members of the presentation its holder proof signs, and a proof that also states `challenge` or `domain` states the same values; an SD-JWT presentation's key-binding JWT carries `nonce` equal to this document's `id` and `aud` equal to the community's DID — exactly as [`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md) requires. It **MUST** sign the document and set `issuedAt`. It presents one credential, the one the member chose to present to this community for this election (VTI-VOT-016), and discloses nothing from it that items 8.2 and 8.4 do not read. Before presenting, a client that acts for a member **MUST** tell them that reading the election shows the community their membership of that party, before they have seen what the election asks (VTI-VOT-090). A producer **MUST NOT** carry a presentation to learn whether an election exists or which elections concern it, **MUST NOT** carry one in a read of an election it can already see, and, after a lost answer, sends a new document — a new `id`, a new presentation — never the same one again. Answered `indeterminate`, it **MAY** send the same document again, after any `retryAfter` and with backoff, while its `issuedAt` is inside the acceptance window: the community kept nothing of it. A client that holds no key of the member reads through the member's VTA ([`vta/pcs/voter/read`](../../../../vta/pcs/voter/read/0.7/spec.md)), naming the credential, and the VTA builds the presentation.

## Authorization

The authority is **the caller's standing toward this election**: being a party its roll admits, or holding `vtc.elections.manage`, read from the community's own records at execution time. The proof or the transport identity says who is asking ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); it is the lookup key for the standing, not the standing. A caller without it is answered as if the election did not exist. Under a `credential` roll the standing to vote is proven only when a ticket is claimed ([`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md), VTI-VOT-025). Where the roll names the community or one of its groups, a caller its records show the roll could admit reads the election, and only the claim decides whether they may vote in it. Where it names a member party, the community's records say nothing about that party's members, so the standing to read is the party's own membership — for the party itself — or a presentation of the reader's membership of the party, carried in this read and checked as a claim's would be (Conformance item 8, VTI-VOT-015); without one, such an election would be shown to the world. The count of tickets issued is an administrator's, because it is the bound certification holds the count to (VTI-VOT-072) and, while voting is open, a running measure of turnout the community chooses whether to publish.

## Definitions

- **`electionId`** (REQUIRED) — the election to read.
- **`manifest`**, **`manifestDigest`** (response) — the election's [`Manifest`](../../../../voting/_shared/0.4/voting.schema.json) and its digest.
- **`state`** (response) — a `RegistrarState`, as [`vtc/elections/list`](../../list/0.5/spec.md) defines it.
- **`parameters`** (response) — the community's [`VotingParameters`](../../../../voting/_shared/0.4/voting.schema.json).
- **`disclosure`** (response) — the community's [`Disclosure`](../../../../voting/_shared/0.4/voting.schema.json) for this election. Under a `credential` roll it carries a `RollCredentialDisclosure` (`rollCredential`).
- **`checkpoints`** (response) — each a `Checkpoint`: a [`BoardHead`](../../../../voting/_shared/0.4/voting.schema.json) and the ballot box's signed [`voting/board/head`](../../../../voting/board/head/0.1/spec.md) response document that carried it, kept whole.
- **`ticketsIssued`** (response) — tickets issued for the election; administrators only.
- **`electionKey`** (response) — the [`ElectionKey`](../../../../voting/_shared/0.4/voting.schema.json) a sealed or tallied-encrypted election's ballots are encrypted under, once formed.
- **`presentation`** — only to read an election whose `credential` roll names a member party: the reader's presentation of their membership credential for the party `roll.memberOf` names, bound to this document — a W3C Verifiable Presentation, or a compact SD-JWT presentation with its key-binding JWT, as [`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md) carries one. Absent in every other read.
- **`certified`** (response) — a `Certification`: the certified [`ElectionResult`](../../../../voting/_shared/0.4/voting.schema.json), when it was certified, and the auditors' attestations it was certified with.

## Request

A member's client reads the election before requesting a ticket. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read one election

```json
{
  "id": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "type": "https://trusttasks.org/spec/vtc/elections/show/0.6#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-02T08:00:00Z",
  "threadId": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "payload": {
    "electionId": "board-2026"
  }
}
```

### Read an election put to a member party's members

The community's roll is `{ "rule": "credential", "memberOf": "did:webvh:QmPartner:partner.example" }`, naming a partner community that is itself a member of this one. The reader is a member of the partner, of whom this community holds no record; they present the membership credential the partner issued them, bound to this document. The presentation is abbreviated. Without it, or with one that does not pass, the answer is `electionUnknown`.

```json
{
  "id": "urn:uuid:c5d6e7f8-0912-4a3b-9c4d-5e6f708192a3",
  "type": "https://trusttasks.org/spec/vtc/elections/show/0.6#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-02T08:05:00Z",
  "expiresAt": "2026-11-02T09:05:00Z",
  "threadId": "urn:uuid:c5d6e7f8-0912-4a3b-9c4d-5e6f708192a3",
  "payload": {
    "electionId": "partner-charter-2026",
    "presentation": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiablePresentation"],
      "holder": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "nonce": "urn:uuid:c5d6e7f8-0912-4a3b-9c4d-5e6f708192a3",
      "domain": "did:webvh:QmCommunity:community.example",
      "verifiableCredential": [
        {
          "@context": ["https://www.w3.org/ns/credentials/v2"],
          "type": ["VerifiableCredential", "MembershipCredential"],
          "issuer": "did:webvh:QmPartner:partner.example",
          "validFrom": "2026-06-01T00:00:00Z",
          "validUntil": "2027-06-01T00:00:00Z",
          "credentialSubject": { "id": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH" },
          "credentialStatus": {
            "type": "BitstringStatusListEntry",
            "statusPurpose": "revocation",
            "statusListIndex": "4211",
            "statusListCredential": "https://partner.example/status/1"
          },
          "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "proofValue": "z5vExampleCredentialProof" }
        }
      ],
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "proofPurpose": "authentication",
        "challenge": "urn:uuid:c5d6e7f8-0912-4a3b-9c4d-5e6f708192a3",
        "domain": "did:webvh:QmCommunity:community.example",
        "proofValue": "z2pExamplePresentationProof"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "created": "2026-11-02T08:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3qExampleProofValue"
  }
}
```

## Response

The community answers with the election — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error`.

### Registration open, one checkpoint kept

```json
{
  "id": "urn:uuid:8192a3b4-c5d6-4e7f-98a9-1a2b3c4d5e6f",
  "type": "https://trusttasks.org/spec/vtc/elections/show/0.6#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T08:00:01Z",
  "threadId": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "payload": {
    "manifest": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:community.example",
      "title": "Board election 2026",
      "questions": [
        {
          "key": "chair",
          "type": "choice",
          "text": "Who should chair the board?",
          "options": [
            { "key": "alice", "label": "Alice" },
            { "key": "bob", "label": "Bob" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": true
        }
      ],
      "secrecy": "open",
      "roll": { "rule": "members" },
      "registrationOpensAt": "2026-11-01T00:00:00Z",
      "votingOpensAt": "2026-11-08T00:00:00Z",
      "votingClosesAt": "2026-11-15T00:00:00Z",
      "revote": "replace",
      "board": { "visibility": "public", "auditors": ["did:web:audit.example"] },
      "ballotBox": "did:web:ballots.example",
      "binding": true,
      "allowEarlyClose": false,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10
    },
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "state": "registration",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z3Xef1JY2x1s2vY6yKWfLu8Ap2vKuoZGwaQLZ2tjq8QkEZbGeZyaNgtznZ8B7N1yf14"
    },
    "disclosure": {
      "ballotBox": "did:web:ballots.example",
      "ballotBoxOperator": "Ballots Example Ltd",
      "sameOperator": false,
      "boardVisibility": "public",
      "auditors": ["did:web:audit.example"],
      "revote": "replace",
      "coercionResistance": false
    },
    "checkpoints": [
      {
        "head": {
          "electionId": "board-2026",
          "treeSize": 0,
          "rootHash": "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n",
          "issuedAt": "2026-11-01T00:00:05Z"
        },
        "document": {
          "id": "urn:uuid:92a3b4c5-d6e7-4f80-a9ba-2b3c4d5e6f70",
          "type": "https://trusttasks.org/spec/voting/board/head/0.1#response",
          "issuer": "did:web:ballots.example",
          "recipient": "did:webvh:QmCommunity:community.example",
          "issuedAt": "2026-11-01T00:00:05Z",
          "threadId": "urn:uuid:a3b4c5d6-e7f8-4091-bacb-3c4d5e6f7081",
          "payload": {
            "head": {
              "electionId": "board-2026",
              "treeSize": 0,
              "rootHash": "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n",
              "issuedAt": "2026-11-01T00:00:05Z"
            }
          },
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "verificationMethod": "did:web:ballots.example#key-1",
            "created": "2026-11-01T00:00:05Z",
            "proofPurpose": "assertionMethod",
            "proofValue": "z5BallotBoxHeadProofValue"
          }
        }
      }
    ]
  }
}
```

The checkpoint's `document` is illustrative of its shape; its payload is whatever [`voting/board/head`](../../../../voting/board/head/0.1/spec.md) defines, kept as received.

### Unknown, or not visible to the caller

```json
{
  "id": "urn:uuid:b4c5d6e7-f809-4a1b-8cdc-4d5e6f708192",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T08:00:01Z",
  "threadId": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "payload": {
    "code": "vtc/elections/show:electionUnknown",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The response carries what the community publishes about one election to the parties who may take part: the manifest, the community's voting verification key, the disclosure, the ballot box's signed heads and, once certified, the result and the auditors' attestations. `ticketsIssued` goes to administrators only. Nothing names a voter: the community never holds a ballot (CONVENTIONS §2), and a checkpoint is a tree size and a hash. A community **MUST NOT** put into `ext` anything about who took a ticket.

A request may carry one more thing: a reader's presentation of their membership credential for the party a `credential` roll names, where that party is a member of the community and the reader is one of its members. It carries one credential, disclosing only what Conformance items 8.2 and 8.4 read, and the community discards it once it has decided (item 8.6).

### Correlation

The community is declared `identifierScope: public`: it is addressed by the DID its manifests name as `community`, which its members, its ballot box and its auditors all know it by, and which every ticket and cast is bound to (CONVENTIONS §3). A pairwise identifier for it would break that binding and tell the parties nothing they could check.

The community learns that a member read an election, and when. Reading the election shortly before requesting a ticket is the expected flow and tells the community nothing it does not learn from the issuance itself. A member's client **SHOULD NOT** read an election and then cast at once: the ballot box sees when casts arrive, and the community sees when reads arrive, and the two together are the timing that CONVENTIONS §6 and VTI-VOT-033 ask both parties not to supply.

Under a credential roll naming a member party, the read itself shows the community that the reader is a member of that party — a fact about a person the community may not otherwise know at all — before the reader has seen what the election asks, and whether or not they go on to claim a ticket. That is why the read carries a presentation only by the member's choice (VTI-VOT-016) and only for the one election (the producer's paragraph): a client that presented its credentials to find out which elections concern it would show the community every membership it holds. It is also why [`vtc/elections/list`](../../list/0.5/spec.md) never carries one, and never shows such an election to a reader who has not presented. The answer to a read that does not pass is the answer to an election that does not exist, so a presentation is no oracle for whether an election exists, or for which of the checks failed.

### Side channels

An outage must not be an oracle either. Were the presentation checked only for a member-party election that exists, an unreachable status list would answer differently for an `electionId` that names one than for one that does not, whatever answer the outage was given. So Conformance item 8.2 checks the presentation — the one step that depends on anyone else: the issuer's DID document, its status list, its trust registry — before the election is looked up, and the same outage gets the same `indeterminate` whatever the request names. What follows reads only the community's own records. A community **SHOULD** make the time from the end of item 8.2 to its answer independent of whether the election exists and of which check refused the read, and **SHOULD NOT** answer an unknown election from a cheaper path than a known one it hides. A community that caches status lists or DID documents **SHOULD** refresh them on a schedule of its own rather than on demand, or make a cached and an uncached check take the same time: a status list fetched on demand and kept answers faster for an issuer someone presented recently, which tells a prober presenting a credential of that issuer that others have been reading under it ([`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md) Security & Privacy). An `indeterminate` answer tells the reader only that a reference of its own presentation could not be obtained, which it could learn by fetching that reference itself.

### Retention

Transient. The community keeps nothing of the request beyond its ordinary request log. Of a request carrying a presentation it keeps only the document's `id`, for its acceptance window, so the presentation cannot be used twice (Conformance item 8.1); never the presentation, the credential or any claim of it. The checkpoints and the certification it returns are records it keeps for as long as the election's result stands, under [`vtc/elections/certify`](../../certify/0.1/spec.md).

### Consent/purpose

The read exists so that a member can decide, on the community's own disclosure, whether to take part, and so that anyone who may see the election can check the board against what the community kept. The community **SHOULD NOT** use a member's reads of an election for any other purpose, and in particular not as a measure of who intends to vote. A presentation made to read an election is collected to decide that one read, and the community **MUST NOT** use it, or the membership it shows, for any other purpose — admission, recognition, the directory, the roll of a later election — or disclose it to anyone (VTI-CRD-043).
