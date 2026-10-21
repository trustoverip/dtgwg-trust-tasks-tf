---
slug: persona/contact/put
version: "1.1"
title: Persona Contact — Put
summary: A holder's agent files what a peer disclosed as a new revision, never overwriting — and keeps the holder's own star, petname, lists and directory link on the contact, which never leave the agent.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords: [persona, contact, revision, phishing-resistance, petname, star, favourite, address-book]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Application
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A contact record is evidence of what a counterparty asserted and when, and its annotations are the holder's private opinion of that counterparty. Attribution must survive the transport so a later dispute about what was filed has an answer that names the key that filed it, and so a compromised channel cannot rewrite the name the holder trusts most — the one they believe they wrote.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Revisions are ordered, and two filings applied out of order would make an older disclosure the current one. Annotations are replaced wholesale, so a replayed earlier request would silently put back a petname or a list the holder has since changed.
sideEffects:
  level: mutating
  rationale: "A filing appends a revision and never overwrites. An annotating request replaces or removes the holder's own marks on one contact in the agent's store, reversibly; it appends no revision and reaches no other party."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: The request carries another party's personal data — the claims they disclosed — and the holder's own words and judgements about that party, into the holder's store. The response returns identifiers, a revision number, change flags and the claim TYPES that changed; it echoes no value and no annotation.
retention:
  class: durable
  rationale: A contact and its annotations are worth having only if they are there next time. They persist in the agent's store until the holder deletes the contact, and go with it.
errorCodes:
  - code: persona/contact/put:documentTooLarge
    meaning: The document exceeds the maintainer's stated per-record cap. The cap and the actual size are both returned, so a producer can tell how far over it is rather than guessing.
    retryable: false
  - code: persona/contact/put:notFound
    meaning: Annotating form only — no contact exists at `contactId` in this context. Answered the same whether the identifier was never assigned or belongs to another context.
    retryable: false
  - code: persona/contact/put:listUnknown
    meaning: An identifier in `lists` names no list in this context. Nothing is changed. A producer creates the list with persona/contact/lists/put first.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [listIds]
      properties:
        listIds:
          type: array
          minItems: 1
          maxItems: 64
          items: { type: string, pattern: "^[0-9A-HJKMNP-TV-Z]{26}$" }
related:
  - persona/contact/get
  - persona/contact/list
  - persona/contact/delete
  - persona/contact/lists/put
  - persona/contact/lists/list
  - persona/disclosure/present
  - rooms/keys/label
---

## Abstract

**Persona Contact — Put** files what a peer disclosed, and keeps what the holder
thinks of them.

Two properties carry over from 1.0 unchanged.

**Revisioned, never overwritten.** When a peer re-discloses, the previous
document is kept and the new one becomes current. An address book that silently
replaces a payment address is a phishing surface; one that reports *"this changed
four minutes ago, here is what it was"* is a defence. The `changedClaims` member
is what turns the history from an archive into that defence.

**A contact belongs to a relationship, not to the holder at large.** The same
peer met through two personas is two contacts. Collapsing them would correlate
the holder's own personas inside their own address book.

### What 1.1 adds

The holder's own **annotations** on a contact: a **star**; a **petname** — the
holder's private name for the contact, distinct from the nickname the contact
chose for themselves; the holder's own **lists** ("Mentors", "Met at the spring
meetup"), kept by [`persona/contact/lists/*`](../../lists/put/1.0/spec.md); and a
**directory link** (`directoryRef`) from a contact to the entry in a community's
member directory it was made through, so the holder's client can show a star or
a petname beside that entry.

Annotations are **the holder's, and stay with the holder's agent.** A community
never stores who its members star or list: if it did, its operator would hold the
graph of who watches whom. Keeping them in the holder's own agent means a list can
span every community the holder belongs to while no community learns it exists.

