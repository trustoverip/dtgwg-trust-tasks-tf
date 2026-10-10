---
slug: "external/sign"
version: "0.1"
title: "External — Sign"
summary: "A bound integration has a key custodian sign a Sui transaction from an external account, only after the custodian has decoded it and checked every call, object, gas budget and coin movement against the account's allow-list and caps."
status: "draft"
targetFrameworkVersion: "0.6.0"
category: "key-management"
keywords:
  - "sui"
  - "walrus"
  - "blockchain"
  - "transaction-signing"
  - "allow-list"
  - "signing-oracle"
parties:
  - role: "Bound integration"
    requirement: "REQUIRED"
    member: "issuer"
    identifierScope: "any"
  - role: "Key custodian"
    requirement: "REQUIRED"
    member: "recipient"
proofRequirement:
  request: "REQUIRED"
  response: "RECOMMENDED"
  rationale: "A signature spends the account's funds. The binding is checked against the request's proven signer, and the request is the evidence of who asked for each payment."
issuedAtRequirement:
  requirement: "REQUIRED"
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor."
sideEffects:
  level: "mutating"
  rationale: "Counts the transaction's gas budget against the account's daily cap and writes an audit row. The signature, once submitted by the caller, moves funds; that effect is on chain, not at the custodian."
consequences:
  - "Signs a payment from the community's address, within the account's allow-list and caps."
exposure:
  discloses: "none"
  ingests: "metadata"
  actsAsSubject: true
  rationale: "The custodian signs as the account's address. The request carries an unsigned transaction; the response a signature and the transaction digest. No key or secret."
retention:
  class: "transient"
  rationale: "Nothing kept but the audit row and the day's gas tally."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:notActive"
    meaning: "The account is `suspended` or `archived` and cannot be used."
    retryable: false
  - code: "external:notBound"
    meaning: "The caller is not the consumer of any binding on this account."
    retryable: false
  - code: "external:rateLimited"
    meaning: "The binding's rate is exhausted. `details.retryAfterSeconds` says when to try again."
    retryable: true
    detailsSchema: {"type": "object", "additionalProperties": false, "required": ["retryAfterSeconds"], "properties": {"retryAfterSeconds": {"type": "integer", "minimum": 1}}}
  - code: "external/sign:notSignOnly"
    meaning: "The account's model is not `sui-signer`."
    retryable: false
  - code: "external/sign:undecodable"
    meaning: "`txBytes` is not a BCS-encoded Sui `TransactionData` of a kind the custodian decodes (only programmable transactions are signed)."
    retryable: false
  - code: "external/sign:senderMismatch"
    meaning: "The transaction's sender, or its gas owner, is not the account's address. Sponsored transactions are not signed."
    retryable: false
  - code: "external/sign:callNotAllowed"
    meaning: "A command calls a Move function, or uses a command kind, the account does not allow. `details` names the first one."
    retryable: false
    detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"commandIndex": {"type": "integer", "minimum": 0}, "package": {"type": "string", "maxLength": 66}, "module": {"type": "string", "maxLength": 128}, "function": {"type": "string", "maxLength": 128}, "commandKind": {"type": "string", "maxLength": 64}}}
  - code: "external/sign:objectNotAllowed"
    meaning: "The transaction takes a shared object the account does not allow."
    retryable: false
  - code: "external/sign:gasBudgetExceeded"
    meaning: "The gas budget exceeds the account's per-transaction cap, or would take the rolling 24-hour total past its daily cap."
    retryable: false
  - code: "external/sign:coinOutExceeded"
    meaning: "The transaction would move a coin out of the account's address beyond its per-transaction cap, or a coin type the account may not spend at all."
    retryable: false
  - code: "external/sign:wrongNetwork"
    meaning: "The transaction's expiration or referenced objects are inconsistent with the account's network."
    retryable: false
related:
  - "external/credentials/issue"
  - "external/accounts/get"
  - "keys/sign-sshsig"
---

## Abstract

Some providers are paid for, not logged in to. Walrus storage is bought by Sui transactions — register a blob, certify it, extend it, delete it — signed by the address that pays. The **External — Sign** Trust Task lets an integration have those transactions signed by a `sui-signer` account the custodian holds, without the integration holding a wallet key.

It is **not** a signing oracle. Like [`keys/sign-sshsig`](../../../keys/sign-sshsig/0.1/spec.md), it signs one externally defined format, which the custodian builds itself: the caller supplies an **unsigned transaction**, never a digest or bytes to sign. The custodian decodes it, checks every command against the account's allow-list and caps, constructs the intent message and its digest itself, and signs that. A caller who could submit a digest could get anything signed; one who submits a transaction can get only what the account allows.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`), before signing:

