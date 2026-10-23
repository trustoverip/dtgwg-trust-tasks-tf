---
slug: vtc/website/files/list
version: "0.2"
title: VTC Website — Files List
summary: List the files served by a community's website, paged, excluding hidden and blocklisted files.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords: [vtc, website, files, list]
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
  rationale: Read-only listing of website files. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the website file tree; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
---

## Abstract

The **VTC Website — Files List** Trust Task pages the files under a community's website root, each as `{ path, size, modifiedAt }`. Hidden files and blocklisted extensions are excluded, matching the public read handler. The byte contents are not returned — reading a file's bytes is a plain binary REST fetch, not a Trust Task.

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`**. It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

Producer: optional `cursor`/`limit`.

Consumer: verify the community-admin capability. Return the visible files, excluding hidden and blocklisted ones, with `nextCursor` when more remain.

## Security & Privacy

### Data carried

**Path metadata.** Discloses file paths and sizes, not contents — `metadata`, behind the admin gate.

The request carries only paging members. Each response item carries a path
relative to the site root, a size, and optionally a content hash (`etag`) and
modification time. Hidden files and blocklisted extensions are excluded,
matching the public read handler.

### Correlation

The `etag` matches what a direct read of the same path returns, so a reader can
tie a listed file to a fetched copy of it, and modification times show when the
site changed. `threadId` correlates request and response, and
`nextCursor`/`cursor` link the pages of one traversal; a responder MUST NOT
accept a cursor it did not issue. The specification imposes no further rule.

### Retention

The task reads the website file tree and persists nothing. The specification
imposes no retention rule on the administrator's copy of the listing.

### Consent/purpose

The purpose is for a community administrator to see which files the
community's website serves, and to detect changes without downloading each
file. The specification states no further purpose limitation.
