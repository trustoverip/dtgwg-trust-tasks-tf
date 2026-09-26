---
slug: vtc/credentials/reissue
version: "0.1"
title: "VTC Credentials — Reissue"
summary: "A community administrator asks the VTC to re-sign its status lists and re-issue the credentials still in force that an attestation key revoked for compromise signed, under the community's current attestation key."
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - credentials
  - reissue
  - status list
  - key roles
  - incident response
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Community administrator
    requirement: REQUIRED
    member: issuer
  - role: VTC
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Re-issuance produces new attestation artefacts for every affected member and re-publishes every status list. The request that caused that must stay attributable to one administrator after the session has gone."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential; the job's start is measured from when the request executes, and after a compromise every hour of delay is an hour members cannot prove membership."
sideEffects:
  level: mutating
  rationale: "Re-signs status lists and issues replacement credentials with the same claims and validity. Recoverable — nothing is revoked, extended or narrowed — and idempotent per cause."
subjectPath: /cause/verificationMethod
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns counts and job state only; never member identities or credential contents."
retention:
  class: durable
  rationale: "The job record and audit rows are the evidence that the obligation of VTI-KEY-133 was met."
errorCodes:
  - code: vtc/credentials/reissue:causeNotFound
    meaning: "The named key is not an attestation key of this community's DID revoked for compromise."
    retryable: false
  - code: vtc/credentials/reissue:noSoundAttestationKey
    meaning: "The community's DID has no active attestation key to sign with. Add or rotate one first."
    retryable: true
related:
  - vta/webvh/dids/keys/revoke
  - vta/webvh/dids/keys/list
  - vtc/members/renew
---

## Abstract

When a community's attestation key is revoked for compromise, verifiers stop accepting what
it signed from the compromise time, and — without independent evidence of issuance time —
everything it signed. The community owes its members replacements, and must re-sign every
status list at once (VTI-KEY-133). The VTA reports how much is owed — `followUp` on
[`keys/revoke`](../../../../vta/webvh/dids/keys/revoke/1.0/spec.md) — but it cannot do the
work: re-issuing a membership credential is the community's decision, made by the VTC and
signed by the VTA's signing oracle.

**VTC Credentials — Reissue** starts that work, and reports its progress.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **consumer** (the VTC) **MUST**:

1. Confirm the cause against its own DID's key roles — the key must be an `attestation` key in
   state `revoked` — and refuse otherwise (`causeNotFound`). A request
   naming a sound key is not a reason to re-issue anything.
2. Treat the cause as the job's identity: a second request with the same `cause` returns the
   existing job's progress and starts nothing.
3. **Re-sign every status list first**, before re-issuing any other artefact, under an active
   attestation key, by signing requests to its VTA. Status lists are the artefact whose loss
   re-admits every suspended member (VTI-KEY-080 rationale).
4. Re-issue each attestation artefact still in force that the VTA's signing record shows the
   key signed, with **the same subject, claims, status entry and expiry** — never extending,
   narrowing or re-dating membership. Re-issuance repairs a signature; it is not a renewal.
5. Deliver each replacement to its holder over the channel the original used, and hold any it
   cannot deliver for the holder's next contact, counting them as `undeliverable`.
6. Record one audit row per status list re-signed and one per job state change; not one row
   per member, which would be a membership log by another name.

A VTC **MAY** start the job itself when it observes the revocation; it records
the job under the VTA's rotation record as cause, and a later request returns that job.

## Authorization

A community administrator who may issue membership credentials. The VTC's own service
identity **MUST NOT** be able to start a job for a cause it did not observe in its DID's log.

## Payload

`payload.cause` (REQUIRED) — the revocation: `verificationMethod`, optional `rotationId`.
`payload.scope` (OPTIONAL) — `all` (default) or `statusListsOnly`.
`payload.dryRun` (OPTIONAL) — report without doing.
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

```json
{
  "id": "urn:uuid:4d5e6f70-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/credentials/reissue/0.1",
  "issuer": "did:key:z6MkCommunityAdmin",
  "recipient": "did:webvh:QmVtcScid:vtc.example",
  "issuedAt": "2026-09-25T11:05:00Z",
  "payload": {
    "cause": {
      "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttestation",
      "rotationId": "rot-0004"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-25T11:05:00Z",
    "verificationMethod": "did:key:z6MkCommunityAdmin#z6MkCommunityAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:4d5e6f70-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/credentials/reissue/0.1#response",
  "issuer": "did:webvh:QmVtcScid:vtc.example",
  "recipient": "did:key:z6MkCommunityAdmin",
  "issuedAt": "2026-09-25T11:05:01Z",
  "threadId": "urn:uuid:4d5e6f70-0000-4000-8000-000000000001",
  "payload": {
    "jobId": "reissue-rot-0004",
    "state": "statusListsDone",
    "dryRun": false,
    "statusLists": { "total": 3, "done": 3, "failed": 0 },
    "artefacts": { "total": 412, "done": 57, "failed": 0, "undeliverable": 4 },
    "signedWith": ["did:webvh:QmVtcScid:vtc.example#z6MkReplacement"]
  }
}
```

## Security & Privacy

### Data carried

The request names a key of the community's own DID; the response carries counts. No member
identity, credential or key material crosses the wire, and a conforming VTC **MUST NOT** place
any in `ext`.

### Correlation

Re-issuing every affected credential at once, keyed to a community-wide event, identifies no
member; re-issuing them one by one at each member's next contact would tie each new signature
to that contact. A VTC **SHOULD** re-issue in bulk and deliver on the next contact, rather than
issue on contact. Keeping claims, status entries and expiry identical means the replacement
reveals nothing its original did not.

### Retention

The job record and its audit rows are kept as evidence that the obligation was met.
Per-member delivery state is kept only until delivery.

### Consent/purpose

The purpose is repair of signatures a compromise invalidated. The job **MUST NOT** be used to
change what a member holds.
