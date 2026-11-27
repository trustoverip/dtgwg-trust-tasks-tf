---
slug: did-management/account/register
version: "0.1"
title: "DID Management — Account Register"
summary: "A party with no account on a DID hosting service registers its own self-certifying DID there, admitted by an operator invite or a trust community's voucher, and the service creates the account and publishes the DID in one commit."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, register, self-registration, onboarding, guest, invite, voucher, webvh]
parties:
  - role: Registrant
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The registrant has no account and no session, and its DID is not resolvable until this task commits, so nothing
    but the document's own proof can show that the party asking controls the DID it registers. The consumer checks
    that proof against the document the submitted log implies; without it, anyone holding a copy of someone else's log
    could register it as their own account.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Registration creates an account and claims a path. A replayed registration would re-create an account its holder
    let expire, and the duplicate-execution record of SPEC §7.2 item 11 is bounded by this timestamp.
sideEffects:
  level: mutating
  rationale: >-
    Creates an account, claims a path and publishes the first DID log under it, and consumes the invite or voucher
    that admitted it. All of it is undone by deleting the DID, or by the account's expiry.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The request carries a public DID log and admission evidence; the response returns the account and DID record the
    service now holds, which describe the registrant's own hosting and nothing about anyone else.
retention:
  class: durable
  rationale: >-
    The account, the DID record and the single-use record of the invite or voucher outlive the exchange: they are
    what the service later authorises the holder's updates against.
maxDocumentBytes:
  request: 1310720
  rationale: A complete webvh log of up to 1 MiB, plus admission evidence, envelope and proof headroom.
uncheckedRules:
  - member: "/$defs/HostingVoucher/properties/type"
    rule: "A voucher's `type` array includes `DidHostingVoucher`."
    source: "This specification, Conformance, consumer item 5"
errorCodes:
  - code: did-management/account/register:registrationClosed
    meaning: The domain does not accept self-registration, or not by the admission kind presented. did-management/server/info says which domains do, and how.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        domain: { type: string }
        admissionsOffered:
          type: array
          items: { type: string, enum: [invite, voucher] }
  - code: did-management/account/register:admissionRefused
    meaning: The invite or voucher does not admit this registrant here. `reason` says why.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [reason]
      properties:
        reason:
          type: string
          enum: [unknownInvite, inviteExpired, inviteRedeemed, inviteRevoked, inviteScope, untrustedIssuer, voucherInvalid, voucherExpired, voucherUsed, voucherSubject, voucherAudience, voucherDomain]
  - code: did-management/account/register:capacityReached
    meaning: The domain holds the most self-registered accounts it admits. Space may free up as accounts expire.
    retryable: true
  - code: did-management/account/register:alreadyRegistered
    meaning: The registrant already holds an account on this service. Later publications use did-management/did/register as the account holder.
    retryable: false
  - code: did-management/account/register:invalidLog
    meaning: The log failed verification, or does not establish the DID the slot names (`domain` and `path`).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
  - code: did-management/account/register:issuerNotSubject
    meaning: The document's `issuer` is neither the DID the log establishes nor a `did:scid` alias that DID's current document lists, or the proof's key is not an `authentication` method of that document.
    retryable: false
  - code: did-management/account/register:pathNotAllowed
    meaning: The path breaks the domain's path rules for self-registered accounts, or the invite's path prefix.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        path: { type: string }
        reason: { type: string }
  - code: did-management/account/register:pathTaken
    meaning: The path is already held.
    retryable: false
  - code: did-management:unknownDomain
    meaning: The submitted `domain` is not a known hosting domain on this consumer. See [the category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        domain: { type: string }
        activeDomains:
          type: array
          items: { type: string }
related:
  - did-management/account/invite/issue
  - did-management/account/renew
  - did-management/account/status
  - did-management/account/expiry-notice
  - did-management/server/info
  - did-management/did/register
---

## Abstract

Every other did-management task assumes its caller already has an account on the hosting service, which some administrator created. **Account Register** is how a party that has none gets one: typically a software agent that minted its own DID before it had anywhere to publish it. The party submits the DID's complete log and the evidence that admits it, and the service creates the account and publishes the DID in one commit.

Two problems are solved separately. **Identity:** the registrant's DID does not resolve yet, so the service cannot look up the key that signed the request. It does not need to: a `did:webvh` log is self-certifying, because its SCID commits to the genesis entry, which names the DID and the keys that control it. The service verifies the log and checks the request's proof against the document the log implies, without resolving anything. **Entitlement:** a self-certifying DID costs nothing to mint, so proving who is asking says nothing about whether they should be given space. That is what the admission evidence is for: an invite an operator issued (did-management/account/invite/issue), or a voucher from a trust community the domain trusts.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the registrant):

1. **MUST** set `issuer` to the DID it registers — the `did:webvh` the log establishes, or that DID's `did:scid` alias — and sign the document with a key in that DID's `authentication` relationship, with `proofPurpose: authentication`.
2. **SHOULD** read the target domain's rules from did-management/server/info **before** minting the DID, because the path is fixed into the DID's identifier, and its SCID, when it is minted.
3. **MUST** submit the complete log, from the genesis entry.

