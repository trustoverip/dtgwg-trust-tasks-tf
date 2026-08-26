---
slug: credential-exchange/request
version: "0.1"
title: Credential Exchange — Request
summary: Holder to issuer — an OID4VCI Credential Request carrying the key-binding proof that ties the credential to be issued to a key the holder controls.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - credential-exchange
  - issuance
  - oid4vci
  - request
  - key-binding
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Issuer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The OID4VCI key-binding proof inside the body is a *different* thing from the document proof and is always required by OID4VCI itself — it binds the credential to a holder key, where a document proof attests the sender. Conflating the two is the mistake this rationale exists to prevent, and requiring both is what keeps them distinct. The document proof is required because execution acts with the subject's authority to obtain a credential in their name, and §7.3 item 8 forbids a declaration weaker than the §4.7.1 default — which is a MUST wherever the exchange may be relied on beyond the original consumer. An authcrypted transport proves the sender only to the immediate peer, leaving nothing attributable once the request is retained or relayed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A request to a holder is made under the subject's authority to ask. Captured and re-sent, it prompts the holder again for material they have already answered about, and a fire-and-forget task leaves no reply to tell the two apart.
sideEffects:
  level: none
  rationale: Asking for a credential mutates nothing at the issuer. Any state the issuer keeps to track the request is its own bookkeeping, not an effect of this task.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: "The holder is asking for a credential about itself, and the embedded key-binding proof names a key it controls — so the sender is the subject of what follows. No claims are disclosed by the request itself, and none are carried into the issuer either: what arrives is a credential configuration identifier and a public key, which is descriptive data about what is being asked for rather than attributes of the person asking."
retention:
  class: exchange
  rationale: The issuer needs the request only for the length of the issuance thread — long enough to verify the key-binding proof, mint against the key it names, and reply with `credential-exchange/issue`. The proof is nonce-bound and has no value once redeemed. Any longer-lived record of who was issued what is the issuer's own bookkeeping, kept under its own policy, and not an effect of this document.
errorCodes:
  - code: credential-exchange/request:invalidProof
    meaning: The key-binding proof is missing, malformed, or does not verify.
    retryable: false
  - code: credential-exchange/request:unknownOffer
    meaning: The request does not correspond to an offer this issuer made.
    retryable: false
related:
  - credential-exchange/offer
  - credential-exchange/issue
---

## Abstract

The **Credential Exchange — Request** Trust Task is the holder's answer to an [offer](../../offer/0.1/): a request for the credential, carrying the proof that binds it to a key the holder holds.

The body is an [OID4VCI](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html) Credential Request, carried verbatim and not re-specified here.

## Two bindings, not one

This task involves two distinct authentications, and treating them as one is the recurring implementation error:

1. **Envelope authentication** proves *who sent this message*. On an authcrypted transport it is intrinsic.
2. **The key-binding proof** inside the OID4VCI request proves *which key the credential should be bound to*. The issuer mints against that key — the `cnf` for SD-JWT-VC, the `credentialSubject.id` for a W3C Data-Integrity credential.

They can legitimately differ. A relayer may carry the envelope for a holder that is not the sender, which is exactly the air-gap onboarding case. **A consumer MUST bind the credential to the key in the proof, never to the envelope sender** — doing the latter issues a credential the intended holder cannot use, and, if the relayer is hostile, issues it to the relayer.

## Conformance

Producer: send the OID4VCI Credential Request under `credential_request`, unmodified, with its key-binding proof. Reply on the offer's thread.

Consumer: verify the key-binding proof before minting anything, and reject with `invalidProof` if it does not verify — an unverified proof means the credential would be bound to a key nobody has demonstrated control of. Answer on-thread with [`issue`](../../issue/0.1/).

## Security & Privacy

### Data carried

The request names a credential configuration and a public key, and nothing else. No
claims are disclosed by it and none are carried into the issuer — the embedded
`openid4vci-proof+jwt` key-binding proof demonstrates control of a key, which is
descriptive data about what is being asked for rather than an attribute of whoever is
asking. That is why `exposure.discloses` is `none` and `ingests` is `metadata`, the two
lowest classes in this family, on a task whose `actsAsSubject` is nonetheless true: the
holder is acting in its own name to acquire something about itself.

There is no room here for a producer to over-share, and that is worth noticing rather
than taking for granted. The payload has exactly one member, `credential_request` is
carried verbatim, and unknown members are rejected — so the only surface a producer can
widen is `ext`. A holder **SHOULD NOT** use it to supply supporting evidence for the
issuance decision: material sent to justify a request is retained by the issuer
alongside the credential it justified, and OID4VCI's own flows put that evidence in the
authorization step where it belongs, not in the redemption.

### Correlation

The key in the key-binding proof is the correlation decision of the whole issuance leg,
and it is made here. It becomes the credential's `cnf` for SD-JWT-VC or its
`credentialSubject.id` for a W3C Data-Integrity credential, and from there it travels to
every verifier the credential is ever shown to. A holder that reuses one key across
issuers, or across credentials from one issuer, hands verifiers a join key that survives
selective disclosure entirely — the claims can be withheld while the identifier that
links the presentations cannot. This is why the Holder declares
`identifierScope: pairwise`: the scope that matters is per-relationship, and a holder
that wants presentations it cannot be linked across needs a distinct key here, not a
better presentation later.

The Issuer declares `identifierScope: public` for the reason the rest of the family
does — a credential is only useful if a verifier who has never dealt with the issuer can
recognise and resolve it.

Envelope and key deliberately need not agree. A relayer may carry the request for a
holder that is not the sender, which is what makes air-gapped onboarding work, and a
consumer **MUST** bind the credential to the key in the proof rather than to the
envelope sender. Binding to the sender issues a credential the intended holder cannot
use and, where the relayer is hostile, issues it to the relayer.

### Retention

Exchange-scoped, and briefly so. The issuer needs the request long enough to verify the
proof, mint against the key it names, and answer with
[`issue`](../../issue/0.1/); the proof is nonce-bound and has no value once redeemed,
and `unknownOffer` is checked against the issuer's record of the offer rather than
against a stored copy of this document.

Whatever the issuer keeps beyond that is its own bookkeeping — the specification's
`sideEffects` rationale says as much, and the distinction is load-bearing rather than
pedantic. An issuance record ("this key was issued this credential at this time") is a
durable record of a relationship, retained under the issuer's policy and answerable
under it; it is not something this task obliges anyone to keep, and a consumer
**SHOULD NOT** justify retaining the request itself by pointing at the record it fed.

### Consent/purpose

The holder asked. That is the entire basis on which this data moves: the request answers
an [`offer`](../../offer/0.1/) on its thread, and the purpose is to obtain a credential
bound to a key the holder controls.

An issuer **MUST NOT** treat a verified key-binding proof as authorization. It proves
control of a key; whether that party is *entitled* to the credential is a policy question
this task does not answer, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10
verifying the `proof` establishes who asked and never that they may. The two
authentications in play are easy to collapse and must not be: the envelope proves who
sent the message, the key-binding proof names the key to mint against, and neither is a
statement that the credential should be issued. What evidence an issuer requires before
it is satisfied — and whether a person reviews it — is its own decision, taken outside
this exchange.
