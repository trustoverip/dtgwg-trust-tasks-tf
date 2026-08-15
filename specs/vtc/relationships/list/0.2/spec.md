---
slug: vtc/relationships/list
version: "0.2"
title: VTC Relationships — List
summary: List the Verifiable Relationship Credentials published about a community member.
status: draft
targetFrameworkVersion: "0.4"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - list
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
  rationale: Read-only listing of a member's relationships. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the relationship store; persists nothing."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/list:permissionDenied
    meaning: The consumer lacks the capability to read this member's relationships.
    retryable: false
  - code: vtc/relationships/list:notFound
    meaning: No member with the supplied did exists.
    retryable: false
---

## Abstract

The **VTC Relationships — List** Trust Task returns the Verifiable Relationship Credentials recorded for a member `did` — each with its `id`, `issuerDid`/`subjectDid`, the `vrcJsonld` body, a `vrcDigestMultibase`, and `createdAt`. Paged by `cursor`/`limit`.

## Changes from 0.1

`0.1` carried the credential digest as **`vrcSha256`**, a bare lowercase-hex
SHA-256. `0.2` carries **`vrcDigestMultibase`**, the framework's
[`DigestMultibase`](../../../../_framework/0.3/framework.schema.json) — a
multibase-encoded multihash.

Three things were wrong with the old form, and the shared definition says so in
its own description. A bare hex string **hard-codes one algorithm into the wire
contract**, so moving off SHA-256 later would need a schema revision rather than
a different multihash prefix. It **names no base encoding**, so a verifier
infers base16 from context. And `0.1` **named no canonicalization** — the digest
was "SHA-256 of the VRC", which is not reproducible for a JSON document, so two
conforming implementations could compute different values for the same
credential and neither would be wrong. `0.2` states [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)
(JCS) explicitly.

This is a breaking change to the wire format, released as a `MINOR` increment
under [SPEC.md §5.2](../../../../../SPEC.md#52-compatibility-rules)'s `draft`
allowance. `0.1` remains published and unchanged; migrate with the
expand-then-contract sequence of
[§5.4](../../../../../SPEC.md#54-migrating-between-versions), and note that a
consumer at `0.2` **MUST** still accept `0.1` documents — which carry
`vrcSha256`, a member `0.2`'s schema rejects — so the two are distinguished by
the *Type URI*'s version, not by sniffing the payload.

## Conformance

Producer: supply `did`; optionally `cursor`/`limit`.

Consumer: resolve the member (`notFound` if absent). Return the relationships where the member is issuer or subject, clamping `limit` to 1..=200 and setting `nextCursor` when more remain.

## Security & Privacy

**Relationship metadata.** The entries name the related DIDs and carry the credential bodies — community-visible metadata behind the read gate.