1. **MUST** check the account and the binding as [`external/credentials/issue`](../../credentials/issue/0.1/spec.md) does — existence, state, caller is a binding's consumer by proof, `external-auth-use`, rate — with the same codes, and refuse an account whose model is not `sui-signer` with `external/sign:notSignOnly`.
2. **MUST** decode `txBytes` as BCS `TransactionData` and refuse anything it cannot fully decode, or that is not a programmable transaction, with `external/sign:undecodable`.
3. **MUST** refuse with `senderMismatch` a sender or gas owner other than the account's address.
4. **MUST** check **every** command: each `MoveCall` names a `(package, module, function)` in `allowedCalls`; only the coin-handling commands the allowed calls need (`SplitCoins`, `MergeCoins`, and `TransferObjects` back to the account's own address) appear; `Publish`, `Upgrade` and transfers to any other address do not. Else `callNotAllowed`. When `allowedObjects` is set, every shared-object input is in it. Else `objectNotAllowed`.
5. **MUST** refuse a gas budget over `maxGasBudgetMist`, or one that would take the rolling 24-hour total over `maxGasPerDayMist` (`gasBudgetExceeded`), and any coin leaving the address beyond `maxCoinOutPerTx` for its type, or of a type not listed (`coinOutExceeded`).
6. **MUST** then build the intent message (intent scope `TransactionData`, version 0, app id Sui) over the decoded bytes and its Blake2b-256 digest **itself**, and sign it with the account's secp256r1 key as Sui specifies, returning a Sui serialized signature (flag, signature, public key). The signature **MUST** be one Sui's reference verifier accepts for that digest; implementations **SHOULD** be tested against Sui's published secp256r1 vectors, since the scheme's internal hashing and its low-`s` normalisation are where independent implementations diverge.
7. **MUST** count the gas budget against the daily cap only once it has signed, and **SHOULD** audit the signature with the decoded calls and digest.

### Retries

This task is **not retry-safe**. A second signature over the *same* bytes is harmless, and a custodian **SHOULD** answer a duplicate document with its original response; but a client that rebuilds the transaction and asks again — new gas object version, new bytes — gets a second, independent transaction signed, which is a second payment. A client **MUST NOT** wrap this task in a retry loop of its own; it retries only by resending the identical document.

## Authorization

The authority this task presupposes is twofold and both parts are required: **`external-auth-use`** in the account's context, held by the caller's access-control entry at the custodian and read at execution time; and **a binding on the account naming the caller as consumer**, which also fixes how far the caller may go. The capability says the caller may consume accounts in the context; the binding says which account, how far, how long and how often. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Signing is not consent-gated per transaction: the account's allow-list and caps are the authority that was consented, when the account was created or updated.

## Definitions

- **Allow-list, caps** — `SuiSignerSettings` in [`external/_shared/0.1/accounts.schema.json`](../../_shared/0.1/accounts.schema.json).
- **Intent message** — Sui's `IntentMessage<TransactionData>`: the three-byte intent prefix followed by the BCS bytes of the transaction. Its Blake2b-256 digest is what Sui signatures commit to, and is the transaction digest.

## Request

A bound integration (`issuer`) asks the custodian (`recipient`).

### Pay for a Walrus blob registration

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000080",
  "type": "https://trusttasks.org/spec/external/sign/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000180",
  "payload": {
    "context": "community",
    "account": "walrus-main",
    "txBytes": "AAADAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ"
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`: the signature and the digest the custodian computed. The caller submits the transaction with this signature itself. A refusal is a `trust-task-error`.

### Signed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000081",
  "type": "https://trusttasks.org/spec/external/sign/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000180",
  "payload": {
    "signature": "AgEjRWeJq83vASNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8BI0VniavN7wEjRWeJq83vASNFZ4mrze8BI0VniavN7wE",
    "digest": "7nq4DPvmbmKmGzYrBkkVJBKyBMyUY5ft2KNi4rFfDS8Q"
  }
}
```

## Security & Privacy

### Data carried

An unsigned transaction in; a signature and a digest out. The transaction will be public on chain once submitted.

### Correlation

Every signed transaction is public and linked to the account's address, which is public too; that is the nature of the chain. The custodian's audit row links each one to the integration that asked.

### Retention

The custodian keeps the audit row and the day's gas tally, nothing else.

### Consent/purpose

The allow-list and caps, consented with the account, are the purpose. A transaction outside them is refused however it is asked for.
