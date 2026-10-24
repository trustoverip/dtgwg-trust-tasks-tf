---
slug: rooms/keys/act
version: "0.7"
title: "Rooms Keys — Act"
summary: "A member's own client asks the party holding the member's keys to complete one host-bound room request, or one request to a community about the rooms it offers the member, as the member — authority narrowed to the one action — and hand it back to send."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - persona
  - attribution
  - presentation
  - signing
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The recipient signs as the member, and what it signs is chosen by the request. A request whose integrity depended on the transport would let a compromised channel choose what the member is made to say to a room's host.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task exercises the member's own authority, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would re-sign a document the member authorized once, at a time they did not choose.
sideEffects:
  level: none
  rationale: >-
    Returns a signed document and stores nothing. The recipient makes no request of the host: the caller sends what comes back, and whatever that document changes at the host is that task's side effect, declared there.
subjectPath: /document/issuer
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` is the whole task: the recipient signs as the member and presents the member's own authority, narrowed to one action. The response discloses that presentation — the member's membership credential and authority chain, which name the member on a room that discloses its subjects — and the signed document. The request carries a room document whose sealed content, where it has any, is ciphertext.
retention:
  class: transient
  rationale: >-
    The recipient completes, signs and returns one document. It keeps neither the document nor the presentation it minted for it.
maxDocumentBytes:
  request: 327680
  response: 393216
  rationale: >-
    The largest document this task completes is rooms/blobs/upload/begin/0.2 without its presentation: a manifest of up to 4096 digests, about 205 KiB, and a part plan of up to 1024, about 50 KiB, plus the envelope, so 320 KiB holds the request with its own envelope and proof. The response carries that document with the presentation inserted — up to 8 authority credentials and a membership credential, about 64 KiB — and the member's proof, so 384 KiB.
errorCodes:
  - code: rooms/keys/act:typeNotActable
    meaning: "The document's `type` is not in this task's table of host-bound room requests, so the recipient will not sign it as the member. `details.type` repeats it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [type]
      properties:
        type:
          type: string
          maxLength: 2048
  - code: rooms/keys/act:notAMember
    meaning: "The recipient holds no credential issued by the document's room to a member its caller may act as."
    retryable: false
  - code: rooms/keys/act:principalAmbiguous
    meaning: "The document's `issuer` is absent and the recipient cannot tell which member to act as: the caller may act as more than one member of the document's room, or the document's type names no room (a download chunk, a direct upload's commit or abort), for which `issuer` is the only way to name the member. Name the member in `issuer` and ask again."
    retryable: false
  - code: rooms/keys/act:issuerNotPrincipal
    meaning: "The document's `issuer` is not a member the recipient may act as for its caller: for a type that names a room, not a member of that room the caller may act as; for a type that names no room, not a member whose keys the recipient holds and the caller may act as. `details.issuer` repeats it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [issuer]
      properties:
        issuer:
          type: string
          maxLength: 2048
  - code: rooms/keys/act:actionNotHeld
    meaning: "The member's own credentials do not confer the action the document's type requires. `details.action` names it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [action]
      properties:
        action:
          type: string
          enum: [read, write, curate, admin]
  - code: rooms/keys/act:keyPackageNotHeld
    meaning: "The document is a vtc/rooms/offers/accept carrying a `keyPackage` the recipient did not mint for this room and this member with rooms/keys/key-package, or one whose private half it no longer holds (used, or expired). Mint one with rooms/keys/key-package and build the acceptance with it (Recipient item 10)."
    retryable: false
  - code: rooms/keys/act:sealedRoomMint
    meaning: "The document is a rooms/epoch/mint for a room whose MLS group the recipient holds. A sealed room's epoch advances only with a commit of its group, which a client cannot make, and a mint without one leaves every member's key holder behind the host. Renew a sealed room with rooms/owner/group/renew, which the owner's key holder performs."
    retryable: false
related:
  - rooms/keys/seal
  - rooms/keys/present
  - rooms/keys/read
  - rooms/keys/browse
  - rooms/records/put
  - rooms/records/get
  - rooms/records/list
  - rooms/records/curate
  - rooms/info
  - rooms/usage
  - rooms/epoch/chain
  - rooms/epoch/commits
  - rooms/epoch/mint
  - rooms/owner/group/add
  - rooms/owner/group/remove
  - rooms/owner/group/renew
  - rooms/blobs/get
  - rooms/blobs/chunk
  - rooms/blobs/upload/begin
  - rooms/blobs/upload/commit
  - rooms/blobs/upload/abort
  - rooms/keys/key-package
  - rooms/keys/welcome
  - vtc/rooms/offers/list
  - vtc/rooms/offers/accept
  - vtc/rooms/memberships/list
---

## Abstract

A member of a data room is a DID — the identifier the room admitted, issued its credentials to, and records as the author of what they write. The keys of that DID live with the member's **key holder**, their own agent, and never leave it. The surfaces the member acts from — a command line, a browser extension, a phone — hold a channel to the key holder and a key of their own, and none of the member's.

Until this task such a surface had two ways to reach a room's host, and both were wrong:

- ask for a presentation with [`rooms/keys/present`](../../present/0.2/spec.md), which grants the leaf to **the caller**, then sign the request with the caller's own key. The host then records the caller — a key on one laptop — as the actor and, on an `attributed` room, as the author of every record it writes. The room never admitted that identifier, and every other member reads the member's words under it.
- not reach the host at all, and ask the key holder to do the whole operation. That serves reads ([`rooms/keys/read`](../../read/0.1/spec.md), [`rooms/keys/browse`](../../browse/0.1/spec.md)), and cannot serve writes of files, whose bytes must never pass through the key holder.

This task is the third way. The client builds the room request it means to send, without a presentation or a proof, and hands it to the key holder. The key holder completes it **as the member**: it inserts the member's own authority narrowed to the one action the request needs and granted to the member itself, sets the member as `issuer`, and signs it with a key of the member's DID. The client sends the result to the host unchanged. The host sees the member — as the presenter, as the signer and, on an `attributed` room, as the author — and nothing of the client or of the key holder.

**Taking up an offer is the member's act too.** A community that owns a room offers its members admission ([`vtc/rooms/community/members/add`](../../../../vtc/rooms/community/members/add/0.1/spec.md)), and each member lists their offers and the renewals of the admissions they hold ([`vtc/rooms/offers/list`](../../../../vtc/rooms/offers/list/0.4/spec.md)), accepts one ([`vtc/rooms/offers/accept`](../../../../vtc/rooms/offers/accept/0.1/spec.md)) and reads where each admission stands ([`vtc/rooms/memberships/list`](../../../../vtc/rooms/memberships/list/0.1/spec.md)) with a document signed as the DID the offer is addressed to. That DID is the member as the community knows them, and its keys are with the same key holder, so no surface the member acts from can sign any of them. All are in the table, as requests made **to the community** rather than to a room's host, by a party the room has not yet admitted.

**`present` is for delegation; `act` is for being the member.** An agent to which the member has delegated a narrower standing is a different party, rightly recorded as itself, and keeps using `present`. A client that *is* the member — that may do anything the member may — uses this.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.5

[`vtc/rooms/offers/list/0.3`](../../../../vtc/rooms/offers/list/0.3/spec.md) also returns the renewals of the admissions a member holds — the fresh membership and authority credentials a community issues before the ones the member holds expire, as a group's room binding requires. `0.5` lists `vtc/rooms/offers/list` at `0.1` and `0.2` only, and its table is closed (Recipient item 1), so a key holder conforming to `0.5` refuses `0.3` with `typeNotActable`: a member whose offers are addressed to a DID their key holder holds could not collect a renewal, and their admission would lapse while they are still entitled to it.

`0.6`:

- **Adds `vtc/rooms/offers/list/0.3` to the table**, in the same row as `0.1` and `0.2` and on the same terms: no presentation inserted, `issuer` **REQUIRED** and the DID the community addresses its offers to, resolved without a room credential (Recipient item 2b).

Nothing else changes. The schema's `type` pattern already admits it. The table widens: a change with wire effect, so a new version per [SPEC §5.2](/SPEC.md#52-compatibility-rules) rather than an in-place edit to `0.5` — a `MINOR` increment.

### What 0.5 changed from 0.4

`0.4` lists the community's offer types — `vtc/rooms/offers/list` at `0.1` and `0.2`, `vtc/rooms/offers/accept/0.1` — and its table is closed (Recipient item 1). A key holder conforming to `0.4` therefore refuses [`vtc/rooms/memberships/list`](../../../../vtc/rooms/memberships/list/0.1/spec.md) with `typeNotActable`: a member whose offers are addressed to a DID their key holder holds could not read the community's record of where they stand in each room — the record that lets every surface they use show the same rooms in the same state, and finish an acceptance another surface began.

`0.5`:

- **Adds `vtc/rooms/memberships/list/0.1` to the table**, on the offer types' terms: no presentation inserted, `issuer` **REQUIRED** and the DID the community addresses its offers to (Producer item 2), resolved without a room credential (Recipient item 2b). With the offer types it makes *the community types*.
- **Widens the schema's `type` pattern** to admit `vtc/rooms/memberships/` beside `vtc/rooms/offers/` and `rooms/`, and nothing else of the `vtc` family.
- **Completes an acceptance carrying a KeyPackage minted with [`rooms/keys/key-package`](../../key-package/0.2/spec.md) at `0.1` or `0.2`** (Recipient item 10, Producer item 5). Under `0.2` a repeated request returns the outstanding KeyPackage, so an acceptance begun from one surface carries the same KeyPackage from another; item 10 already accepts it, because it is one the recipient minted and still holds.

The table and the `type` pattern widen: changes with wire effect, so a new version per [SPEC §5.2](/SPEC.md#52-compatibility-rules) rather than an in-place edit to `0.4` — a `MINOR` increment.

### What 0.4 changed from 0.3

[`vtc/rooms/offers/list/0.2`](../../../../vtc/rooms/offers/list/0.2/spec.md) shows each offered
room's current label and its description, so a member can tell what a room is for before
accepting. `0.3`'s table lists only `vtc/rooms/offers/list/0.1`, and its table is closed
(Recipient item 1), so a key holder conforming to `0.3` refuses a `0.2` listing with
`typeNotActable` — and a member whose offers are addressed to a DID their key holder holds, the
case the offer rows exist for, could never read a room's description.

`0.4`:

- **Adds `vtc/rooms/offers/list/0.2` to the table**, in the same row as `0.1` and on the same
  terms: no presentation inserted, `issuer` **REQUIRED** and the DID the offer is addressed to
  (Producer item 2), resolved without a room credential (Recipient item 2b). `0.2` changes only
  what the community answers; its request is the same empty payload, signed the same way.

Nothing else changes: the request and response schemas are `0.3`'s (the `type` pattern already
admits `vtc/rooms/offers/`), no error code is added, and `vtc/rooms/offers/accept` has no
version but `0.1`. Still, a type the recipient refused under `0.3` it now completes — a change to
what the closed table admits, which is wire behaviour, not an editorial or normalization change of
the kind [SPEC §5.2](/SPEC.md#52-compatibility-rules) requires to be made in place. So, as `0.3`
did when it first widened the table, it is a new version rather than an in-place edit to `0.3` — a
`MINOR` increment.

### What 0.3 changed from 0.2

`0.2` completed only requests to a room's host, from a party the room had admitted. The two
requests by which a member takes up a community's offer of admission —
[`vtc/rooms/offers/list/0.1`](../../../../vtc/rooms/offers/list/0.1/spec.md) and
[`vtc/rooms/offers/accept/0.1`](../../../../vtc/rooms/offers/accept/0.1/spec.md) — must be
signed as the DID the offer is addressed to, and a member whose keys their key holder holds
(the case this task exists for) had no way to get them signed: the `type` pattern refused
them, and the table did not list them. The member could be offered a room and never reach it.

`0.3`:

- **Adds the offer types to the table**, with no presentation inserted (*The table*). The
  schema's `type` pattern admits `vtc/rooms/offers/` beside `rooms/`, and nothing else of the
  `vtc` family.
- **Requires `issuer` for them** (Producer item 2) and resolves it without a room credential
  (Recipient item 2b): the member is not in the room yet.
- **Completes an acceptance only with a KeyPackage the recipient minted** for that room and
  that member, whose private half it still holds (Recipient item 10, Producer item 5), and
  refuses any other with the new code `keyPackageNotHeld`. See *An acceptance carries only the
  key holder's own KeyPackage* under Security & Privacy.

The table and the `type` pattern widen, a refusal is added, and a code is declared: changes with
wire effect, so a new version per [SPEC §5.2](/SPEC.md#52-compatibility-rules) rather than an
in-place edit to `0.2` — a `MINOR` increment.

### What 0.2 changed from 0.1

`0.1` declared `subjectPath: /document/payload/roomId` and resolved the member to act as
"among the members of the document's room". Three types in its own table carry no `roomId`:
`rooms/blobs/chunk/0.1` (`downloadId`, `index`) and `rooms/blobs/upload/{commit,abort}/0.2`
(`uploadId`). For them `0.1` defined neither the subject nor how the recipient finds "the
document's room", and a producer **MAY** omit `issuer` — so a conforming request could leave
the recipient nothing to resolve the member from.

`0.2`:

- **Names the member as the subject.** `subjectPath` is `/document/issuer`, not
  `/document/payload/roomId`. This task's subject is the party whose authority it exercises
  (`actsAsSubject: true`) — the member, whose signature the key holder makes. The room is the
  resource the completed document addresses at its host, not a party the caller speaks for, and
  three of the table's types do not name one. See *The subject* below.
- **Requires `issuer` for a type that names no room** (*The table*, Producer item 2), and
  defines how the recipient resolves the member for one (Recipient item 2a). It is the member
  whose request opened the transfer.
- Widens the meanings of `principalAmbiguous` and `issuerNotPrincipal` to the room-less case.
  No error code is added; the request and response schemas are unchanged.

A producer that omitted `issuer` on one of those three types conformed to `0.1` and is refused
here, and a registry-driven policy engine reads a different subject: changes with wire effect,
not editorial ones, so they are a new version per [SPEC §5.2](/SPEC.md#52-compatibility-rules)
rather than an in-place edit to `0.1` — a `MINOR` increment, as a narrowing change to a `draft`
may be.

## Changes from 0.6

`0.6`'s table is closed (Recipient item 1), so a key holder conforming to it refuses with `typeNotActable` every version of a listed type that the table does not name — and several of those types have newer versions: [`vtc/rooms/offers/list/0.4`](../../../../vtc/rooms/offers/list/0.4/spec.md) (`renewals[].renews` → `renewsAt`), and newer versions of four host-bound room requests. `0.7` adds them to the table, each in the same row as the versions it already lists and on the same terms — the same action, the same presentation rule, the same `issuer` rule:

- **`vtc/rooms/offers/list/0.4`** — community type: no presentation inserted, `issuer` **REQUIRED** and resolved by Recipient item 2b.
- **`rooms/records/put/0.3`** — `write`.
- **`rooms/records/list/0.3`, `0.4` and `0.5`** — `read`.
- **`rooms/info/0.2` and `0.3`** — `read`.
- **`rooms/epoch/chain/0.2`** — `read`.

Each of these changes only what the request or the response of its own type carries; the request this task completes is still a document of that type, signed the same way. The schema's `type` pattern already admits them, and the `issuer` description is updated to name `vtc/rooms/offers/list/0.4`. Nothing else changes: the request and response schemas are `0.6`'s, and no error code is added.

Every type `0.6` completes, `0.7` completes on the same terms, so the change is backwards-compatible: the table only widens. A type the recipient refused under `0.6` it now completes, which is wire behaviour rather than an editorial change, so it is a new version per [SPEC §5.2](/SPEC.md#52-compatibility-rules) rather than an in-place edit to `0.6` — a `MINOR` increment. `0.6` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The table

The recipient completes a document only when its `type` is one of these host-bound room requests, or one of the community types at the end of the table, and narrows the member's authority to exactly the action the table names for it.

| `type` (request) | Action | Presentation inserted |
|---|---|---|
| `rooms/records/put/0.1`, `rooms/records/put/0.2`, `rooms/records/put/0.3` | `write` | yes |
| `rooms/records/get/0.1`, `rooms/records/get/0.2` | `read` | yes |
| `rooms/records/list/0.1`, `rooms/records/list/0.2`, `rooms/records/list/0.3`, `rooms/records/list/0.4`, `rooms/records/list/0.5` | `read` | yes |
| `rooms/records/curate/0.1` | `curate` | yes |
| `rooms/info/0.1`, `rooms/info/0.2`, `rooms/info/0.3` | `read` | yes |
| `rooms/usage/0.1` | `admin` | yes |
| `rooms/epoch/chain/0.1`, `rooms/epoch/chain/0.2` | `read` | yes |
| `rooms/epoch/commits/0.1` | `read` | yes |
| `rooms/epoch/mint/0.1` | `admin` | yes — an `open` room only: see Recipient item 9 |
| `rooms/blobs/get/0.1`, `rooms/blobs/get/0.2` | `read` | yes |
| `rooms/blobs/chunk/0.1` | — | no: a download chunk's authority is the open download, which the member's `rooms/blobs/get` opened |
| `rooms/blobs/upload/begin/0.2` | `write` | yes |
| `rooms/blobs/upload/commit/0.2`, `rooms/blobs/upload/abort/0.2` | — | no: their authority is the open upload, which the member's begin opened. Signed as the member only for an upload in **direct** mode; a proxied upload's are signed by its upload key, and a host refuses the member's |
| `vtc/rooms/offers/list/0.1`, `vtc/rooms/offers/list/0.2`, `vtc/rooms/offers/list/0.3`, `vtc/rooms/offers/list/0.4` | — | no: its authority is the community's own record of the offers addressed to `issuer`, which the community reads against the document's proof |
| `vtc/rooms/memberships/list/0.1` | — | no: its authority is the community's own record of the admissions of `issuer`, which the community reads against the document's proof |
| `vtc/rooms/offers/accept/0.1` | — | no: the member holds no authority in the room yet, and the offer addressed to `issuer` is what the community checks. A `keyPackage` is completed only when the recipient minted it (Recipient item 10) |

**The community types** are addressed to the community that made the offer — the room's owner,
and its host — rather than to a room's host as such, and are made by a party the room may not
yet have admitted, so there is no room credential for the recipient to check and none to insert.
`vtc/rooms/memberships/list` is among them although it reads rooms the member may already be
in: it is a question to the community about its own record, which a room credential does not
answer. For
them `issuer` is **REQUIRED**: it is the DID the community addressed its offer to, which the
community checks against the document's proof. The recipient resolves it by Recipient item 2b.

`rooms/blobs/chunk/0.1` and `rooms/blobs/upload/{commit,abort}/0.2` carry no `roomId`. For
them the member is the document's `issuer`, which the producer **MUST** set to the member whose
request opened the transfer — the member whose `rooms/blobs/get` opened the download, or whose
`rooms/blobs/upload/begin` opened the upload. The host binds the transfer to that member, so a
document completed as any other is refused there.

Deliberately absent:

- **`rooms/blobs/upload/chunk`.** A chunk document carries a file's bytes, which never pass through the key holder: in proxied mode it is signed by the upload's own key, which [`rooms/blobs/upload/begin/0.2`](../../../blobs/upload/begin/0.2/spec.md) names, and in direct mode there is none, because the bytes go straight to storage. `rooms/blobs/upload/{begin,commit,abort}/0.1`, which bind every chunk to the member, are absent for the same reason.
- **`rooms/epoch/prune`, `rooms/owner/transfer`, `rooms/owner/claim` and `rooms/create`.** Pruning destroys a room's ability to read its past, and the other three change who owns a room. Each has, or warrants, a task of its own that states its consequences, rather than riding a signature asked for one operation at a time.

### Producer

A conforming **producer** (the member's own client):

1. **MUST** use this task only when it acts **as** the member. A party acting under a chain the member attenuated to it is an agent, and **MUST** use [`rooms/keys/present`](../../present/0.2/spec.md) and sign as itself.
2. **MUST** build `document` as the room request it will send — `id`, `type`, `recipient` (the host, or for the community types the community that made the offer), `issuedAt` and `payload` — with no `proof`, and with no `presentation` in `payload`. It **MUST** set `issuer` to name the member it acts as when it may act as more than one member of the room; for a type that names no room (*The table*), where it **MUST** be the member whose request opened the transfer; and for the community types, where it **MUST** be the DID the community addresses its offers to. Otherwise it **MAY** set it. It **MUST NOT** put its own identifier, or the key holder's, in any member of `document`.
3. **MUST** send the returned document to its `recipient` unaltered, and **MUST NOT** send it to anyone else.
4. **MUST NOT** ask to complete a `rooms/epoch/mint` for an `attributed` or `private` room. Its epoch advances only with a commit of its group, which the producer cannot make; the owner renews it with [`rooms/owner/group/renew`](../../../owner/group/renew/0.1/spec.md).
5. **MUST**, in a `vtc/rooms/offers/accept/0.1` for an `attributed` or `private` room, carry as `keyPackage` the KeyPackage the same recipient gave it for the member and the room with [`rooms/keys/key-package`](../../key-package/0.2/spec.md), against the offer's invitation, unaltered. It **SHOULD** obtain it with `rooms/keys/key-package` at `0.2` or later, so that an acceptance begun from another surface is finished with the KeyPackage that surface sent. It **MUST NOT** carry one the producer made itself or obtained anywhere else.

### Recipient

A conforming **recipient** (the key holder):

1. **MUST** refuse with `typeNotActable` a document whose `type` is not in the table.
2. **MUST** resolve the member to act as, for a type that names a room (`payload.roomId`) other than the community types: among the members of the document's room whose keys it holds, those its caller may act as. Where `issuer` is set it **MUST** be one of them, or the recipient **MUST** refuse with `issuerNotPrincipal`; where it is absent there **MUST** be exactly one, or the recipient **MUST** refuse with `principalAmbiguous` (more than one) or `notAMember` (none).

   a. For a type that names no room, the recipient **MUST** act as `issuer` if it is a member it holds keys for and the caller may act as, and **MUST** refuse with `issuerNotPrincipal` otherwise; where `issuer` is absent it **MUST** refuse with `principalAmbiguous`. It **MUST NOT** pick a member on the caller's behalf — not the only member it holds keys for, not one it infers from the transfer identifier — because the transfer belongs to the member who opened it, which only the producer knows, and a document signed as the wrong member reaches the host as that member's act.

   b. For the community types (`vtc/rooms/offers/list/0.1`, `0.2`, `0.3` and `0.4`, `vtc/rooms/offers/accept/0.1`, `vtc/rooms/memberships/list/0.1`), the recipient **MUST** act as `issuer` if it holds a key that `issuer`'s DID document lists for `authentication` and the caller may act as `issuer`, and **MUST** refuse with `issuerNotPrincipal` otherwise; where `issuer` is absent it **MUST** refuse with `principalAmbiguous`. It does not ask for a credential from the document's room — an offer is made to a party the room has not admitted, and whether `issuer` is the party the offer names is the community's check, against the proof — and, as in item 2a, it **MUST NOT** pick the identity for the caller.
3. **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) a caller to which the member has delegated narrower standing — an agent holding a chain attenuated to itself. Acting as the member is not something a delegate may ask for.
4. **MUST**, for a type that takes a presentation, insert `payload.presentation`: the member's membership credential and the member's own authority credential narrowed to **exactly** the table's action for the type and granted to **the member** — leaf first, every link included, and on a `private` room the same-subject binding. It **MUST** refuse with `actionNotHeld` when the member's credentials do not confer that action, and **MUST NOT** insert one conferring any other. A narrowed leaf **SHOULD** live minutes to hours, never days; it is minted for one document.
5. **MUST** set `issuer` to the member, and **MUST** sign the completed document with a Data Integrity proof made with a key of the member's DID document listed for `authentication`, with `proofPurpose: authentication`, covering every member including `recipient`.
6. **MUST NOT** change any other member of `document`, and **MUST NOT** add its own identifier or its caller's to it. The member is the only party the returned document names besides its recipient.
7. **MUST NOT** send the document anywhere: it returns it. A key holder that sent it would be making an outbound request the caller cannot see the answer to, and the caller is the party that needs it.
8. **MUST** refuse with the standard `malformedRequest` a document whose `issuedAt` it would not accept as fresh, or whose `expiresAt` has passed — a document is signed to be sent now — and one whose payload already carries a `presentation`, or does not validate against its type's schema once the presentation is inserted.
9. **MUST** refuse with `sealedRoomMint` a document of type `rooms/epoch/mint` for a room it holds group state for — for any member it serves, not only the member item 2 resolves — and **MUST** do so before resolving the member. On a sealed room the room's epoch is the MLS group's epoch plus one, and the document carries no commit, because the caller holds no group key to make one: a host advances anyway ([`rooms/epoch/mint`](../../../epoch/mint/0.1/spec.md)), and then every member's key holder is an epoch behind the room, every write is refused as sealed under a superseded epoch, and the owner's next admission or removal fails `epochConflict`. A sealed room is renewed with [`rooms/owner/group/renew`](../../../owner/group/renew/0.1/spec.md). For a room it holds no group state for it cannot tell the tier, and completes a mint as the table says; Producer item 4 is what keeps a sealed room's mint from reaching it.
10. **MUST**, for a `vtc/rooms/offers/accept/0.1` whose payload carries a `keyPackage`, complete it only when that KeyPackage is one it minted itself with [`rooms/keys/key-package`](../../key-package/0.2/spec.md), at `0.1` or `0.2`, for the document's `roomId`, whose leaf credential identifies the member item 2b resolved, and whose private half it still holds — not yet used, not expired — and **MUST** refuse with `keyPackageNotHeld` otherwise. It **MUST** compare the KeyPackage by its `KeyPackageRef` ([RFC 9420 §5.2](https://www.rfc-editor.org/rfc/rfc9420#section-5.2)), never by what its leaf credential says, and **MUST NOT** complete an acceptance carrying a KeyPackage it did not mint, whatever the leaf names.

A key holder serving several members — several principals, or several personas of one — keeps one set of credentials per member per room, and uses only those of the member item 2 resolved.

## Authorization

**The entitlement is acting as the member** — being, or being the client of, the key holder's own principal. A key holder holds a member's room credentials and signing keys; this asks it to use both, once, for a document the caller wrote.

That is a wider grant than `present` makes, and deliberately so. `present` hands a delegate a presentation only it can use, for one action, and the delegate signs for itself: the host records the delegate, which is the truth. A client acting as the member must be recorded as the member, which only a signature by the member can achieve — and only the key holder can make one. So this task is refused to delegates (item 3), and is otherwise bounded by the table: one request of the table at a time, its action fixed by its type, never a general signing oracle.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. The host that receives the completed document authorizes it from the presentation, as it authorizes any room request.

### The subject

The subject of this task — the value at `subjectPath`, `/document/issuer` — is the **member**:
the party whose own authority is exercised and whose key signs. That is the question a
delegated-execution policy point asks of a subject ("does the executing agent speak for this
subject?"), and it is exactly what item 2 decides. The room is the resource the completed
document addresses at its host; the caller never speaks *for* a room, and three of the table's
types do not name one.

Where `issuer` is absent — permitted only for a type that names a room, when the caller may act
as exactly one of its members — `subjectPath` does not resolve in the request. The subject is
then the member item 2 resolves, which becomes the completed document's `issuer` and the
response's `principal`. A recipient that evaluates a policy or records an audit entry against
the subject **MUST** do so against that resolved member, and **MUST NOT** treat an unresolved
`subjectPath` as a request with no subject.

## Definitions

**Member** — the DID a room admitted: the subject of the membership and authority credentials it issued. Often a persona the key holder holds for a person in one of their contexts, distinct from every key the person's devices hold.

**`document`** — the room request to complete: a Trust Task document of a type in the table, unsigned, with no presentation.

**`document`** (response) — the same request, completed: `issuer` the member, `payload.presentation` inserted where the type takes one, and the member's `proof`.

**`principal`** (response) — the member the recipient signed as, which is the completed document's `issuer`.

**`expiresAt`** (response) — when the inserted presentation's leaf lapses. Absent when no presentation was inserted.

## Request

A **Member**'s client sends this to the member's **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Writing a sealed record, as the member

The record was sealed with [`rooms/keys/seal/0.2`](../../seal/0.2/spec.md) under `signAsPrincipal`, so its body is already the member's. This completes the request that stores it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/act/0.7#request",
  "issuer": "did:example:client",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "document": {
      "id": "urn:uuid:00000000-0000-4000-8000-0000000000a1",
      "type": "https://trusttasks.org/spec/rooms/records/put/0.2#request",
      "recipient": "did:webvh:example.com:northwind-community",
      "issuedAt": "2026-01-01T00:00:00Z",
      "payload": {
        "roomId": "did:webvh:example.com:rooms:northwind",
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "expectedVersion": 0,
        "sealed": {
          "ciphertext": "3QhV1sVvR0m5xAqZ7Aw2mQnR4vBk8QLp3wXc9TgYw",
          "nonce": "b0Zt8Qm2Yq1sVvR0",
          "epoch": 4
        }
      }
    }
  }
}
```

