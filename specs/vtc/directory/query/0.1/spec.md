---
slug: vtc/directory/query
version: "0.1"
title: VTC Directory — Query
summary: Read the fields a community member has consented to publish, projected through the community's directory policy rather than read straight from the record.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - directory
  - consent
  - projection
  - privacy
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The projection depends on who is asking, so an attributable caller gets a correctly scoped answer.
sideEffects:
  level: none
  rationale: "Evaluates the directory policy and projects fields; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/directory/query:notFound
    meaning: No member with that DID, or none whose directory projection is visible to this caller.
    retryable: false
related:
  - vtc/directory/entries/show
  - vtc/directory/entries/list
  - vtc/directory/card/update
---

## Abstract

The **VTC Directory — Query** Trust Task returns the subset of a member's record that the community's directory policy permits this caller to see. `fields` narrows the request; the policy decides what is actually returned.

The projection is a **policy decision, not a database read**. Two callers asking for the same subject and the same fields can legitimately receive different answers, because the directory ceremony evaluates who is asking against what the subject consented to publish.

## Relationship to the directory family

This task predates the member directory's card model. Its successor is [`vtc/directory/entries/show`](../../entries/show/0.1/spec.md), which addresses a member by an opaque, community-scoped entry identifier and returns the full projection — the member's own visibility choices intersected with the community's ceilings and policy. New producers **SHOULD** use `entries/show`. A community that serves both **SHOULD** answer this task through the same projection, so the two never disagree about what a caller may see; addressing by DID here remains subject to the `notFound` rule below.

## Conformance

Producer: name the `subject` DID. `fields` is an optional comma-separated narrowing hint.

Consumer: run the directory ceremony rather than reading the membership record directly. Return only fields the policy permits for this caller. A requested field the policy withholds MUST be **absent** from the response, not present-and-null — an explicit null discloses that the field exists and is being withheld, which is itself information the subject did not consent to share.

Where the subject exists but nothing is visible to this caller, return `notFound` rather than an empty projection, so a caller cannot enumerate members by distinguishing "no such member" from "nothing visible".

## Security & Privacy

Conflating "absent because unset" with "absent because withheld" is the whole privacy property here, and it is why the two indistinguishable-outcomes rules above are normative rather than advisory. A directory that answers "that member exists but you may see nothing" is a membership oracle; one that answers `notFound` is not.

`fields` is a hint, never an authorisation. A caller asking for a field they may not see gets it withheld, not an error — an error would confirm the field exists.
