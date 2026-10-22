---
slug: vault/credentials/present
version: "0.1"
title: "Vault Credentials — Present"
summary: "Present one stored credential to a verifier the consumer names, bound to that verifier and its nonce, disclosing only the named claims — so a client that holds no holder key can still carry a presentation to a verifier itself."
status: draft
targetFrameworkVersion: "0.6.0"
category: credentials
keywords:
  - credential-vault
  - holder
  - presentation
  - selective-disclosure
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
    The maintainer signs as the holder on the consumer's word. The record of which consumer key asked for a presentation, to which verifier, disclosing what, is the only account of that disclosure afterwards, so it must be attributable to a key rather than to whatever holds the transport session.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A captured request is worth replaying: resent, it yields a fresh holder-signed presentation every time. `issuedAt` lets a maintainer bound how old a request it will act on.
sideEffects:
  level: mutating
  rationale: >-
    The maintainer records the disclosure — what was disclosed, to whom, under which consumer's request — before it answers. The credential itself is unchanged.
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: true
  rationale: >-
    The maintainer signs as the credential's subject, and the presentation discloses the named claims to whoever the consumer carries it to.
errorCodes:
  - code: vault/credentials/present:notFound
    meaning: >-
      The maintainer holds no credential under this identifier that this consumer may present, or holds no key for its subject that this consumer may use. Deliberately conflates the cases — see Custody scope.
    retryable: false
  - code: vault/credentials/present:notPresentable
    meaning: >-
      The credential is revoked, suspended, outside its validity period, archived or deleted.
    retryable: false
  - code: vault/credentials/present:formatUnsupported
    meaning: >-
      The maintainer cannot present a credential of this format to a named verifier.
    retryable: false
  - code: vault/credentials/present:claimNotHeld
    meaning: >-
      `claims` names a claim the credential does not carry.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required:
        - claims
      properties:
        claims:
          type: array
          maxItems: 64
          items:
            type: string
            maxLength: 128
  - code: vault/credentials/present:wouldOverDisclose
    meaning: >-
      Every presentation of the credential discloses claims `claims` does not name — all of them, for a credential without selective disclosure. `details.undisclosed` lists those claims.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required:
        - undisclosed
      properties:
        undisclosed:
          type: array
          maxItems: 256
          items:
            type: string
            maxLength: 128
related:
  - vault/credentials/query
  - vault/credentials/get
  - credential-exchange/present
  - vtc/directory/card/present
---

## Abstract

The **Vault Credentials — Present** Trust Task asks the maintainer — the agent that holds the credential and its subject's keys — to make a presentation of one stored credential **for a verifier the consumer names**, bound to a nonce the consumer supplies, disclosing only the claims the consumer names. The maintainer returns the presentation; the consumer carries it to the verifier inside whatever task the verifier defines.

[`credential-exchange/present`](../../../../credential-exchange/present/0.1/spec.md) covers the other direction, where the verifier asks the agent and the agent answers the verifier. This task is for a verifier that never talks to the agent: a task whose request document carries the presentation, such as [`vtc/directory/card/present`](../../../../vtc/directory/card/present/0.1/spec.md), where the presentation's nonce is that document's `id` and its audience the community's DID. A consumer that holds no holder key — a browser wallet, a phone — can then present through the agent that does.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **maintainer** **MUST**:

1. Resolve `id` to a credential the consumer may present and its subject to a key the consumer may use, both within the consumer's custody scope, and answer `notFound` otherwise — the same answer whichever of them failed.
2. Refuse with `notPresentable` a credential that is archived or deleted, revoked or suspended, or outside its validity period at the time of the request. Where the credential names a status list, the maintainer **SHOULD** re-read it before answering rather than rely on what it read at receipt.
3. Refuse with `claimNotHeld` a request naming a claim the credential's subject does not carry.
4. Disclose nothing outside `claims`. A claim every presentation of the credential discloses — every subject claim of a Data Integrity credential without a selective-disclosure cryptosuite, a claim an SD-JWT carries in the clear — **MUST** be named in `claims`, and the request refused with `wouldOverDisclose` otherwise.
5. Bind the presentation to `audience` and `nonce`, signed by the credential subject's key:
   - a **Data Integrity** presentation carries `holder` = the subject, `nonce` = `nonce` and `domain` = `audience` as members of the presentation its holder proof signs, and a proof that also states `challenge` or `domain` states the same values;
   - an **SD-JWT** presentation carries the disclosures for `claims` only and a key-binding JWT with `nonce` = `nonce` and `aud` = `audience`, signed by the key the credential's `cnf` names.
