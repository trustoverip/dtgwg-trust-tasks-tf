---
slug: voting/election/register
version: "0.1"
title: "Voting — Election Register"
summary: "A community registers an election — its signed manifest and the voting key its tickets are issued under — with the ballot box the manifest names."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - election
  - ballot-box
  - poll
  - manifest
parties:
  - role: Community (the election's registrar)
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The registration fixes, for the life of the election, what is asked, who may vote and which
    key admits a cast. Anyone recounting the election later needs to show that the community
    itself registered exactly this manifest and key, after the transport that carried it has
    closed, so the request is signed by the community. The response carries the board's first
    head, which is signed by being carried in a document the ballot box signs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor: the ballot box
    must be able to place a registration in a bounded acceptance window.
sideEffects:
  level: mutating
  rationale: >-
    Creates the election at the ballot box: its manifest, its voting parameters and an empty
    board, whose lifecycle the ballot box then runs.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the manifest the community publishes and its public voting key — nothing about any
    voter. Out: the ballot box's summary of the election (state, counts, head).
retention:
  class: durable
  rationale: >-
    The manifest and the voting key are what every cast is checked against and what any
    recount starts from, so the ballot box keeps them for as long as it keeps the board.
errorCodes:
  - code: voting/election/register:alreadyRegistered
    meaning: "This election is already registered here with another manifest digest or voting key; a registered manifest never changes (VTI-VOT-013)."
    retryable: false
  - code: voting/election/register:manifestMismatch
    meaning: "`manifestDigest` is not the digest of `manifest`."
    retryable: false
  - code: voting/election/register:unsupportedSecrecy
    meaning: "The manifest's secrecy level is not served by this version (0.1 serves `open` only)."
    retryable: false
  - code: voting/election/register:invalidManifest
    meaning: "The manifest is well-formed but cannot be run: it names another ballot box, its times are out of order, or a question cannot be answered as specified."
    retryable: false
related:
  - voting/election/show
  - voting/election/close
  - voting/election/cancel
  - voting/ballot/cast
  - vtc/elections/create
---

## Abstract

The **Voting — Election Register** Trust Task is how a community hands an election to its ballot box. The community signs a request carrying the election's manifest, the manifest's digest and the voting key the election's tickets are issued under; the ballot box checks that the community is who registers it, recomputes the digest, checks the manifest can be run, and creates the election's empty board. The community's VTC sends it from inside [`vtc/elections/create`](../../../../vtc/elections/create/0.1/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the community) **MUST**:

1. Sign the request as the community's DID, and set `community` and `manifest.community` to it.
2. Set `manifestDigest` to the SHA-256 multihash of the RFC 8785 (JCS) canonicalization of `manifest` (VTI-VOT-011).
3. Set `manifest.ballotBox` to the DID of the ballot box it sends the request to.
4. Send `parameters` with the voting key, never the community's hidden-vetting helper key (VTI-VOT-020).

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements). A `textReleaseThreshold` below 10 fails the schema and is `malformedRequest`.
2. Refuse with `permissionDenied` unless the proven signer, `community` and `manifest.community` are the same DID (VTI-VOT-012).
3. Recompute the digest of `manifest` as received; refuse with `voting/election/register:manifestMismatch` if it is not `manifestDigest`.
4. Refuse with `voting/election/register:unsupportedSecrecy` unless `manifest.secrecy` is `open` (VTI-VOT-060).
5. Refuse with `voting/election/register:invalidManifest` when any of these holds:
   - `manifest.ballotBox` is not the ballot box's own DID (the document's `recipient`);
   - `registrationOpens` ≤ `votingOpens` < `votingCloses` does not hold, or `votingCloses` has already passed;
   - two questions share a `key`, or two options of one question share a `key`;
   - a `choice` question has `minSelections` > `maxSelections` or `maxSelections` greater than its number of options;
   - a `score` question has `min` ≥ `max`;
   - `trustees` is present on an `open` manifest.
6. Treat a registration of the same `(community, electionId)` with the same `manifestDigest` and the same `parameters` as a repeat: answer with the election as it now stands and change nothing. Refuse one with another digest or other parameters with `voting/election/register:alreadyRegistered` (VTI-VOT-013).
7. Otherwise create the election with an empty board, in state `registered` (or `registration` once `registrationOpens` has passed), and answer with its summary.

Once an election is registered, the ballot box **MUST** run its lifecycle on its own clock:

