---
slug: payments/settle
version: "1.0"
title: Payments — Settle
summary: Settle an amount owed to the recipient. A third-party specification under acme's own authority (SPEC §6.5), used by the trust-tasks CLI tests.
status: draft
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

# Payments — Settle 1.0

A test fixture: a specification published by a third party, `acme`, under
`https://tasks.acme.example`.