6. Record the disclosure — the credential, `audience`, the disclosed claims, `purpose` when given, and the consumer — before answering, and not afterwards: a presentation handed back with no record of it is a disclosure nobody can review.
7. Return in `disclosed` exactly the claims the presentation discloses.

The **consumer** carries `presentation` to the verifier unchanged: it carries a proof over its own bytes.

## Authorization

**Authority: custody of the credential and its subject's key.** The maintainer signs as the subject, so a consumer that may read a credential but not use its subject's key may not have it presented. Being able to read a credential with [`get`](../../get/0.1/spec.md) does not imply being able to present it, and the reverse.

Where that entitlement is expressed as a capability, the registered value is **`credentialPresent`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign` and is not conferred by it: the maintainer signs only a presentation it builds itself, of a credential it holds, for one audience and nonce, so a holder of `credentialPresent` obtains that and nothing else. Nor is it `vaultRead`: reading a credential is not disclosing it to a third party under its subject's signature.

## Definitions

- **`id`** — the local handle a [`query`](../../query/0.1/spec.md) descriptor carried. Opaque, as there.
- **subject claim** — a member of the credential's `credentialSubject` other than `id` (Data Integrity), or a selectively disclosable claim of the SD-JWT.

## Request

The credential-vault consumer names the credential, the verifier, the verifier's nonce and the claims to disclose. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Presenting an employer to a community's directory

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/vault/credentials/present/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-10-22T09:00:00Z",
  "payload": {
    "id": "cred-7f3a91c2",
    "audience": "did:example:community",
    "nonce": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001b",
    "claims": ["employer"],
    "purpose": "A verified employer on my directory card"
  }
}
```

## Response

The maintainer returns the presentation and the claims it discloses. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents.

### An SD-JWT presentation

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/vault/credentials/present/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-10-22T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000011",
  "payload": {
    "presentation": "eyJhbGciOiJFZERTQSJ9.example-sd-jwt~WyJzYWx0IiwiZW1wbG95ZXIiLCJBY21lIl0~eyJhbGciOiJFZERTQSJ9.example-kb-jwt",
    "disclosed": ["employer"]
  }
}
```

## Security & Privacy

### Data carried

The request carries an opaque identifier, the verifier's identifier, its nonce and claim names. The response carries the named claims, the issuer's signature over the credential, and a holder signature binding them to the verifier.

### A presentation is a bearer artefact until the verifier checks it

The binding to `audience` and `nonce` is what stops a presentation obtained for one verifier being used at another, or twice. A consumer **MUST** take `nonce` from the verifier — for a task that carries the presentation in its request, that request document's `id` — and **MUST NOT** reuse one.

### Correlation

The maintainer learns the verifier and when a presentation was made for it; that is the record item 6 requires. The verifier learns what any presentation discloses: the named claims, the credential's issuer, and the subject identifier the binding needs.

### Retention

The maintainer keeps the record of the disclosure as long as it keeps its other consent and disclosure records. Neither party needs the presentation after the verifier has checked it.

### Consent/purpose

The consumer's signed request is the holder's instruction to disclose. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required. A consumer that acts for a person **SHOULD** show them the claims and the verifier before it asks.

### Custody scope

A maintainer holds credentials and keys on behalf of more than one context. Where the consumer's scope does not reach the credential or its subject's key, the maintainer **MUST** answer exactly as for an identifier it does not hold, as [`get`](../../get/0.1/spec.md) does, so that the task cannot be used to map another context's vault.
