---
slug: vtc/endorsement-types/list
version: "0.2"
title: VTC Endorsement-Types — List
summary: List the endorsement types a community recognises.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords: [vtc, endorsements, endorsement-types, list]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only listing of registered endorsement types. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the endorsement-type registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
---

## Abstract

The **VTC Endorsement-Types — List** Trust Task returns the endorsement types the community recognises, as [`EndorsementType`](../../../_shared/0.1/endorsement-type.schema.json) entries, paged by `cursor`/`limit`.

The registry lists predicate IRIs only, and includes the four DTG core predicates — `https://registry.trustoverip.org/dtg/vsc/endorses/1`, `.../witnessed/1`, `.../vetted/1` and `.../presented/1` — which the community seeds when it starts. The reserved value `role:vetter` never appears: it is a record kind of `vtc/endorsements/*`, not a registration.

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`**. It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

Producer: optional `cursor`/`limit`.

Consumer: verify the community-admin capability. Return the registered types, `nextCursor` when more remain.

## Security & Privacy

### Data carried

**Registry metadata.** Discloses the recognised type URIs and their schemas — `metadata`, behind the admin gate. Each [`EndorsementType`](../../../_shared/0.1/endorsement-type.schema.json) entry may also carry `createdByDid` and `createdAt`, naming the member who registered the type and when. The request carries only `cursor` and `limit`. This specification imposes no further data-minimisation rule.

### Correlation

`createdByDid` attributes each registered type to a member DID, which joins that member to the community's endorsement vocabulary. `nextCursor` links the pages of one listing, and `threadId` joins each request to its response. This specification imposes no further rule on correlation.

### Retention

The recipient persists nothing on account of this task. This specification sets no rule for how long a requester retains the listing.

### Consent/purpose

The listing is returned so an administrator can see the endorsement types the community recognises. This specification places no further limit on its reuse.
