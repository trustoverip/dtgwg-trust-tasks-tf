---
slug: vtc/join-requests/query
version: "0.1"
title: "VTC Join Requests — Query"
summary: An administrator asks the community to open a credential exchange with a prospective member — issue a presentation challenge, build the credential-exchange/query for a registered Accepts criterion, and deliver it.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The task makes the community address a third party — the holder — under the community's own identity, asking them for credentials, on the signer's administrator standing — so the signer must be attributable on every transport, and the act attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Issues a single-use presentation challenge bound to the community's DID and a new thread, and queues a credential-exchange/query to the holder. The challenge expires unused if the holder never answers; nothing is decided until the holder presents.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a holder and a criterion; the response returns the query the holder will see, including its nonce, so the caller can relay it. The nonce is a freshness value bound to the community's DID and the thread, not a credential: presenting it authorizes nothing without the holder's own key-bound presentation.
retention:
  class: exchange
  rationale: >-
    The challenge lives for the exchange — until the holder presents on the thread or it expires — and is consumed by the presentation.
errorCodes:
  - code: vtc/join-requests/query:criterionNotFound
    meaning: "No Accepts criterion is registered under `criterionId`."
    retryable: false
related:
  - credential-exchange/query
  - credential-exchange/present
  - vtc/schemas/accepts/register
  - vtc/join-requests/submit
---

## Abstract

A community admits members on evidence: a holder presents credentials satisfying one of its Accepts criteria. Usually the applicant starts the exchange. Sometimes the community does — an administrator wants a particular prospective member to present against a particular criterion — and the community, as the verifier, has to build the query itself: only it can issue a presentation challenge bound to its own DID and a thread it will recognise when the presentation arrives.

The **VTC Join Requests — Query** Trust Task is the administrator's instruction to do that. The community loads the criterion, issues the challenge, builds a [`credential-exchange/query/0.1`](../../../../credential-exchange/query/0.1/spec.md), queues it to the holder over whichever transport the holder speaks, and returns it — so a relayer can deliver it where the community cannot. The holder answers with [`credential-exchange/present`](../../../../credential-exchange/present/0.1/spec.md) on the returned thread.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** refuse with `criterionNotFound` when no Accepts criterion is registered under `criterionId`, before issuing anything.
3. **MUST** mint a fresh thread id, issue a single-use presentation challenge keyed by it and bound to the community's own DID, and build the query from the criterion's DCQL query, the challenge as `nonce`, and the criterion's `description` as `purpose` (or a statement naming the criterion when it has none).
4. **MUST** make the query document's `id` the returned `threadId`, so the holder's presentation threads back to the challenge.
5. **SHOULD** queue the query to the holder for guaranteed delivery, over the highest-preference transport both parties advertise; a failure to route or queue it **MUST NOT** fail the task — it is reported as `delivered: false`, and the caller may relay `query`.
6. **MUST** consume the challenge when the presentation on the thread is verified, so the query can be answered once.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. The holder is not a party to this document and presupposes nothing: they receive a query and decide for themselves whether to answer it.

## Why this is its own task, not a `credential-exchange/*` one

`credential-exchange/*` is the exchange itself — verifier and holder (or issuer and holder) talking to each other: `query` goes verifier → holder, `present` comes back. This task is neither party talking to the other. It is an **administrator instructing the verifier** to begin an exchange with a third party, and the two things that make it distinct are exactly what a `credential-exchange/query` cannot carry:

- **The verifier must author the query.** Its `nonce` is a challenge the community issues and binds to its own DID and to a thread it will recognise. An administrator could not construct one the community would accept, and sending the community a `credential-exchange/query` would mean the opposite — *asking the community to present credentials*.
- **The instruction names a criterion, not a query.** The administrator selects a registered Accepts criterion; the DCQL query, the purpose shown to the holder and the challenge are the community's to derive, so that what the holder is asked always matches what the community published.

So the query the holder receives *is* a `credential-exchange/query/0.1` document — this task's response carries its payload — and the administrator's request that caused it is this task.

## Definitions

- **Holder** — the prospective member the query is for.
- **Challenge** — the single-use freshness value the community issues, carried as the query's `nonce`.
- **Thread** — the exchange's identity; the holder presents on it.

## Request

A community administrator (`issuer`) names a holder and a criterion to the community (`recipient`).

### Ask a prospective member to present against the membership criterion

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/join-requests/query/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "holderDid": "did:example:prospect",
    "criterionId": "membership"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Queued for delivery

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/join-requests/query/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "threadId": "urn:uuid:9a8b7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d",
    "holderDid": "did:example:prospect",
    "query": {
      "dcql_query": {
        "credentials": [
          {
            "id": "membership",
            "format": "vc+sd-jwt",
            "meta": {
              "vct_values": [
                "https://openvtc.org/credentials/MembershipCredential"
              ]
            }
          }
        ]
      },
      "nonce": "b3Blbi12dGMtY2hhbGxlbmdlLTAx",
      "purpose": "Show a membership credential from a partner community."
    },
    "delivered": true
  }
}
```

## Security & Privacy

### Data carried

A holder DID and a criterion id in; the query the holder will see out — DCQL, nonce and purpose. `purpose` is the criterion's description, written by an administrator and shown to the holder as the community's reason for asking; it is attributed to the community.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The community learns, and records, that an administrator asked this holder for credentials; the holder learns the community is asking. The thread id links the query to the presentation that answers it, which is its purpose.

### Retention

The challenge is held for the exchange and consumed by the presentation or discarded at expiry. The instruction is audited against the administrator.

### Consent/purpose

The purpose is to ask a prospective member for the evidence one published criterion requires, and nothing more. The holder's agent decides whether to present, what to disclose, and may defer for consent under credential-exchange/query; this task obliges the holder to nothing.
