---
slug: vault/release
version: "0.2"
wireCompatibleWith: "0.1"
title: Vault — Release
summary: A vault consumer requests the cleartext secret material of an entry; the maintainer returns it inside an HPKE-sealed envelope with a strict cache TTL. The fallback when proxy-login is not viable.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vault
  - credentials
  - release
  - autofill
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Release transfers long-term secret material to the consumer. Even though wrapped in HPKE, the consumer becomes the secret's custodian for the TTL window. The producer's identity MUST be verifiable for audit and so policy can enforce per-consumer release rules.
sideEffects:
  level: mutating
  rationale: "Returns an entry's cleartext secret in an HPKE-sealed envelope with a strict TTL; the release is logged."
consequences:
  - "Discloses the entry's secret material to the requesting consumer (sealed in transit)."
subjectPath: /target
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: false
  rationale: "Returns the entry's cleartext secret material to the caller (HPKE-sealed in transit). Inbound, `consumerContext` carries `deviceId`, `lastUserVerificationAt` — the moment a human last unlocked their device — and a `networkClass` of `home`, `corp`, `public`, or `vpn`, so every release also reports where the principal is and when they were last physically present at the device. `target` additionally names the site or app being acted against."
retention:
  class: transient
  rationale: "`ttlSeconds` is the point of the task: the consumer MUST wipe the unsealed cleartext when the window expires, and the ceiling is 600 seconds, so the design's expectation is that nothing survives ten minutes. This class records that expectation and not a guarantee — the maintainer hands over material it can no longer reach, so a non-compliant consumer that retains the secret cannot be detected from here. The maintainer's own durable residue is the audit line, not the secret."
errorCodes:
  - code: vault/release:notFound
    meaning: No entry with this id exists in the consumer's scope.
    retryable: false
  - code: vault/release:stepUpRequired
    meaning: Policy demands a step-up proof. Same shape as vault/proxy-login:stepUpRequired.
    retryable: true
    detailsSchema:
      $ref: "../../_shared/0.2/consumer-context.schema.json#/$defs/StepUpChallenge"
  - code: vault/release:policyDeny
    meaning: Policy refuses to release this secret to this consumer.
    retryable: false
  - code: vault/release:envelopeUnsupported
    meaning: The consumer's published recipient key advertises envelope kinds the maintainer does not implement (e.g. consumer requests `tspMessage` against a maintainer that only emits `didcommAuthcrypt`). Producers SHOULD consult `trust-task-discovery/0.1` for the maintainer's emit set.
    retryable: false
    detailsSchema:
      $ref: "../../_shared/0.2/sealed-envelope.schema.json#/$defs/EnvelopeMismatch"
---

## Abstract

The **Vault — Release** Trust Task is the fallback when `vault/proxy-login/0.1` cannot be used: the consumer needs the raw secret bytes (autofill into a desktop app, browser-bound channel binding at the third party, copy-to-clipboard, etc.). The maintainer returns the secret in an HPKE-sealed envelope with a short TTL the consumer is required to enforce.

Release is the higher-risk path. Use proxy-login when possible.

## Conformance

A conforming **producer** **MUST**:

1. Populate `entryId`. **MAY** populate `target` and `consumerContext`.
2. Carry a `proof`.
3. **MUST** enforce the maintainer's returned `ttlSeconds` — wipe the cleartext from memory after that window.
4. **MUST NOT** persist the cleartext beyond the TTL — not to disk, not to logs, not to syncing storage.
5. On `stepUpRequired`, satisfy the demanded method and retry with `stepUpProof`.

A conforming **consumer** (the vault maintainer) **MUST**:

1. Verify proof and `FillRelease` capability on the entry.
2. Evaluate policy. Possible outcomes: `allow`, `requireStepUp`, `deny`.
3. On allow: unseal the stored secret, validate against `vault-secret.schema.json` for the entry's `secretKind`, re-seal under HPKE to the consumer's published recipient key.
4. Cap `ttlSeconds` at the maintainer's policy ceiling (RECOMMENDED ≤ 300 seconds for `password`, ≤ 60 for `bearerToken` and `oauthTokens`, ≤ 30 for `sshKey`).
5. Record `lastUsedAt = now` on the entry; emit a `sync/event/0.1` of kind `vaultUpserted`.
6. Audit-log the release with `{ who, when, entryId, ttlSeconds, outcome }` — NOT the secret bytes.

## Payload

`payload.entryId` (REQUIRED).

`payload.target` (optional).

`payload.consumerContext` (optional).

`payload.stepUpProof` (REQUIRED on retry).

`payload.ttlSecondsHint` (optional) — caller's preferred TTL; maintainer caps.

## Response

`payload.sealedSecret` — HPKE-sealed VaultSecret.

`payload.secretKind` — discriminator.

`payload.ttlSeconds` — enforced cache TTL.

## Security & Privacy

