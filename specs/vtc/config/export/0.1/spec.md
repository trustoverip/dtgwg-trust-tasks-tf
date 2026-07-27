---
slug: vtc/config/export
version: "0.1"
title: VTC Config — Export
summary: Export a Verifiable Trust Community's portable configuration — its profile and its stored configuration overrides — as one self-describing document.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - config
  - export
  - portability
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Reads the community's whole configuration surface in one call. The caller must be attributable.
sideEffects:
  level: none
  rationale: Reads state and returns a document; nothing is mutated.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: The document carries operational configuration and the community's public profile. No key material and no member data — unlike vtc/backup/export, which is why this task needs no passphrase.
errorCodes:
  - code: vtc/config/export:permissionDenied
    meaning: The consumer lacks the community administrator capability.
    retryable: false
related:
  - vtc/config/import
  - config/show
  - vtc/backup/export
---

## Abstract

The **VTC Config — Export** Trust Task returns a community's portable configuration as a single [`ConfigExportDocument`](#response): the community profile, and the configuration overrides the maintainer stores for itself. An operator saves that document and later feeds it to [`vtc/config/import`](../../import/0.1/) — to stand up a replacement maintainer, to move a community between hosts, or to review what a deployment is actually configured to do.

It takes no parameters. `config/show` has a `keys` selector because an operator inspecting a running maintainer often wants one key; an export does not, because a document carrying an arbitrary subset is not portable — importing it would silently leave the unselected keys at whatever the target already had, which is the failure this task exists to avoid.

## What is portable, and what is not

`configOverrides` carries **only** the maintainer's own stored overrides — the layer `config/patch` writes. Values that come from the host, such as environment variables or an on-disk configuration file, are deliberately excluded. They describe *where a maintainer runs*, not *what the community is*: a document carrying the source host's listen address or log destination would, on import, reconfigure the target host to impersonate the source. Excluding them is what lets the same document be imported onto a differently-deployed maintainer.

For the same reason this is **not** a backup. [`vtc/backup/export`](../../../backup/export/0.1/) captures every backed-up keyspace plus the community signing key bundle, needs a passphrase, and restores a community wholesale. This task captures the configuration surface an operator would otherwise re-enter by hand, carries no key material and no member data, and needs no passphrase.

## Self-describing by design

`schemaVersion` is not redundant with this specification's `0.1`. The Type URI versions the *envelope*; the moment an operator writes the response to a file, the envelope is gone and the document is bare JSON. `schemaVersion` is the only thing a later reader — possibly a different implementation, possibly across a version boundary — has to check it against. A consumer that does not implement the version it finds MUST reject the document rather than interpret it optimistically.

`exportedAt` is provenance for the operator. A consumer does not act on it: an older document is not thereby wrong, and this task defines no staleness rule.

## Conformance

Producer: send an empty payload. Persist the returned `document` verbatim — re-serialising it through a partial model is how `schemaVersion` and unrecognised members get dropped, and a document that has lost them is no longer safe to import.

Consumer: verify the community administrator capability. Return the community profile under `communityProfile`, omitting the member entirely when no profile exists yet (a maintainer exported before bootstrap) — do not synthesise an empty profile, because an import cannot distinguish a synthesised one from a real one that happens to be blank. Populate `configOverrides` from the stored-override layer only. Set `schemaVersion` to the version of the document shape actually emitted.

## Security & Privacy

`exposure.discloses` is `metadata`, matching [`config/show`](../../../../config/show/0.1/): the document describes how a community is configured and presents itself, but it carries no secrets. The community signing key bundle, the audit log, and member records are all outside this task — an operator who needs those needs [`vtc/backup/export`](../../../backup/export/0.1/) and the passphrase discipline that comes with it.

The one member worth noting is `extensions`, an opaque community-defined bag. A community that has put sensitive values in it exports them here. That is a property of what was stored rather than of this task, but it is why an export still warrants the administrator capability instead of being a public read.
