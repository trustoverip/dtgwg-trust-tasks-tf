---
slug: auth/oob/grant
version: "0.3"
title: "Auth — OOB Grant"
summary: "A member's signed decision on one out-of-band request: let this starter key act as me at this origin until sessionEndsAt, having seen this context, and sign these task families as me for that session — or decline. Signed for assertionMethod, carried only inside auth/oob/respond."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, grant, session-key, user-verification, vta, wallet, delegation, member-portal]
parties:
  - role: Approving identity
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the grant. It is an attestation in the sense of SPEC §4.7.3: the identified DID's assertionMethod key attests that the named starter key may act as it, and — where the grant carries a signing scope — that the key may sign documents of the named task families in its name. The service relies on that attestation to create a session and to accept the key's documents, and a third party relies on it to attribute them. An unsigned grant authorizes nothing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A grant arms a session in the member's name, and an issue time bounds how long a captured grant could be carried to the service; it is accepted only inside the request's decision window.
sideEffects:
  level: none
  rationale: >-
    The document does nothing on its own. It is evidence carried inside auth/oob/respond, and the effects — recording the decision and, at redeem, creating the session — are those tasks'. The documents a signing scope lets the starter key sign are each their own task, with their own effects.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Names a request, two throwaway keys, an origin, a digest, a time and, optionally, a list of task families. Nothing secret moves; the DID's private key stays in the VTA. The authority the grant confers is exercised through auth/oob/respond, auth/oob/redeem and the tasks the signing scope names, which declare it.
retention:
  class: durable
  rationale: >-
    The service keeps the signed grant as the audit record of who let which key act for them, where, for which tasks, and after seeing what — and, for as long as it keeps a document the starter key signed, as the evidence that attributes that document to the member. The VTA keeps its id so it never signs the same grant twice.
errorCodes: []
related:
  - auth/oob/prove
  - auth/oob/respond
  - auth/oob/redeem
  - auth/authenticate
  - auth/signing-key/enroll
  - consent/decision
---

## Abstract

The grant is the heart of the key-grant sign-in. The member does not hand the browser a session; the member signs a statement naming the browser's own key, and only the holder of that key can turn the statement into a session ([`auth/oob/redeem`](../../redeem/0.1/spec.md)). The grant binds:

- the **request** (`requestId`);
- the **starter key** that may act (`sessionKey`) and the **lock** that delivered it (`approverKey`);
- the **origin** at which it may act;
- **what the member was shown** (`contextDigest`, the digest of the signed step 2);
- **how long** (`sessionEndsAt`);
- and, since 0.2, **which task families** the starter key may sign documents for in the member's name (`signingScope`).

The wallet builds it unsigned with the identified DID as `issuer`. The member's VTA signs it, for `assertionMethod`, only when the request carries a `consent/decision` from the device's user-verification key over the digest of the unsigned grant. A decline is the same document with `decision: decline`, and needs no user verification. It travels as the `grant` member of [`auth/oob/respond/0.3`](../../respond/0.3/spec.md).

### Changes from 0.1

- **The grant may delegate signing, within a scope the member sees.** A sign-in surface that authors signed Trust Task documents — a community's member portal reading and editing its directory, say — has no key of the member's to sign with, and asking the member's wallet for a signature on every read would make the surface unusable. The starter key is already the key the member lets act as them for the session; 0.1 left unsaid whether that extends to signing documents, and for which tasks. In 0.2 the service offers a `signingScope` in step 2 ([`auth/oob/prove/0.2`](../../prove/0.2/spec.md)), the grant copies it, and the member's signature over the grant is the delegation: *this key may sign documents of these task families as me, for this session, at this service*. A grant without `signingScope` delegates no signing, exactly as 0.1.
- **What a delegated key may never sign is stated here, once**, rather than left to each task: anything that needs the member's own `assertionMethod` key, that confers or withdraws authority, that gives or withdraws a consent, that publishes the member's data beyond the service, or that vouches for an identifier in the member's name (Conformance, consumer item 6).

This is the same delegation a console signing key is under [`auth/signing-key/enroll`](../../../signing-key/enroll/0.2/spec.md) — a key that confers nothing, authorized by the identity's standing read at execution time — with three differences that make it the right shape for a sign-in surface: it needs no second enrolment, because the sign-in grant already carries the identity's own signature; it lives exactly as long as the session, where a console key is a durable device credential; and its scope is signed by the identity itself, so a third party holding a delegated document can attribute it with the grant alone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.2

- **`notAfter`** (integer UTC epoch seconds) is renamed **`sessionEndsAt`** and
  becomes an RFC 3339 date-time (`type: "string"`, `format: "date-time"`). It is
  still required and still names the latest instant the session key may act as
  the issuer. `1791655200` in 0.2 is `"2026-10-10T18:00:00Z"` here.
