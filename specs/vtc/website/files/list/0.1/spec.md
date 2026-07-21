---
slug: vtc/website/files/list
version: "0.1"
title: VTC Website — Files List
summary: List the files served by a community's website, paged, excluding hidden and blocklisted files.
status: draft
targetFrameworkVersion: "0.2"
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
errorCodes:
  - code: vtc/website/files/list:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Website — Files List** Trust Task pages the files under a community's website root, each as `{ path, size, modifiedAt }`. Hidden files and blocklisted extensions are excluded, matching the public read handler. The byte contents are not returned — reading a file's bytes is a plain binary REST fetch, not a Trust Task.

## Conformance

Producer: optional `cursor`/`limit`.

Consumer: verify the community-admin capability. Return the visible files, excluding hidden and blocklisted ones, with `nextCursor` when more remain.

## Security & Privacy

**Path metadata.** Discloses file paths and sizes, not contents — `metadata`, behind the admin gate.
