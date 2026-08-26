---
slug: credential-exchange/offer
version: "0.1"
title: Credential Exchange — Offer
summary: Issuer to holder — an OID4VCI Credential Offer that opens an issuance thread, leaving the credential format to be negotiated rather than fixed by the envelope.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - issuance
  - oid4vci
  - offer
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
proofRequirement:
  requirement: OPTIONAL
  rationale: The transport already proves the sender — an authcrypted DIDComm envelope yields a cryptographically authenticated `from` DID, which is what a holder checks before acting on an offer. A document proof is permitted for a relayed or store-and-forward path where the envelope's authentication does not survive the hop, but requiring one on every offer would duplicate work the transport has already done.
sideEffects:
  level: none
  rationale: An offer is a proposal. Nothing is issued and no credential exists until the holder replies and the issuer answers.
exposure:
  discloses: none
  ingests: secret
  actsAsSubject: false
  rationale: An offer asserts nothing about the holder, so nothing is disclosed. It can nonetheless carry a bearer secret INTO the holder — an OID4VCI pre-authorized code grant places a redeemable code in `credential_offer.grants`, and anyone who reads it can claim the credential. The offer body is carried verbatim, so this specification cannot rule that path out and declares the class that covers it.
retention:
  class: exchange
  rationale: An offer is consumed by the issuance thread it opens. A holder keeps it only until it replies with `credential-exchange/request` and the credential arrives — longer where the offer carries a pre-authorized code that has not yet been redeemed, and no longer than that, since a spent or lapsed code is a bearer secret with no remaining function.
errorCodes:
  - code: credential-exchange/offer:unsupportedCredential
    meaning: The holder cannot accept any credential configuration named in the offer.
    retryable: false
related:
  - credential-exchange/request
  - credential-exchange/issue
---

## Abstract

The **Credential Exchange — Offer** Trust Task opens the issuance leg: an issuer tells a holder which credentials are available and how to ask for them.

The division of labour is the point of this family. **The Trust Task is the transport, authentication, threading and relayer envelope; the body is [OID4VCI](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html).** The offer object is carried verbatim and is deliberately not re-specified here — re-stating a foreign specification inside this one would create a second source of truth that drifts from the first the moment OID4VCI revises.

## Format-agnostic by construction

Nothing in this task is specific to a credential format. Which format is issued follows from the credential configuration the offer references and is finalised by the DCQL `format` selector at presentation time. A deployment can move from SD-JWT-VC to a W3C Data-Integrity credential without touching this task or any consumer's handling of it.

## Conformance

Producer: send the OID4VCI Credential Offer under `credential_offer`, unmodified. Open a thread — the holder's [`request`](../../request/0.1/) and the eventual [`issue`](../../issue/0.1/) both reply on it.

Consumer: read the offer to learn which credentials are available and which grant flow applies, then reply on-thread. A holder that supports none of the offered configurations SHOULD say so with `unsupportedCredential` rather than abandoning the thread silently — an issuer that gets no answer cannot distinguish rejection from a lost message.

Relayer and holder may differ. The party that carries the envelope is not necessarily the subject, which is what makes air-gapped and invite-based onboarding work; a consumer MUST NOT infer the holder's identity from the envelope sender alone.

## Security & Privacy

### Data carried

An offer names credential *configurations*, not claims about anybody. Nothing about the
holder is asserted, no credential exists yet, and there is no response payload — which
is why `exposure.discloses` is `none` and why this is the least revealing document in
the family.

It is not, however, empty of things worth protecting. `credential_offer` is an OID4VCI
object carried verbatim, and one of the grant types it may carry is the **pre-authorized
code**: a bearer value that redeems for the credential, held by whoever reads it. That
is why `exposure.ingests` is `secret` rather than `none`. The body is not re-specified
here, so this task cannot forbid the grant or bound what an issuer puts in it; what it
can say is that a producer using that grant is placing a redeemable secret on the wire
and **MUST** treat the channel accordingly — an offer that could be sent over an
unauthenticated relay when it names only configurations cannot be, once it carries a
code. Where a transaction code or user PIN is part of that flow, it belongs out of band,
which is the whole reason OID4VCI separates them.

The configuration identifiers themselves are mildly revealing in aggregate. An offer for
a residency credential says what the issuer believes it is in a position to attest about
the recipient, which is a statement about the recipient even though it is not a claim
about them.

### Correlation

The Issuer declares `identifierScope: public`. A holder decides whether an offer is
worth answering by recognising who made it, and that recognition has to survive across
exchanges and match whatever the ecosystem's trust list names — the same identifier that
will sign the credential at [`issue`](../../issue/0.1/) and that a verifier will resolve
later. A pairwise issuer identifier would make every offer arrive from a stranger and
would leave the eventual credential unverifiable by third parties.

Nothing is declared for the Holder, and that is deliberate rather than an omission. An
offer may be addressed to a party that is not yet known — the invite and air-gap cases —
and relayer and holder may legitimately differ, which is why a consumer **MUST NOT**
infer the holder's identity from the envelope sender. There is no identifier here whose
scope this task is in a position to describe.

What a relayer on the path does learn is the pairing: this issuer offered this
configuration to this recipient, at this time. That is unavoidable given the envelope,
and it is the reason an offer carrying a pre-authorized code needs a channel it can
trust rather than merely a channel that works.

### Retention

An offer is consumed by the thread it opens. The holder keeps it long enough to reply
with [`request`](../../request/0.1/) and receive the credential, and the issuer keeps it
long enough to recognise the request as corresponding to an offer it made — which is
what `unsupportedCredential` and the eventual `unknownOffer` on the request side are
checked against. After the credential is delivered neither side has a use for it.

Where the offer carries a pre-authorized code the retention question sharpens: a stored,
unredeemed offer is a stored bearer secret. A holder **SHOULD** discard it as soon as it
is spent or lapses, and an issuer **SHOULD** expire the code rather than rely on the
holder to forget it, because an offer that stays redeemable indefinitely is a credential
waiting for whoever finds the message.

### Consent/purpose

The purpose of an offer is to open a conversation, and its limit is that it opens
nothing else. **An offer is not an authorization.** It states availability; the issuer's
own policy still governs whether a request made against it is honoured, and a consumer
**MUST NOT** treat having received an offer as a grant. Nor does it commit the holder:
declining is expected, and a holder that supports none of the offered configurations
**SHOULD** say so with `unsupportedCredential` rather than abandon the thread, since an
issuer receiving silence cannot distinguish a refusal from a lost message — and will
otherwise reasonably retry, which is a worse outcome for both.
