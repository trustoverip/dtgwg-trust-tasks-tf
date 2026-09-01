---
slug: vta/credentials/list
version: "0.1"
title: VTA Credentials — List
summary: Enumerate the credentials an agent has issued, as metadata. Bodies are not returned.
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - credentials
  - issuer
  - inventory
  - revocation
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: issuing agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Read-only, and the agent already authenticates the caller through the transport's session. A proof lets the agent attribute the request to a specific administrator key where the transport cannot, and agents MAY require one unconditionally as policy.
sideEffects:
  level: none
  rationale: >-
    Reads issuance records the agent already keeps. Nothing is minted, revoked or altered.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/credentials/list:cursorInvalid
    meaning: >-
      The supplied `cursor` cannot be honoured — expired, malformed, or issued against agent state no longer recognised. Consumers SHOULD retry from the first page without a cursor.
    retryable: true
related:
  - vta/credentials/issue
  - vta/credentials/revoke
  - vault/list
---

## Abstract

The **VTA Credentials — List** Trust Task enumerates the credentials an agent has issued. It answers the question an issuer currently cannot ask its own agent: *what have I issued, to whom, and which of it is still good?*

Today `vta/credentials` declares `issue` and `revoke` and nothing else. The `credentialId` that `revoke` is keyed on is returned exactly once, in the `issue` response, and a caller that did not record it at that moment has no way to recover it from the agent. An issuer can mint and revoke; it cannot look.

**Bodies are not returned.** This task enumerates; the claims stay where they are. See [Why metadata only](#why-metadata-only).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

Unlike the rest of `vta/credentials`, **this task has no implementation yet** — it is a proposal, raised from
[#337](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/337). What it needs, an agent already stores: see [No new storage](#no-new-storage).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

### Request

Every member is OPTIONAL. Those present are combined with **AND**.

- **`holder`** — only credentials issued to this DID.
- **`credentialType`** — only credentials carrying this type tag beyond `VerifiableCredential`.
- **`status`** — only credentials in this state. See [Status is derived](#status-is-derived).
- **`pageSize`** — maximum records to return. The agent MAY return fewer and caps the value.
- **`cursor`** — continue a previous page. Opaque; a consumer **MUST NOT** construct or parse one.

### An unfiltered request is allowed here

Unlike [`vault/credentials/query`](../../../../vault/credentials/query/0.1/), which refuses a filter that constrains nothing, this task answers an empty request.

The two look alike and are not. There, the caller is reading a *holder's* private store, and an unconstrained answer is the shape of that person's affiliations. Here, the caller is the **issuer**, reading a record of **its own past actions** against its own agent — an issuer that may issue at all already knows it issued. Refusing to enumerate would withhold from a party what it did itself, while the ACL that decides whether it may call this at all is the control that actually matters.

This also matches the sibling reads an operator uses in the same sitting: [`acl/list/0.1`](../../../../acl/list/0.1/) and [`policy/list/0.2`](../../../../policy/list/0.2/) are unconstrained and paginated.

### Status is derived

`status` is computed by the agent when it answers, never stored:

- **`revoked`** — the agent recorded a revocation. Takes precedence over expiry: a credential revoked before its window closed is revoked, and reporting it as merely expired would hide that somebody acted.
- **`expired`** — not revoked, and `expiresAt` has passed.
- **`active`** — neither.

Deriving rather than storing is what keeps the answer true: a stored status is a copy of a fact about the clock, and it is wrong from the first second after it is written.

### Revoked credentials stay listed

A revocation does not remove a record. "What did I revoke, and when" is a question an issuer has to be able to answer — to a verifier querying an anomaly, or to the holder who wants to know why their credential stopped working.

A list that dropped them could not distinguish a revoked credential from one that was never issued, which are opposite answers to give someone asking why something failed. The `status` filter is how a consumer that wants only live credentials narrows to them.

## Why metadata only

Each record is a summary: which credential, to whom, when, and whether it is still good. The credential body is not carried.

The reasoning is the one [`vault/list/0.1`](../../../../vault/list/0.1/) already states for secrets — *list exists to enumerate; release exists to use* — and it transfers. An issuer listing its own issuances is answering an inventory question, and a response that carried every claim body would make the routine act of looking at a roster into a bulk disclosure of everything the issuer ever asserted about anyone. It would also make the response size a function of claim size, so the cost of asking would scale with the sensitivity of the answer.

An issuer that needs a body has it: it minted the credential, and the `issue` response returned it. Where a case for re-reading one is made, that is a separate `vta/credentials/get` keyed on `credentialId` — deliberately not proposed here, so the disclosure decision is taken on its own terms rather than inherited from this one.

## No new storage

An agent implementing `vta/credentials/issue` already retains everything this task returns. In the reference implementation the issuance record carries the credential id, the holder, the issued and expiry instants, and the revocation instant and reason — and revocation is written as a **tombstone rather than a delete**, precisely so the audit and verifier trail survives it.

So this proposes a read over records that already exist, not a new thing to keep.

## Request

An administrator asks the issuing agent. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Live credentials issued to one holder

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/credentials/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "holder": "did:example:holder",
    "status": "active"
  }
}
```

## Response

The agent returns one summary per matching credential. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents; an empty list is a successful answer.

`truncated` says the agent stopped early. A consumer **MUST NOT** read a truncated page as a complete account — concluding "nothing else was issued" from a partial one is the failure this member exists to prevent.

### One active credential and one revoked

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/credentials/list/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "credentials": [
      {
        "credentialId": "urn:uuid:11111111-1111-4111-8111-111111111111",
        "holder": "did:example:holder",
        "credentialType": "MembershipCredential",
        "issuedAt": "2026-01-01T00:00:00Z",
        "expiresAt": "2027-01-01T00:00:00Z",
        "status": "active"
      },
      {
        "credentialId": "urn:uuid:22222222-2222-4222-8222-222222222222",
        "holder": "did:example:holder",
        "credentialType": "RoleCredential",
        "issuedAt": "2025-06-01T00:00:00Z",
        "expiresAt": "2027-06-01T00:00:00Z",
        "status": "revoked",
        "revokedAt": "2026-02-14T09:30:00Z",
        "revocationReason": "role ended"
      }
    ],
    "truncated": false
  }
}
```

## Security & Privacy

### Data carried

The request carries filters. The response carries, per credential, an identifier, the holder's DID, the type tag, two instants, a derived status, and — where one exists — a revocation instant and reason.

No claim body is carried. The smallest payload that answers "what have I issued and is it still good" does not include the claims, which is the whole reason this task is separate from the body-returning read that does not yet exist.

`revocationReason` is free text and is disclosed to every caller entitled to this list. An issuer **MUST NOT** write anything into it at revocation time that it would not show to that audience — "role ended" rather than the circumstances behind it.

### Correlation

The response is a map of who the issuer has issued to. A caller reading it learns the holder set, which is more than any single credential discloses: the *pattern* of issuance can be sensitive where an individual credential is not — an issuer of health or membership credentials discloses its cohort by listing them.

That is inherent to an inventory, and it is why the ACL gating this task matters more than any filter in it.

### Retention

The agent retains issuance records for the life of the credential and beyond, because revocation is a tombstone — a verifier checking a revoked credential needs the record to still be there. This task reads that retention; it does not extend it.

A consumer **SHOULD NOT** cache the result: `status` is derived at read time, and a cached page reports a credential as active after it has been revoked.

### Consent/purpose

The list is disclosed to an administrator of the issuing agent, for administering issuance. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required.
