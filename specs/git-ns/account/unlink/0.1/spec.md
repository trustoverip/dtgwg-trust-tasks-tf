---
slug: git-ns/account/unlink
version: "0.1"
title: "Git Namespaces — Unlink Forge Account"
summary: "A member removes the link between their DID and their account on one forge. The VTC deletes the binding and re-projects, so the bridge withdraws the forge roles it gave that account; the member's git rights are unchanged."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - account
  - unlink
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
  rationale: "Unlinking takes forge roles away from an account. It must be attributable to the DID the account is linked to on every transport, or anyone could strip another member of their forge access."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An unlink replayed after the member linked an account again would remove the account linked since. Placing the request in time lets the VTC refuse the replay; `accountId` closes the same window for a request that is merely late."
sideEffects:
  level: mutating
  rationale: "Deletes the member's binding on one forge, and through the next projection removes the forge roles the bridge gave that account. Reversible by linking again with git-ns/account/link, after which the projection gives the roles back."
consequences:
  - "The forge roles the bridge gave this account — on the organisation and on every repository of every namespace on that forge — are withdrawn at the next projection. Roles the bridge did not give are left as they are and reported as drift."
  - "The member's git rights are unchanged, and so is what they may sign: commit signing is by DID, not by forge account. Without a linked account they contribute to that forge's repositories by fork pull request."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a forge host and, optionally, a forge account id. The response returns the account that was unlinked, which is the caller's own; a caller with nothing linked learns only that."
retention:
  class: exchange
  rationale: "The request is needed only to perform it. What remains is an audit record that the member unlinked an account on that forge, without the account."
errorCodes:
  - code: git-ns/account/unlink:notLinked
    meaning: "No account is linked to the caller's DID on this forge — the answer for any caller with nothing linked, member or not — or `accountId` was given and the account linked there now is a different one."
    retryable: false
related:
  - git-ns/account/link
  - git-ns/account/link-status
  - git-ns/view
  - git-ns/bridge/job
  - git-ns/drift/resolve
---

## Abstract

A member links their account on a forge with [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md) so the bridge can give that account the forge roles their git rights call for. This task undoes it: the VTC deletes the binding of the member's DID to their account on one forge, and the next projection no longer names the account, so the bridge withdraws the roles it gave it.

A member unlinks when the account is no longer theirs to use — a work account left behind, an account they are retiring, one they suspect is compromised — or when they no longer want the community to join it to their DID. Linking another account on the same forge replaces the old one without this task; unlinking leaves the member with none there.

Only the member's forge access changes. Their git rights stay recorded and published, and what they may sign does not change: `did-git-sign verify-trust` checks commits by DID, never by forge account.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the DID the account is linked to**: there is no subject member, and a caller unlinks only the account linked to their own DID. That includes a member whose access has lapsed but who has not left — their link still stands, projecting nothing, and removing it is theirs to do. A caller with no link, a non-member included, is answered as having nothing linked, so the answer reveals nothing about who is a member. The consent class is the same as linking's — `normal` in the delegation pillar's terms — because the task removes access from the caller alone and linking again restores it.

A community administrator removing another member's link — say, for an account reported compromised — is **not defined by this version**. It is a different entitlement over a different subject, with its own consent class and notice to the member, and a later version may add it with a `subject` member. Until then a community administrator who needs a member's forge roles withdrawn revokes the rights, or resolves the drift, instead.

## Definitions

**`forge`** — the forge whose linked account to unlink.

**`accountId`** — the forge id of the account the member means, as they last read it (from [`git-ns/view`](../../../../git-ns/view/0.2/spec.md) or [`git-ns/account/link-status`](../../../../git-ns/account/link-status/0.1/spec.md)). A guard, not a selector: a member has at most one account linked per forge.