- Step 2, which `sessionKey`, `origin`, `signingScope` and `contextDigest` are
  taken from, is now the [`auth/oob/prove/0.3`](../../prove/0.3/spec.md)
  response, whose `decisionDeadline` is renamed `decisionClosesAt`.
  `contextDigest` is computed exactly as before, over that document as signed.

The reason is the registry-wide convention that an instant on the wire is an
RFC 3339 string whose member name ends in `At`. Nothing else in the payload
moves.

Breaking on the wire, released as a `MINOR` increment under
[§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains
published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the wallet with the member's VTA):

1. **MUST** set `issuer` to the DID that proved membership (`identifiedAs` in step 2, the [`auth/oob/prove/0.3`](../../prove/0.3/spec.md) response), `recipient` to the service DID, `sessionKey` and `origin` from step 2, `approverKey` to the lock key, and `contextDigest` to the SHA-256 multihash, in base58btc multibase (`z…`), of the [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) canonical form of the signed step 2 response document, proof included.
2. **MUST** set `signingScope` to step 2's `signingScope`, unchanged and in the same order, when step 2 carries one, and **MUST** omit it when step 2 does not. A wallet that will not delegate what step 2 offers declines; it **MUST NOT** approve with a narrower or a wider scope, because the service compares the two.
3. **MUST** sign with a key the issuer's DID document lists under `assertionMethod`, with `proofPurpose: assertionMethod`.
4. A signing VTA, for `decision: approve`, **MUST** require a `consent/decision` signed by the requesting device's user-verification key whose payload digest equals the digest of the unsigned grant; **MUST** refuse unless `issuer` is the principal of the vault entry used and `recipient` is the DID that entry targets; and **MUST NOT** sign the same grant `id` twice. Because the digest covers `signingScope`, the device's decision covers the delegation too.
5. **SHOULD** refuse to approve a grant whose `signingScope` names `auth`, `acl`, `consent`, `task-consent` or `confirm`, or a family under one of them, whatever step 2 offered.
6. **MUST NOT** send the document on its own.

A conforming **consumer** (the service, executing `auth/oob/respond` and accepting documents the starter key signs):

1. **MUST NOT** act on a grant received on its own, and **SHOULD** answer one with `unsupportedType`.
2. **MUST** verify the embedded grant as a Trust Task document in its own right ([SPEC §7.2](/SPEC.md#72-consumer-requirements)) — recipient is itself, fresh, `id` new — with its proof checked against the issuer's **`assertionMethod`** relationship.
3. **MUST** require `issuer` to equal the request's identified DID, `approverKey` the lock, `sessionKey` the starter key, `origin` the request's origin, `contextDigest` the stored digest of the signed step 2, comparing the decoded multihash rather than the multibase text, and `signingScope` to equal step 2's — both absent, or the same families in the same order — else refuse as [`auth/oob/respond/0.3`](../../respond/0.3/spec.md) states.
4. **Delegated signing.** **MAY** accept a document whose `issuer` is the starter key, with a proof that verifies as made by it for `authentication`, as issued by the grant's `issuer` — the member — only while all of these hold, and **MUST** refuse it, exactly as it would refuse a document signed by no key it knows, when any does not:
   1. the grant approved and carries a `signingScope`, and the document's Type URI names a task whose slug is a family the scope lists or lies under one, matched on whole segments;
   2. the session that `auth/oob/redeem` created from this grant is live: not past the grant's `sessionEndsAt` or the service's own session limit, not idle past the service's limit, not signed out and not revoked — a delegated document **MUST NOT** keep the session alive, renew it, or extend it;
   3. the document's `recipient` is this service;
   4. the task is not excluded by item 6.
5. **MUST** authorize a document accepted under item 4 by the member's standing **read at execution time** — never by a copy taken at sign-in — and only by the standing the session's surface carries: a service that runs separate surfaces for different audiences (a member portal and an administration console) **MUST NOT** let a key delegated by a sign-in to one reach authority the member holds only on another. Whatever the scope names, the key reaches no further than the member could reach on that surface with their own key.
6. **Exclusions.** **MUST NOT** accept a document signed by the starter key, whatever `signingScope` names:
   1. where any specification requires a proof by a key the signer's DID document lists under `assertionMethod` — in particular [`auth/step-up/approve-response`](../../../step-up/approve-response/0.5/spec.md), [`task-consent/decision`](../../../../task-consent/decision/0.1/spec.md), [`confirm/response`](../../../../confirm/response/0.1/spec.md) and this specification — nor as satisfying any requirement for a user-verified gesture or step-up;
   2. for an operation that confers, withdraws or changes authority, for the member or anyone else: a role, a capability, an access-control entry, a membership, a group membership that carries a role or a capability, a credential, a key, a delegation or a session. Asking for a credential the member receives only as an offer their own agent must accept, and ending the renewal of the member's own credential, confer and withdraw nothing in this sense: the key never receives or accepts the credential — so joining or leaving a group that carries no role or capability is not excluded here;
   3. for an operation that gives, changes or withdraws a consent, or that publishes the member's data beyond the service's own members;
   4. for an operation that asserts, in the member's name and to another party, an identifier, a key or a statement that party will rely on as the member's — for example a reply identifier handed to another member as the member's own.

   Where a task's own specification marks an operation as taking the member's own signature, that operation is excluded too. The exclusions apply per document, by what that document would do: a task some of whose operations are excluded remains available for the rest.
7. **MUST** delete the record that the starter key may sign no later than it ends the session, and **MUST** audit every document accepted under item 4 against the member, naming the starter key as the acting credential and the grant `id` as the authority.

## Authorization

The grant is a delegation by the member: it is the authority, for one session, for `sessionKey` to act as the issuer at `origin` until `sessionEndsAt`, and — where it carries `signingScope` — to sign documents of those task families as the issuer for that session. It is an assertion whose proof *is* the authorization ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15), and only for that purpose.

The delegation confers no standing of its own. A document the starter key signs is authorized by the member's standing at the service, read when it executes, so it can never reach further than the member could with their own key; ending the membership, or the session, disarms the key at once. Within that standing it is narrowed three ways — by the families the member signed, by the exclusions of consumer item 6, and by the surface the session belongs to — and it never satisfies a check that needs the member's own `assertionMethod` key. The service still checks membership at respond, at redeem, and on every delegated document.

## Definitions

- **`decision`** — `approve` or `decline`.
- **`sessionKey`** — the starter key `K_b` the grant lets act as the issuer.
- **`approverKey`** — the lock key `K_a`.
- **`origin`** — the portal origin at which the session key may act.
- **`contextDigest`** — the digest of the signed step 2 response, binding the decision to what the member was shown.
- **`sessionEndsAt`** — the latest instant the session key may act as the issuer, as an RFC 3339 date-time.
- **`signingScope`** — the task families, as slug prefixes of whole segments, that the starter key may sign documents for as the issuer, for the session this grant arms. Copied from step 2.
- **Delegated document** — a document whose `issuer` is the starter key, accepted under consumer item 4 as the member's.

## Request

The approving identity (`issuer`) addresses the service (`recipient`); the wallet carries the document inside `auth/oob/respond`. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member approves the sign-in, and lets the portal sign directory and group tasks

```json
{
  "id": "urn:uuid:8d4e5f60-7182-4c9d-8e0f-2a3b4c5d6e01",
  "type": "https://trusttasks.org/spec/auth/oob/grant/0.3",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:30Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "decision": "approve",
    "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "origin": "https://members.community.example",
    "contextDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "sessionEndsAt": "2026-10-10T18:00:00Z",
    "signingScope": ["vtc/directory", "vtc/groups"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-10-10T10:01:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### A document the portal then signs under it

Not a grant: a `vtc/directory/entries/list` request, issued and signed by the starter key, which the service accepts as Alice's for the session the grant above armed.

```json
{
  "id": "urn:uuid:1f2e3d4c-5b6a-4798-8a7b-6c5d4e3f2a01",
  "type": "https://trusttasks.org/spec/vtc/directory/entries/list/0.1",
  "issuer": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:05:00Z",
  "payload": {
    "query": "ana",
    "limit": 20
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp#z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "created": "2026-10-10T10:04:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Security & Privacy

### Data carried

A handle, two throwaway public keys, an origin, a digest, a time, a decision and, optionally, a short list of task-family names. Nothing secret and nothing about the member beyond the DID that is already the issuer.

### Correlation

The grant links the member's DID to one starter key and one origin, for one session, at the community the member chose. Every delegated document the key signs is linkable to that grant and to each other — that linkage is the delegation — and to nothing outside the session: the starter key is generated for one request and used nowhere else. The identity's identifier scope is `any`; the service has identifier scope `public` because the VTA's vault entry targets the community's published DID.

### Retention

The service keeps the signed grant as the audit record of the sign-in, and for as long as it keeps any delegated document as evidence, because the grant is what attributes that document to the member: a delegated document with its grant is verifiable by anyone who can resolve the member's DID, without the service's records. The record that the key may sign is deleted with the session. The VTA keeps the grant `id` and the user-verification decision in its own audit log, and refuses to sign that `id` again.

### Consent/purpose

The grant records one decision for one request, bound to the context the member was shown, including the signing scope offered. It **MUST NOT** be reused as evidence of consent to anything else, and in particular a delegated document is never evidence that the member made a decision consumer item 6 excludes. Whether a given deployment requires user verification for a grant is the signing VTA's policy; this specification describes the evidence, not the policy.

**What a script in the portal's origin gets.** Use of the starter key, not the key itself where it is non-extractable, for as long as the script runs and the session lasts — within the scope, the exclusions and the member's standing. It cannot leave a durable credential behind, because nothing a delegated key may sign confers a key, a session or authority, and the delegation ends with the session. That is the same bound a session cookie already had, with the difference that a signed document is a durable artefact: the service's acceptance window and replay record are what limit it.