1. At `votingOpens`, append a `LifecycleEntry` of kind `opened` and enter `voting`.
2. At `votingCloses`, unless the election was closed early or cancelled, append a `LifecycleEntry` of kind `closed` and enter `closed`.
3. Once closed, compute the result from the board and the registration alone (VTI-VOT-070), append it as a `ResultEntry`, and enter `tallied`. A cancelled election is never tallied.

## Authorization

The authority is **being the community the manifest names**: only a community registers its own elections ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)). Consumer item 2 enforces it by comparing the proven signer with `community` and `manifest.community`. The proof establishes who signed; the authority is the equality, and a document signed by anyone else — an administrator's own DID included — registers nothing. Whether a community's ballot box accepts registrations from a given community at all (an operator serving several communities, or one) is the ballot box's own policy, outside this specification.

## Definitions

- **`community`** — the community's DID; the registrar of the election.
- **`manifest`** — the election's manifest (`Manifest`).
- **`manifestDigest`** — the manifest's digest, as the community computed it.
- **`parameters`** — the community's voting parameters (`VotingParameters`); `votingKey` is the key every cast's attestation is checked against ([conventions §3](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)).
- **`election`** (response) — the ballot box's summary of the election (`ElectionSummary`), including the board's current head.

## Request

The community sends the request to the ballot box its manifest names. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

A public-board board election with a two-seat choice and an optional motion.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000a001",
  "type": "https://trusttasks.org/spec/voting/election/register/0.1#request",
  "issuer": "did:webvh:QmCommunity:vtc.example.org",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-01T10:00:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000a0ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "manifest": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "title": "Board election 2026",
      "questions": [
        {
          "key": "board",
          "type": "choice",
          "text": "Elect up to two board members",
          "options": [
            { "key": "ana", "label": "Ana" },
            { "key": "ben", "label": "Ben" },
            { "key": "cai", "label": "Cai" }
          ],
          "minSelections": 0,
          "maxSelections": 2,
          "optional": false
        },
        {
          "key": "motion-7",
          "type": "choice",
          "text": "Adopt the revised code of conduct",
          "options": [
            { "key": "yes" },
            { "key": "no" },
            { "key": "abstain" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": true
        }
      ],
      "secrecy": "open",
      "roll": { "rule": "members" },
      "registrationOpens": "2026-11-02T09:00:00Z",
      "votingOpens": "2026-11-09T09:00:00Z",
      "votingCloses": "2026-11-16T09:00:00Z",
      "revote": "replace",
      "board": { "visibility": "public", "auditors": [] },
      "ballotBox": "did:webvh:QmVote:vote.example.net",
      "binding": true,
      "allowEarlyClose": false,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10
    },
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z4rVtgKeyBS12381G2abcdefghXyZ9wQ7nPkM"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-01T10:00:00Z",
    "verificationMethod": "did:webvh:QmCommunity:vtc.example.org#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5Vy7mR3ExampleCommunitySignatureOverTheRegistration"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the election's summary, with the board's first (empty) head. Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000a002",
  "type": "https://trusttasks.org/spec/voting/election/register/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "recipient": "did:webvh:QmCommunity:vtc.example.org",
  "issuedAt": "2026-11-01T10:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000a0ff",
  "payload": {
    "election": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "state": "registered",
      "ballotsAdmitted": 0,
      "head": {
        "electionId": "board-2026",
        "treeSize": 0,
        "rootHash": "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n",
        "issuedAt": "2026-11-01T10:00:01Z"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-01T10:00:01Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3hJ8sKqExampleBallotBoxSignatureOverTheFirstHead"
  }
}
```

## Security & Privacy

### Data carried

The manifest (questions, options, times, roll rule, board visibility, auditor DIDs) and the community's public voting key. Nothing about any voter: the roll rule names a rule, and the `listed` DIDs it may carry are parties the community chose to enfranchise by name, which the community publishes anyway. A community that does not want its ballot box to hold the question text **MAY** register a manifest without `text` and `label` (VTI-VOT-014); the digest then covers the manifest without them, and members see the text from the community. The manifest's `title`, `description`, `text` and `label` are free text written by the community, which signs the document; a ballot box shows them, where it shows them at all, as the community's words.

### Correlation

The ballot box learns which community runs which election and when. Both parties use public identifiers — the identifier scope of both is `public`, because a voter must be able to find the ballot box the manifest names and recognise the community whose election it is, and neither identifier says anything about a voter.

### Retention

Durable. The registration is the starting point of every recount (VTI-VOT-070): a ballot box keeps the signed request for as long as it keeps the board, and a community keeps the signed response with it.

### Consent/purpose

To run one election as its manifest describes. The ballot box uses the manifest and key only to admit casts, keep the board and compute the result. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
