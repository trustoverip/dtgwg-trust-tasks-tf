---
slug: vta/witness/open
version: "0.2"
title: "VTA Witness — Open"
summary: "A client asks its VTA to open a witnessed session for one of its principal's relationships — the VTA checks custody and the credentials, and returns the witness/session document, signed as the relationship DID, for the client to send."
status: draft
targetFrameworkVersion: "0.6.0"
category: credentials
keywords:
  - witness
  - vwc
  - relationship
  - session
  - vta
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: The principal's VTA
    requirement: REQUIRED
    member: recipient
  - role: Witness
    requirement: REQUIRED
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA sign as the principal's relationship DID, addressed to a third party the
    caller names, and binds to the session the credentials that will be shown to it. A request whose
    integrity depended on the transport would let a compromised channel choose the witness, and so the
    party the principal's credentials are shown to.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3 (the VTA acts as the relationship DID and opens a record), so SPEC §7.3
    item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    The VTA opens a session record, which expires on its own if the session goes no further. It contacts
    nobody: the session exists at the witness only once the client sends the document.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA signs, as the principal's relationship DID, a document naming both relationship DIDs. In:
    DIDs and vault handles. Out: that document. No credential travels either way.
retention:
  class: exchange
  rationale: >-
    The session record lives until the session is accepted or the record expires, minutes at most. The
    session document it holds outlives it only as evidence stored with the VWC by vta/witness/accept.
errorCodes:
  - code: vta/witness/open:notFound
    meaning: "A named credential is not in the credential vault in a context the caller may act in, or is not active there."
    retryable: false
  - code: vta/witness/open:credentialMismatch
    meaning: "A named credential does not fit its role: the relationship credential is not a RelationshipCredential issued by relationshipDid naming counterpartyDid, or the membership credential is not a MembershipCredential naming the expected DID, or either is outside its validity period."
    retryable: false
  - code: vta/witness/open:witnessUnresolvable
    meaning: "The witness's DID could not be resolved, so no answer from it could be verified."
    retryable: true
related:
  - vta/witness/present
  - vta/witness/accept
  - witness/session
---

## Abstract

The first of three steps by which a VTA is the participating party of a witnessed session for its principal, while the client carries the messages ([conventions](../../../_shared/0.6/CONVENTIONS.md)). The client names the witness, the relationship and the credentials to present. The VTA checks that it can act as the relationship DID for this caller and that the credentials fit, opens its record of the session, and returns the [`witness/session/0.2`](../../../../witness/session/0.2/spec.md) document, built and signed as the relationship DID. The client sends it to the witness from the relationship DID and hands the witness's answer to [`vta/witness/present`](../../present/0.2/spec.md).

Everything that will be presented is chosen and checked here, before the witness learns anything: a session that cannot succeed fails before it exists.

## What changed from 0.1

The session is conducted under [`witness/session/0.2`](../../../../witness/session/0.2/spec.md) and [`witness/session/submit/0.2`](../../../../witness/session/submit/0.2/spec.md), which defines the member presentation's carriage itself (`memberPresentation`). So a VTA no longer needs to know a witness's own carriage before it opens a session with a member DID, and `vta/witness/open:memberLinkUnsupported` is gone. A session opened here is continued only by `vta/witness/present/0.2` and `vta/witness/accept/0.2` ([conventions](../../../_shared/0.6/CONVENTIONS.md)).

