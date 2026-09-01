---
slug: vault/credentials/query
version: "0.1"
title: "Vault Credentials — Query"
summary: "A filtered, body-free search over the credentials a holder's agent stores; claim bodies are never returned."
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - credential-vault
  - holder
  - inventory
  - dcql
parties:
  - role: credential-vault consumer
    requirement: REQUIRED
    member: issuer
  - role: credential-vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Query is read-only and the maintainer already authenticates the consumer, typically through the transport's session. A proof lets a maintainer attribute the request to a specific consumer key where the transport cannot — a single Trust Task delivered over DIDComm with no prior handshake — and maintainers MAY require one unconditionally as policy.
sideEffects:
  level: none
  rationale: >-
    A read over stored metadata. Nothing is created, altered or destroyed, and no claim body leaves the maintainer.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/credentials/query:filterRequired
    meaning: >-
      The filter carried no constraint the maintainer can scan on. An unconstrained query would enumerate the consumer's whole vault, so it is refused rather than answered. `includeArchived` and `includeDeleted` are modifiers and do not satisfy this requirement on their own.
    retryable: false
related: []
---

## Abstract

The **Vault Credentials — Query** Trust Task returns the **body-free metadata view** of the W3C credentials a holder's agent stores on its behalf — invitations, memberships, roles. A consumer uses it to render a wallet, answer "which membership do I hold for this community", and choose which credential to present before fetching a body with [`vault/credentials/get`](../../get/0.1/spec.md).

It is a Trust Task rather than an API call because the maintainer is a distinct party that must authenticate the consumer, evaluate its custody scope, and record the access. The answer is scoped to what that particular consumer is entitled to see, which is a decision only the maintainer can make.

**This task never returns credential contents.** Enumerate here; fetch to use.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents a family that maintainers already implement and drive in production tooling, written down so that the shapes stop being recoverable only by reading an implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

Every member is OPTIONAL, and the members that are present are combined with **AND**: a credential matches only if it satisfies all of them.

- **`type`** — matches a single VC `type` tag. A credential matches if *any* of its type tags equals this value.
- **`communityDid`** — matches credentials held for this community or context DID.
- **`issuerDid`** — matches credentials issued by this DID.
- **`purpose`** — matches the maintainer's semantic classification of the credential: `invite`, `membership`, `role`, `endorsement`, `personhood`, or a maintainer-defined string. Derived by the maintainer when the credential was received, so a consumer can find a credential by what it is *for* without knowing its type tags.
- **`status`** — matches the validity dimension: `valid`, `expired`, `revoked`, `unknown`.
- **`includeArchived`** — a **modifier**, not a filter. When `true`, archived credentials matching the other constraints are returned as well. Defaults to `false`.
- **`includeDeleted`** — a modifier on the same terms, surfacing soft-deleted tombstones so a consumer can offer restore or purge. Defaults to `false`.

### At least one filter is REQUIRED

A request that sets no filter — even one setting both modifiers — **MUST** be refused with `vault/credentials/query:filterRequired`.

This is a requirement on the **maintainer**, not on the payload schema. Expressing it schema-side would need a top-level `anyOf`, which the registry's code generators render as an untagged enum in Rust and as an index signature in TypeScript — both of which are worse than the rule being enforced where it is actually decided. A conforming maintainer **MUST NOT** treat a schema-valid empty filter as a request to enumerate.

This is not a performance limit. An unconstrained query returns the shape of the holder's entire life: every community they belong to, every role they hold, every issuer they have dealt with. A consumer that has been granted read access to answer one question does not thereby acquire the right to ask all of them, and a maintainer that answers an empty filter has no way to tell those two apart. The modifiers are excluded from the requirement for the same reason: `{ "includeDeleted": true }` is an enumeration wearing a flag.

A stored credential carries two orthogonal states, and a consumer that collapses them will mis-render its own vault:

- **Validity** (`status`) — `valid`, `expired`, `revoked` or `unknown`. Driven by the credential's own validity window and by status-list checks. The maintainer does not choose it.
- **Archival lifecycle** (`lifecycle`) — `active`, `archived` or `deleted`. Chosen by the consumer through this family. The maintainer records it.

The two do not constrain each other. A credential can be `valid` and `archived`; it can be `revoked` and `active`. "Can I present this?" is answered by both together — only an `active` credential may be presented, and only a `valid` one is worth presenting.

## Request

The credential-vault consumer sends the filter to the maintainer. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Finding the membership credential for one community

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vault/credentials/query/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "purpose": "membership",
    "communityDid": "did:web:community.example"
  }
}
```

## Response

The maintainer answers with the matching descriptors. The sub-schema is reachable via `$anchor: "response"`. A failure is a `trust-task-error` document, not a `#response` with an empty list — "no credential matches" and "the maintainer would not answer" are different facts and a consumer acts differently on each.

Each descriptor carries `id`, `types` and `status`; `issuerDid`, `purpose`, `validFrom` and `validUntil` when the stored envelope records them; and `lifecycle` with `archivedAt`, `deletedAt` and `graceUntil` on the rows the modifiers surface. `lifecycle` is omitted for active credentials.

**No descriptor carries the credential body.** A maintainer **MUST NOT** add one, and a consumer **MUST NOT** treat this response as a substitute for a presentation.

### One active membership

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vault/credentials/query/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "credentials": [
      {
        "id": "cred-7f3a91c2",
        "types": ["VerifiableCredential", "MembershipCredential"],
        "issuerDid": "did:web:community.example",
        "purpose": "membership",
        "status": "valid",
        "validFrom": "2026-01-01T00:00:00Z",
        "validUntil": "2027-01-01T00:00:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a filter over indexed envelope fields. The response carries metadata only: identifiers, type tags, issuer, purpose, validity window and lifecycle state.

Claim bodies are **never** carried. A search result that could return contents would be a way to read a credential without the maintainer evaluating a presentation, and the smallest payload that answers "which credential should I use" does not include them.

A consumer **SHOULD** send the narrowest filter that answers its question. `communityDid` alone returns every credential from that community; adding `purpose` narrows it to the one being asked about.

### Correlation

The maintainer necessarily learns which community, issuer or purpose the consumer is asking about, and can join those questions across requests by consumer identity. That is unavoidable — it is the query.

An observer of the transport sees request timing and size. Response size varies with the number of matches, so a maintainer handling a padding-sensitive deployment **MAY** pad.

`threadId` joins the request to its response and to nothing else.

### Retention

The maintainer needs to retain nothing from the request beyond answering it. Where a maintainer records the access for audit, it **SHOULD** record the filter rather than the result: the filter is what the consumer asked, and the result set changes as the vault does.

### Consent/purpose

The filter is collected to answer this query. A maintainer **MUST NOT** reuse it to profile the consumer's interests or to seed recommendations. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required; whether one is needed is the maintainer's policy decision.

### Custody scope

A maintainer holds credentials on behalf of more than one context. A consumer's authority is scoped, and the maintainer **MUST** evaluate that scope against the credential's own context before acting: a consumer scoped to one context **MUST NOT** be able to read, alter or destroy a credential held for another.

Where that check fails, the maintainer **MUST** answer exactly as it would for an identifier it does not hold. Answering `permissionDenied` for a credential that exists and `notFound` for one that does not would let a consumer map another context's vault one identifier at a time, which is the enumeration this family is built to refuse.
