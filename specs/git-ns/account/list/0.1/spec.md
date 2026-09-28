---
slug: git-ns/account/list
version: "0.1"
title: "Git Namespaces — List Linked Forge Accounts"
summary: "A community administrator lists every member's forge account linked through git-ns/account/link, community-wide, since a member's own view of git-ns/view returns only their own."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - account
  - administration
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The answer is the community's whole forge-account roster — every member's linked identity on every forge — disclosed to nobody but community administrators. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's member and account-link records; changes nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns, for every current or former member with a linked account, the forge, the forge's own account id, the display login, when the link was recorded, and whether the member is still current — the community's whole forge-identity roster in one answer."
retention:
  class: exchange
  rationale: "The request carries at most a member filter, a forge filter and a paging cursor, and is needed only to answer it."
errorCodes:
  - code: git-ns/account/list:notCommunityAdministrator
    meaning: "The caller does not hold the community-administrator capability. Linked accounts are not namespace-scoped — a member's account is linked once, community-wide, and used wherever their rights reach — so holding git.ns.admin on a namespace does not entitle a caller to this list."
    retryable: false
related:
  - git-ns/account/link
  - git-ns/account/link-status
  - git-ns/account/unlink
  - git-ns/view
  - git-ns/bridge/job
---

## Abstract

A member links their own forge account once, community-wide, through [`git-ns/account/link`](../../link/0.1/spec.md); [`git-ns/view`](../../../view/0.5/spec.md) then shows that member their own linked accounts and nobody else's — correctly, since a member's forge identity is theirs to see, not another member's business. But an administrator diagnosing why a bridge job's `desiredRoles` skipped someone, or auditing which members have never linked an account and so hold no forge role at all, needs the whole roster. This task is that roster: every member's linked account on every forge, with the membership fact (`memberCurrent`) that says whether the link still projects anything. It is an admin console's *Linked accounts* table, a command-line `accounts list`.

It is a read. It changes nothing, links nothing and unlinks nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The caller is the DID the document's verified `proof` binds to its `issuer`; an unsigned document is refused with `proofRequired`.

The entitlement is **the community-administrator capability** in the VTC's own access control, exactly as in [`git-ns/right/list`](../../../right/list/0.1/spec.md). A linked account is not scoped to any namespace — the same link is used by every bridge, in every namespace, that needs to know a member's forge identity — so holding `git.ns.admin` on one or more namespaces does not entitle a caller to this community-wide roster. A caller who does not hold the community-administrator capability is refused with `git-ns/account/list:notCommunityAdministrator`.

The capability is read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Linked account** — a member's account on one forge, recorded against their DID through [`git-ns/account/link`](../../link/0.1/spec.md). At most one per member per forge, as `git-ns/account/link` enforces.

**Member current** — whether the linked member is still a current member of the community. A member whose access lapsed keeps the link — nobody else may claim the account — but it projects no forge role, and a right recorded for a forge role the account holds outside the projection cannot be adopted.

## Request

The community administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Confirms the caller holds the community-administrator capability, refusing with `git-ns/account/list:notCommunityAdministrator` otherwise.
3. Returns `accounts`: every linked account matching `member` and `forge` where given, ordered by `member` then `account.forge`.
4. Returns at most `limit` accounts (clamped to 1..=500, default 100) and, when more match, a `nextCursor` to continue.

A VTC **MUST NOT** change anything on this task.

### Dana looks up Frank's linked accounts

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6f01",
  "type": "https://trusttasks.org/spec/git-ns/account/list/0.1",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6f01",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:30:00Z",
  "payload": {
    "member": "did:webvh:QmFrankScid12:acme-vtc.example:frank"
  }
}
```

## Response

The VTC answers with the matching links, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/account/list:notCommunityAdministrator`.

### What Dana sees

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6f02",
  "type": "https://trusttasks.org/spec/git-ns/account/list/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6f01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-26T09:30:01Z",
  "payload": {
    "accounts": [
      {
        "member": "did:webvh:QmFrankScid12:acme-vtc.example:frank",
        "account": {
          "forge": "github.com",
          "id": "9931204",
          "login": "frank-acme"
        },
        "linkedAt": "2026-09-24T11:00:00Z",
        "memberCurrent": true
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a member filter, a forge filter and a paging cursor. The response carries, for every linked account community-wide: the member's DID, the forge, the forge's own account id (authoritative), the display login (renamed and re-registered on the forge, so display only), when the link was recorded, and whether the member is still current. A forge login can itself be personally identifying — many people use a recognisable handle across services — which is exactly why this roster is a community-administrator read and not something `git-ns/view` hands to any member. A producer **MUST NOT** put anything beyond a member or forge identifier into `ext`.

### Correlation

`member` lets a caller assemble one person's accounts across every forge they have linked; `forge` lets a caller assemble every member's identity on one forge. Both are correlation instruments the same way a filtered audit read is: convenient for an administrator answering a real question, and equally capable of building a roster nobody asked for. The entitlement does not depend on narrowing — a community administrator who asks unfiltered receives the whole roster, a page at a time — so the filters exist for usability, not for containing disclosure.

The **VTC** declares `identifierScope: public`, for the same reason as the rest of this family: the recipient is the community's own VTC, and a community administrator must recognise it as the same VTC across every account read. The **community administrator** carries no such declaration.

### Retention

The request is kept only as long as the VTC keeps request logs. A VTC that audits administrative reads **MAY** record that the caller listed accounts, and **SHOULD NOT** record the response, whose rows are the VTC's own durable link records under their own retention.

### Consent/purpose

The purpose is operational: diagnosing why a bridge's role projection did or did not reach a member, and auditing who has linked an account at all. It is not a directory for any other use. A caller **MUST NOT** republish a member's forge login or account id outside the community's own administration.
