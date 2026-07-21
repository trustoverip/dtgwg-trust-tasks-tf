---
slug: vtc/website/files/delete
version: "0.1"
title: VTC Website — File Delete
summary: Delete a single file from a community's website root.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, website, files, delete]
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
  requirement: REQUIRED
  rationale: Deleting a served file changes the live site and destroys content; it MUST be attributable.
sideEffects:
  level: destructive
  rationale: "Removes a file from the website root; the bytes are gone unless separately backed up."
subjectPath: /path
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/website/files/delete:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/website/files/delete:notFound
    meaning: No file exists at the supplied path.
    retryable: false
---

## Abstract

The **VTC Website — File Delete** Trust Task removes one file at `path` from the community's website root. Live mode only.

## Conformance

Producer: supply `path`. Carry a proof.

Consumer: verify the community-admin capability. Delete the file (`notFound` if absent) and return `{ path, deleted: true }`. Audit the deletion.

## Security & Privacy

**Destructive.** The bytes are removed from the live site, so the task is proof-REQUIRED and audited. Path traversal outside the site root MUST be rejected.
