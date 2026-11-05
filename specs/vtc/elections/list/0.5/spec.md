---
slug: vtc/elections/list
version: "0.5"
extends: "0.4"
title: "VTC Elections — List"
summary: "A member or an administrator pages through the community's polls, surveys and elections the caller may see, with each one's schedule, state and whether the caller already holds its ticket."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - poll
  - registrar
parties:
  - role: member or administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    What the page holds depends on who is asking — which elections their roll admits, and whether they hold a ticket.
    The community can establish that from a verified proof or from a transport-authenticated sender; a proof makes the
    read attributable on every transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing; a bounded `issuedAt` still keeps a captured request from being replayed to learn, much
    later, which elections a member held a ticket for.
sideEffects:
  level: none
  rationale: "Reads the community's election records and the caller's own issuance records. Persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses, for each election the caller may see, its identifier, title, schedule, state and whether it is binding,
    and whether the caller holds its ticket. Never whether anyone else does, never anything of a ballot, and nothing at
    all — not even its identifier — of an election whose credential roll names a member party, except to that party and
    to administrators.
retention:
  class: transient
  rationale: "A view of records the community keeps; the caller refetches as elections move through their states."
errorCodes: []
related:
  - vtc/elections/show
  - vtc/elections/create
  - vtc/elections/tickets/issue
  - voting/election/show
---

## Abstract

**VTC Elections — List** pages through a community's elections — its polls, surveys and elections — as the caller may see them. A member sees the elections whose roll admits them, with whether they already hold a ticket for each; an administrator holding `vtc.elections.manage` sees every election, including one still being registered with its ballot box. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

This is the read-many half of a split pair: [`vtc/elections/show`](../../show/0.5/spec.md) reads one election whole, and answers `electionUnknown` for one the caller may not see, where a list simply leaves it out.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.4

An election whose `credential` roll names a member party — a party other than the community and its groups that is itself a member of the community — is no longer shown to any caller. In `0.4` it was, because the community holds no record of who that party's members are, and so its title and schedule were public, and its questions and options through [`vtc/elections/show/0.4`](../../show/0.4/spec.md). Here a list shows it only to a caller holding `vtc.elections.manage` and to the party `roll.memberOf` names, by that party's own membership; to every other caller it is left out entirely, without even a stub of its identifier and schedule. A member of that party reads it through [`vtc/elections/show`](../../show/0.5/spec.md), presenting their membership of the party, having learnt the election's identifier from outside this list (VTI-VOT-015). A list never carries a presentation. Conformance item 1, the Authorization section and Security & Privacy say how and why. Elections under every other roll, and under a `credential` roll naming the community or one of its groups, are listed as in `0.4`.

The payload is unchanged; `0.5` declares `extends: "0.4"`. Released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules). `0.4` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** take the caller from the verified proof or the transport-authenticated sender, and **MUST** include an election only when the caller may see it (CONVENTIONS §4): a caller holding `vtc.elections.manage` (VTI-VOT-080) sees every election; any other caller sees an election only when the election's roll rule admits them as it stands now (VTI-VOT-021) — an active member under `members` or `membersAndListed`, a DID the manifest's `roll.listed` names under `membersAndListed` or `listed`. Under `credential`, which the roll proves only when a ticket is claimed (VTI-VOT-025), the community shows the election to the callers it could admit: where `roll.memberOf` is the community's own DID, an active member; where it is one of the community's groups, an active member the community's records hold in that group; where it is a party that is itself a member of the community (a **member party**), only that party itself, by its own active membership of the community — never any other caller, because the community holds no record of who that party's members are, and a list proves nothing about its caller (VTI-VOT-015). Where `roll.memberOf` is a group the community no longer holds, the election is shown to no caller but one holding `vtc.elections.manage`, who is the one to cancel it: the deleted group has no members in the community's records, and its DID is never read as a member party's (VTI-VOT-029). An election the caller may not see is left out, never marked, and never shown as a stub of its identifier, schedule or `roll.memberOf`.
2. **MUST NOT** show a caller without `vtc.elections.manage` an election in `pendingRegistration` or `registrationRefused`: no ticket can be issued for it, and its manifest may yet be withdrawn.
3. **MUST** set `ticketHeld` from its own record of tickets issued to the caller (VTI-VOT-022), and **MUST NOT** carry, in this task or any other answer to this caller, whether any other party holds one.
4. **MUST** apply `state` as a filter, and bind `cursor` to the caller and to `state`; a cursor presented by another caller or with another filter is `malformedRequest`.
5. **MUST NOT** accept, ask for or act on a presentation of any credential in this task (VTI-VOT-015). A caller who could read an election by presenting a membership reads it through [`vtc/elections/show`](../../show/0.5/spec.md), one election at a time.
6. **MUST** return at most `limit` elections, and `nextCursor` only when more remain. The order is the community's, stable across pages; the reference order is `votingOpensAt`, most recent first.

## Authorization