*Non-normative.* A witness that serves only `witness/session/0.1` either refuses the 0.2 session document with `unsupportedVersion`, or processes it forward-compatibly ([SPEC §5.2](/SPEC.md#52-compatibility-rules)) and then cannot read the member presentation at submit. The client learns which from the witness's answers, and a 0.1 answer to a 0.2 session is `vta/witness/present:responseInvalid`. The VTA never contacts the witness to find out first ([conventions §2](../../../_shared/0.6/CONVENTIONS.md#2-custody)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.6/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Name in `witness` a witness the principal has chosen to show these credentials to.
2. Name `membership.memberDid` only when it intends the member DID to be disclosed to the witness, and **SHOULD** name it only for a witness whose published policy asks for membership evidence: a witness that asks for none may ignore the member presentation, and the member DID would have been disclosed for nothing.
3. Send the returned `document` to the witness **from `relationshipDid`** — where the transport authenticates its sender, the sender is `relationshipDid` — unaltered, and to nobody else.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse a caller without standing ([conventions §4](../../../_shared/0.6/CONVENTIONS.md#4-authorization)), or one for whose contexts it holds no key of `relationshipDid` — or, when `membership.memberDid` is present, of `memberDid` — with `permissionDenied`.
2. Refuse with `malformedRequest` unless `relationshipDid`, `counterpartyDid` and `witness` are three different DIDs and `membership.memberDid`, when present, is none of them.
3. Refuse with `vta/witness/open:notFound` a credential handle that names no active credential in a context the caller may act in, and with `vta/witness/open:credentialMismatch` a relationship credential that is not a `RelationshipCredential` issued by `relationshipDid` whose subject is `counterpartyDid`, or a membership credential that is not a `MembershipCredential` whose subject is `membership.memberDid` (or `relationshipDid` when `memberDid` is absent), or either outside its validity period.
4. Resolve `witness`, refusing with `vta/witness/open:witnessUnresolvable` when it cannot.
5. Build the `witness/session/0.2` document: a fresh `id`, `threadId` equal to that `id`, `parentThreadId` equal to `relationshipThreadId`, `issuer` `relationshipDid`, `recipient` `witness`, a current `issuedAt`, and `parties` `[relationshipDid, counterpartyDid]`. It **MUST** sign it with a key of `relationshipDid` ([conventions §2](../../../_shared/0.6/CONVENTIONS.md#2-custody)), and put nothing in it beyond what `witness/session` defines — no relationship context in `ext`.
6. Open the session record ([conventions §3](../../../_shared/0.6/CONVENTIONS.md#3-the-session-record)) in state `opened`, keyed by the document's `id`, holding the document exactly as signed and the digests of the credentials it checked.
7. Answer with the session id, the document, and the record's expiry.

## Authorization

The authority is **standing over the contexts the session uses** — the one holding `relationshipDid`'s key, and every one holding a named credential or `memberDid`'s key ([conventions §4](../../../_shared/0.6/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`witnessRequest`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task nor satisfy it under `sign` alone. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

This is the step at which the principal's credentials are committed to being shown to the named witness, and a VTA's own policy may gate it ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

## Definitions

- **`witness`** — the witness's public DID.
- **`relationshipDid`** — the principal's relationship DID; the participating party.
- **`counterpartyDid`** — the other party's relationship DID.
- **`relationshipThreadId`** — the `threadId` of the relationship exchange the session is nested in.
- **`relationshipCredentialId`** — the vault handle of the relationship credential to present.
- **`membership`** — OPTIONAL membership evidence: `credentialId`, the vault handle of a membership credential, and `memberDid`, the DID it names when that is not `relationshipDid`.
- **`storeAs`** — OPTIONAL vault handle for the VWC.
- **`sessionId`** (response) — the session's id: the id of `document`.
- **`document`** (response) — the signed `witness/session/0.2` document to send.
- **`expiresAt`** (response) — when the VTA's record of the session expires.

### Membership evidence, and the member link

A witness may witness only members of the communities it serves, and its evidence is a membership credential (VMC). A VMC names the member's own DID, not the pairwise relationship DID the party acts as, so there are two cases:

- **The VMC names `relationshipDid`.** Omit `memberDid`. The VMC rides in the relationship presentation.
- **The VMC names a member DID** — the common case. Name it in `memberDid`. At [`vta/witness/present`](../../present/0.2/spec.md) the VTA builds a second presentation by that DID, bound to the same session and carried in the submit document's `memberPresentation`, so the witness can see that whoever holds the relationship key also held the member key, in this one session.

The second case discloses `memberDid` to the witness, and lets it link the pairwise relationship DID to the member's community identity. That is why it takes an explicit `memberDid`: the VTA could read the DID off the credential, and does not, so that no session discloses a member DID its caller did not name.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Witnessed as a member, through the member link

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000101",
  "type": "https://trusttasks.org/spec/vta/witness/open/0.2#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-15T09:00:00Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000001ff",
  "payload": {
    "witness": "did:webvh:QmWitness:witness.example.org:a",
    "relationshipDid": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
    "counterpartyDid": "did:peer:2.Ez6LSBobForAliceRelationship.Vz6MkBobForAliceRelationship",
    "relationshipThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
    "relationshipCredentialId": "vrc-alice-to-bob",
    "membership": {
      "credentialId": "vmc-community-example",
      "memberDid": "did:webvh:QmAlice:alice.example.com"
    },
    "storeAs": "vwc-alice-to-bob"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The session document to send

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000102",
  "type": "https://trusttasks.org/spec/vta/witness/open/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-15T09:00:01Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000001ff",
  "payload": {
    "sessionId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
    "document": {
      "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
      "type": "https://trusttasks.org/spec/witness/session/0.2#request",
      "issuer": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
      "recipient": "did:webvh:QmWitness:witness.example.org:a",
      "issuedAt": "2026-10-15T09:00:01Z",
      "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
      "parentThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
      "payload": {
        "parties": [
          "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
          "did:peer:2.Ez6LSBobForAliceRelationship.Vz6MkBobForAliceRelationship"
        ]
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-10-15T09:00:01Z",
        "verificationMethod": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship#key-1",
        "proofPurpose": "authentication",
        "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSbwUM6dcvcRnzsBGVX9AsF3EyLoJvxfsMEKmYYCsCS4rBFPKVJTF"
      }
    },
    "expiresAt": "2026-10-15T09:10:01Z"
  }
}
```

## Security & Privacy

### Data carried

DIDs and vault handles in; one document naming two relationship DIDs and the enclosing thread out. No credential travels in either direction at this step. A producer **SHOULD** use handles that are handles — `storeAs` included — not descriptions of the relationship, and **MUST NOT** put relationship context in `ext`: the witness specifications ask that a witness not be told what an exchange is about, and a VTA told it writes it into a record.

### Correlation

The document is what the witness will see first: both relationship DIDs, which `witness/session` uses as the join key between the two parties' sessions, and the enclosing thread. The witness is public by design (`identifierScope: public`) — recognisable by both parties and by every verifier, so a verifier can tell one witness attested both halves of an exchange — and choosing it is therefore the principal's substantive privacy decision, which this task leaves to the caller. Naming `memberDid` here commits the session to disclosing the member DID at the next step; see [`vta/witness/present`](../../present/0.2/spec.md).

### Retention

The VTA keeps the session record until the session is accepted or the record expires ([conventions §3](../../../_shared/0.6/CONVENTIONS.md#3-the-session-record)). It holds no presentation and no challenge. The session document itself is durable evidence once a VWC cites it, and [`vta/witness/accept`](../../accept/0.2/spec.md) stores it with the VWC.

### Consent/purpose

The DIDs and credential handles are given so the VTA can open one witnessed session, with one witness the principal chose, presenting exactly the credentials named. The VTA does not select a witness, substitute one, or widen what is presented. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
