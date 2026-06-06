# device — category conventions

This document records the conventions shared by every specification under the
`device/` category, and by the `push/` family that reuses this category's
shared schema. It is descriptive — patterns already present across the specs.
Where it could conflict with a specific spec, **that spec's own front matter and
payload schema are authoritative.**

## 1. Shared schema component

Device (and push) payloads reference one shared component
([SPEC.md §6.6](../../../../SPEC.md#66-shared-schema-components)):

[`device-binding.schema.json`](device-binding.schema.json) — `$defs`:
`DeviceBinding`, `Capability`, `ConsumerKind`, `DeviceAttestation`,
`KeyCustody`, `PushRegistration`, `WakeHandle`, `WakeTriggerPolicy`.

The `push/` specs (`push/register`, `push/provision`) have no `_shared`
directory of their own; they reference `device/_shared/.../device-binding.schema.json`
for `WakeHandle`, `WakeTriggerPolicy`, and `PushRegistration`.

## 2. Recipient party

Most `device/` tasks are addressed to the **vault maintainer** (the party tagged
`member: recipient`, declared `REQUIRED`): a device is registered and managed by
the maintainer. The exception is `device/wipe`, whose `recipient` is the
**device** itself — the maintainer issues the wipe *to* the device. Per
[SPEC.md §7.2 item 5](../../../../SPEC.md#72-consumer-requirements) the
`recipient` is enforced in-band.

## 3. Proof convention

Tasks that **mutate device state** declare `proofRequirement: REQUIRED`
(`device/disable`, `device/register`, `device/set-wake`, `device/wipe`);
read-only/telemetry tasks declare `RECOMMENDED` (`device/heartbeat`,
`device/list`). The per-spec declaration is authoritative.

## 4. Wake model

The platform-push wake flow has three roles — gateway, trigger, device — and is
defined by the [push wake-up binding](../../../../bindings/push/0.1/spec.md). A
device hands its platform push token to a gateway and receives an opaque
`WakeHandle`; producers **MUST NOT** place a raw platform push token in a
payload — only the opaque handle (see the prose of `device/set-wake` and
`push/register`). The `WakeTriggerPolicy` allowlist is VTA-owned, not
device-supplied.

## 5. Attestation and capability values

`DeviceAttestation` kinds (`appleAppAttest`, `playIntegrity`, `nitroEnclave`,
…) and `Capability` values (`vaultRead`, `proxyLogin`, …) are spec-defined
enumerated values in lowerCamelCase ([SPEC.md §4.10](../../../../SPEC.md#410-naming-conventions)).
Externally-owned tokens that appear in the same schema (push-provider names such
as `apns`/`fcm`/`webpush`) are carried verbatim per the same section.
