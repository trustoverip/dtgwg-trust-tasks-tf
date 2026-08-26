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
    identifierScope: public
  - role: Holder
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: The credential inside carries its own issuer signature, so a document proof is not what makes the credential trustworthy — that reasoning stands. What it does not cover is the delivery. This message hands over an asset, and on a relayed path the delivering party may not be the credential's issuer, so a holder receiving an unexpected credential has no way to attribute the delivery itself without an envelope proof. Repudiation of the hand-off, and substitution by an intermediary that re-wraps a genuine credential into a delivery it did not make, are the threats addressed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Issuance mints a credential that outlives the exchange, and the task returns no response the issuer can correlate against. The issue time is what lets the consumer bound how long a captured issue request stays executable.
sideEffects:
  level: mutating
  rationale: The holder receives the credential into its wallet. Recoverable — a credential can be deleted, and re-issuance is an ordinary flow.
exposure:
  discloses: secret
  ingests: secret
  actsAsSubject: false
  rationale: "The message body IS a credential — signed claims about the subject, and on the cleartext path they are on the wire in the clear. The `sealed` alternative exists precisely because that is not always acceptable. The same body is what the request carries into the holder, so the ingest class matches: the holder receives a credential it must thereafter protect, and on the `sealed` path a bundle that may be secret-bearing outright."
retention:
  class: durable
  rationale: The holder keeps the credential — that is what issuance is for, and a wallet that discarded what it received would make the whole family pointless. The credential's own `validUntil` or equivalent bounds its usefulness, not its storage; a holder that deletes it loses the ability to present, and any status or revocation check a verifier later runs is against the issuer's record rather than this document.
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

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is the consumer's own decision that this **issuer** is one it will accept a credential from, taken against the credential's own issuer signature rather than against whoever delivered it.

Conformance already states the rule this section exists to name: *the envelope proving who delivered it says nothing about who issued it.* On a relayed path the delivering party and the issuing party are routinely different, and accepting a credential because it arrived from a trusted relayer is the confusion the two-layer model exists to prevent. Verifying either proof establishes attribution; neither establishes that the credential should be accepted.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

### Data carried

The body **is** the credential, which makes this the one task in the family where the
sensitive data and the payload are the same object. On the `credential_response` path
everything the credential asserts about the subject is on the wire in the clear —
whatever the SD-JWT-VC or W3C Data-Integrity credential inside happens to contain,
which the issuer chose and this specification cannot bound. That is why the cleartext
path is conditioned on an authenticated, confidential channel *and* a known holder, and
why it is not the default.

`sealed` exists because the cleartext path's protection comes from the transport, and
the transport is exactly what cannot be trusted when the holder is not yet known. An
invite or air-gap credential may sit in a relayer's queue before it reaches anyone; the
armored bundle is encrypted to the holder's key, so the relayer carries a string it
cannot read. Digest pinning is mandatory there — the recipient verifies the bundle
against a digest obtained out of band, and rejects with `unopenableBundle` on mismatch —
because a bundle whose integrity is anchored only in the bundle is anchored in nothing,
and trust-on-first-use here means accepting a credential from whoever reached the queue
first.

`oneOf` is enforced rather than advisory, and the reason is a data-protection one as
much as a parsing one: a message carrying both members leaves a consumer to guess which
is authoritative, and the guess that reads `credential_response` when the issuer meant
`sealed` processes in the clear a credential the issuer had decided to seal.

### Correlation

The Issuer declares `identifierScope: public` and has no alternative. A credential is
only worth holding if a verifier that has never met the issuer can recognise it — the
issuer's identifier is what a trust list names, what the credential's signature resolves
to, and what a verifier checks status against. A pairwise issuer identifier would make
the credential unverifiable by anyone but the party it was issued through, which is the
opposite of the point.

The Holder declares `identifierScope: pairwise`, and the caveat is the one that matters
in this family. The holder can perfectly well use a distinct identifier with each issuer,
and **SHOULD**, so that issuers cannot join their records. But whatever identifier ends
up inside the credential as `credentialSubject.id` or as the `cnf` key stops being
pairwise the moment the credential is presented: it travels to every verifier the
credential is shown to, and becomes a join key across all of them. The choice is made
here, at issuance, and cannot be repaired at
[`present`](../../present/0.1/) — a holder that wants unlinkable presentations needs
either a credential format that supports it or a separately-issued credential per
relationship.

Delivery adds one more join: on a relayed path the relayer learns that this issuer
issued to this holder, and when, without reading anything. The sealed path protects the
contents from the relayer; it does not hide the fact of the delivery.

### Retention

The holder keeps the credential. That is not a policy choice a deployment makes, it is
what issuance is *for*, and a wallet that discarded what it received would make the rest
of the family pointless. The credential's own validity window bounds how long it is
useful, not how long it is stored, and status or revocation checks a verifier later runs
resolve against the issuer's record rather than against this document — so an expired
credential in a wallet is stale data with no remaining function, and a holder
**SHOULD** delete it rather than let it accumulate.

`sideEffects` is `mutating` rather than `destructive` precisely because this is
recoverable: a credential can be deleted and re-issuance is an ordinary flow. The issuer,
for its part, retains its own issuance record — who it issued what to, and when — which
is a durable record of the relationship and is governed by the issuer's own policy, not
by this task.

### Consent/purpose

The credential is delivered because the holder asked for it: this document closes the
thread that [`offer`](../../offer/0.1/) opened and
[`request`](../../request/0.1/) answered, and the key-binding proof in that request is
the record of what the holder asked to be bound to. The purpose is possession — the
holder is being given an asset to present later, under its own control, at times the
issuer will not know about.

That has a consequence worth stating: the issuer's involvement ends here. It does not
learn where the credential is subsequently presented, and **MUST NOT** design a status
or revocation mechanism that turns every verification into a call home, because that
would convert an issued credential into a standing surveillance channel over the
holder's use of it. Whether the holder is entitled to the credential at all is the
issuer's own policy decision, taken before this document exists; per
[SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying either proof
establishes who delivered or who issued, never that anyone may.