### Committing a direct upload, as the member who began it

`rooms/blobs/upload/commit/0.2` names no room, so `issuer` is required: the member whose
`rooms/blobs/upload/begin` opened the upload. It is also this task's subject.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/rooms/keys/act/0.7#request",
  "issuer": "did:example:client",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "document": {
      "id": "urn:uuid:00000000-0000-4000-8000-0000000000a2",
      "type": "https://trusttasks.org/spec/rooms/blobs/upload/commit/0.2#request",
      "issuer": "did:webvh:example.com:people:alice",
      "recipient": "did:webvh:example.com:northwind-community",
      "issuedAt": "2026-01-01T00:05:00Z",
      "payload": {
        "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a"
      }
    }
  }
}
```

### Accepting a community's offer of a private room, as the member it is addressed to

The KeyPackage is the one this key holder minted for the member and the room with
`rooms/keys/key-package` (Producer item 5). No presentation is inserted; `issuer` is required.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/rooms/keys/act/0.7#request",
  "issuer": "did:example:client",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:10:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fc",
  "payload": {
    "document": {
      "id": "urn:uuid:00000000-0000-4000-8000-0000000000a3",
      "type": "https://trusttasks.org/spec/vtc/rooms/offers/accept/0.1#request",
      "issuer": "did:webvh:example.com:people:alice",
      "recipient": "did:webvh:example.com:northwind-community",
      "issuedAt": "2026-01-01T00:10:00Z",
      "payload": {
        "roomId": "did:webvh:example.com:rooms:board",
        "keyPackage": "AAEAAgADAAQ"
      }
    }
  }
}
```