**`unlinked`**, **`unlinkedAt`** — the account the binding was to, and when it was deleted.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses with `git-ns/account/unlink:notLinked` when no account is linked to the caller's DID on `forge` — whoever the caller is, member or not — or when `accountId` is present and is not the id of the account linked there. A client that means "make sure nothing is linked" may treat this refusal as done. The VTC **MUST NOT** answer a non-member differently from a member with nothing linked.
2. Performs items 3 to 6 atomically with respect to link completion ([`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md), *Request* item 4), so that a link completing at the same moment is either unlinked, when it is the account named, or kept.
3. Deletes the binding of the caller's DID to that account, and **MUST** delete with it anything else it holds joining the two — a credential it issued attesting the binding **MUST** be revoked.
4. Re-projects every bridge-mode namespace on `forge`: it sends each namespace's bridge the complete `desiredRoles` of [`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md) for the namespace itself and for each of its repositories, computed without the binding. The account is not listed in any of them, and under that task's rule — *people not listed lose any role the bridge manages* — the bridge withdraws exactly the roles it gave the account: its repository roles, and its organisation role where the namespace projects one. The VTC **SHOULD** send these projections promptly rather than wait for a periodic pass, since the member asked for the access to end. A namespace on the forge that is `manual`-mode, or has no bridge, has nothing to re-project.
5. **MUST NOT** name the account in `removeAccounts` for this. A role the bridge did not give was not given through the link, and whether the account keeps it is for the repository's owners: the bridge reports it as `roleAdded` drift for an account linked to no member, and an owner reverts it with [`git-ns/drift/resolve`](../../../../git-ns/drift/resolve/0.2/spec.md) if they choose. The same happens to a role the bridge gave but no longer remembers giving.
6. Records the unlink in its audit trail, naming the member and the forge. The audit record **MUST NOT** keep the account's id or login: the binding is deleted, and keeping the join in the audit trail would keep it.

The member's git rights are not touched. A link attempt the member began on `forge` and has not completed is not cancelled either: completing it links that account afresh, which is the member's own doing.

A member who has left the community has no binding to unlink — the VTC deletes it on departure (`git-ns/account/link`, *Consent/purpose*).

### Bob unlinks his GitHub account

```json
{
  "id": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a01",
  "type": "https://trusttasks.org/spec/git-ns/account/unlink/0.1",
  "threadId": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a01",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T09:30:00Z",
  "payload": {
    "forge": "github.com",
    "accountId": "9120045"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBobScid2:acme-vtc.example:bob#key-1",
    "created": "2026-09-25T09:30:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zMASi45ub7Qe4ZE36UT5G6cU4ud8Fhhe4deS4F3cw9KTAb8dLcukC7edhDQ7cn5d4gEYkbUrMWeWQLGsCmrG6dLa"
  }
}
```

## Response

The VTC, now responding, returns the account that was unlinked, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). The response does not wait for the bridge: the roles are withdrawn when the bridge runs the projections, and a failure there is reported as the bridge reports any `projectRoles` job. Refusals use `trust-task-error`.

```json
{
  "id": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a02",
  "type": "https://trusttasks.org/spec/git-ns/account/unlink/0.1#response",
  "threadId": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-25T09:30:01Z",
  "payload": {
    "unlinked": {
      "forge": "github.com",
      "id": "9120045",
      "login": "bob-builds"
    },
    "unlinkedAt": "2026-09-25T09:30:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-25T09:30:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zYyNoVKf58ZTBqNAYT3j5qcdsyuMNmPfYetW5v6JXmj54omLidkuVKnRyjP2WPBg8Y4ErK9pGSSxY6BVScJy9uUx"
  }
}
```

### The account linked there has changed

Bob relinked a different GitHub account after reading `9120045`, so the guard refuses.

```json
{
  "id": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-25T09:30:01Z",
  "payload": {
    "code": "git-ns/account/unlink:notLinked",
    "message": "the account linked on github.com is not 9120045",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/git-ns/account/unlink/0.1",
      "id": "urn:uuid:5b1f3c7e-0d8a-4b91-9a57-2f3c6e1d7a01"
    }
  }
}
```

## Security & Privacy

### Data carried

A forge host and optionally a forge id in; the member's own account out. Nothing here is disclosed to anyone but the member who linked it.

### Withdrawal is by projection, and bounded by it

The roles withdrawn are the ones the bridge gave. That is deliberate: the projection is a pure function of the VTC's records, and removing the binding is a change to the records like any other. It also means an unlink is not a way to take arbitrary access off an account: a role someone granted on the forge by hand stays, reported as drift, until an owner decides. And an unlink cannot reach access the account has through a team or as an organisation owner outside the projection; the bridge reports what remains.

A member who believes their forge account is compromised should unlink it, and also tell the namespace's admins, who can resolve any drift the account is left holding.

### Correlation

Unlinking ends the join of the member's DID to their forge identity inside the VTC and its bridge. The forge keeps its own history of who held which role; the Trust Registry never held the account.

### Retention

The binding is deleted. The audit record keeps that the member unlinked an account on the forge, and not which account.

### Consent/purpose

The purpose is to stop projecting the member's rights onto that account. After an unlink the VTC and its bridge **MUST NOT** use the account for this member again unless the member links it again.
