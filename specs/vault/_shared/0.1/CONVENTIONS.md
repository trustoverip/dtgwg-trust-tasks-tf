# vault — category conventions

This document records the conventions shared by every specification under the
`vault/` category. It is descriptive: it summarises patterns that are already
present across the category's specs and shared schemas so authors and reviewers
can see them in one place. Where a statement could conflict with a specific
spec, **that spec's own front matter and payload schema are authoritative.**

## 1. Shared schema components

Vault payloads reference these `vault/_shared/` components rather than
re-declaring common shapes (see [SPEC.md §6.6](../../../../SPEC.md#66-shared-schema-components)):

| Component | `$defs` | Used for |
|---|---|---|
| [`vault-entry.schema.json`](vault-entry.schema.json) | `VaultEntry`, `SecretKind`, `SiteTarget`, `AttachmentRef` | the metadata record for a stored item |
| [`vault-secret.schema.json`](vault-secret.schema.json) | `VaultSecret` + per-kind defs (`PasswordSecret`, `PasskeySecret`, `OAuthTokensSecret`, `BearerTokenSecret`, `SshKeySecret`, `DidSelfIssuedSecret`, `DidCommPeerSecret`, `TotpSeed`, `CustomSecret`, …) | the secret material itself |
| [`sealed-envelope.schema.json`](sealed-envelope.schema.json) | `SealedEnvelope` (`DidcommAuthcryptEnvelope`, `HpkeArmoredEnvelope`, `TspMessageEnvelope`) | the cipher-bearing envelope that carries released secret material |
| [`consumer-context.schema.json`](consumer-context.schema.json) | `ConsumerContext`, `StepUpProof` | context + step-up evidence for tasks that consult the policy engine |
| [`session-blob.schema.json`](session-blob.schema.json) | `SessionBlob`, `CookieJarEntry`, `RequestHeader`, `StorageEntry` | the result of a `vault/proxy-login` |

## 2. Recipient party

Every `vault/` task is addressed to the **vault maintainer** — the party tagged
`member: recipient` in each spec's front matter, declared `REQUIRED`. The
producer is the requesting *vault consumer* (or a device acting as one). Per
[SPEC.md §7.2 item 5](../../../../SPEC.md#72-consumer-requirements), the
`recipient` is therefore enforced in-band.

## 3. Proof convention

Across the category, tasks that **mutate state or release secret material**
declare `proofRequirement: REQUIRED` (`vault/delete`, `vault/upsert`,
`vault/release`, `vault/proxy-login`, `vault/sign-trust-task`); read-only tasks
declare `RECOMMENDED` (`vault/get`, `vault/list`, `vault/sync`, `vault/usage`).
This mirrors the threat model in [SPEC.md §4.7.1](../../../../SPEC.md#471-when-to-include-a-proof):
a released secret is retained and relied upon after delivery, so it needs a
transport-independent integrity guarantee. The per-spec declaration is
authoritative.

## 4. Step-up and policy

Tasks whose authorization depends on the policy engine carry a
`ConsumerContext` and, where the policy demands re-authentication, a
`StepUpProof` (both from `consumer-context.schema.json`). Step-up evidence
kinds (`webauthnUv`, `pushApproval`, …) are spec-defined enumerated values and
follow the lowerCamelCase rule of [SPEC.md §4.10](../../../../SPEC.md#410-naming-conventions).
