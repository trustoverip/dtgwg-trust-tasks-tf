---
slug: vault/credentials/receive
version: "0.1"
title: "Vault Credentials — Receive"
summary: "A holder's agent verifies a received W3C credential and stores it in the credential vault."
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - credential-vault
  - holder
  - storage
parties:
  - role: credential-vault consumer
    requirement: REQUIRED
    member: issuer
  - role: credential-vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Receive writes into the holder's vault. Without a proof a maintainer cannot attribute the write to a consumer key, and an unattributable write into a credential store is a way to place a credential the holder never accepted where the holder will later find it and believe they did.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed receive re-stores a credential the holder may since have deleted, which silently undoes a deletion. `issuedAt` bounds how old a write a maintainer will act on.
sideEffects:
  level: mutating
  rationale: >-
    Creates a stored record. Re-receiving the same credential under the same id replaces that record rather than adding a second, so the task is idempotent on id.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: personal
  rationale: >-
    The request carries the full credential, so the maintainer ingests every claim the issuer asserted about the holder. The response is metadata only.
errorCodes:
  - code: vault/credentials/receive:verificationFailed
    meaning: >-
      The credential's proof did not verify against the issuer key resolved from its DID. The credential is not stored. A consumer MUST NOT retry unchanged.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum: ["proofInvalid", "issuerUnresolvable", "proofMissing", "unsupportedCryptosuite"]
  - code: vault/credentials/receive:formatUnsupported
    meaning: >-
      The maintainer does not implement the declared `format`. Distinguished from a verification failure so a consumer can tell "this maintainer cannot hold this credential" from "this credential is not valid".
    retryable: false
related: []
---

## Abstract

The **Vault Credentials — Receive** Trust Task hands a W3C verifiable credential to a holder's agent to **verify and store**. It is how a credential a holder has been issued — an invitation, a membership, a role — comes to live somewhere the holder can find and present it later.

The maintainer verifies before it stores. That ordering is the whole task: a credential store that accepts unverified material is a store whose contents mean nothing, and every later `query` result would be a claim the maintainer cannot stand behind.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents a family that maintainers already implement and drive in production tooling, written down so that the shapes stop being recoverable only by reading an implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

- **`credential`** — the verifiable credential, as a JSON object. Mutually exclusive with `credentialBase64`.
- **`credentialBase64`** — the same credential, base64url-encoded without padding, for formats whose canonical form is not JSON. Mutually exclusive with `credential`.
- **`format`** — the credential format, when it is not evident from the body. A maintainer that does not implement the named format **MUST** refuse with `vault/credentials/receive:formatUnsupported` rather than storing something it cannot later verify or present.
- **`id`** — the local handle to store under. OPTIONAL: absent, the maintainer derives one from the credential's own `id`. Supplying it makes the write idempotent under the consumer's own naming, which is what lets a retry after an ambiguous failure not produce a second copy.
- **`contextId`** — the context to hold the credential in. Absent, the maintainer uses the consumer's own context.

### Verification happens before storage

A maintainer **MUST** verify the credential's proof against the issuer key resolved from the issuer's DID before storing anything, and **MUST NOT** store a credential that fails. On failure it answers `vault/credentials/receive:verificationFailed` and the vault is unchanged.

### Purpose is derived, not declared

The maintainer classifies the credential — `invite`, `membership`, `role`, `endorsement`, `personhood` — from its `type` tags, and records that classification so [`query`](../../query/0.1/spec.md) can filter on it.

It is deliberately not a request member. A consumer that could declare the purpose could file a credential under a classification its contents do not support, and a later query for "my memberships" would return something that is not one. Deriving it means the classification and the credential cannot disagree.

## Request

The credential-vault consumer hands over one credential. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Storing a membership credential just received

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vault/credentials/receive/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "credential": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "MembershipCredential"],
      "issuer": "did:web:community.example",
      "validFrom": "2026-01-01T00:00:00Z",
      "credentialSubject": {
        "id": "did:example:holder",
        "memberOf": "did:web:community.example"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-01-01T00:00:00Z",
        "verificationMethod": "did:web:community.example#key-1",
        "proofPurpose": "assertionMethod",
        "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSbwUM6dcvcRnzsBGVX9AsF3EyLoJvxfsMEKmYYCsCS4rBFPKVJTF"
      }
    },
    "contextId": "community-example"
  }
}
```

## Response

The maintainer confirms what it stored. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents.

The response carries the handle the credential can be fetched by, the type tags the maintainer read, the purpose it derived, and the validity status it computed. It does **not** echo the credential — the consumer just sent it.

### Stored, and classified as a membership

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vault/credentials/receive/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "types": ["VerifiableCredential", "MembershipCredential"],
    "purpose": "membership",
    "status": "valid"
  }
}
```

## Security & Privacy

### Data carried

The request carries the **entire credential**, so the maintainer ingests every claim the issuer asserted about the holder. There is no smaller payload — a partial credential does not verify, and a maintainer that stored one could not present it.

`contextId` and a consumer-supplied `id` are the only free members, and a consumer **MUST NOT** encode claim content in either. An id is a handle, not a description: `cred-7f3a91c2` discloses nothing to whoever later sees a query result, and `hiv-clinic-membership-2026` discloses a great deal.

### Correlation

The maintainer learns the issuer, the type and the full contents of every credential the holder accepts — which, accumulated, is a map of the holder's affiliations. That is inherent to being the store.

Where a consumer receives a credential and immediately stores it, the timing links the maintainer's record to the issuance. A consumer that wants to break that link **MAY** delay the write, at the cost of a window in which the credential exists nowhere durable.

### Retention

The maintainer keeps the credential until the holder removes it through [`delete`](../../delete/0.1/spec.md) or [`purge`](../../purge/0.1/spec.md). That is the point of the task, and it is the one member of this family where "retain indefinitely" is the correct answer rather than a lapse.

A maintainer **SHOULD NOT** retain a credential that failed verification, including for diagnostics: it was never the holder's, and a store of rejected credentials is a store of claims about the holder that nobody vouched for.

### Consent/purpose

The credential is stored so the holder can present it later. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required.

### Custody scope

A maintainer holds credentials on behalf of more than one context. A consumer's authority is scoped, and the maintainer **MUST** evaluate that scope against the credential's own context before acting: a consumer scoped to one context **MUST NOT** be able to read, alter or destroy a credential held for another.

Where that check fails, the maintainer **MUST** answer exactly as it would for an identifier it does not hold. Answering `permissionDenied` for a credential that exists and `notFound` for one that does not would let a consumer map another context's vault one identifier at a time, which is the enumeration this family is built to refuse.