## Response

The **KeyHolder** returns the completed document, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Completed as the member

The presentation and proof are shortened for the example. Nothing in the completed document names the client.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/act/0.7#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:client",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "principal": "did:webvh:example.com:people:alice",
    "expiresAt": "2026-01-01T01:00:01Z",
    "document": {
      "id": "urn:uuid:00000000-0000-4000-8000-0000000000a1",
      "type": "https://trusttasks.org/spec/rooms/records/put/0.2#request",
      "issuer": "did:webvh:example.com:people:alice",
      "recipient": "did:webvh:example.com:northwind-community",
      "issuedAt": "2026-01-01T00:00:00Z",
      "payload": {
        "roomId": "did:webvh:example.com:rooms:northwind",
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "expectedVersion": 0,
        "sealed": {
          "ciphertext": "3QhV1sVvR0m5xAqZ7Aw2mQnR4vBk8QLp3wXc9TgYw",
          "nonce": "b0Zt8Qm2Yq1sVvR0",
          "epoch": 4
        },
        "presentation": {
          "membership": "{\"type\":[\"VerifiableCredential\",\"MembershipCredential\"]}",
          "authority": [
            "{\"type\":[\"VerifiableCredential\",\"AuthorityCredential\"],\"credentialSubject\":{\"id\":\"did:webvh:example.com:people:alice\"}}",
            "{\"type\":[\"VerifiableCredential\",\"AuthorityCredential\"]}"
          ]
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:example.com:people:alice#key-1",
        "proofPurpose": "authentication",
        "created": "2026-01-01T00:00:01Z",
        "proofValue": "z5vgFc8Q"
      }
    }
  }
}
```

### Refused: an agent asked to be the member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "code": "permissionDenied",
    "message": "A delegate presents under its own identifier: use rooms/keys/present.",
    "retryable": false
  }
}
```

