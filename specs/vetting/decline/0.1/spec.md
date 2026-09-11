---
slug: vetting/decline
version: "0.1"
title: Vetting — Decline
summary: A vetter tells an applicant it will not issue a Vetting Statement for a request it accepted. The reason is optional, and the decline never goes to the community.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - vetting
  - identity-vetting
  - decline
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: applicant
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: A decline ends an applicant's wait for a statement from this vetter. Unattributed, anyone who learned a requestId could end a vetting on the vetter's behalf, and the applicant would move on believing the vetter had refused them.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A decline closes a request in the applicant's records. Its issue time places it against any statement delivered for the same request, so a replayed or delayed decline cannot overturn a statement that arrived after it.
sideEffects:
  level: mutating
  rationale: "The applicant's agent marks the request declined and stops waiting on this vetter."
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "The optional `code` is a judgement about the applicant — that their identity could not be verified, or that a document did not match — and the optional `message` is free text a vetter wrote about them. Both are personal data about the recipient, which is why neither is required. Nothing is returned."
retention:
  class: exchange
  rationale: The applicant needs the decline to close the request and to decide whether to approach another vetter. Nothing in it is evidence that anyone else relies on.
errorCodes:
  - code: vetting/decline:unknownRequest
    meaning: The applicant has no open request with this vetter under the named requestId.
    retryable: false
related:
  - vetting/request
  - vetting/session
---

## Abstract

A vetter that accepted a [`vetting/request`](../../request/0.1/spec.md) sends this when it will not issue a Vetting Statement for it. That might be before any session, after a [`vetting/session`](../../session/0.1/spec.md) in which something did not check out, or simply because the vetter is not comfortable attesting.

A vetter never has to say why. `code` is optional, and `message` is optional too. The decline goes to the applicant and nobody else. It is not reported to the community, so declining someone creates no record anywhere the applicant cannot see. The applicant can approach other vetters.

The task defines no success response.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **vetter** (`issuer`):

1. **MAY** decline a request it accepted at any time before issuing a statement for it. It **MUST NOT** decline a request for which it has issued a statement; withdrawing a statement is a separate act, addressed to the community that relies on it.
2. **MUST NOT** issue a statement for a request it has declined.
3. **MUST** set `parentThreadId` to the request exchange's `threadId`.
4. **MUST NOT** send the decline, its code or its message to the community, or to anyone but the applicant. A vetter who suspects fraud raises that separately, under the community's own process.

A conforming **applicant** (`recipient`):

1. Accepts a decline only from the vetter that accepted `requestId`, and otherwise returns `vetting/decline:unknownRequest`.
2. Treats the request as closed, and refuses any later session under it with `vetting/session:unknownRequest`.
3. Where it shows `message` to its person, **MUST** attribute it to the vetter, and **SHOULD** render `code` in plain language rather than as a code.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority to decline is the **accepted request**. The vetter that accepted `requestId` may decline it, and nobody else may. Declining needs no further authority and no justification. The REQUIRED `proof` establishes that this vetter sent it, which is what makes Conformance item 1 for the applicant checkable. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, it establishes nothing more.

## Definitions

**`code`** — optional:

- `could-not-verify` — the vetter could not establish the claimed identity;
- `document-mismatch` — the documentation did not match the card or the person;
- `liveness-failed` — the match code could not be confirmed with the person;
- `not-comfortable` — the vetter prefers not to attest;
- `other` — none of these.

**`message`** — optional free text from the vetter to the applicant, at most 500 characters.

## Request

The vetter sends the decline to the applicant. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### After a session, with a reason

```json
{
  "id": "urn:uuid:3c5a7e9b-2d4f-4a6c-8e1b-5f7d9a2c4e01",
  "type": "https://trusttasks.org/spec/vetting/decline/0.1",
  "parentThreadId": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmAliceScid1:alice.example",
  "issuedAt": "2026-09-17T15:12:00Z",
  "payload": {
    "requestId": "urn:uuid:4b2e8f10-7a6c-4d3b-9e21-0f5a6b7c8d01",
    "code": "document-mismatch",
    "message": "The name on the document you showed did not match the card. Happy to try again another day."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
    "created": "2026-09-17T15:12:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2RA8945kouBqzqifZqkbB8ZSrj1sfVLZPvr6wz4RvHSaYqXySHQoep9vM1fRYit6tNfmaTDThA2ibMPhBMFh8w3N"
  }
}
```

## Security & Privacy

### Data carried

`requestId` alone is a complete decline, and a vetter **SHOULD** send no more than it thinks helps the applicant. A reason code tells the applicant something about how they came across. That can help — the wrong card, a document the vetter does not accept — but it is still a judgement about a person, and silence about the reason is always conforming.

`message` is bounded, written by the vetter, and attributed to the vetter wherever it appears. A vetter **MUST NOT** use it to record documentation details or suspicions. What the applicant showed stays in the session, and a fraud concern is not the applicant's to receive.

### Correlation

Both parties declare `identifierScope: public`, for the reasons [`vetting/request`](../../request/0.1/spec.md) gives. The applicant's join DID is the one DID every vetter of an application sees. The vetter's DID is its member DID in the community, so that what it does as a vetter stays attributable to a member. A decline has to come from the same identifiers as the request it closes — a pairwise vetter identifier minted just for the decline could not be matched to the vetter that accepted the request, and the applicant could not tell a real decline from a forged one.

The decline carries no identifier the request did not already carry. It links to the request through `requestId` and `parentThreadId`, which is its purpose. It creates no new record at the community, so declines cannot be pooled into a list of people vetters turned away. Keeping it that way — no copies, no community reporting under this task — is what prevents a decline becoming a quiet blacklist.

### Retention

The applicant keeps the decline as part of its application history, for as long as that is useful to its person. The vetter **SHOULD** keep no more than the fact that it declined this request, and **SHOULD NOT** keep `message`. Neither party needs the decline as evidence, and nobody else is entitled to it.

### Consent/purpose

The decline exists to tell the applicant, promptly, that this vetter will not attest, so they can go elsewhere. It is not a finding, and it is not for the community. Using declines to rate applicants, or sharing them among vetters, is a purpose neither party addressed. Whether an agent confirms with its person before sending a decline is that agent's policy; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification takes no position.
