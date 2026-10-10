---
slug: "external/accounts/delete"
version: "0.1"
title: "External Accounts — Delete"
summary: "A manager deletes an archived external account: its record, wrapped keys and secrets are destroyed and its derived key path is retired for good."
status: "draft"
targetFrameworkVersion: "0.6.0"
category: "key-management"
parties:
  - role: "Account manager"
    requirement: "REQUIRED"
    member: "issuer"
    identifierScope: "any"
  - role: "Key custodian"
    requirement: "REQUIRED"
    member: "recipient"
proofRequirement:
  requirement: "REQUIRED"
  rationale: "The task changes an authority the community's integrations rely on, and where consent applies it is counted against the request's own digest. Both need the request to be attributable to its signer whatever transport, or relayer, carried it."
issuedAtRequirement:
  requirement: "REQUIRED"
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor."
sideEffects:
  level: "destructive"
  rationale: "Removes the account record and destroys its wrapped key or secret; irreversible. The provider-side configuration is not touched."
exposure:
  discloses: "metadata"
  ingests: "metadata"
  actsAsSubject: false
  rationale: "An account reference and a reason in; the account out. No secret."
retention:
  class: "durable"
  rationale: "The state change persists; the audit row keeps the reason."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external/accounts/delete:invalidTransition"
    meaning: "The account is not in a state this task applies to (`archived`)."
    retryable: false
related:
  - "external/accounts/suspend"
  - "external/accounts/resume"
  - "external/accounts/archive"
  - "external/accounts/restore"
---

## Abstract

The **External Accounts — Delete** Trust Task removes an account for good. Only an `archived` account can be deleted, so a deletion is always preceded by a period in which the account was visibly out of use.

The custodian destroys the account's wrapped RSA key or secret. A derived P-256 key cannot be destroyed — it is a function of the custodian's seed — so its path is **retired**: recorded as never to be derived again for any account. The provider side is the administrator's to clean up: a deleted account's trust anchor, pool provider or app credential should be removed there too, which `external/accounts/setup` described when it was created.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** accept the task only for an account that is `archived`. **MUST** refuse an `archived` account, where `archived` is not a starting state, with `external:archived`, and any other state with `external/accounts/delete:invalidTransition`.
2. **MUST** destroy the account's wrapped key or secret and **MUST** record its derived key path, if any, as retired, never to be derived for another account.
3. **MUST NOT** reuse the account id in the context while any audit row names it.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

**Consent.** A custodian **SHOULD** subject this task to consent under its approvals policy ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)): the request is held, and approvers — the context's administrators other than the requester — each decide with a `task-consent/decision` signed by their own DID. A decision counts for the DID that signed it, never for a relayer that delivered it. Without consent, one administrator, or one compromised console relaying for administrators, can destroy an authority the community relies on, irreversibly.

## Definitions

- **Account state** — `AccountState` in [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json). The lifecycle is `active` ⇄ `suspended`, either → `archived`, `archived` → `suspended` (restore), `archived` → gone (delete).

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### Delete

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000044",
  "type": "https://trusttasks.org/spec/external/accounts/delete/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000134",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary",
    "reason": "Rooms storage moved to the new account."
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`.

### Done

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000064",
  "type": "https://trusttasks.org/spec/external/accounts/delete/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000134",
  "payload": {
    "id": "eu-s3-primary",
    "deletedAt": "2026-10-10T10:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

An account reference and an optional reason in; the account out.

### Correlation

None beyond the account itself.

### Retention

The state change persists; the audit trail keeps who changed it, when and why.

### Consent/purpose

Reductions need no one else; restorations of authority are consented where the operator enforces consent (conventions §4).