### Refused: a download chunk that does not name its member

`rooms/blobs/chunk/0.1` names no room, and the request left `issuer` out. The recipient does not
guess, even when it holds keys for only one member (Recipient item 2a).

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:client",
  "issuedAt": "2026-01-01T00:06:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {
    "code": "rooms/keys/act:principalAmbiguous",
    "message": "rooms/blobs/chunk/0.1 names no room: set issuer to the member whose rooms/blobs/get opened the download.",
    "retryable": false
  }
}
```

### Refused: an acceptance carrying a KeyPackage the key holder did not mint

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000007",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:client",
  "issuedAt": "2026-01-01T00:10:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fc",
  "payload": {
    "code": "rooms/keys/act:keyPackageNotHeld",
    "message": "This KeyPackage was not minted here for this room and member: mint one with rooms/keys/key-package.",
    "retryable": false
  }
}
```

## Security & Privacy

**The member is the only identity the host sees.** The client's key authenticates the client to its key holder and goes no further; the key holder's identifier never appears at all. On an `attributed` room that is what makes the recorded author the member the room admitted, and on every tier it keeps a host from learning which device, or which agent, serves which member.

**Not a signing oracle.** The table fixes which documents are signed and what authority each carries. A caller cannot ask for a presentation wider than its document's type needs, cannot have a non-room document signed as the member, and cannot route a signature to a party the document does not name, because the proof covers `recipient`.

