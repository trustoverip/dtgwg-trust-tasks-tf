---
slug: vtc/vetting/vetters/show
version: "0.1"
title: "VTC Vetting — Show Vetter"
summary: Ask a community about one vetter by DID, and learn whether their grant is live, revoked, expired, or was never made.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: applicant or member
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The community answers only a caller it can identify, and an authenticated transport identifies one. A proof is recommended so a community that rate-limits or audits these reads can attribute them on every transport, relayed ones included — the same strength vtc/vetting/vetters/list asks for, since this task answers about the same people.
sideEffects:
  level: none
  rationale: >-
    Reads one vetter's grant record and, for a live grant, whether a profile is stored as listed. Persists nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response discloses one member's standing as a vetter in this community — that they hold, held, or never held a grant, and when it ends or ended. That is personal data about the vetter, disclosed to identified callers only, and it is narrower than what vtc/vetting/vetters/list discloses about the same person. The request carries the vetter's DID, which tells the community that this caller is asking about this vetter.
retention:
  class: transient
  rationale: >-
    A read changes nothing and needs no record. What a community may keep is that an identified caller asked, for rate limiting. The caller's copy goes stale the moment the grant is revoked or lapses, which is the event this task exists to surface.
errorCodes: []
related:
  - vtc/vetting/vetters/list
  - vtc/vetting/vetters/grant
  - vtc/vetting/vetters/profile
  - vetting/request
---

## Abstract

A community's vetter listing ([`vtc/vetting/vetters/list`](../../list/0.1/spec.md)) omits a vetter with no published profile and a vetter whose grant has been revoked in exactly the same way: both are simply absent. An applicant mid-exchange with a vetter who has gone quiet therefore cannot tell whether that vetter was removed as a vetter or never chose to be listed, and a vetter cannot check their own standing at all. The only way to find out today is to fetch the grant credential's status list, which requires already holding the credential.

This task answers the question directly: given a vetter's DID, the community says whether the grant is `live`, `revoked`, `expired` or `none`. It is a Trust Task rather than an API call because the answer is about a named third party, it is disclosed only to callers the community can identify, and the same request travels over every transport a community offers.

It is a separate task rather than a `status` member added to the listing response because adding a member to an existing response is a breaking change for consumers that reject unknown members, and because the two answer different questions: the listing is discovery, and this is a lookup about someone you can already name.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **caller** (`issuer`) sends one `vetterDid` and **MUST NOT** read `none` as a statement that the DID is not a member: it says only that this member holds no vetter grant here.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline. It **MUST NOT** answer a caller it cannot identify — one with neither a verified `proof` nor a transport-authenticated sender — and refuses it with `permissionDenied`.
2. **MUST** answer with exactly one `status`, chosen in this order, so that a vetter with more than one grant record is reported by the strongest state that holds:
   - `live` — a grant exists, is not revoked, and its `validUntil` is in the future.
   - `revoked` — a grant exists and was revoked, whether or not it had also expired. Revocation is the stronger statement: it says the community withdrew its trust rather than let it lapse.
   - `expired` — a grant exists, was not revoked, and its `validUntil` has passed.
   - `none` — no grant record exists for this DID in this community.
3. **MUST** answer `none` for a DID it holds no grant record for, whether or not that DID is a member, and whether or not it exists. A community **MUST NOT** distinguish "not a member" from "a member who is not a vetter" in this response: the caller asked about vetting, and membership is not theirs to learn here.
4. **MUST** include `grantId` for `live`, `revoked` and `expired`, and **MUST NOT** include it for `none`.
5. **MUST** include `validUntil` for `live` and `expired`, **MUST** include `revokedAt` for `revoked`, and **MUST NOT** include `revokedAt` for any other status. It **MUST NOT** carry the revocation's reason.
6. **MUST** include `listed` for `live`, and **MUST NOT** include it otherwise. This is what tells a caller that a live vetter is absent from the listing by the vetter's own choice.
7. **SHOULD** answer at the same cost whatever the status, so that response timing does not distinguish `none` from the rest.

## Definitions

