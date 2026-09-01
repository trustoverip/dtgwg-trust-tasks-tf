---
slug: vtc/members/credentials
version: "0.1"
title: VTC Members — Credentials
summary: Fetch the membership-pair credential bodies a community holds for one member — the community-issued grant, the member-issued acknowledgement, and the role endorsement.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - members
  - vmc
  - membership-edge
  - credentials
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Unlike `members/show`, this returns credential bodies rather than identifiers. A maintainer must be able to attribute the disclosure to a specific administrator key, because the record of who read a member's credentials is the only thing that makes the disclosure accountable afterwards.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request that discloses credential bodies is worth replaying: captured once, it returns the same documents every time it is resent. `issuedAt` bounds how old a request a maintainer will answer, and so bounds the window in which a captured one is still useful.
sideEffects:
  level: none
  rationale: >-
    Reads the credentials already stored against one membership record. Nothing is issued, altered or persisted.
subjectPath: /did
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: >-
    Returns the credential bodies themselves, including every claim the community asserted about the member and every claim the member asserted about the community. `members/show` returns the identifiers; this returns what they name.
errorCodes:
  - code: vtc/members/credentials:notFound
    meaning: >-
      No member with the supplied `did` exists in this community. Distinguished from a member who exists and holds no credentials, which is a successful answer with every document absent.
    retryable: false
related:
  - vtc/members/show
  - vtc/members/vmc
  - vtc/relationships/list
---

## Abstract

The **VTC Members — Credentials** Trust Task returns the **credential bodies** a community holds for one member: the community-issued Verifiable Membership Credential, the member-issued reciprocal VMC that completes the edge, and the role Verifiable Endorsement Credential.

[`vtc/members/show`](../../show/0.1/) already returns the *identifiers* — `currentVmcId`, `currentRoleVecId`, `memberVmcId` — and its schema says outright that the body is not echoed there. So an operator can see *that* a member holds a membership credential and cannot see *what it says*. This task is the read that closes that.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Why this is its own task

`MemberResponse` in [`_shared/0.1/member.schema.json`](../../../_shared/0.1/member.schema.json) is `$ref`d by `members/show`, `members/list` and `members/update`. Adding credential bodies to it would put full JSON-LD on **every row of a paginated roster** — which is presumably why the "not echoed here" sentence is in that schema already.

The response shape there is right; the registry was missing a task. This is the same split the community profile makes between its record and its read-shaped view: a separate view, not another member on a shared one.

## Definitions

### Request

- **`did`** — the member to read. Carried in the payload rather than only a transport path, so the task dispatches identically over REST and DIDComm and the subject is visible to policy evaluation via `subjectPath: /did`.

### Response

Every credential member is OPTIONAL, and **absent means the community holds no such document** — which is a real answer, not a failure. A member who has been granted membership but never returned the reciprocal credential is exactly the case this task exists to make visible.

- **`membershipCredential`** — the community-issued VMC. The grant.
- **`roleCredential`** — the role VEC.
- **`memberVmc`** — the member-issued reciprocal VMC. The acknowledgement.
- **`memberVmcReceivedAt`** — when that acknowledgement arrived. Paired with `memberVmc`; a maintainer **MUST NOT** send one without the other.
- **`memberVmcBound`** — whether the acknowledgement's `digest` was verified against the grant.

Credential bodies are carried as opaque objects. The schema does not describe their internals, matching [`vtc/relationships/list/0.2`](../../../relationships/list/0.2/), which returns `vrcJsonld` the same way: a credential carries a proof over its own bytes, and a schema that constrained the shape would invite a maintainer to normalise it and destroy the signature.

### `memberVmcBound` is REQUIRED, and declares no default

The member is REQUIRED so that "not verified" is always stated rather than inferred from silence. A consumer reading a membership edge needs to distinguish three things: the acknowledgement is present and its digest matches the grant; it is present and does not; it is absent entirely. Only the first completes the edge.

**It declares no JSON Schema `default`, deliberately.** A declared default is materialised by the generated bindings — the member becomes non-optional with `#[serde(default)]`, an absent value reappears on re-serialisation, and round-trip idempotence breaks for every existing document. The same defect has now been caught twice in this registry, and once more in the `vault/credentials` family where a `default: false` on a boolean broke the generated example test outright.

## Request

An administrator names one member. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Reading one member's membership edge

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/members/credentials/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "did": "did:example:member"
  }
}
```

## Response

The community maintainer returns the documents it holds. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents; a member with no credentials is not a failure.

Bodies are returned **verbatim**. A maintainer **MUST NOT** re-serialise, re-order or otherwise normalise them: each carries a proof over its own bytes, and a maintainer that rewrites them destroys the signature it was holding.

### A complete membership edge

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/members/credentials/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "did": "did:example:member",
    "membershipCredential": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "VerifiableMembershipCredential"],
      "issuer": "did:web:community.example",
      "credentialSubject": { "id": "did:example:member" }
    },
    "memberVmc": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "VerifiableMembershipCredential"],
      "issuer": "did:example:member",
      "credentialSubject": { "id": "did:web:community.example" }
    },
    "memberVmcReceivedAt": "2026-01-01T00:00:00Z",
    "memberVmcBound": true
  }
}
```

### A grant the member has not acknowledged

`memberVmcBound` is `false` and the acknowledgement is absent. The community cannot produce the member-issued VMC, so it cannot complete the edge for a verifier that requires one.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/members/credentials/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "did": "did:example:member",
    "membershipCredential": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiableCredential", "VerifiableMembershipCredential"],
      "issuer": "did:web:community.example",
      "credentialSubject": { "id": "did:example:member" }
    },
    "memberVmcBound": false
  }
}
```

## Why it matters beyond an operator console

DTG Core Credentials, *Membership Edge Completion*, requires that where a community asserts an entity's membership, it **must be able to produce the member-issued VMC that completes the edge**.

A community that holds that credential with no task to produce it can discharge that requirement only through its own admin UI — not over the protocol, and so not to any counterparty that speaks Trust Tasks. That makes this a conformance gap rather than only an ergonomic one, and it is why `memberVmcBound` is REQUIRED: a consumer must be able to tell a complete edge from an asserted one without inspecting the documents itself.

## Security & Privacy

### Data carried

The request carries one member DID. The response carries **credential bodies** — every claim the community asserted about the member, and every claim the member asserted about the community.

There is no smaller payload that answers this task: a partial credential does not verify. That is why it is separate from `members/show`, which answers "who is a member" with identifiers and is the right task for a roster. Minimisation here happens by calling `members/show` instead, and a consumer **MUST NOT** call this task to populate a list.

### Correlation

The maintainer learns which member's credentials an administrator is inspecting, and when. Read against a roster in sequence, that is a map of which members an administrator is scrutinising.

`threadId` joins the request to its response and to nothing else.

### Retention

A consumer **SHOULD** hold the returned bodies only as long as the check they were fetched for. The maintainer remains the record — only it sees a later revocation, rotation or departure — and a cached credential outlives all three.

The maintainer **SHOULD** record that the read happened. A disclosure of a member's credentials that leaves no trace cannot be reviewed afterwards, and members are entitled to know that their community's administrators looked.

### Consent/purpose

The bodies are disclosed to an administrator so they can verify the membership edge. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required. A maintainer that gates this task more tightly than `members/show` is making a policy choice this specification neither mandates nor forbids, and one the exposure class above is meant to inform.
