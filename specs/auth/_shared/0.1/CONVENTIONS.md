# auth — category conventions

This document records the conventions shared by every specification under the
`auth/` category. It is descriptive; where it could conflict with a specific
spec, **that spec's own front matter and payload schema are authoritative.**

## 1. Shared schema components

Auth payloads reference these `auth/_shared/` components
([SPEC.md §6.6](../../../../SPEC.md#66-shared-schema-components)):

| Component | `$defs` | Used for |
|---|---|---|
| [`session.schema.json`](session.schema.json) | `Session` | the authenticated session a task establishes or references |
| [`tokens.schema.json`](tokens.schema.json) | `TokenBundle` | access/refresh token material |
| [`webauthn.schema.json`](webauthn.schema.json) | `AssertionResponse`, `AttestationResponse`, `CredentialCreationOptions`, `CredentialRequestOptions`, `CredentialDescriptor` | WebAuthn ceremony messages |

The WebAuthn `$defs` mirror the W3C WebAuthn dictionaries. Their member names
and enumerated values (`public-key`, `cross-platform`, `platform`, `none`,
`usb`, …) are **externally owned and carried verbatim** — never re-cased — per
[SPEC.md §4.10](../../../../SPEC.md#410-naming-conventions).

## 2. Recipient party

Most `auth/` tasks are addressed to the **Auth service** (the party tagged
`member: recipient`, declared `REQUIRED`). The step-up sub-family is the
exception: `auth/step-up/approve-request` is addressed to the **Approver**,
`auth/step-up/approve-response` back to the **Relying party**, and
`auth/step-up/policy` to the **ACL maintainer** — read each spec's front matter.
Per [SPEC.md §7.2 item 5](../../../../SPEC.md#72-consumer-requirements) the
`recipient` is enforced in-band.

## 3. Proof convention

Proof requirements vary because the *task* sometimes is the proof. Tasks that
assert or mutate authenticated state declare `proofRequirement: REQUIRED`
(`auth/authenticate`, the `passkey/enroll/*` set, `auth/revoke-session`,
`auth/sessions/list`, `auth/whoami`). Tasks whose own payload carries the
authentication evidence — a challenge request, or a WebAuthn assertion at
login/refresh — declare `OPTIONAL`/`RECOMMENDED` (`auth/challenge`,
`auth/passkey/login/start`, `auth/passkey/login/finish`, `auth/refresh`). The
per-spec declaration is authoritative.