### Data carried

This is the task in which the secret actually moves. The response's `sealedSecret`
unseals to a `VaultSecret` — a password and username, a passkey's private key, an
OAuth refresh token, an SSH key — and once the consumer has unsealed it, the
consumer holds the principal's long-term credential in cleartext. `secretKind` is
returned alongside so the consumer can pre-allocate the right type before
unsealing, and `ttlSeconds` states how long it may keep it.

**Prefer proxy-login.** Whenever the maintainer can perform the login itself, it
should — release is the last resort, and the difference between the two tasks is
the whole design of this family. A [`vault/proxy-login`](../../proxy-login/0.2/spec.md)
gives the consumer a session blob: cookies and headers, scoped to one origin,
expiring on their own. A release gives it the credential that mints sessions,
which is unscoped, does not expire on its own, and works from anywhere including
from a device the maintainer never sees again. Consumers **SHOULD** attempt
proxy-login first and fall back to release only on `vault/proxy-login:notProxyable`.

Inbound, the request is small but not innocuous. `entryId` names which credential;
`target` names which of the entry's sites or apps is being acted against, so the
maintainer learns not just that a credential was wanted but where it is about to be
used. `consumerContext` is the member that carries a person rather than a
credential: `deviceId` identifies the device, `networkClass` reports whether the
principal is on a `home`, `corp`, `public`, or `vpn` network, and
`lastUserVerificationAt` is a timestamp of the last time a human physically
unlocked that device. Producers populate what they can observe and the maintainer
**MUST** cross-check anything security-relevant against its own state rather than
trusting it; a producer **SHOULD NOT** enrich these members beyond what its own
policy engine consumes, because everything supplied here is retained in the release
log described below.

### Correlation

A release is a strong signal of real-world activity, and the maintainer accrues one
per use. The tuple of `entryId`, `target`, timestamp, and `networkClass` says that
this principal used this account at this time from this kind of network — a
behavioural record of a person's day assembled from a security control. Repeated
`lastUserVerificationAt` values sharpen it further, since they mark when the
principal was physically at the device rather than merely when software ran.

Downstream, the correlation is worse and unreachable from here. Once the consumer
holds the raw credential it can use it anywhere, and the relying party sees only
the principal — no marker distinguishes a login the maintainer proxied from one an
agent performed with a released secret. Proxy-login at least confines the consumer
to a session bound to one origin; a release removes that confinement, which is why
it is the fallback and not the default.

`deviceId` is deliberately stable across releases so the maintainer can apply
per-device policy and revoke a single device; that stability is what makes the
release log joinable, and it is a cost the design accepts in exchange for
revocability.

### Retention

**TTL is contractual, and the contract is the whole mechanism.** The consumer is
bound to wipe the cleartext within `ttlSeconds` — capped at 600 by the schema, so a
release can never be sanctioned for longer than ten minutes — and **MUST** do so even
if the user has not finished interacting with it. `ttlSecondsHint` lets a consumer
ask for less; the maintainer **MAY** cap it and the consumer **MUST** honour the
maintainer's decision.

Maintainers **SHOULD** assume a non-compliant consumer cannot be detected directly.
Nothing in this task can enforce the wipe: the material is out of the maintainer's
reach the moment the envelope is opened. Defence is in the TTL ceiling itself —
short windows minimise the exposure a defection buys — plus device attestation and
ACL revocation when misuse is suspected. This is why the declared `transient` class
describes the design's expectation rather than a property anyone can verify.

**Replay.** The `id` is the maintainer's idempotency key. A retry within the
idempotency window returns the same sealed material with the same TTL, and the TTL
does **not** reset on retry — a consumer cannot extend its window by asking again.

**Audit reach.** Every release is logged, and that log is the durable artefact this
task leaves behind: it necessarily records the `consumerContext` a producer
supplied, which is why over-populating it has a retention cost and not merely a
disclosure one.

### Consent/purpose

The purpose is narrow and it is stated by the request: use *this* credential
against *this* target, now, for at most `ttlSeconds`. Everything beyond that is
outside what was asked for — a consumer that releases a credential to log into one
site and then reuses it elsewhere has exceeded the purpose the maintainer's policy
evaluated, and no member of the released secret carries the limit with it.

**Step-up disposition.** Release **SHOULD** require step-up more aggressively than
proxy-login does, precisely because the consumer ends up holding the secret; the
default policy **SHOULD** demand a recent user verification (≤ 30 seconds) for any
release. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) this
specification describes that disposition rather than requiring it — whether a human
is asked is the maintainer's policy decision, expressed through
`vault/release:stepUpRequired` and `vault/release:policyDeny`.

**Permission scope.** AI Agent consumers holding `FillRelease` are high-risk, since
the capability lets them obtain long-term credentials without a human in the loop.
Maintainers **SHOULD** prefer narrow per-site capability grants for them over a
blanket release capability.
