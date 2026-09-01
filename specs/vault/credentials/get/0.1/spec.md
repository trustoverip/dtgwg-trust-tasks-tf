---
slug: vault/credentials/get
version: "0.1"
title: "Vault Credentials — Get"
summary: "Fetch one stored credential's full body by id, for presentation."
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - credential-vault
  - holder
  - presentation
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
    This is the one task in the family that returns credential contents. A maintainer must be able to attribute the disclosure to a specific consumer key rather than to whatever holds the transport session, because the record of who read a credential body is the only thing that makes the disclosure accountable after the fact.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request to read credential contents is worth replaying: captured once, it discloses the same body every time it is resent. `issuedAt` is what lets a maintainer bound how old a request it will act on, and so bound the window in which a captured one is still useful.
sideEffects:
  level: none
  rationale: >-
    A read. Nothing is created, altered or destroyed — though what it returns is the credential itself, which is why the exposure class rather than the side-effect class is what governs this task.
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: >-
    Returns the full credential body, including every claim the issuer asserted about the holder. That is the material a verifier would otherwise only see through a presentation the holder consented to.
errorCodes:
  - code: vault/credentials/get:notFound
    meaning: >-
      The maintainer holds no credential under this identifier that this consumer may read. Deliberately conflates "no such credential" with "not yours" — see Custody scope.
    retryable: false
related: []
---

## Abstract

The **Vault Credentials — Get** Trust Task returns one stored credential's **full body** by the identifier a [`query`](../../query/0.1/spec.md) descriptor carried. A consumer uses it when it has already decided which credential to act on and now needs the material to present.

Query enumerates; get discloses. Splitting them is the point: a consumer can browse its own vault continuously while the far narrower act of reading a credential's contents stays a separate, separately-authorised, separately-recorded request.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents a family that maintainers already implement and drive in production tooling, written down so that the shapes stop being recoverable only by reading an implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

- **`id`** — the local handle a `query` descriptor carried. Opaque to the consumer: a maintainer chooses its own form and a consumer **MUST NOT** parse it, derive one, or assume ids are contiguous, guessable or stable across maintainers.

## Request

The credential-vault consumer names one credential. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Fetching a credential chosen from a query result

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vault/credentials/get/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2"
  }
}
```

## Response

The maintainer returns the credential as stored. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents.

`credential` is the verifiable credential itself, carried verbatim. A maintainer **MUST NOT** re-serialise, re-order or otherwise normalise it: the bytes carry a proof over themselves, and a maintainer that rewrites them destroys the signature it was holding.

A credential whose `lifecycle` is `archived` or `deleted` **MUST NOT** be returned. Those states mean "not for use", and a body handed back is a body that can be presented.

### The stored credential

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vault/credentials/get/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-01-01T00:00:01Z",
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
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries one opaque identifier. The response carries **the entire credential**, including every claim the issuer asserted about the holder and the proof over them.

There is no smaller payload that answers this task — a partial credential does not verify. That is precisely why the task is separate from `query`: minimisation happens by *not calling this*, and a consumer **MUST NOT** call it to populate a list.

### Correlation

The maintainer learns which credential the consumer intends to use, and when. Where a consumer fetches a credential immediately before presenting it, that timing links the maintainer's record to the verifier's — the maintainer can infer that a presentation occurred even though it never sees the verifier.

A consumer that wants to break that link **MAY** fetch ahead of use and cache, accepting that a cached body outlives any revocation the maintainer would otherwise have refused to serve.

### Retention

A consumer **SHOULD** hold the returned body only as long as the presentation it was fetched for, and **MUST NOT** treat it as a durable copy: the maintainer remains the record, and only the maintainer sees a later revocation or lifecycle change.

The maintainer **SHOULD** record that the read happened, and the record is the point — a disclosure of credential contents that leaves no trace cannot be reviewed afterwards.

### Consent/purpose

The body is disclosed to the consumer for the presentation it asked about. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required. A maintainer that gates this task on a fresh human confirmation is making a policy choice this specification neither mandates nor forbids.

### Custody scope

A maintainer holds credentials on behalf of more than one context. A consumer's authority is scoped, and the maintainer **MUST** evaluate that scope against the credential's own context before acting: a consumer scoped to one context **MUST NOT** be able to read, alter or destroy a credential held for another.

Where that check fails, the maintainer **MUST** answer exactly as it would for an identifier it does not hold. Answering `permissionDenied` for a credential that exists and `notFound` for one that does not would let a consumer map another context's vault one identifier at a time, which is the enumeration this family is built to refuse.
