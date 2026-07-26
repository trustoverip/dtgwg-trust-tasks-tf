---
slug: vtc/join-requests/submit-receipt
version: "0.1"
title: VTC Join Requests — Submit Receipt
summary: A community's acknowledgement that an applicant's join request was accepted for processing, naming the request id the applicant polls on.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join
  - receipt
  - applicant
  - ceremony
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
  - role: applicant
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The applicant must be able to confirm the receipt came from the community they applied to before trusting the request id in it.
sideEffects:
  level: none
  rationale: "Acknowledges a request already recorded; the receipt itself changes nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/submit-receipt:unknownRequest
    meaning: The recipient has no record of submitting a request matching this receipt.
    retryable: false
---

## Abstract

The **VTC Join Requests — Submit Receipt** Trust Task is the community's answer to [`vtc/join-requests/submit`](../../submit/0.1/). It tells the applicant their request was accepted for processing and hands them the `requestId` that [`vtc/join-requests/status`](../../status/0.1/) is polled with.

It is an acknowledgement of **receipt**, not of admission. A join request may be auto-admitted, queued for an administrator, or rejected — this receipt says only that the community has it.

## Conformance

Producer (the community): send after the request is durably recorded, never on merely parsing it. `status` reports the request's state at the moment of acknowledgement, which may already be terminal where policy auto-admitted or auto-rejected.

Consumer (the applicant): verify the sender is the community applied to. Treat `status` as a snapshot, not a verdict — poll `vtc/join-requests/status` for the outcome unless the status is already terminal.

## Security & Privacy

Sending only after durable recording is what makes the `requestId` meaningful: a receipt issued on parse would hand the applicant an identifier for a request that could still be lost, and they would poll forever on a request the community never had.

The proof requirement matters more here than it might appear. The receipt teaches the applicant which identifier to poll; a forged one could point them at a request that does not exist, or at another applicant's, turning a status poll into an information leak.
