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
  actsAsSubject: false
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

`exposure.discloses` is `none`: an offer names credential *configurations*, not claims about anybody. Nothing about the holder is asserted, and no credential exists yet to disclose.

The offer is not an authorization. It states availability; the issuer's own policy still governs whether a request against it is honoured, and a consumer MUST NOT treat having received an offer as a grant.
