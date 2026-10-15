---
slug: vta/witness/present
version: "0.1"
title: "VTA Witness — Present"
summary: "A client hands its VTA the witness's answer to an opened session — the VTA verifies it is the witness's own, builds the presentations bound to its challenge, and returns the witness/session/submit document, signed as the relationship DID, for the client to send."
status: draft
targetFrameworkVersion: "0.6.0"
category: credentials
keywords:
  - witness
  - vwc
  - relationship
  - presentation
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
    The request makes the VTA sign presentations as the principal's relationship DID — and, under the
    member link, as its member DID — over the principal's credentials. The witness's own proof on the
    carried response is what the VTA trusts for the challenge; the request's proof attributes the act of
    asking to a caller with standing, which a transport-only check would not survive a relay.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3 (the VTA acts as the relationship DID and the member DID), so SPEC §7.3
    item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    The VTA moves its session record to `presented`. It contacts nobody; a repeat builds a fresh submit
    document, and the witness spends its challenge on the first it accepts.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the relationship DID, and under the member link the member DID, to present the
    principal's own credentials. In: the witness's challenge and domain, inside its signed answer. Out: the
    submit document, which carries the presentations — the principal's relationship credential and, when
    named, its membership credential — to the principal's own delegate, bound to one session.
retention:
  class: transient
  rationale: >-
    The VTA keeps neither presentation nor the submit document. Its session record already holds what
    vta/witness/accept needs.
errorCodes:
  - code: vta/witness/present:sessionUnknown
    meaning: "No session record with this id in state `opened` or `presented`, in a context the caller may act in — including one that expired or was accepted."
    retryable: false
  - code: vta/witness/present:responseInvalid
    meaning: "The document handed back is not the witness's witness/session/0.1 #response for this session: wrong type, issuer, recipient or thread, or a proof that does not verify for the witness's DID."
    retryable: false
  - code: vta/witness/present:notFound
    meaning: "A credential bound to the session at open is no longer in the vault, or is no longer active."
    retryable: false
  - code: vta/witness/present:credentialMismatch
    meaning: "A credential bound to the session at open has changed since, or has left its validity period."
    retryable: false
related:
  - vta/witness/open
  - vta/witness/accept
  - witness/session
  - witness/session/submit
---

## Abstract