**What the caller can do with the result is what the member could.** The completed document is audience-bound to its host and placed in time by `issuedAt`; the host's duplicate-execution protection keyed on its `id` stops a second use. A caller that may act as the member could have asked for the same document again, so holding one confers nothing new.

**An acceptance carries only the key holder's own KeyPackage** (Recipient item 10). The KeyPackage in an acceptance becomes the member's leaf in the room's group. One the key holder did not mint has init and signature keys someone else holds — typically the client asking — and once the community's VTA adds it, that party opens every record sealed to the group from then on and signs group messages as the member: outside the key holder, and beyond any later withdrawal of the client's standing with it, until the owner removes the member. A KeyPackage's leaf credential names the member but is signed by the leaf's own key, not by the member's DID, so the community, which checks only that the leaf names the signer, cannot tell a foreign KeyPackage from the member's. Only the key holder, which minted its own, can — and the signature it is asked for is the moment to check. This is the one way a community type could carry key material past the key holder, and item 10 closes it; the two lists carry none.

**Why the community types are here, and not in a task of their own.** The entitlement is the one this task already grants — acting as the member — with the same refusal of delegates (item 3), the same closed table and the same rule that the producer names the member and the recipient never guesses. A separate task would restate all of it for four rows and give a client two signing-as-the-member surfaces to audit instead of one. A task in which the key holder accepted the offer itself would have it call the community and route the answer, which item 7 rules out for the reason it gives: the caller is the party that needs the answer — here the Welcome, which it then hands back with [`rooms/keys/welcome`](../../welcome/0.1/spec.md).

### Data carried

A room request in; the same request completed out. The request is the member's own — a record's ciphertext, a manifest, a key — and the response adds the member's presentation and signature. No private key material crosses in either direction; an offer acceptance carries a public KeyPackage the recipient itself minted.

### Correlation

The key holder learns which room operations its member's clients perform and when, a picture of the member's own activity it largely holds already. The host learns the member, as it would from any presentation on a tier that discloses one; it learns nothing of the client or of the key holder.

### Retention

The key holder retains neither the document nor the presentation it minted for it. An implementation **MAY** record that it signed a document of a given type for a given room, for its member's own audit, and **SHOULD NOT** record the document's payload.

### Consent/purpose

The member's client acts as the member; the key holder serves the member. The completed document does what the member, through their own client, asked of the room's host, under the authority the room issued to the member.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
