---
slug: credential-exchange/issue
version: "0.1"
title: Credential Exchange — Issue
summary: Issuer to holder — the issued credential, either cleartext to a known holder or as a sealed bundle only the holder can open.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - issuance
  - oid4vci
  - issue
  - sealed-transfer
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Issuer
    requirement: REQUIRED
    member: issuer
  - role: Holder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The credential inside carries its own issuer signature, so a document proof is not what makes it trustworthy. It is RECOMMENDED rather than OPTIONAL because this message delivers an asset — a holder that receives an unexpected credential should be able to attribute the delivery itself, not only the credential's issuer, which may be a different party on a relayed path.
sideEffects:
  level: mutating
  rationale: The holder receives the credential into its wallet. Recoverable — a credential can be deleted, and re-issuance is an ordinary flow.
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: The message body IS a credential — signed claims about the subject, and on the cleartext path they are on the wire in the clear. The `sealed` alternative exists precisely because that is not always acceptable.
errorCodes:
  - code: credential-exchange/issue:unopenableBundle
    meaning: The sealed bundle could not be opened, or its out-of-band digest did not match.
    retryable: false
  - code: credential-exchange/issue:unsupportedFormat
    meaning: The holder cannot process the delivered credential's format.
    retryable: false
related:
  - credential-exchange/offer
  - credential-exchange/request
---

## Abstract

The **Credential Exchange — Issue** Trust Task delivers the credential, closing the issuance thread. It carries **exactly one** of two shapes, and which one is a security decision rather than a preference:

- **`credential_response`** — the cleartext [OID4VCI](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html) Credential Response. For a **known** holder over an authenticated, confidential channel.
- **`sealed`** — an armored sealed-transfer bundle. For a **secret-bearing** credential, or an **unknown** holder — the invite and air-gap cases.

`oneOf` is enforced, not advisory. A message carrying both leaves a consumer to guess which is authoritative, and the wrong guess either drops a credential or processes one the issuer did not intend as the delivery.

## Why the sealed path exists

On the cleartext path the credential is protected only by the transport. That is adequate when the channel is confidential and the holder is already known and authenticated.

It is not adequate when the holder is **not yet known** — the invite case, where the credential is minted for whoever holds a particular key and may sit in a relayer's queue before reaching them. There the credential must be protected *from the transport*, not by it. The sealed bundle is encrypted to the holder's key, so a relayer carries something it cannot read.

Digest pinning is mandatory on that path: the recipient MUST verify the bundle against a digest obtained out of band. There is no trust-on-first-use, because a bundle whose integrity is anchored only in the bundle is anchored in nothing.

## Format-agnostic

The `credential` inside a cleartext response is a JSON **string** (SD-JWT-VC compact serialization) or a JSON **object** carrying a `proof` (a W3C Data-Integrity VC). A consumer selects its verification path from the value's shape, not from a discriminator member. That is deliberate: a new format needs no new field here, and a producer cannot mislabel one.

## Conformance

Producer: send exactly one of the two members. Choose `sealed` whenever the credential is secret-bearing or the holder is not yet authenticated, and communicate the bundle digest out of band. Reply on the request's thread.

Consumer: reject a payload carrying both members or neither. On the sealed path, verify the out-of-band digest **before** opening, and reject with `unopenableBundle` on mismatch. On the cleartext path, verify the credential's own issuer signature — the envelope proving who delivered it says nothing about who issued it.

## Security & Privacy

`exposure.discloses` is `secret`: the body is the credential. On the cleartext path everything the credential asserts is on the wire, which is why that path is conditioned on an authenticated, confidential channel and a known holder.

A consumer MUST verify the credential's issuer signature independently of the delivery. On a relayed path the delivering party and the issuing party are routinely different, and accepting a credential because it arrived from a trusted relayer is precisely the confusion the two-layer model exists to prevent.
