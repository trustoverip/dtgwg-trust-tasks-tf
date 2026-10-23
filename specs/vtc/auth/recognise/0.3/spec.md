---
slug: vtc/auth/recognise
version: "0.3"
title: VTC Auth — Recognise
summary: Mint a scoped cross-community session by presenting a foreign community's role (authority) and membership credentials; maps the foreign role to a local one via policy.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - auth
  - recognise
  - cross-community
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: foreign community member
    requirement: REQUIRED
    member: issuer
  - role: community
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Sender authentication is carried by the presented credentials (the vec/vmc
    proofs), not the framework proof. A framework proof is recommended for
    channel binding but the credential proofs are the primary evidence.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Recognition is a challenge-response exchange, and a response accepted outside a bounded window is a response to a challenge that has already been answered. This is the case SPEC §7.2 item 11 protects most directly.
sideEffects:
  level: mutating
  rationale: "Mints a cross-community session token; recoverable by letting it expire or revoking it."
consequences:
  - "Issues a scoped access token granting the caller a mapped local role, effective until expiry."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/auth/recognise:credentialInvalid
    meaning: The role credential (VAC) or membership credential (VMC) failed proof verification, was expired, or was revoked via credentialStatus.
    retryable: false
  - code: vtc/auth/recognise:issuerNotRecognised
    meaning: The foreign issuer DID is not in this community's cross_community_roles policy.
    retryable: false
  - code: vtc/auth/recognise:roleNotMapped
    meaning: The foreign role has no mapping to a local role under policy.
    retryable: false
---

## Abstract

The **VTC Auth — Recognise** Trust Task lets a member of a *foreign* Verifiable
Trust Community obtain a scoped session in *this* community by presenting a
**holder-signed Verifiable Presentation** embedding their foreign-issued
membership credential and role credential. The role credential is a DTG
Verifiable Authority Credential (`AuthorityCredential`) issued by the foreign
community, whose `credentialSubject.authority` is
`{ "scope": <foreign community DID>, "actions": ["role:<name>"] }`; the foreign
role is the `<name>` a `role:` action carries. `0.1` called it `vec`, from when
roles were carried in endorsement credentials.

The community's `cross_community_roles` policy decides whether the foreign
issuer is recognised and which **local** role the foreign role maps to.

### Changes from 0.1

`0.1` took the two credentials directly, as `{vec, vmc}`. **That made the pair
a replayable impersonation token.**

Both are bearer artifacts. Anyone who obtained them — from a relayed join, an
audit log, a backup, a compromised member device — held everything the payload
required, and a community minting a session off them had no way to tell the
subject from someone holding a copy. There was no proof the caller controlled
the subject's key, no freshness, and no audience binding, so the same pair
worked at every community that recognised the issuer, indefinitely.

`0.2` requires a presentation instead, holder-signed with `proofPurpose:
authentication`, committing to the single-use `nonce` from
[`vtc/auth/recognise/challenge`](../challenge/0.2/) and naming the
recognising community's DID as `domain`. Each embedded credential still
carries its own issuer proof.

That is three separate properties, and dropping any one restores the attack:
the holder signature proves possession of the subject key, the nonce defeats
replay, and `domain` stops a presentation minted for one community being spent
at another. A consumer MUST also refuse unless the presentation's holder is
the credentials' subject — otherwise one party is admitted on another's
evidence.

A captured credential pair is inert under `0.2`: an attacker cannot produce the
holder signature over a fresh challenge, and a replayed presentation finds its
nonce already consumed.

The response returns a `sessionId` prefixed `xc-` (cross-community, distinct
from a local-member session) and a short-lived `accessToken` whose expiry is
clamped to the earliest of the community default and the credentials'
`validUntil`.

## Changes from 0.2

- The response's **`data.accessExpiresAt`** changes type from an integer Unix
  timestamp (seconds) to an RFC 3339 date-time (`type: "string"`,
  `format: "date-time"`). Its name and meaning are unchanged: the access
  token's expiry, clamped to the earliest of the community default and the
  credentials' `validUntil`. `1792747800` in 0.2 is `"2026-10-23T09:30:00Z"`
  here.
- The `nonce` comes from
  [`vtc/auth/recognise/challenge/0.2`](../challenge/0.2/), whose `expiresAt`
  makes the same change.

The reason is the registry-wide convention that an instant on the wire is an
RFC 3339 string whose member name ends in `At`. The request payload is
unchanged.

Breaking on the wire, released as a `MINOR` increment under
[§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains
published.

## Conformance

Producer: obtain a `nonce` from `vtc/auth/recognise/challenge`, then present a
`presentation` embedding the foreign `vmc` and `vec`, holder-signed with
`proofPurpose: authentication`, committing to that `nonce` and naming the
recognising community's DID as `domain`.

Consumer: verify the holder proof and each embedded credential's issuer proof;
consume the nonce; refuse unless the holder is the credentials' subject. Reject
on invalid/expired/revoked (`credentialInvalid`). Resolve the foreign issuer
against
`cross_community_roles` (`issuerNotRecognised` if absent). Map the foreign role
to a local role (`roleNotMapped` if unmapped). Mint an `xc-` session with the
**mapped local role** — never the raw foreign role — and an expiry clamped to
`min(local default, earliest credential validUntil)`.

## Security & Privacy

**Cross-trust boundary.** The `mappedRole` MUST come from local policy; a
foreign credential never asserts a local role directly. The `xc-` session
prefix keeps cross-community sessions auditably distinct from local ones.
Expiry clamping ensures a session cannot outlive the evidence that justified
it.

### Data carried

The request carries a holder-signed presentation embedding the caller's
foreign membership credential and role credential, so the recognising
community receives both credentials in full, together with the holder's DID.
The response carries an `xc-` `sessionId` and a bearer `accessToken` (an
authenticator for the minted session), its `accessExpiresAt`, the
`foreignIssuerDid`, and the local `mappedRole`. `discloses: metadata` — the
response reveals the foreign issuer DID and the mapped role. This
specification imposes no further data-minimisation rule.

### Correlation

The presentation names the recognising community's DID as `domain` and
commits to a single-use `nonce` from `vtc/auth/recognise/challenge`, which
binds it to one community and one exchange. The holder DID and the embedded
credentials identify the same subject at every community it presents them to,
and the `xc-` `sessionId` joins later requests under the session to this
exchange. This specification imposes no further rule on correlation.

### Retention

The minted session's expiry is clamped to `min(local default, earliest
credential validUntil)`. This specification sets no rule for how long the
recognising community retains the presented credentials or the exchange.

### Consent/purpose

The credentials are presented so the recognising community can decide whether
to grant a scoped cross-community session under its `cross_community_roles`
policy. This specification places no further limit on their reuse.