Annotations are **contact-level, not revision-level.** They are the holder's
current view of a relationship, not evidence of what the counterparty asserted, so
changing one appends no revision. A second request form — **annotating**, which
names an existing `contactId` and no document — changes them without re-filing
anything. A filing may carry annotations too, so a contact made from an
introduction can be filed, starred and linked in one request.

A third form — **linking** — keeps annotations on a member of a community's
directory whose DID the holder has not been shown. A directory shows members'
DIDs to other members only where each member chooses to, so most entries a holder
reads carry no DID to file a contact under. Linking names the entry
(`directoryRef`) and the persona that knows it, and the agent finds the contact it
keeps for that entry and persona, or makes one: no `subjectDid`, no disclosed
document, and no revision appended. A later filing that names the member's DID is
a contact of its own; the agent does not merge the two (Conformance item 4).

`notes` keeps its 1.0 meaning, and now follows the annotation rules: absent is
unchanged, and `removeNotes` removes it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues). Version 1.0 remains in force; 1.1 is a backwards-compatible MINOR increment ([SPEC.md §5.2](/SPEC.md#52-compatibility-rules)) — every 1.0 request is a valid 1.1 filing.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/put/1.1`, populate `contextId` and
exactly one of the three forms the schema defines, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof). It **MUST NOT** put a credential, a key, a
password or any other secret in a petname, in `notes` or in `ext`.

A conforming **maintainer** (the holder's agent) **MUST**:

1. Confine the caller to its own context, and answer `notFound` for a `contactId` it holds in any other context exactly as for one it never assigned.
2. On a filing, append a revision rather than replacing one, and assign `rev` itself — a recipient counts what it received, which is the only sequence it can vouch for. The publisher's `cardVersion` is advisory and **MUST NOT** be used to order revisions.
3. On a filing, compute `changedClaims` against the immediately previous revision.
4. Key a filing on `(contextId, subjectDid, knownByPersona)`, and a linking request on `(contextId, directoryRef, knownByPersona)`, making the contact on the first request for that key with `rev` 1, an empty document and no `subjectDid`, and appending no revision on any later one. It **MUST NOT** merge contacts that differ only in `knownByPersona` — nor contacts that share a `directoryRef`, nor a linked contact with a filed one.
5. **MUST NOT** treat a claimed `credentialBacked` provenance as verified.
6. Apply every annotation change the request names together with the filing, if any, or not at all: each annotation given replaces the stored one, each `removeX` removes one, each annotation absent is unchanged, and `lists` replaces the whole set. It **MUST** refuse with `listUnknown`, changing nothing, a `lists` naming a list this context does not hold.
7. **MUST NOT** append a revision, or move `rev`, for an annotating request; and **MUST NOT** record annotations in any revision.
8. Answer `annotationsChanged: false`, and write nothing, when every annotation given already matches the stored one and every one named for removal is already absent. A petname and `notes` are compared exactly, as given; `lists` as a set.
9. **MUST NOT** case-fold, normalise, translate or otherwise rewrite a petname or a note.
10. **MUST NOT** send an annotation, or any part of one, to anyone but a caller authorised for this context: not to the contact, not to the community a `directoryRef` names, not to a verifier, and not in any document it signs, presents or seals for the holder — [`persona/disclosure/present`](../../../disclosure/present/1.0/spec.md) included.
11. **MUST NOT** write a petname, a note or a list membership to a log or an audit record. It **MAY** record that a contact's annotations changed, and when.
12. Discard a contact's annotations when it deletes the contact ([`persona/contact/delete`](../../delete/1.0/spec.md)), and **MUST NOT** retain them for a disclosure reference: a disclosure record evidences what the counterparty presented, which no annotation is.

## Authorization

**Context-scoped**, confined to the caller's own context. Filing a contact writes
into the context the relationship lives in, and annotating one writes the holder's
own marks beside it; neither needs anything from the pool.

The entitlement is being authorised for the context — the same standing every
`persona/contact/*` task asks for. Per [SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements),
the `proof` establishes *which key sent this* and *that it is unaltered*; it lets
the agent look that key up in its own record of which contexts it serves, where
the entitlement lives. A `directoryRef` confers nothing: it names an entry at a
community, it is not evidence that the contact is a member there, and no
authorization anywhere may rest on it.

**No optimistic concurrency.** Two of the holder's own clients changing the same
annotation at once: the later write wins. Annotations are replaced one by one, so
a star set on one device does not overwrite a petname written on another, and an
annotation is one person's, always visible to them — a lost edit is seen and
repaired. Whether anything more is required before writing is the consumer's
policy, which this specification does not declare.

## Definitions

- `Petname`, `ListIds`, `DirectoryRef` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../_shared/0.1/contact-annotations.schema.json).
- **Annotations** — `starred`, `petname`, `lists`, `directoryRef` and `notes`: what the holder keeps *about* a contact, as distinct from what the contact disclosed.
- **Filing form** — `subjectDid`, `knownByPersona` and `document`, without `contactId`. Appends a revision.
- **Annotating form** — `contactId` and at least one annotation change, without `subjectDid`, `knownByPersona`, `document` or `credentialRefs`. Appends nothing.
- **Linking form** — `directoryRef` and `knownByPersona`, without `contactId`, `subjectDid`, `document` or `credentialRefs`. Finds or makes the contact for that directory entry and persona; appends nothing.

## Request

An **Application** sends this to the holder's **Agent**. The payload validates
against the top-level schema in [`payload.schema.json`](payload.schema.json).

### File a contact met through a community's directory, and star it

```json
{
  "id": "urn:uuid:4f0c1f6e-2d7b-4f4a-9a59-0b6f3f0c1a01",
  "type": "https://trusttasks.org/spec/persona/contact/put/1.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:00:00Z",
  "payload": {
    "contextId": "ctx-personal",
    "subjectDid": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc",
    "knownByPersona": "did:peer:2.Ez6LSghwSE437wnDE1pt3X6hVDUQzSjsHzinpX3XFvMjRAm7y",
    "document": {
      "claims": [
        { "type": "name.display", "valueType": "string", "value": "Sam Okafor" }
      ]
    },
    "starred": true,
    "directoryRef": { "vtcDid": "did:web:climbing.example", "entryId": "e_7Hq2mZp9Lx4v" }
  }
}
```

### Star a directory member whose DID you have not been shown

The agent makes the contact for this entry and persona, or finds the one it keeps.

```json
{
  "id": "urn:uuid:4f0c1f6e-2d7b-4f4a-9a59-0b6f3f0c1a03",
  "type": "https://trusttasks.org/spec/persona/contact/put/1.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:10:00Z",
  "payload": {
    "contextId": "ctx-personal",
    "knownByPersona": "did:peer:2.Ez6LSghwSE437wnDE1pt3X6hVDUQzSjsHzinpX3XFvMjRAm7y",
    "directoryRef": { "vtcDid": "did:web:climbing.example", "entryId": "e_Q2wE9rT4yU1i" },
    "starred": true,
    "petname": "the route-setter"
  }
}
```

### Give the contact a petname and put them on a list

Only the annotations named change. No revision is appended.

```json
{
  "id": "urn:uuid:4f0c1f6e-2d7b-4f4a-9a59-0b6f3f0c1a02",
  "type": "https://trusttasks.org/spec/persona/contact/put/1.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:05:00Z",
  "payload": {
    "contextId": "ctx-personal",
    "contactId": "01JA8Z3Q4R5S6T7V8W9X0Y1Z2A",
    "petname": "Sam (belay partner)",
    "lists": ["01JA8Z5M2N3P4Q5R6S7T8V9W0X"]
  }
}
```

## Response

The **Agent** answers with the sub-schema reachable via `$anchor: "response"`: the
contact identifier, its current revision, whether this was the first revision,
whether any annotation changed, and — on a filing that updated a contact — which
claim **types** changed. A refusal is a `trust-task-error`.

`changedClaims` names types and not values, and no annotation is echoed. A
producer that needs a value reads it with [`persona/contact/get`](../../get/1.1/spec.md),
which is an explicit act.

### Annotated, nothing re-filed

```json
{
  "id": "urn:uuid:9a3c2b7e-1d4f-4e8a-b0c6-5f2e7d9a1b02",
  "type": "https://trusttasks.org/spec/persona/contact/put/1.1#response",
  "threadId": "urn:uuid:4f0c1f6e-2d7b-4f4a-9a59-0b6f3f0c1a02",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T09:05:01Z",
  "payload": {
    "contactId": "01JA8Z3Q4R5S6T7V8W9X0Y1Z2A",
    "rev": 1,
    "created": false,
    "annotationsChanged": true
  }
}
```

## Security & Privacy

### Data carried

A filing carries **another party's personal data**: the claims they chose to
disclose. Every request may carry the holder's **annotations** — a star, a
petname, list memberships, a directory link and a private note. Those are the
holder's own words and judgements about a third party, and they are the most
sensitive members here: their subject cannot see them, never consented to them,
and may be harmed by them if they leak ("people I don't trust", "owes me money").

These annotations **never leave the holder's agent.** They are not sent to the
contact, to the community a `directoryRef` names, or to any other party, and they
appear in no document the agent signs, presents or seals (Conformance item 10).
A community's directory and the holder's annotations are joined only on the
holder's own client, from two reads neither server sees the other half of.

A producer **MUST NOT** place secret material in a petname, `notes` or `ext`. The
response echoes no value and no annotation, so nothing written here reaches the
logs of the transport that carried the answer.

### Correlation

`subjectDid` is normally pairwise, so the record names *this relationship* rather
than the person across all of theirs. A maintainer **MUST NOT** index contacts by
subject DID across contexts or personas.

**Being on a holder's list, or starred, or given a petname, MUST NOT be inferable
by the person annotated or by any community.** That rules out more than sending
the annotation: a maintainer **MUST NOT** let an annotation change anything a
counterparty or a community can observe — what is presented to them, when or how
often the agent contacts them, which directory entries it reads, or the order in
which it reads them. A client joining a community's directory with the holder's
contacts **SHOULD** read the directory as it would without annotations (the same
pages, the same filters), and apply stars, petnames and lists locally; fetching
starred entries first, or one list's entries alone, would tell the community whom
the holder favours.

`directoryRef` is itself a correlator, held privately: it links a pairwise
relationship to a community-scoped directory entry. It is safe only because it
never leaves the agent. Two contacts of different personas carrying the same
`directoryRef` show, inside the holder's own store, that both personas know one
member; the maintainer **MUST NOT** merge them (Conformance item 4).

The revision history remains a correlation surface: it records when a
counterparty changed what they present. Annotations add none, because they are
not revisioned.

### Retention

The current revision and the contact's annotations are retained until the contact
is deleted. Superseded revisions are reference-counted as in 1.0. Annotations have
no history: a replaced petname or note is gone, which is what a holder expects of
their own scribbles and is the smallest record a later breach could expose. They
go with the contact, even where a disclosure reference keeps a revision
(Conformance item 12).

### Consent/purpose

The purpose of a filing is memory: the holder keeps what a counterparty told
them, so a later interaction can be checked against an earlier one. The purpose of
an annotation is the holder's own organisation of their relationships — finding
people, recognising them, grouping them — and nothing else. Using annotations to
rank, recommend or profile the people annotated, or reporting them to any party,
is outside that purpose.

Reusing a contact's data under another of the holder's personas would present the
counterparty with an identity they never disclosed to, and a maintainer **MUST
NOT** do so implicitly. What further gate a producer applies is the consumer's
policy and is deliberately not specified here.
