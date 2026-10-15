---
slug: rooms/keys/act
version: "0.1"
title: "Rooms Keys — Act"
summary: "A member's own client asks the party holding the member's keys to complete one host-bound room request as the member — the member's own authority narrowed to the one action, the member as issuer, the member's signature — and hand it back to send."
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
subjectPath: /document/payload/roomId
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
    meaning: "The caller may act as more than one member of this room, and the document's `issuer` does not say which. Name the member in `issuer` and ask again."
    retryable: false
  - code: rooms/keys/act:issuerNotPrincipal
    meaning: "The document's `issuer` is not a member of this room the caller may act as. `details.issuer` repeats it."
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
---

## Abstract

A member of a data room is a DID — the identifier the room admitted, issued its credentials to, and records as the author of what they write. The keys of that DID live with the member's **key holder**, their own agent, and never leave it. The surfaces the member acts from — a command line, a browser extension, a phone — hold a channel to the key holder and a key of their own, and none of the member's.

Until this task such a surface had two ways to reach a room's host, and both were wrong:

- ask for a presentation with [`rooms/keys/present`](../../present/0.2/spec.md), which grants the leaf to **the caller**, then sign the request with the caller's own key. The host then records the caller — a key on one laptop — as the actor and, on an `attributed` room, as the author of every record it writes. The room never admitted that identifier, and every other member reads the member's words under it.
- not reach the host at all, and ask the key holder to do the whole operation. That serves reads ([`rooms/keys/read`](../../read/0.1/spec.md), [`rooms/keys/browse`](../../browse/0.1/spec.md)), and cannot serve writes of files, whose bytes must never pass through the key holder.

This task is the third way. The client builds the room request it means to send, without a presentation or a proof, and hands it to the key holder. The key holder completes it **as the member**: it inserts the member's own authority narrowed to the one action the request needs and granted to the member itself, sets the member as `issuer`, and signs it with a key of the member's DID. The client sends the result to the host unchanged. The host sees the member — as the presenter, as the signer and, on an `attributed` room, as the author — and nothing of the client or of the key holder.

**`present` is for delegation; `act` is for being the member.** An agent to which the member has delegated a narrower standing is a different party, rightly recorded as itself, and keeps using `present`. A client that *is* the member — that may do anything the member may — uses this.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The table

The recipient completes a document only when its `type` is one of these host-bound room requests, and narrows the member's authority to exactly the action the table names for it.

| `type` (request) | Action | Presentation inserted |
|---|---|---|
| `rooms/records/put/0.1`, `rooms/records/put/0.2` | `write` | yes |
| `rooms/records/get/0.1`, `rooms/records/get/0.2` | `read` | yes |
| `rooms/records/list/0.1`, `rooms/records/list/0.2` | `read` | yes |
| `rooms/records/curate/0.1` | `curate` | yes |
| `rooms/info/0.1` | `read` | yes |
| `rooms/usage/0.1` | `admin` | yes |
| `rooms/epoch/chain/0.1` | `read` | yes |
| `rooms/epoch/commits/0.1` | `read` | yes |
| `rooms/epoch/mint/0.1` | `admin` | yes — an `open` room only: see Recipient item 9 |
| `rooms/blobs/get/0.1`, `rooms/blobs/get/0.2` | `read` | yes |
| `rooms/blobs/chunk/0.1` | — | no: a download chunk's authority is the open download, which the member's `rooms/blobs/get` opened |
| `rooms/blobs/upload/begin/0.2` | `write` | yes |
| `rooms/blobs/upload/commit/0.2`, `rooms/blobs/upload/abort/0.2` | — | no: their authority is the open upload, which the member's begin opened. Signed as the member only for an upload in **direct** mode; a proxied upload's are signed by its upload key, and a host refuses the member's |

Deliberately absent:

- **`rooms/blobs/upload/chunk`.** A chunk document carries a file's bytes, which never pass through the key holder: in proxied mode it is signed by the upload's own key, which [`rooms/blobs/upload/begin/0.2`](../../../blobs/upload/begin/0.2/spec.md) names, and in direct mode there is none, because the bytes go straight to storage. `rooms/blobs/upload/{begin,commit,abort}/0.1`, which bind every chunk to the member, are absent for the same reason.
- **`rooms/epoch/prune`, `rooms/owner/transfer`, `rooms/owner/claim` and `rooms/create`.** Pruning destroys a room's ability to read its past, and the other three change who owns a room. Each has, or warrants, a task of its own that states its consequences, rather than riding a signature asked for one operation at a time.

### Producer