The authority is **the caller's standing toward each election**: being a party the election's roll admits, or holding `vtc.elections.manage` at the community. Under a `credential` roll the standing to vote is proven only by the presentation a ticket claim carries ([`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md)), so seeing an election under one is not standing to take its ticket. Where the roll names a member party, the community's records hold no standing for the party's members, and a list carries no presentation that could prove one: a reader who presented every membership credential they hold to find the elections that concern them would show the community all of them. Such an election is therefore listed only to the party itself — whose members are the ones asked, and which could in any case issue itself a credential and present it — and to administrators. Both are read from the community's own records at execution time, per election. The proof or the transport identity says who is asking ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); it is the key the standing is looked up by, not the standing. A caller with standing toward no election is answered with an empty page, not a refusal: a list's empty page is not a statement that nothing exists.

## Definitions

- **`state`** — a [`RegistrarState`](payload.schema.json): the ballot box's [`ElectionState`](../../../../voting/_shared/0.1/voting.schema.json) once the election is registered, or the registrar's `pendingRegistration` / `registrationRefused` before that.
- **`cursor`**, **`limit`** — paging, as above.
- **`elections`** (response) — the page, each an `ElectionListItem`: the election's identifier, title, schedule (`votingOpensAt`, `votingClosesAt`), state, whether it is binding, and `ticketHeld`.
- **`ticketHeld`** — whether the community has issued this caller a ticket for the election. A ticket the community issued is not necessarily one the caller's agent unblinded or voted with; the community cannot know either.
- **`nextCursor`** (response) — the token for the next page; absent on the last page.

## Request

A member asks for the elections they can take part in. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json); every member is optional.

### Elections open for voting

```json
{
  "id": "urn:uuid:5e6f7081-92a3-4b4c-85d6-e7f8091a2b3c",
  "type": "https://trusttasks.org/spec/vtc/elections/list/0.5#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-09T10:00:00Z",
  "threadId": "urn:uuid:5e6f7081-92a3-4b4c-85d6-e7f8091a2b3c",
  "payload": {
    "state": "voting",
    "limit": 20
  }
}
```

## Response

The community answers with one page — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error`.

### One election, ticket already taken

```json
{
  "id": "urn:uuid:6f708192-a3b4-4c5d-96e7-f8091a2b3c4d",
  "type": "https://trusttasks.org/spec/vtc/elections/list/0.5#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-09T10:00:01Z",
  "threadId": "urn:uuid:5e6f7081-92a3-4b4c-85d6-e7f8091a2b3c",
  "payload": {
    "elections": [
      {
        "electionId": "board-2026",
        "title": "Board election 2026",
        "state": "voting",
        "votingOpensAt": "2026-11-08T00:00:00Z",
        "votingClosesAt": "2026-11-15T00:00:00Z",
        "binding": true,
        "ticketHeld": true
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a filter and paging. The response carries, per election, what its manifest already publishes to the parties its roll admits — identifier, title, schedule, whether it is binding — plus its state and one fact about the caller: whether the community issued them its ticket. A community **MUST NOT** put into `ext` anything about another party's ticket, and nothing here concerns a ballot, which the community never holds (CONVENTIONS §2).

### Correlation

The community is declared `identifierScope: public`: it is addressed by the DID its manifests name as `community`, which its members, its ballot box and its auditors all know it by, and which every ticket and cast is bound to (CONVENTIONS §3). A pairwise identifier for it would break that binding and tell the parties nothing they could check.

The community already knows who took which ticket — it issued them — so `ticketHeld` tells it nothing new; it tells the caller only what is theirs. An observer of the transport learns that a member asked about elections, and the size of the page. The member's request is not linkable to any cast: casts are signed by one-time keys and carry no identifier of the voter (VTI-VOT-003), and nothing in this exchange is a value a cast carries. A member who wants their reading of the list unlinkable in time to their cast **SHOULD** not list immediately before voting, for the same reason a cast is delayed after a ticket (VTI-VOT-033).

Leaving an election whose roll names a member party out of every other caller's list, rather than listing a stub of it, is a trade. A stub — the identifier, the party and the schedule, without the title or the questions — would let that party's members find the election here. But a list proves nothing about its caller, so a stub would go to anyone, and would publish that the community holds the election, when, under what identifier (which may be descriptive), and that the party is a member of the community, which that party's own listing in the community's directory may keep from the caller (VTI-DIR-040). The cost is discovery: the party's members learn of the election from outside this list — from the party, which this list does show it to and which alone holds their roster, or from a link the community or the party sends them. They had to learn of the community from somewhere in any case, since they hold no relationship with it.

### Retention

Transient. The community keeps nothing of the request beyond its ordinary request log; the records it reads are kept under [`vtc/elections/create`](../../create/0.5/spec.md) and [`vtc/elections/tickets/issue`](../../tickets/issue/0.3/spec.md).

### Consent/purpose

The read lets a party find the elections they may take part in and see whether they have taken a ticket. It **SHOULD NOT** be logged in a form that records which member viewed which election beyond what the community's ordinary request log holds.
