---
slug: vta/witness/accept
version: "0.2"
title: "VTA Witness — Accept"
summary: "A client hands its VTA the witness's answer to the submission — the VTA verifies the Verifiable Witness Credential against the session and stores it in the credential vault with its evidence."
status: draft
targetFrameworkVersion: "0.6.0"
category: credentials
keywords:
  - witness
  - vwc
  - relationship
  - outcome-evidence
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
    The request writes a credential into the principal's vault. Its contents are trusted on the witness's
    proofs, not the caller's, but an unattributable write into a credential store is a way to place there a
    credential the principal never sought. The response carries the witness's own signed documents, which
    are evidence on their own proofs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3 (it writes to the vault), so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Stores the VWC, with its evidence, in the credential vault, and consumes the session record. The same
    outcome handed back again stores nothing new.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    In: the witness's response, carrying the VWC — a credential about the principal's relationship DID,
    whose subject and object digest name it and its relationship credential. Out: the vault handle and the
    two evidence documents, returned to the principal's own delegate. The VTA signs nothing as the
    principal at this step.
retention:
  class: durable
  rationale: >-
    The VTA keeps the VWC with its evidence — the session document as built and the submit response as
    received — for the useful life of the VWC, because a verifier cannot pair the credential with its
    session without both.
errorCodes:
  - code: vta/witness/accept:sessionUnknown
    meaning: "No session record with this id in state `presented`, in a context the caller may act in — including one that expired, was accepted with a different outcome, or was opened by vta/witness/open/0.1."
    retryable: false
  - code: vta/witness/accept:responseInvalid
    meaning: "The document handed back is not the witness's witness/session/submit/0.2 #response for this session: wrong type, issuer, recipient or thread, or a proof that does not verify for the witness's DID."
    retryable: false
  - code: vta/witness/accept:vwcInvalid
    meaning: "The response is the witness's, but the VWC does not verify against the session: not issued by the witness, a failing proof, a taskContext or task digest naming another session document, a subject or object digest naming another party or credential, or a vwcDigestMultibase that does not match. Nothing is stored."
    retryable: false
related:
  - vta/witness/open
  - vta/witness/present
  - witness/session/submit
  - vta/witness/list
  - vault/credentials/get
---

## Abstract

The last of three steps by which a VTA is the participating party of a witnessed session for its principal ([conventions](../../../_shared/0.6/CONVENTIONS.md)). The client hands back the witness's answer to the submit document from [`vta/witness/present`](../../present/0.2/spec.md). The VTA verifies the Verifiable Witness Credential it carries as a verifier of the evidence would, and stores it in the credential vault **with its evidence** — the session document and the witness's submit `#response` — so that the principal can later show a verifier the exchange was witnessed ([`witness/session/submit/0.2`](../../../../witness/session/submit/0.2/spec.md), *This response is the outcome evidence*).

## What changed from 0.1

