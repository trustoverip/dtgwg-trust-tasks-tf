---
slug: vtc/ceremonies/list
version: "0.1"
title: VTC Ceremonies — List
summary: Return the community's ceremony manifests — the declarative descriptions an operator UI renders to drive each governance decision.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - ceremonies
  - manifest
  - governance
  - admin-ui
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
  rationale: Read-only description of the community's own capabilities. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Returns static manifests compiled into the maintainer; persists nothing."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
---

## Abstract

The **VTC Ceremonies — List** Trust Task returns the community's ceremony manifests: for each governance decision the community can run, a declarative description of its purpose, the fields it takes, and how they should be presented.

An operator UI renders forms from these rather than hard-coding them, so a maintainer that gains a ceremony exposes it without a client release.

## Conformance

Producer: no payload members are required.

Consumer: return every ceremony this maintainer implements. The manifests describe *capability*, not community state — two communities running the same maintainer version return the same list. A consumer MUST treat an unrecognised field type as opaque and render it generically rather than dropping the field, so an older UI degrades instead of silently omitting an input the ceremony needs.

## Security & Privacy

`exposure.discloses` is `none`: the manifests describe what the software can do, not what the community has done. No member, credential, or policy data is reachable here.

They are still authenticated rather than public, because the set of ceremonies a deployment exposes is a reasonable fingerprint of its configuration and version.

**Free text.** Two members of each returned manifest are free text and are now
bounded: `purpose` at 128, matching the decision-slot bound the `policy/*` family
already declares, and `blurb` at 256, matching the `label` it is displayed
beside. Both are authored by the maintainer that publishes the manifest and are
read by the operator console rendering the ceremony picker; nothing else reads
them, and no decision is taken from either. The maintainer **retains** both for
as long as the ceremony is published — they are part of the manifest, not part
of any enactment — and a console SHOULD NOT copy them into the record of a
ceremony it ran, because the manifest can be republished with different wording
under the same `purpose`.

