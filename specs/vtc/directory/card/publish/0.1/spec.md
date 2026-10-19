---
slug: vtc/directory/card/publish
version: '0.1'
title: VTC Directory — Card — Publish
summary: A member publishes their external card — their public directory values as a JSContact card they signed — which the community serves as signed and points its trust registry to, until the values change, the member withdraws it or leaves.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- directory
- external card
- trust registry
- jscontact
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Publishing a person's card beyond the community is the widest disclosure the directory makes. The member's signed decision is the community's evidence that they made it, and must outlive the transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed publication could re-publish a card the member has since withdrawn.
sideEffects:
  level: mutating
  rationale: Turns external publication on, stores the signed card, serves it, and adds a pointer to it to the member's recognition record in the community's trust registry. Undone by vtc/directory/card/update with `discoverability.external` false.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: true
  rationale: The member hands over the card they want published — personal data they chose to make public. The response is the card's publication state.
retention:
  class: durable
  rationale: The signed card is served, and kept, only while it is published; withdrawn, its pointer is removed first and the card erased.
errorCodes:
- code: vtc/directory/card/publish:notMember
  meaning: The signer is not an active member of this community.
  retryable: false
- code: vtc/directory/card/publish:notAvailable
  meaning: This community does not publish external cards. Nothing was changed.
  retryable: false
- code: vtc/directory/card/publish:consentRequired
  meaning: The member has not consented to registry publication (vtc/members/registry-consent), which an external card hangs off and requires (VTI-DIR-047). Nothing was changed.
  retryable: false
- code: vtc/directory/card/publish:superseded
  meaning: '`expectedRevision` differs from the card''s revision: the card changed after the member signed. Nothing was published.'
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      revision:
        type: integer
        minimum: 0
- code: vtc/directory/card/publish:cardMismatch
  meaning: The card is not exactly the member's current public values — its `uid` is not their entry, or it carries a value that is not public, or leaves one out, or differs from one. Nothing was published.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      fields:
        type: array
        maxItems: 64
        items:
          type: string
          maxLength: 128
        description: The directory fields that differ, by key.
- code: vtc/directory/card/publish:signatureInvalid
  meaning: The card's proof does not verify, or its verification method is not one of the membership DID's authorized for `assertionMethod`. Nothing was published.
  retryable: false
related:
- vtc/directory/card/public
- vtc/directory/card/update
- vtc/directory/card/show
- vtc/members/registry-consent
- registry/record/put
---

## Abstract

The **VTC Directory — Card — Publish** Trust Task is how a member makes their card findable beyond the community. The member — never the community — writes and signs a JSContact card holding exactly the values on their card whose visibility is `public`, and sends it here. The community checks it is exactly those values and exactly the member's signature, then:

- serves it, unchanged, to anyone through [`vtc/directory/card/public`](../../public/0.1/spec.md);
- adds a pointer to it to the member's recognition record in its trust registry, stating the purpose;
- turns the card's `discoverability.external` on.

The community never signs, composes or edits the card. Profile data about a person beyond the community is the person's own statement, which the community mirrors; it is never the community asserting things about them (VTI-DIR-046, VTI-REG-004 item 4).

The published card goes stale the moment the member's public values change. The community then withdraws it — stops serving it and removes the pointer — and reports the card as `awaitingSignature` until the member signs and publishes the current values. It never serves a card the member did not sign, and never one that says something other than what their card now shows the public.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`, §5.3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is the member, signing both the card and this document with their membership DID — a key they delegated under [`auth/oob/grant/0.2`](../../../../../auth/oob/grant/0.2/spec.md) never publishes. Before the member signs, the producer **MUST** show them what will be published — every value, as the card carries it — that it is published so that anyone can find and read their card, and that anyone means anyone, outside the community too (VTI-REG-004 item 2). It builds the card from the member's card as [`vtc/directory/card/show`](../../show/0.1/spec.md) returns it, mapping each value whose visibility is `public` as `JSContactCard` maps it, with `uid` `urn:vtc-directory:<entryId>`, and signs it with [`keys/sign-document`](../../../../../keys/sign-document/0.1/spec.md) or its own key: a Data Integrity proof (`eddsa-jcs-2022` or `ecdsa-jcs-2019`) by a verification method of the membership DID, `proofPurpose: assertionMethod`. It **SHOULD** send the `revision` it built from as `expectedRevision`.

A conforming **consumer** (the community) **MUST**:

1. Refuse a signer who is not an active member with `notMember`, and a document signed by a delegated key with `permissionDenied`.
2. Refuse with `notAvailable` when it does not publish external cards, and with `consentRequired` while the member has not consented to registry publication.
3. Refuse with `superseded` when `expectedRevision` is present and differs from the card's revision.
4. Compute the member's public values — the values on their card that the public would be shown: their visibility `public`, the field's ceiling `public`, and permitted to a viewer with no standing by the community's `directory` policy — and refuse with `cardMismatch`, naming the fields that differ, unless the card's `uid` names the member's own entry and the card carries exactly those values, mapped as `JSContactCard` maps them, and no other directory value. JSContact members that carry no directory value (`@type`, `version`, `uid`, `kind`, `updated`) are permitted.
5. Check what `PublishedCard` leaves to it — `@type` `Card`, `version` `1.0`, `uid` present, and a `proof` that is a `DataIntegrityProof` with `cryptosuite` `eddsa-jcs-2022` or `ecdsa-jcs-2019` and `proofPurpose` `assertionMethod`, refusing a card that fails any with `cardMismatch` or `signatureInvalid` as the failure concerns the card or its proof — and verify `proof` over the card without it, against the membership DID's `assertionMethod` verification methods, and refuse with `signatureInvalid` where it does not verify.
6. Publish atomically: store the card exactly as received, set `discoverability.external` true, set the card's `externalCard` to `published` with its `cardDigest` and `publishedAt`, increment the card's revision, and answer `externalCard`. A card already published is replaced.
7. Add to the member's recognition record in its trust registry ([`registry/record/put`](../../../../../registry/record/put/0.1/spec.md)) the pointer defined under *The registry pointer*, replacing any earlier one. Publication does not wait for the registry: the card is served by the community, and the pointer tells the registry's readers where. The community retries the pointer until it is written or the card is withdrawn.
8. Never serve, and never point to, a card other than the one the member signed, nor one that differs from their current public values. When those values change for any reason — the member's update, an administrator's assertion, a ceiling narrowed, a field disabled or retired, a value expired, a policy change — it **MUST** at once stop serving the card, remove the pointer, and set `externalCard` to `awaitingSignature`. `discoverability.external` stays on: the member chose publication, and only their signature of the new values resumes it.
9. Withdraw the card — remove the pointer first, then stop serving it and erase it — when the member sets `discoverability.external` to false ([`vtc/directory/card/update`](../../update/0.1/spec.md)), when they withdraw registry-publication consent (VTI-DIR-047), and before anything else at their departure (VTI-DIR-120).
10. Audit the publication by reference — the entry, the card's digest and the signed document — never a value.

### Computing `cardDigest`

```
cardDigest = multibase( multihash( SHA-256( JCS( card ) ) ) )
```

over the card exactly as the member signed and sent it, `proof` included. `JCS` is [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) serialized as UTF-8. SHA-256 and base58btc (`z`) are **RECOMMENDED**, and a reader compares decoded multihash bytes. The digest names the exact signed card, so a reader holding a registry pointer can tell whether the card it is served is the one pointed to.

### The registry pointer

The community adds to the `context` of the member's recognition record (`record_type: recognition`) one member, `directoryCard`:

```json
{
  "directoryCard": {
    "purpose": "A member's own directory card, published at their request so that anyone may find it.",
    "entryId": "dir_Q2xhcmFRMmxoY21GUk1t",
    "cardDigest": "zQmRtaiZdzghK4hrtHNZFU11qm134M72fmH7JzwwjnP7Rou",
    "readWith": "https://trusttasks.org/spec/vtc/directory/card/public/0.1"
  }
}
```

`purpose` states, in the published entry, what it is for (VTI-REG-004 item 1). `entryId` and `cardDigest` are what a reader sends to [`card/public`](../../public/0.1/spec.md) at the community the record's authority names. The pointer carries no profile value: the card is fetched from the community, as signed. Nothing else about the member is added to the record.

## Authorization

**Authority: being the member, with their own key.** Who signed *is* the authorization question: the `issuer` is the member whose card is published, matched to their membership, and the card's proof is theirs. A delegated key is refused because external publication is the widest disclosure the directory makes, and a browser session is not the member deciding it. No administrator can publish a member's card. The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `PublishedCard`, `ExternalCard`, `JSContactCard` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).
- **Public values** — the values on a member's card that a viewer with no standing would be shown: visibility, ceiling and the community's policy all admitting the public.

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Carol publishes her name and bio

Carol has chosen `public` for her display name and bio, and nothing else. Her card is exactly those two values.

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000021",
  "type": "https://trusttasks.org/spec/vtc/directory/card/publish/0.1",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T09:10:01Z",
  "payload": {
    "card": {
      "@type": "Card",
      "version": "1.0",
      "uid": "urn:vtc-directory:dir_Q2xhcmFRMmxoY21GUk1t",
      "name": { "full": "Carol Example" },
      "notes": { "n1": { "note": "Rust, decentralised identity, and long walks." } },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:example:carol#key-1",
        "created": "2026-10-19T09:10:00Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
      }
    },
    "expectedRevision": 7
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-19T09:10:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/card/publish/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) with the codes declared above.

### Published

The `cardDigest` shown is the real value for the card as printed.

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000022",
  "type": "https://trusttasks.org/spec/vtc/directory/card/publish/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-19T09:10:02Z",
  "threadId": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000021",
  "payload": {
    "externalCard": {
      "state": "published",
      "cardDigest": "zQmRtaiZdzghK4hrtHNZFU11qm134M72fmH7JzwwjnP7Rou",
      "publishedAt": "2026-10-19T09:10:02Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries the card the member wants public — values they already chose to show anyone — and their signature over it. The response carries its publication state. The registry pointer carries no profile value at all, only where to read the card and how to recognise it.

### Correlation

A published card is public by the member's choice, and is a durable correlator the community counts (VTI-DIR-053): anyone holding it can link the member's entry identifier, their recognition record and whatever they chose to show. That is the purpose of publishing, and why it is a separate decision from recognition (VTI-DIR-047), made by the member with their own key, after being shown what it means. The card's `uid` names the entry, not the member's DID; the recognition record it hangs from already names the DID, which registry-publication consent covered.

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and its trust registry already know it by, and a pairwise identifier per caller would make the community differ by who asks.

### Retention

The community keeps the signed card only while it is published. Withdrawal — by the member, by consent being withdrawn, by a change to the public values, or by departure — removes the pointer first and erases the card. What a third party copied while it was public cannot be recalled, which is why departure withdraws publication before anything else (VTI-DIR-120).

### Consent/purpose

The purpose is that anyone may find and read the card the member chose to make public, and it is stated in the registry pointer itself. The community publishes nothing beyond that card (VTI-REG-004 item 3), publishes it only while it is exactly what the member signed, and never composes or signs profile data about a member itself.