The session is conducted under [`witness/session/0.2`](../../../../witness/session/0.2/spec.md), so the outcome handed back is a [`witness/session/submit/0.2`](../../../../witness/session/submit/0.2/spec.md) `#response`, and this task continues only a session opened by `vta/witness/open/0.2` ([conventions](../../../_shared/0.6/CONVENTIONS.md)). `evidence` is the witness family's shared `OutcomeEvidence` ([`outcome-evidence.schema.json`](../../../../witness/_shared/0.1/outcome-evidence.schema.json)) — the same two members as 0.1's — so the evidence a holder gets here is, unchanged, what it ships to a verifier such as [`vtc/members/personhood/assert/0.2`](../../../../vtc/members/personhood/assert/0.2/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.6/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST** hand back in `response` the document the witness returned for the submission, exactly as received. A refusal (`challengeMismatch`, `presentationInvalid`) ends the session at the client and is not handed back.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse a caller without standing with `permissionDenied`.
2. Find the session record by `sessionId` among the contexts the caller may act in, opened by `vta/witness/open/0.2`. When it is `accepted` and `response` carries the VWC digest it accepted, answer again as it answered then, storing nothing ([conventions §5](../../../_shared/0.6/CONVENTIONS.md#5-retries)). Otherwise it **MUST** be `presented`, or the answer is `vta/witness/accept:sessionUnknown`.
3. Check `response` against the record before reading it ([conventions §3 item 3](../../../_shared/0.6/CONVENTIONS.md#3-the-session-record)): `type` `https://trusttasks.org/spec/witness/session/submit/0.2#response`, `issuer` the record's witness, `recipient` the record's relationship DID, `threadId` the session id, and a proof that verifies for the witness's DID; otherwise `vta/witness/accept:responseInvalid`.
4. Verify the VWC, refusing anything that fails with `vta/witness/accept:vwcInvalid` and storing nothing:
   1. its `issuer` is the record's witness — the session document's `recipient` — and its proof verifies for the witness's `assertionMethod`;
   2. its `taskContext` is the session id, and its `taskDigestMultibase` the task digest of the session document the record holds ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names));
   3. its `credentialSubject.id` is the record's relationship DID, and its `credentialSubject.object.digestMultibase` the digest of the relationship credential bound to the session;
   4. `vwcDigestMultibase` is the digest of the VWC as received, proof included.
5. Store the VWC in the credential vault, in the context of the relationship DID's key, under the record's `storeAs` or a handle it derives, **together with** the session document as the record holds it and `response` as received, each byte for byte.
6. Move the record to `accepted`, keeping only the stub [conventions §3 item 5](../../../_shared/0.6/CONVENTIONS.md#3-the-session-record) describes, and answer.

## Authorization

The authority is **standing over the context holding the relationship DID's key**, where the VWC is stored — the capability **`witnessRequest`**, never `sign` ([conventions §4](../../../_shared/0.6/CONVENTIONS.md#4-authorization)). The write is narrow: only a VWC that the session's own witness issued for the session's own document and credential, verified under Conformance item 4. A caller cannot use this task to store any other credential.

The witness's proofs establish what the VWC says and who said it; they authorize nothing. Verifying the request's `issuer`, transport identity or `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`sessionId`** — the session, as `vta/witness/open` returned it.
- **`response`** — the witness's `witness/session/submit/0.2` `#response`, as received.
- **`credentialId`** (response) — the vault handle the VWC is stored under.
- **`witness`** (response) — the witness that issued it.
- **`vwcDigestMultibase`** (response) — the VWC's digest, as the witness returned it and the VTA checked it.
- **`evidence`** (response) — `session`, the `witness/session` document as the VTA built it, and `outcome`, the witness's `witness/session/submit` `#response` as received, which carries the VWC.

### Why the VWC is not repeated in the response

It travels once, inside `evidence.outcome`, under the witness's proof. A second copy beside it could only disagree with it, and a stored credential is never re-serialised.

### Reading the evidence back later

The VTA stores both evidence documents with the VWC, so that a later presentation of the VWC can ship them. [`vault/credentials/get/0.1`](../../../../vault/credentials/get/0.1/spec.md) returns a stored credential and nothing beside it. [`vta/witness/list`](../../list/0.1/spec.md) reads back every stored VWC with its evidence, exactly as stored, so a client that lost this response — or another device of the same principal — can still present the VWC.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The witness's answer to the submission, handed back

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000105",
  "type": "https://trusttasks.org/spec/vta/witness/accept/0.2#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-15T09:00:06Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000003ff",
  "payload": {
    "sessionId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
    "response": {
      "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000204",
      "type": "https://trusttasks.org/spec/witness/session/submit/0.2#response",
      "issuer": "did:webvh:QmWitness:witness.example.org:a",
      "recipient": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
      "issuedAt": "2026-10-15T09:00:05Z",
      "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
      "parentThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
      "payload": {
        "vwc": {
          "@context": [
            "https://www.w3.org/ns/credentials/v2",
            "https://registry.trustoverip.org/dtg/context/v1"
          ],
          "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"],
          "issuer": "did:webvh:QmWitness:witness.example.org:a",
          "issuerScope": "public",
          "validFrom": "2026-10-15T09:00:05Z",
          "credentialSubject": {
            "id": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
            "predicate": "https://registry.trustoverip.org/dtg/vsc/witnessed/1",
            "object": {
              "digestMultibase": "zQmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk"
            }
          },
          "taskContext": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
          "taskDigestMultibase": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "created": "2026-10-15T09:00:05Z",
            "verificationMethod": "did:webvh:QmWitness:witness.example.org:a#key-0",
            "proofPurpose": "assertionMethod",
            "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSbwUM6dcvcRnzsBGVX9AsF3EyLoJvxfsMEKmYYCsCS4rBFPKVJTF"
          }
        },
        "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-10-15T09:00:05Z",
        "verificationMethod": "did:webvh:QmWitness:witness.example.org:a#key-0",
        "proofPurpose": "authentication",
        "proofValue": "z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEcqdoDwLWxaqA3Q1geV6"
      }
    }
  }
}
```

## Response

Once the VWC is verified and stored, the VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Stored, with its evidence

`evidence.session` is the document from the [`vta/witness/open`](../../open/0.2/spec.md#response) example and `evidence.outcome` the `response` above; both are abbreviated here to their first members.

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000106",
  "type": "https://trusttasks.org/spec/vta/witness/accept/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-15T09:00:07Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000003ff",
  "payload": {
    "sessionId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
    "credentialId": "vwc-alice-to-bob",
    "witness": "did:webvh:QmWitness:witness.example.org:a",
    "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "evidence": {
      "session": {
        "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
        "type": "https://trusttasks.org/spec/witness/session/0.2#request",
        "payload": {
          "parties": [
            "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
            "did:peer:2.Ez6LSBobForAliceRelationship.Vz6MkBobForAliceRelationship"
          ]
        }
      },
      "outcome": {
        "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000204",
        "type": "https://trusttasks.org/spec/witness/session/submit/0.2#response",
        "payload": {
          "vwc": { "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"] },
          "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
        }
      }
    }
  }
}
```

## Security & Privacy

### Data carried

In: the witness's response, which carries the VWC — a statement, signed by the witness, about the principal's relationship DID and the digest of its relationship credential, meant to be shown to verifiers. Out: the vault handle, the digest and the two evidence documents. No presentation travels at this step, and the VWC names the relationship DID only: under the member link the member DID appears in no VWC.

### Correlation

The VWC is a correlator by design: its `taskContext` and `taskDigestMultibase` bind it to one session document, so any two verifiers shown it can tell it is the same. A principal presenting it to several verifiers gives them a shared, exact join key. The witness is public by design (`identifierScope: public`): the VTA checks that the VWC's issuer is the session's witness and that the same witness signed the response, which a pairwise witness identifier would make the same string only by accident — and the cost is that the witness's DID names the collection of exchanges it observed. That is what makes the attestation checkable, and it is the reason to present it only where it is needed.

### Retention

The VTA keeps the VWC **with its evidence**, byte for byte, for the useful life of the VWC. A holder presenting it must ship both documents: a verifier cannot recompute the task digest without the session document, and cannot establish that the session completed without the witness's signed response. Deleting either leaves a credential that still verifies on the witness's proof and can no longer be shown to belong to a session that happened. The session record is consumed, leaving a stub until its expiry ([conventions §3](../../../_shared/0.6/CONVENTIONS.md#3-the-session-record)); beyond that the VTA keeps its audit record of the operation.

### Consent/purpose

The VWC is stored so that the principal can later show a verifier that the relationship was formed in a witnessed exchange. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
