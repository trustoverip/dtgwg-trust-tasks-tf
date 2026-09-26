---
slug: git-ns/account/link
version: "0.1"
title: "Git Namespaces — Link Forge Account"
summary: "A member begins linking their account on a forge to their DID, so the bridge can give them the forge roles their rights call for. The response says where to authorise; the member polls git-ns/account/link-status."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - account
  - oauth
  - device-flow
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The link binds a forge identity to the requesting DID. It must be attributable to that DID on every transport, or anyone could start binding their own forge account to someone else's rights."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A link request replayed later would start a new binding flow the member did not ask for."
sideEffects:
  level: mutating
  rationale: "Starts a binding flow at the bridge that, when the member completes it, records their forge account against their DID. Reversible with git-ns/account/unlink, or by linking another account."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a forge host. The response returns a link identifier, an authorisation URL, and for device flows a short-lived code."
retention:
  class: exchange
  rationale: "The request lives as long as the link attempt; the binding it leads to is recorded separately."
errorCodes:
  - code: git-ns/account/link:unsupportedForge
    meaning: "This VTC has no bridge-mode namespace on this forge, so no bridge can complete the link."
    retryable: false
related:
  - git-ns/account/link-status
  - git-ns/account/unlink
  - git-ns/bridge/job
  - git-ns/bridge/event
---

## Abstract

To project a member's rights onto a forge, the bridge must know which forge account is theirs. A member links it once per forge: this task begins the link and returns where to authorise it; the member then polls [`git-ns/account/link-status`](../../../../git-ns/account/link-status/0.1/spec.md) until it completes. On GitHub the link uses the OAuth device flow, so it works from a terminal: the response carries a `userCode` to type at `url`. On Forgejo, which has no device flow, `url` is an authorisation page (authorisation code with PKCE) and there is no code.

A member with no linked account can still hold every right. They just get no role on the forge, which is right for fork-based contribution.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **membership of the community**. The link binds the forge account to the caller's own DID and to nobody else's: there is no subject member. The forge's authorisation of the account holder is the second half of the evidence: the link completes only when the forge confirms who authorised it.

## Definitions

**`forge`** — the forge whose account to link.

**`linkId`** — this attempt, for [`git-ns/account/link-status`](../../../../git-ns/account/link-status/0.1/spec.md).

**`url`**, **`userCode`**, **`expiresAt`** — where to authorise, the device code when there is one, and when the attempt lapses.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a non-member with `permissionDenied`.
2. Refuses a forge on which it has no bridge-mode namespace with `git-ns/account/link:unsupportedForge`: there is no bridge to complete the flow, and nothing to project to.
3. Sends that namespace's bridge a `beginAccountLink` job for the caller ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.1/spec.md)) and returns what the bridge answers, under a new `linkId`.
4. When the bridge reports `accountLinked`, refuses the link if the forge account's id is already linked to another member — one who has not left, whether or not their access is current (the attempt ends `failed`). The check and the recording **MUST** be atomic with respect to every other link and unlink, so that two members completing links to the same account at once cannot both succeed. Otherwise records the account against the caller's DID for that forge, replacing any account previously linked there, and **SHOULD** issue the member a credential attesting the binding of their DID to the account's forge id.

A member may link one account per forge, and accounts on several forges, and removes one with [`git-ns/account/unlink`](../../../../git-ns/account/unlink/0.1/spec.md). The forge id is authoritative; the login is display only, because logins are renamed and re-registered.

### Linking a GitHub account

```json
{
  "id": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d801",
  "type": "https://trusttasks.org/spec/git-ns/account/link/0.1",
  "threadId": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d801",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "forge": "github.com"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBobScid2:acme-vtc.example:bob#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zC9FQHmXCvSgqeqtZQtB8ZrLjda82d8fLMMBJ2fxytAsHQqCMDjvBA5QNRFHCeUPy66TtSzFn7qeuYn4hT4G2cf"
  }
}
```

### Linking a Codeberg account

```json
{
  "id": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d803",
  "type": "https://trusttasks.org/spec/git-ns/account/link/0.1",
  "threadId": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d803",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "forge": "codeberg.org"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBobScid2:acme-vtc.example:bob#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zrhiYoJTFEXmh1bTbFczT5qchgpygHgyJdyYVcCwanQvkQbBKkL3UjCAEqg2xC5zUv8h7U315sDJ6B525DQDSQm"
  }
}
```

## Response

The VTC, now responding, returns where to authorise, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Device flow

```json
{
  "id": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d802",
  "type": "https://trusttasks.org/spec/git-ns/account/link/0.1#response",
  "threadId": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d801",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "linkId": "lnk_4Tq9Xw2P",
    "url": "https://github.com/login/device",
    "userCode": "WDJB-MJHT",
    "expiresAt": "2026-09-23T10:15:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zUFtQjyzA2TWvwh2uk6Uqk1twGq9NqeYA618M7mo8KooyZs6CJVU9T8LEKCdaqCFgZ2XbSUGd24FAcRMzeqi1nL"
  }
}
```

### Authorisation code with PKCE

```json
{
  "id": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d804",
  "type": "https://trusttasks.org/spec/git-ns/account/link/0.1#response",
  "threadId": "urn:uuid:2d66a264-6830-4b7f-a2c9-22c5d5b2d803",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "linkId": "lnk_8Rm3Kd7Q",
    "url": "https://codeberg.org/login/oauth/authorize?client_id=acme-vgi&response_type=code&state=Zp4v&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&code_challenge_method=S256",
    "expiresAt": "2026-09-23T10:15:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zG6dB4niuDxvww3pvw5ogaRG1P6PCF2MMNja3xLxPmMWfQGWMkVScbYKYaP7ydMYJ9bXY3MX2wBz42ifGrMCQk6"
  }
}
```

## Security & Privacy

### Data carried

A forge host in; a link identifier, a URL, a device code and an expiry out. The device code is short-lived and useful only together with the member's own sign-in to the forge; still, a client **MUST** show it only to the member and **MUST NOT** log it.

### Correlation

The VTC declares `identifierScope: public`; the member `pairwise`. Completing the link joins the member's DID to their forge identity inside the VTC and its bridge. That join is the point of the task, and it is not published: the Trust Registry holds DIDs, never forge accounts. Anyone comparing the registry with the forge's own role list can still infer it for members who hold forge roles.

### Retention

The attempt lives until it completes or lapses. The resulting binding is durable: it is kept until the member unlinks it with [`git-ns/account/unlink`](../../../../git-ns/account/unlink/0.1/spec.md), links another account on the same forge, or leaves the community.

### Consent/purpose

The purpose is to project the member's rights onto their forge account. The VTC and its bridge **MUST NOT** use the binding for anything else, and **MUST** delete it when the member leaves.
