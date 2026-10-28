---
slug: payments/settle
version: "1.1"
title: Payments — Settle
summary: Settle an amount owed to the recipient, with an optional memo. Extends 1.0.
status: draft
extends: "1.0"
parties:
  - role: Payer
    requirement: REQUIRED
    member: issuer
  - role: Payee
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
errorCodes:
  - code: payments/settle:insufficientFunds
    meaning: The payer's balance does not cover the amount.
    retryable: false
---

# Payments — Settle 1.1

Adds an optional `memo` to 1.0.
