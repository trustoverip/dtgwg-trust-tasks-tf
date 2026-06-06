# policy — category conventions

This document records the conventions shared by every specification under the
`policy/` category. It is descriptive; where it could conflict with a specific
spec, **that spec's own front matter and payload schema are authoritative.**

## 1. Shared schema component

Policy payloads reference one shared component
([SPEC.md §6.6](../../../../SPEC.md#66-shared-schema-components)):

[`policy.schema.json`](policy.schema.json) — `$defs`: `PolicyModule` (a stored
policy), `PolicyInput` (the request context a decision is evaluated against),
and `PolicyDecision` (the outcome, e.g. `allow` / `deny` / `requireStepUp`).
`PolicyInput`/`PolicyDecision` themselves reference the `device/_shared` and
`vault/_shared` components, so the policy engine speaks the same vocabulary as
the tasks it gates.

## 2. Recipient party

Every `policy/` task is addressed to the **vault maintainer** — the party tagged
`member: recipient`, declared `REQUIRED`. Policy is stored and enforced by the
same maintainer that runs the vault. Per
[SPEC.md §7.2 item 5](../../../../SPEC.md#72-consumer-requirements) the
`recipient` is enforced in-band.

## 3. Proof convention

Tasks that **mutate stored policy** declare `proofRequirement: REQUIRED`
(`policy/upsert`, `policy/delete`); read/evaluate tasks declare `RECOMMENDED`
(`policy/list`, `policy/evaluate`). The per-spec declaration is authoritative.

## 4. Decision and method values

`PolicyDecision` outcomes and step-up methods (`requireStepUp`,
`stepUpResponse`, `webauthnUv`, `pushApproval`, …) are spec-defined enumerated
values in lowerCamelCase ([SPEC.md §4.10](../../../../SPEC.md#410-naming-conventions)).
