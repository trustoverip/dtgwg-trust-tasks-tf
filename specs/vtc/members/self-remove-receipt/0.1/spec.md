---
slug: vtc/members/self-remove-receipt
version: "0.1"
title: VTC Members — Self-Remove Receipt
summary: A community's asynchronous acknowledgement that a member's own departure request has been carried out, naming how their published record was dispositioned.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - self-remove
  - receipt
  - departure
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
  - role: member
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The departing member must be able to confirm the receipt came from the community they left, not from a third party asserting it.
sideEffects:
  level: none
  rationale: "An acknowledgement of work already done; the receipt itself changes nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/self-remove-receipt:unknownRequest
    meaning: The recipient has no record of a departure request matching this receipt.
    retryable: false
---

## Abstract

The **VTC Members — Self-Remove Receipt** Trust Task is the community's asynchronous answer to [`vtc/members/self-remove`](../../self-remove/0.1/). It tells the departing member that the removal actually happened and how their published record was dispositioned.

It exists as a separate task because self-removal is not synchronous: the member asks to leave, the community carries it out — revoking credentials, flipping status-list bits, applying the departure preference — and only then can it say what was done.

## Conformance

Producer (the community): send after the removal has taken effect, not on accepting the request. `removed: true` means the record is gone; `disposition` names how the published record was handled, echoing the preference the member expressed or the community default where they expressed none.

Consumer (the member): verify the sender is the community being left. A receipt is the member's only evidence that a departure they requested was honoured, so an unauthenticated one is worth nothing.

## Security & Privacy

The receipt is the member's evidence of erasure. That is precisely why the proof requirement is REQUIRED rather than RECOMMENDED: a forged receipt would let a community — or anyone impersonating one — persuade a member that their data was removed when it was not.

The receipt discloses only what the member already knows or is entitled to know about their own departure. It carries no information about other members.
