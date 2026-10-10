---
slug: "external/accounts/probe"
version: "0.1"
title: "External Accounts — Probe"
summary: "A manager has the key custodian prove an external account works end to end — sign, exchange at the provider, and put, get and delete one canary object at the narrowest bound scope — and records the result."
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
  rationale: "The probe makes the custodian authenticate to a provider as the community and write to its storage; an unattributable one is a stranger exercising the account."
issuedAtRequirement:
  requirement: "REQUIRED"
  rationale: "Consequential, so SPEC §7.3 item 17 sets the floor. A replayed probe would act at the provider again."
sideEffects:
  level: "mutating"
  rationale: "Obtains a credential from the provider, writes and deletes one canary object there, and stores the report on the account (`lastProbe`)."
exposure:
  discloses: "metadata"
  ingests: "none"
  actsAsSubject: true
  rationale: "The custodian authenticates to the provider as the account. The report discloses step outcomes and the provider's error text, never the credential it obtained."
retention:
  class: "durable"
  rationale: "The latest report is kept on the account so approvers and consoles can see it."
errorCodes:
  - code: "external:notFound"
    meaning: "No account with this id exists in the named context that the caller may see. See the family conventions §2."
    retryable: false
  - code: "external:archived"
    meaning: "The account is `archived`; restore it before changing or using it. See the family conventions §2."
    retryable: false
  - code: "external:providerUnavailable"
    meaning: "The provider could not be reached or did not answer in time."
    retryable: true
related:
  - "external/accounts/setup"
  - "external/accounts/create"
  - "external/accounts/keys/rotate"
---

## Abstract

The **External Accounts — Probe** Trust Task answers the question an approver should ask before approving an account change: *does it work, and does it do only what it says?* The custodian:

1. **signs** the model's assertion or session request (for `sui-signer`, builds a test transaction and runs it through its own allow-list — never submitting it);
2. **exchanges** it at the provider for a credential, downscoped as an issuance would be;
3. at the narrowest scope any binding allows — or, with no bindings, under the account's `probePrefix` — **puts, gets and deletes** one canary object.

The first failing step ends the probe and its report carries the provider's own error, verbatim. A probe that could not reach step 3 because nothing names a canary prefix reports `complete: false`: it shows the account authenticates, not that it works. A probe is not consent-gated: it is how a change is shown to work **before** consent is asked for, and approvers see its report beside the request.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the family conventions in [`external/_shared/0.1/CONVENTIONS.md`](../../../../external/_shared/0.1/CONVENTIONS.md).

A conforming **custodian** (`recipient`):

1. **MUST** refuse with `external:archived` an account that is `archived`. It **MAY** probe a `suspended` account, since checking an account before resuming it is a reason to probe; the credential it obtains is used for the canary only.
2. **MUST** run the steps in order and stop at the first failure, reporting it with the provider's error text — truncated, and with any credential, signature or token removed.
3. **MUST** write the canary under a name it chooses inside the narrowest bound prefix — with no bindings, inside the settings' `probePrefix` — and **MUST** delete it. With neither, **MUST** stop after `exchange` and report `complete: false`.
4. **MUST** discard the credential it obtained when the probe ends, and **MUST NOT** return it.
5. **MUST** report `complete: true` only when the canary steps ran, and **MUST** store the report as the account's `lastProbe`. **MUST NOT** clear `providerSetupRequired` except on a report that is both `ok` and `complete`, with no rotation pending confirmation: an exchange alone proves the provider trusts the key, not that the account can do what its bindings will ask.
6. A provider refusal is a **successful** probe response with `ok: false`, not a `trust-task-error`; `external:providerUnavailable` is for a provider that could not be reached at all.

## Authorization

The authority this task presupposes is the capability **`external-accounts-manage`** in the account's context, held by the signer's access-control entry at the custodian and read at execution time through the entry's act scope ([conventions §4](../../../../external/_shared/0.1/CONVENTIONS.md#4-capabilities-and-consent)). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller without it is refused with `permissionDenied`.

This task is **not** consent-gated: it changes nothing an approver is asked to decide, and its report is what they decide with.

## Definitions

- **AccountProbeReport** — as [`external/_shared/0.1/accounts.schema.json`](../../../_shared/0.1/accounts.schema.json).

## Request

A manager (`issuer`) asks the custodian (`recipient`).

### Probe an account after setup

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000017",
  "type": "https://trusttasks.org/spec/external/accounts/probe/0.1#request",
  "issuer": "did:example:community-console",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000109",
  "payload": {
    "context": "community",
    "id": "eu-s3-primary"
  }
}
```

## Response

The sub-schema reachable via `$anchor: "response"`.

### Every step succeeded

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000018",
  "type": "https://trusttasks.org/spec/external/accounts/probe/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:community-console",
  "issuedAt": "2026-10-10T10:00:04Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000109",
  "payload": {
    "report": {
      "at": "2026-10-10T10:00:03Z",
      "ok": true,
      "complete": true,
      "steps": [
        {
          "step": "sign",
          "ok": true,
          "durationMs": 4
        },
        {
          "step": "exchange",
          "ok": true,
          "durationMs": 212,
          "providerRequestId": "7c1d9a3e-2f4b-4e6a-8b0c-1d3f5a7b9c2e"
        },
        {
          "step": "put",
          "ok": true,
          "durationMs": 88
        },
        {
          "step": "get",
          "ok": true,
          "durationMs": 41
        },
        {
          "step": "delete",
          "ok": true,
          "durationMs": 39
        }
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

An account reference in; a report out. The credential obtained during the probe never leaves the custodian.

### Correlation

The provider sees an authentication and three object operations from the account, at the time of the probe. The connections go only to hosts in the account's `egressHosts`.

### Retention

The latest report is stored on the account; the canary object is deleted.

### Consent/purpose

The purpose is to show an account works before anyone relies on it.
