---
slug: credential-exchange/present
version: "0.1"
title: Credential Exchange — Present
summary: Holder to verifier — an OID4VP vp_token disclosing exactly the consented claims, bound to the verifier's nonce and audience.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - presentation
  - oid4vp
  - vp-token
  - selective-disclosure
  - holder-binding
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
  - role: Verifier
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The `vp_token` already carries holder binding over the verifier's nonce and audience, and that remains the proof that establishes the presentation — an envelope proof does not strengthen it. The envelope proof is required for a different reason. Execution exercises the subject's own authority and the response discloses secret material the caller retains, so the exchange is one a third party may later be asked to rely on, which is the case §4.7.1 makes a MUST. Without a document proof the transaction as a whole is repudiable even though the presentation inside it is not, and an intermediary can relay a genuine `vp_token` under an envelope of its own.
sideEffects:
  level: none
  rationale: Presenting asserts; it does not mutate the verifier. Any record the verifier keeps is its own.
exposure:
  discloses: secret
  actsAsSubject: true
  rationale: The body is a disclosure of the holder's own claims to a named audience. It is the point at which private attributes leave the wallet, and it cannot be undone.
errorCodes:
  - code: credential-exchange/present:staleNonce
    meaning: The presentation is bound to a nonce the verifier no longer considers fresh.
    retryable: false
  - code: credential-exchange/present:audienceMismatch
    meaning: The presentation is bound to a different audience than the verifier.
    retryable: false
related:
  - credential-exchange/query
  - credential-exchange/pending/approve
---

## Abstract

The **Credential Exchange — Present** Trust Task answers a [query](../../query/0.1/) with an [OID4VP](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) `vp_token` — a selectively-disclosed, holder-bound presentation of a matched credential.

## Format-agnostic, with an honest asymmetry

`vp_token` is a JSON **string** or a JSON **object**, and a consumer selects its verification path from the value's type:

- **String** — an SD-JWT-VC presentation: exactly the consented disclosures, plus a mandatory key-binding JWT over the verifier's `nonce` and audience.
- **Object** — a W3C Data-Integrity VP whose proof carries the same `nonce` and `domain`.

These are not equivalent in what they can withhold, and a specification that glossed over it would be misleading. Plain `eddsa-jcs-2022` has **no claim-level selective disclosure**: presenting a credential that way discloses all of it. A holder MUST therefore refuse to present on that path unless the credential's claims are a subset of what was consented to. The alternative — disclosing more than was consented to because the format could not do better — is the failure this rule exists to prevent, and it is silent unless checked for.

## Holder binding is mandatory

Every presentation binds the verifier's `nonce` (freshness) and audience, in the key-binding JWT for SD-JWT-VC or the proof's `nonce` and `domain` for Data Integrity.

Both bindings are load-bearing and neither substitutes for the other. Without the nonce a presentation can be replayed later; without the audience a presentation captured by one verifier can be forwarded to another and accepted. A consumer MUST verify both and reject with `staleNonce` or `audienceMismatch` accordingly.

## Conformance

Producer: disclose exactly the consented claims and no more. Bind the verifier's nonce and audience. On a format without claim-level selective disclosure, present only if the whole credential is within the consented set — otherwise refuse. Reply on the query's thread; for a deferred query, present on the original thread once approval lands.

Consumer: verify holder binding, freshness and audience before reading any claim. Verify the underlying credential's issuer signature and status. A presentation that verifies cryptographically may still be a credential you should not accept — signature validity is not the same question as issuer trust.

## Authorization

*Stated in anticipation of [SPEC §7.3](../../../../SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is **holder binding** — that the presenting party controls the credential it presents — together with the audience and freshness bindings the verifier checks before reading any claim.

The specification already states the principle this section names: *a presentation that verifies cryptographically may still be a credential you should not accept — signature validity is not the same question as issuer trust.* Holder binding establishes who is presenting; whether the underlying credential's issuer is one the verifier trusts, and whether the disclosure serves a purpose it will honour, are separate decisions the verifier makes under its own policy.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](../../../../SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

`exposure.discloses` is `secret` and `actsAsSubject` is true: this is the moment private claims leave the wallet, to a named audience, irreversibly. Every other task in this family exists to make sure this one happens only when it should.

A verifier MUST NOT retain more than its stated purpose requires. The holder consented to a disclosure for a reason; retention beyond it is outside what was agreed, and this task's `purpose` binding is the record of what that was.
