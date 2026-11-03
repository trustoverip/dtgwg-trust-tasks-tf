---
slug: vtn/directory/search
version: "0.1"
title: "VTN Directory — Search"
summary: "Search the communities and networks a network publishes, each result stating whether it is a member, as far as the network's directory visibility admits the caller."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, directory, curation, discovery]
parties:
  - role: reader
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: OPTIONAL
  rationale: >-
    The published set is public when the network's directory visibility is `public`, so a page that has not signed in can still search it. A network whose visibility is `members` or `admins` decides who may read from the signer, so it refuses an unproved request with `permissionDenied`.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; replaying it yields nothing the replayer could not ask for itself. A restricted network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads the published set; changes nothing.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: published entries, which are organisations' public identifiers, names and credential ids. In: the query and, in a `members` network, a person's membership credential (VMC), which names that person and is used only to decide admission.
retention:
  class: transient
  rationale: >-
    The answer is computed from the published set and nothing in the request is kept: the query and any membership credential are discarded once admission is decided.
errorCodes: []
related:
  - vtn/directory/federated
  - vtn/directory/summary
  - vtn/entries/show
  - vtn/graph/show
---

## Abstract

A Verifiable Trust Network publishes the communities and networks it lists, and whether each has acknowledged — only an acknowledged entry is a member ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-013). **VTN Directory — Search** searches that published set by free text, kind and state. It reads this network's own set only; [`vtn/directory/federated`](../../federated/0.1/spec.md) extends the search into its member networks. Who may search is decided by the network's directory visibility.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** sign the request when it carries `membership`, or when the network's profile shows a directory visibility other than `public`.
2. **MUST** send `membership` only to a network whose visibility is `members`, and only a credential naming the identity the request is signed as (or the person that delegated the signing key).
3. **MUST NOT** treat a result whose `acknowledged` is `false` as a member, whatever its `state` (VTI-VTN-013).

A conforming **network** (`recipient`):

1. **MUST** admit the request under its configured directory visibility, read at execution time, and **MUST** refuse with `permissionDenied` a caller that visibility does not admit, an unproved request included:
   - `public` — any caller, with or without a proof;
   - `members` — an administrator (the signer is in the network's administrator list, or is a key one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md)); a listed party whose entry is `acknowledged`; or a person whose `membership` verifies under network item 2;
   - `admins` — an administrator only.
2. **MUST**, when admitting a person by `membership`, verify that the credential is a membership credential issued — and signed for `assertionMethod` — by a community whose entry in this network is `acknowledged`; that it is a grant (it carries no `digestMultibase`), not an acknowledgement; that its subject is the request's signer or the person that delegated the signing key; and that its status, resolved against the community's own status lists, is valid. A status that cannot be determined **MUST** be treated as a failure (VTI-REG-012). Any failure is `permissionDenied`.
3. **MUST** search only published entries — every state but `removed` and `departed` — and **MUST** return each with its own `state` and `acknowledged`, where `acknowledged` is `true` exactly when the state is `acknowledged` (VTI-VTN-013).
4. **MUST** apply every filter given: `kind` and `state` match exactly; `membersOnly` keeps only results whose `acknowledged` is `true`; and every whitespace-separated term of `q` **MUST** match, case-insensitively, as a substring of the entry's DID, label, description or one of its tags.
5. **SHOULD** order results members first, then by label (or DID where there is none), case-insensitively.
6. **MUST NOT** keep `membership` past admission, nor use it for anything but admission.

## Definitions

- **Directory visibility** — the network's setting for who may read its published set: `public`, `members` or `admins`. Its profile states it, so a reader knows before asking whether to sign and whether to send `membership`.
- **Published set** — every entry not `removed` or `departed`. It includes `listed` entries, which are not members, and says so of each.
- **`q`** — free text; every whitespace-separated term must match.
- **`kind`**, **`state`** — exact filters on the entry's kind ([`EntryKind`](../../../_shared/0.1/entry.schema.json)) and lifecycle state.
- **`membersOnly`** — when `true`, only members (`acknowledged: true`). Absent means `false`.
- **`membership`** — a person's membership credential (VMC) from a community that is a member of this network, holder-bound to the signer. It is what admits a person to a `members` directory; it is ignored where visibility is `public`.
- **`entries`** — the matching results ([`DirectoryEntry`](../../../_shared/0.1/directory.schema.json)), each carrying the ids of its listing and grant so a reader can fetch and verify them.

This task is the read-many half of a split pair; [`vtn/entries/show`](../../../entries/show/0.1/spec.md) reads one entry by DID and answers `notFound` for an absent one.

## Request

A reader (`issuer`) sends its query to the network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An anonymous search of a public directory

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000501",
  "type": "https://trusttasks.org/spec/vtn/directory/search/0.1#request",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "q": "hospital",
    "kind": "community",
    "membersOnly": true
  }
}
```

### A person searches a members-only directory

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000503",
  "type": "https://trusttasks.org/spec/vtn/directory/search/0.1#request",
  "issuer": "did:example:alice",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:05:00Z",
  "payload": {
    "q": "cardiology",
    "membership": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0031" }
  }
}
```

The membership credential is abbreviated to its `id`; the person sends it in full, and signs the request.

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One member found

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000502",
  "type": "https://trusttasks.org/spec/vtn/directory/search/0.1#response",
  "issuer": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000501",
  "payload": {
    "entries": [
      {
        "did": "did:example:community",
        "kind": "community",
        "state": "acknowledged",
        "acknowledged": true,
        "label": "St Vincent's Hospital",
        "description": "A teaching hospital.",
        "url": "https://stvincents.example",
        "tags": ["hospital", "cardiology"],
        "listingId": "urn:uuid:00000000-0000-4000-8000-0000000c0011",
        "grantId": "urn:uuid:00000000-0000-4000-8000-0000000c0012"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries organisations' public identifiers, the names and descriptions the network's administrators gave them, and credential ids. The request carries a query and, for a `members` directory, a person's membership credential. A reader **MUST NOT** put personal data in `q`: it is a search over organisations.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it; a directory nobody can name the same way twice could not be relied on. The reader declares `identifierScope: any`: an anonymous reader of a public directory sends no identifier at all, and a person who signs may use whatever identifier their community's credential is bound to. That credential does link the person to their community for the network; this is the minimum a `members` directory needs, and the network keeps no record of it (network item 6). The results are no more correlatable than the published set they are drawn from.

### Retention

Nothing in the request is retained. A network **MAY** log that a search happened, as for any request, but **MUST NOT** log `membership`.

### Consent/purpose

The published entries are published with each party's knowledge: a listed party is told it is listed and every result says whether it agreed (VTI-VTN-010, VTI-VTN-013). The person's membership credential is used solely to decide whether they may read, and for nothing else.