A conforming **consumer** (the hosting service):

1. **MUST** refuse a register from a party that already holds an account with `alreadyRegistered`.
2. **MUST** refuse with `registrationClosed` unless the target domain accepts self-registration by the admission kind presented, and **SHOULD** apply per-source and service-wide rate limits before any check below, answering a limited request with `unavailable` and a `retryAfter`.
3. **MUST** check the admission evidence **before** verifying the log, so that a party without valid evidence costs it no log verification and no resolution:
   - an invite: both halves match one `pending`, unexpired invite for this domain, and the path satisfies its `pathPrefix`;
   - a voucher: its `issuer` is a community the domain trusts; `credentialSubject.hostingService` is this service's DID; `credentialSubject.id` is the identity `issuer` names (a `did:scid` without its `src`); `credentialSubject.domain`, if present, is the target domain; the current time is within `validFrom`–`validUntil`; (`issuer`, `id`) has not been accepted before. Only then its proof, against the community's DID document, for `assertionMethod` and under that relationship.

   Any failure is `admissionRefused`.
4. **MUST** verify the log in full — the SCID against the genesis entry, every entry's proof against the update keys then in force — and refuse with `invalidLog` unless the **latest** entry's DID is the one `domain` and `path` build. (A portable DID's genesis entry may name another location.)
5. **MUST** accept `issuer` only if it is that DID, or its `did:scid` alias **and** the latest document lists that alias in `alsoKnownAs`; and **MUST** verify the document's proof against the document the log implies, never one resolved over the network, with the key in its `authentication` relationship. Otherwise `issuerNotSubject` (or `proofInvalid`, where the proof itself fails). A voucher's `type` MUST include `DidHostingVoucher`.
6. **MUST** refuse with `capacityReached` when the domain already holds the most self-registered accounts it admits, and with `pathNotAllowed` or `pathTaken` as their meanings say. It **MUST NOT** replace any existing slot.
7. **MUST** create the account, publish the log and consume the invite or record the voucher in one atomic commit: a failure anywhere leaves the invite redeemable and the voucher unused. The account's `subject` is the identity `issuer` names, without any `src`.
8. **SHOULD** create the account under its most restricted DID-owner role, with quotas, and subject to its inactivity lifecycle (did-management/account/renew).

After the commit the DID resolves from the service, and the holder publishes later log entries with did-management/did/register like any other account holder.

## Authorization

The entitlement is **the admission evidence**: possession of both halves of an invite an administrator of this service issued, or a voucher from a trust community that the domain's operator chose to trust, naming this registrant and this service. The invite's authority is the issuing administrator's; the voucher's is the community's, and only as far as the operator delegated it by listing the community.

The envelope proof is **not** the authorization. Checked against the document the log implies, it establishes that the party asking controls the DID being registered — so that a registrant can register only itself, and an invite or voucher cannot be spent on someone else's log — and nothing more. Self-certification answers *who*; admission answers *may*.

## Definitions

* **Registrant.** The party registering, identified by `issuer`; after the commit, the account holder.
* **Admission evidence.** `admission`: an invite (`token` and `claimCode`) or a voucher.
* **Self-certifying DID.** A DID whose identifier commits to the keys that control it, so that a copy of its log proves control without resolution. For `did:webvh`, the SCID.
* **`did:scid` alias.** The `did:scid:vh:1:<scid>` form of a `did:webvh`. It is a distinct identity verified by the same log; the service accepts it as `issuer` only where the document lists it.
* `domain`, `path`, `method`, `didData` — as their schema descriptions say.

## Request

The registrant sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An embedded agent registers with an operator invite

```json
{
  "id": "urn:uuid:7d2f0a1e-5c3b-4e8a-9f61-2b7c4d9e0a11",
  "type": "https://trusttasks.org/spec/did-management/account/register/0.1",
  "issuer": "did:scid:vh:1:QmScidExample",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-01T09:00:00Z",
  "payload": {
    "domain": "did.example.com",
    "path": "g:agent-7f3a",
    "method": "webvh",
    "didData": "{\"versionId\":\"1-QmEntry\",\"versionTime\":\"2026-12-01T08:59:00Z\",\"parameters\":{\"method\":\"did:webvh:1.0\",\"scid\":\"QmScidExample\",\"updateKeys\":[\"z6MkUpdate\"],\"portable\":true},\"state\":{\"id\":\"did:webvh:QmScidExample:did.example.com:g:agent-7f3a\"},\"proof\":[]}",
    "admission": {
      "invite": { "token": "inv_4b1f9c2e8a7d6e5f", "claimCode": "K7QP-29XD" }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-01T09:00:00Z",
    "verificationMethod": "did:scid:vh:1:QmScidExample#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z3FqPoX3xdRbW"
  }
}
```

### Admitted by a trust community's voucher

```json
{
  "id": "urn:uuid:0b9e6c4d-3a2f-4d1e-8c7b-6a5f4e3d2c1b",
  "type": "https://trusttasks.org/spec/did-management/account/register/0.1",
  "issuer": "did:webvh:QmScidOther:did.example.com:g:agent-91cc",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-01T09:05:00Z",
  "payload": {
    "domain": "did.example.com",
    "path": "g:agent-91cc",
    "method": "webvh",
    "didData": "{\"versionId\":\"1-QmOther\",\"parameters\":{},\"state\":{},\"proof\":[]}",
    "admission": {
      "voucher": {
        "type": ["VerifiableCredential", "DidHostingVoucher"],
        "id": "urn:uuid:5a4b3c2d-1e0f-4a9b-8c7d-6e5f4a3b2c1d",
        "issuer": "did:webvh:QmVtcScid:community.example",
        "validUntil": "2026-12-02T09:00:00Z",
        "credentialSubject": {
          "id": "did:webvh:QmScidOther:did.example.com:g:agent-91cc",
          "hostingService": "did:webvh:QmHostScid:did.example.com"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "created": "2026-12-01T09:05:00Z",
          "verificationMethod": "did:webvh:QmVtcScid:community.example#key-1",
          "proofPurpose": "assertionMethod",
          "proofValue": "z2VqPoX3xdRbW"
        }
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-01T09:05:00Z",
    "verificationMethod": "did:webvh:QmScidOther:did.example.com:g:agent-91cc#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z4GqPoX3xdRbW"
  }
}
```

## Response

The hosting service answers with the account it created and the DID record it published, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). `account` is a [HostingAccount](../../../_shared/0.4/hosting-account.schema.json); `record` is the [DidRecord](../../../_shared/0.1/did-record.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The account is created

```json
{
  "id": "urn:uuid:1c2d3e4f-5a6b-4c7d-8e9f-0a1b2c3d4e5f",
  "type": "https://trusttasks.org/spec/did-management/account/register/0.1#response",
  "threadId": "urn:uuid:7d2f0a1e-5c3b-4e8a-9f61-2b7c4d9e0a11",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:scid:vh:1:QmScidExample",
  "issuedAt": "2026-12-01T09:00:01Z",
  "payload": {
    "account": {
      "subject": "did:scid:vh:1:QmScidExample",
      "role": "guest",
      "domain": "did.example.com",
      "state": "active",
      "admission": "invite",
      "registeredAt": "2026-12-01T09:00:01Z",
      "lastActiveAt": "2026-12-01T09:00:01Z",
      "disableAt": "2027-03-01T09:00:01Z",
      "maxDidCount": 1,
      "maxTotalSizeBytes": 65536
    },
    "record": {
      "mnemonic": "g:agent-7f3a",
      "owner": "did:scid:vh:1:QmScidExample",
      "createdAt": "2026-12-01T09:00:01Z",
      "updatedAt": "2026-12-01T09:00:01Z",
      "versionCount": 1,
      "didId": "did:webvh:QmScidExample:did.example.com:g:agent-7f3a",
      "method": "webvh",
      "domain": "did.example.com",
      "disabled": false
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-01T09:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z5HqPoX3xdRbW"
  }
}
```

### The invite was already used

```json
{
  "id": "urn:uuid:9e8d7c6b-5a4f-4e3d-2c1b-0a9f8e7d6c5b",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "urn:uuid:7d2f0a1e-5c3b-4e8a-9f61-2b7c4d9e0a11",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:scid:vh:1:QmScidExample",
  "issuedAt": "2026-12-01T09:00:01Z",
  "payload": {
    "code": "did-management/account/register:admissionRefused",
    "message": "The invite has already been redeemed.",
    "retryable": false,
    "details": { "reason": "inviteRedeemed" }
  }
}
```

## Security & Privacy

### Data carried

The request carries the registrant's DID log, which is public once published, and its admission evidence. The invite's token and claim code are secrets: a consumer **MUST** store only slow hashes of them and **MUST NOT** log either. A voucher is not a secret, but it is single-use and names its subject, so a copy is no use to anyone else. The response carries the account and DID record, which describe only the registrant's own hosting. A producer **MUST NOT** put anything in `ext` it would not publish with its DID.

### Correlation

`issuer` is the registrant's public DID (`identifierScope: public`): the task exists to publish it, so pairwise identifiers do not apply. A voucher links the registrant to the community that issued it; the hosting service learns that membership, and a registrant that does not want it known uses an invite instead. The service learns nothing from an invite beyond which one was used.

### Retention

The consumer keeps the account and DID record for the account's life, and the single-use record of a voucher's (`issuer`, `id`) until its `validUntil`. A redeemed invite's summary (did-management/account/invite/list) is operator audit data. The request document itself has evidentiary value as the account's origin — it proves the registrant controlled the DID when it registered — and a consumer **MAY** keep it for the account's life.

### Consent/purpose

The DID log is submitted to be published, which is its only purpose. The admission evidence is collected to decide admission and to stop its reuse; a consumer **MUST NOT** use a voucher's community membership for anything else, such as profiling the registrant across services.