A conforming **producer** (the member's own client):

1. **MUST** use this task only when it acts **as** the member. A party acting under a chain the member attenuated to it is an agent, and **MUST** use [`rooms/keys/present`](../../present/0.2/spec.md) and sign as itself.
2. **MUST** build `document` as the room request it will send — `id`, `type`, `recipient` (the host), `issuedAt` and `payload` — with no `proof`, and with no `presentation` in `payload`. It **MAY** set `issuer` to name the member it acts as, and **MUST** set it when it may act as more than one member of the room. It **MUST NOT** put its own identifier, or the key holder's, in any member of `document`.
3. **MUST** send the returned document to its `recipient` unaltered, and **MUST NOT** send it to anyone else.
4. **MUST NOT** ask to complete a `rooms/epoch/mint` for an `attributed` or `private` room. Its epoch advances only with a commit of its group, which the producer cannot make; the owner renews it with [`rooms/owner/group/renew`](../../../owner/group/renew/0.1/spec.md).

### Recipient

A conforming **recipient** (the key holder):

1. **MUST** refuse with `typeNotActable` a document whose `type` is not in the table.
2. **MUST** resolve the member to act as: among the members of the document's room whose keys it holds, those its caller may act as. Where `issuer` is set it **MUST** be one of them, or the recipient **MUST** refuse with `issuerNotPrincipal`; where it is absent there **MUST** be exactly one, or the recipient **MUST** refuse with `principalAmbiguous` (more than one) or `notAMember` (none).
3. **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) a caller to which the member has delegated narrower standing — an agent holding a chain attenuated to itself. Acting as the member is not something a delegate may ask for.
4. **MUST**, for a type that takes a presentation, insert `payload.presentation`: the member's membership credential and the member's own authority credential narrowed to **exactly** the table's action for the type and granted to **the member** — leaf first, every link included, and on a `private` room the same-subject binding. It **MUST** refuse with `actionNotHeld` when the member's credentials do not confer that action, and **MUST NOT** insert one conferring any other. A narrowed leaf **SHOULD** live minutes to hours, never days; it is minted for one document.
5. **MUST** set `issuer` to the member, and **MUST** sign the completed document with a Data Integrity proof made with a key of the member's DID document listed for `authentication`, with `proofPurpose: authentication`, covering every member including `recipient`.
6. **MUST NOT** change any other member of `document`, and **MUST NOT** add its own identifier or its caller's to it. The member is the only party the returned document names besides its recipient.
7. **MUST NOT** send the document anywhere: it returns it. A key holder that sent it would be making an outbound request the caller cannot see the answer to, and the caller is the party that needs it.
8. **MUST** refuse with the standard `malformedRequest` a document whose `issuedAt` it would not accept as fresh, or whose `expiresAt` has passed — a document is signed to be sent now — and one whose payload already carries a `presentation`, or does not validate against its type's schema once the presentation is inserted.
9. **MUST** refuse with `sealedRoomMint` a document of type `rooms/epoch/mint` for a room it holds group state for — for any member it serves, not only the member item 2 resolves — and **MUST** do so before resolving the member. On a sealed room the room's epoch is the MLS group's epoch plus one, and the document carries no commit, because the caller holds no group key to make one: a host advances anyway ([`rooms/epoch/mint`](../../../epoch/mint/0.1/spec.md)), and then every member's key holder is an epoch behind the room, every write is refused as sealed under a superseded epoch, and the owner's next admission or removal fails `epochConflict`. A sealed room is renewed with [`rooms/owner/group/renew`](../../../owner/group/renew/0.1/spec.md). For a room it holds no group state for it cannot tell the tier, and completes a mint as the table says; Producer item 4 is what keeps a sealed room's mint from reaching it.

A key holder serving several members — several principals, or several personas of one — keeps one set of credentials per member per room, and uses only those of the member item 2 resolved.

## Authorization

**The entitlement is acting as the member** — being, or being the client of, the key holder's own principal. A key holder holds a member's room credentials and signing keys; this asks it to use both, once, for a document the caller wrote.

That is a wider grant than `present` makes, and deliberately so. `present` hands a delegate a presentation only it can use, for one action, and the delegate signs for itself: the host records the delegate, which is the truth. A client acting as the member must be recorded as the member, which only a signature by the member can achieve — and only the key holder can make one. So this task is refused to delegates (item 3), and is otherwise bounded by the table: one host-bound room request at a time, its action fixed by its type, never a general signing oracle.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. The host that receives the completed document authorizes it from the presentation, as it authorizes any room request.

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
  "type": "https://trusttasks.org/spec/rooms/keys/act/0.1#request",
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

## Response

The **KeyHolder** returns the completed document, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Completed as the member

The presentation and proof are shortened for the example. Nothing in the completed document names the client.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/act/0.1#response",
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

## Security & Privacy

**The member is the only identity the host sees.** The client's key authenticates the client to its key holder and goes no further; the key holder's identifier never appears at all. On an `attributed` room that is what makes the recorded author the member the room admitted, and on every tier it keeps a host from learning which device, or which agent, serves which member.

**Not a signing oracle.** The table fixes which documents are signed and what authority each carries. A caller cannot ask for a presentation wider than its document's type needs, cannot have a non-room document signed as the member, and cannot route a signature to a party the document does not name, because the proof covers `recipient`.

**What the caller can do with the result is what the member could.** The completed document is audience-bound to its host and placed in time by `issuedAt`; the host's duplicate-execution protection keyed on its `id` stops a second use. A caller that may act as the member could have asked for the same document again, so holding one confers nothing new.

### Data carried

A room request in; the same request completed out. The request is the member's own — a record's ciphertext, a manifest, a key — and the response adds the member's presentation and signature. No key material crosses in either direction.

### Correlation

The key holder learns which room operations its member's clients perform and when, a picture of the member's own activity it largely holds already. The host learns the member, as it would from any presentation on a tier that discloses one; it learns nothing of the client or of the key holder.

### Retention

The key holder retains neither the document nor the presentation it minted for it. An implementation **MAY** record that it signed a document of a given type for a given room, for its member's own audit, and **SHOULD NOT** record the document's payload.

### Consent/purpose

The member's client acts as the member; the key holder serves the member. The completed document does what the member, through their own client, asked of the room's host, under the authority the room issued to the member.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