The second of three steps by which a VTA is the participating party of a witnessed session for its principal ([conventions](../../../_shared/0.5/CONVENTIONS.md)). The client hands back the witness's answer to the session document from [`vta/witness/open`](../../open/0.1/spec.md). The VTA checks that the answer is the witness's own, for this session, then builds the relationship presentation — and, under the member link, the member presentation — bound to the challenge and domain it carries, and returns the [`witness/session/submit/0.1`](../../../../witness/session/submit/0.1/spec.md) document, signed as the relationship DID. The client sends it to the witness and hands the witness's answer to [`vta/witness/accept`](../../accept/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.5/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Hand back in `response` the document the witness returned for the session, exactly as received. A refusal (`witness/session:refused`) ends the session at the client and is not handed back.
2. Send the returned `document` to the witness from `relationshipDid`, unaltered, and to nobody else, and **SHOULD** discard it once the witness has answered.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse a caller without standing with `permissionDenied`.
2. Find the session record by `sessionId` among the contexts the caller may act in, in state `opened` or `presented`; otherwise `vta/witness/present:sessionUnknown`.
3. Check `response` against the record before reading it ([conventions §3 item 3](../../../_shared/0.5/CONVENTIONS.md#3-the-session-record)): `type` `https://trusttasks.org/spec/witness/session/0.1#response`, `issuer` the record's witness, `recipient` the record's relationship DID, `threadId` the session id, and a proof that verifies for the witness's DID. Anything else is `vta/witness/present:responseInvalid` — a challenge the VTA cannot attribute to the witness is one it **MUST NOT** sign over.
4. Read the credentials bound at open from the vault, refusing with `vta/witness/present:notFound` one that is gone or inactive and with `vta/witness/present:credentialMismatch` one whose digest differs from the record's or that has left its validity period.
5. Build, itself ([conventions §2](../../../_shared/0.5/CONVENTIONS.md#2-custody)):
   1. the **relationship presentation**: holder `relationshipDid`, carrying the relationship credential — and the membership credential, when the record has no member DID — bound to the response's `challenge` and `domain` under a proof by `relationshipDid` for `authentication`, both values covered by that proof;
   2. when the record has a member DID, the **member presentation**: holder that DID, carrying the membership credential, bound to the **same** `challenge` and `domain` under a proof by that DID for `authentication`.

   It **SHOULD** present no credential beyond those bound at open, and the narrowest derivation its credential format supports where the witness accepts one.
6. Build the `witness/session/submit/0.1` document: a fresh `id`, `threadId` the session id, `parentThreadId` the record's relationship thread, `issuer` `relationshipDid`, `recipient` the witness, a current `issuedAt`, `vp` the relationship presentation, and the member presentation in the witness's carriage ([below](#the-member-presentations-carriage)). Sign it with a key of `relationshipDid`.
7. Move the record to `presented`, and answer with the session id and the document. It **MUST NOT** keep either presentation or the document.

A repeat for a session in `presented` builds a fresh document by the same steps ([conventions §5](../../../_shared/0.5/CONVENTIONS.md#5-retries)).

## Authorization

The authority is **standing over the contexts the session uses**, as at [`vta/witness/open`](../../open/0.1/spec.md#authorization) — the capability **`witnessRequest`**, never `sign` ([conventions §4](../../../_shared/0.5/CONVENTIONS.md#4-authorization)). Holding the session id confers nothing: a session is found only among the contexts the caller may act in, so a caller cannot present into a session it could not have opened.

The witness's proof on `response` establishes that the challenge is the witness's; it authorizes nothing. Verifying the request's `issuer`, transport identity or `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

This is the step at which the principal's credentials, and under the member link its member DID, are put in a document addressed to the witness, and a VTA's own policy may gate it ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

## Definitions

- **`sessionId`** — the session, as `vta/witness/open` returned it.
- **`response`** — the witness's `witness/session/0.1` `#response`, as received.
- **`document`** (response) — the signed `witness/session/submit/0.1` document to send.

### The member presentation's carriage

[`witness/session/submit/0.1`](../../../../witness/session/submit/0.1/spec.md) defines one presentation, `vp`, and no member for a second. Until it does, a witness that accepts a member presentation defines its own carriage, and this specification does not: per [SPEC §4.5.1](/SPEC.md#451-the-ext-extension-member) the semantics of an `ext` namespace belong to its controller. A VTA builds a member presentation only for a witness whose carriage it knows, which [`vta/witness/open`](../../open/0.1/spec.md) checks.

*Non-normative.* The OpenVTC witness reads it from the submit payload's `ext["org.openvtc.witness"].memberPresentation`, as the example below shows.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The witness's answer, handed back

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000103",
  "type": "https://trusttasks.org/spec/vta/witness/present/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-15T09:00:03Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000002ff",
  "payload": {
    "sessionId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
    "response": {
      "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000202",
      "type": "https://trusttasks.org/spec/witness/session/0.1#response",
      "issuer": "did:webvh:QmWitness:witness.example.org:a",
      "recipient": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
      "issuedAt": "2026-10-15T09:00:02Z",
      "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
      "parentThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
      "payload": {
        "challenge": "z5ZQ8hLqv3yXkJpT2mW9cRb4NfDs7AeGu",
        "domain": "did:webvh:QmWitness:witness.example.org:a"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-10-15T09:00:02Z",
        "verificationMethod": "did:webvh:QmWitness:witness.example.org:a#key-0",
        "proofPurpose": "authentication",
        "proofValue": "z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEcqdoDwLWxaqA3Q1geV6"
      }
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The submit document to send, with the member link

The credentials and proofs are abbreviated.

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000104",
  "type": "https://trusttasks.org/spec/vta/witness/present/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-15T09:00:04Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000002ff",
  "payload": {
    "sessionId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
    "document": {
      "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000203",
      "type": "https://trusttasks.org/spec/witness/session/submit/0.1#request",
      "issuer": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
      "recipient": "did:webvh:QmWitness:witness.example.org:a",
      "issuedAt": "2026-10-15T09:00:04Z",
      "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
      "parentThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
      "payload": {
        "vp": {
          "@context": ["https://www.w3.org/ns/credentials/v2"],
          "type": ["VerifiablePresentation"],
          "holder": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
          "verifiableCredential": [
            { "type": ["VerifiableCredential", "DTGCredential", "RelationshipCredential"] }
          ],
          "proof": {
            "type": "DataIntegrityProof",
            "proofPurpose": "authentication",
            "challenge": "z5ZQ8hLqv3yXkJpT2mW9cRb4NfDs7AeGu",
            "domain": "did:webvh:QmWitness:witness.example.org:a"
          }
        },
        "ext": {
          "org.openvtc.witness": {
            "memberPresentation": {
              "@context": ["https://www.w3.org/ns/credentials/v2"],
              "type": ["VerifiablePresentation"],
              "holder": "did:webvh:QmAlice:alice.example.com",
              "verifiableCredential": [
                { "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"] }
              ],
              "proof": {
                "type": "DataIntegrityProof",
                "proofPurpose": "authentication",
                "challenge": "z5ZQ8hLqv3yXkJpT2mW9cRb4NfDs7AeGu",
                "domain": "did:webvh:QmWitness:witness.example.org:a"
              }
            }
          }
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-10-15T09:00:04Z",
        "verificationMethod": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship#key-1",
        "proofPurpose": "authentication",
        "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSbwUM6dcvcRnzsBGVX9AsF3EyLoJvxfsMEKmYYCsCS4rBFPKVJTF"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

In: the witness's signed answer, whose payload is a challenge and a domain — binding material, not data about anyone. Out: the submit document, which carries the presentations, and with them the principal's relationship credential and, when named, its membership credential, in full.

**The client sees the presentations it carries.** They are the principal's own material, carried by the principal's own delegate, and each is bound under its holder's proof to this session's challenge and domain, which the witness issued for this session alone and spends on the first submission it accepts. Taken from the client, a presentation is useless in any other session and, once the witness has answered, in this one. The client **SHOULD** discard the document once it is answered.

**What the witness sees is the larger disclosure.** [`witness/session/submit`](../../../../witness/session/submit/0.1/spec.md) types `vp` as an opaque object: every claim in the presented credentials reaches the witness in full, and the witness must read all of it to verify it — the credentials' issuers, claims, identifiers and any status entries.

### Correlation

At the witness, the presentation links the pairwise relationship DID to whatever the credentials carry, and the membership evidence adds the community. **The member link discloses more**: the witness learns the member DID, the member's durable community identity, beside the pairwise relationship DID — *which member*, not only *which community*. A member DID seen by one witness across several relationships links them all. The VTA cannot prevent that; it can only refuse to disclose a member DID the caller did not name, which [`vta/witness/open`](../../open/0.1/spec.md) enforces.

The witness is public by design (`identifierScope: public`), as at [`vta/witness/open`](../../open/0.1/spec.md#correlation). The member presentation never appears on the wire outside the submit document, which only the witness opens.

### Retention

The VTA keeps neither presentation nor the document. Once the witness answers they have no evidentiary role: the VWC is bound to the session document and the relationship credential, never to a presentation. The session record keeps what [`vta/witness/accept`](../../accept/0.1/spec.md) needs.

### Consent/purpose

The credentials are presented so that the witness the principal chose can attest that the principal's relationship DID took part in this exchange; the member DID, when named, so that the witness can judge the principal eligible under its policy. Neither is for anything else. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