- **`vetterDid`** — the DID of the vetter the caller is asking about, chosen by the caller. A community matches it against the subject of its vetter grants.
- **`status`** — the community's answer, one of `live`, `revoked`, `expired` or `none`, as Conformance item 2 defines.
- **`grantId`** — the identifier of the grant the status is about, as [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) issued it. A caller can quote it when asking a maintainer about a refusal.
- **`validUntil`** — when the grant ends or ended.
- **`revokedAt`** — when the community revoked the grant.
- **`listed`** — whether a profile is stored for this vetter with `listed: true`, and so whether they appear in [`vtc/vetting/vetters/list`](../../list/0.1/spec.md).

## Request

An applicant or member (`issuer`) asks a community (`recipient`) about one vetter. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): a single required `vetterDid`.

### An applicant checks a vetter who stopped responding

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/show/0.1#request",
  "issuer": "did:example:applicant",
  "recipient": "did:example:community",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "vetterDid": "did:example:vetter"
  }
}
```

## Response

The community (`recipient` of the request, now responding) answers with the sub-schema reachable via `$anchor: "response"`. It carries the DID asked about, the `status`, and the members Conformance items 4 to 6 attach to that status. A refusal is a `trust-task-error`, not a `#response` document with a refusing status: `none` is an answer, not a failure.

### The grant was revoked

The applicant now knows to find another vetter, rather than waiting on someone whose vetting this community would not accept.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:applicant",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "vetterDid": "did:example:vetter",
    "status": "revoked",
    "grantId": "urn:uuid:11111111-1111-4111-8111-111111111111",
    "revokedAt": "2025-12-20T09:15:00Z"
  }
}
```

### The grant is live, and the vetter chose not to be listed

The same answer a vetter gets when checking their own standing: they are a vetter here, and the reason they cannot find themselves in the directory is their own `listed: false`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-01-01T00:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "vetterDid": "did:example:vetter",
    "status": "live",
    "grantId": "urn:uuid:22222222-2222-4222-8222-222222222222",
    "validUntil": "2026-06-30T00:00:00Z",
    "listed": false
  }
}
```

## Security & Privacy

### Data carried

The request carries one vetter's DID and nothing else. The response carries that DID, the grant's state, its identifier, the timestamp that ended or will end it, and — for a live grant — whether the vetter is listed. That is the smallest answer that still separates the four cases the caller cannot otherwise distinguish.

The revocation **reason** is deliberately absent. A community's reason for withdrawing a grant is its own record and frequently concerns a third party; the caller needs to know the grant does not hold. A community **MUST NOT** put a reason, a note, or any other free-form text about the vetter into `ext` on this response either — the same rule, and `ext` is not a way around it.

### Correlation

The recipient learns that this caller is interested in this vetter, and repeated calls describe an exchange in progress. An observer of an unencrypted transport learns the same. `threadId` joins the request and its response and nothing else; a caller who does not want two lookups joined **SHOULD** use a fresh thread for each, and a pairwise identifier where the transport allows one — `identifierScope: any` on the caller exists so that it can.

The vetter is not told that they were asked about, which is a deliberate asymmetry: notifying them would tell a vetter which applicants are checking on them.

### Retention

A community needs to keep nothing from this exchange. Where it keeps a record for rate limiting or abuse investigation, that record is about the caller, not the vetter, and the grant data in the response is already held for the grant's own sake.

A caller's copy is a point-in-time reading. A grant can be revoked a second after the answer is sent, so a caller **MUST NOT** treat a `live` answer as durable evidence of eligibility; eligibility is proven by the vetter's own credential in `vetting/request`, not by this lookup.

### Consent/purpose

The purpose is to let a caller act on a vetter's current standing: to stop waiting on a revoked vetter, to choose another, or to understand one's own absence from a listing. The vetter consented to being a vetter in this community, which is what is disclosed.

Reusing the answer to build a directory of a community's vetters is outside that purpose: [`vtc/vetting/vetters/list`](../../list/0.1/spec.md) is the task for discovery, and it returns only vetters who chose to be found. A community that sees enumeration **SHOULD** rate-limit it, which is why it identifies every caller.
