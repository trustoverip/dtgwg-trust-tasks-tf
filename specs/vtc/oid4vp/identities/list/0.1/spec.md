---
slug: vtc/oid4vp/identities/list
version: "0.1"
title: VTC OpenID4VP — Verifier Identities List
summary: An administrator reads the OpenID4VP verifier identities the community presents to swiyu and EUDI wallets — each one's client identifier, key custody, access certificate and verifier attestations with their expiry — and what, if anything, stops each from serving a wallet.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - oid4vp
  - verifier
  - wallet
  - diagnostics
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The response describes the community's verifier configuration, including where its signing keys are held. The proof's signer is what the consumer checks the administrator's authority against.
sideEffects:
  level: none
  rationale: Reads verifier identity records and configuration; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response carries what every wallet the community sends a request to already sees — client identifiers, DIDs, certificates — together with where each signing key is held and what is misconfigured, which is operational detail for administrators only. It never carries a private key or the identifier of a key a custodian holds."
retention:
  class: transient
  rationale: A read of current configuration, discarded once shown.
errorCodes: []
related:
  - vtc/join-requests/wallet-session/start
  - vtc/join-requests/wallet-session/status
  - vtc/registry/diagnostics
---

## Abstract

The **VTC OpenID4VP — Verifier Identities List** Trust Task lets a community's administrator see the identities the community uses as an [OpenID for Verifiable Presentations 1.0](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) verifier, the identities [`vtc/join-requests/wallet-session/start`](../../../../join-requests/wallet-session/start/0.1/spec.md) signs request objects with.

A verifier identity is installed out of band — a DID registered with an ecosystem, an access certificate from a registrar, trust statements from a registry — and each part of it expires or can be misconfigured on its own schedule. A wallet that refuses a request object says nothing to the community about why. This task gathers, per identity, the facts an administrator needs to see the trouble before an applicant does: the client identifier wallets are shown, where the signing key is held, the certificate's validity period, each attestation's expiry, and a list of problems in a fixed vocabulary.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Producer (administrator)

1. **MUST** send an empty payload; only the framework `ext` point is allowed.
2. **MUST** carry a proof signed by the administrator's DID.

### Consumer (community)

1. **MUST** refuse a signer who does not hold the community's authority to administer its configuration, with the framework's `permissionDenied`.
2. **MUST** return every verifier identity it holds, in a stable order, and **MUST NOT** return a private key, or the identifier under which a remote custodian holds a key, in any member, `ext` included.
3. **MUST** report each problem in the tables below that applies at the time of the request, and **MUST** set `usable` to true exactly when no `error` problem applies to the identity and no `error` other than `defaultIdentityNotInstalled` applies to the community as a whole.
4. **MUST** report `custody` as `agent` when the identity's key is held by the community's agent and every request object is signed there, and `cached` when the community holds it.
5. **MUST** list `attestations` in the order request objects carry them in `verifier_info`, and **SHOULD** give each one's `expiresAt` when it carries an expiry the consumer can read without verifying it against its issuer.
6. **SHOULD** report `certificateExpiresSoon` and `statementExpiresSoon` within a window of its own choosing, long enough for an administrator to obtain a replacement; thirty days is **RECOMMENDED**.
7. **MAY** report a problem's `detail` naming the action that fixes it. `detail` is prose for the administrator and **MUST NOT** be parsed.

### Problems with one identity

| `code` | `severity` | Meaning |
|---|---|---|
| `keyCustodyInvalid` | error | The record names neither a cached key nor one the agent holds, or names both |
| `signerNotLoaded` | error | The identity is recorded but the running maintainer did not load a signer for it — it was installed after start, or did not load |
| `agentLinkMissing` | error | The agent holds the key, and the community has no live link to the agent to sign over |
| `certificateMissing` | error | An `eudi` identity has no access certificate installed, so it has no client identifier |
| `certificateNotYetValid` | error | The access certificate's validity begins after now |
| `certificateExpired` | error | The access certificate's validity ended before now |
| `certificateExpiresSoon` | warning | The access certificate expires within the consumer's warning window |
| `identityStatementMissing` | warning | A `swiyu` identity carries no current identity trust statement; wallets that require one under swiyu Trust Protocol 2.0 will refuse its requests |
| `statementExpired` | warning | An attestation has expired and is left out of every request object |
| `statementExpiresSoon` | warning | An attestation expires within the consumer's warning window |

`statementExpired` is a warning, not an error, because an expired attestation is left out rather than sent: the identity still opens sessions, and whether a wallet accepts a request without it is the wallet's policy. When the attestation that expired is the identity trust statement, `identityStatementMissing` is reported as well.

### Problems with the community's verifier

| `code` | `severity` | Meaning |
|---|---|---|
| `defaultIdentityNotInstalled` | error | The configured default identity names no usable identity the community holds, so a session that names no client identifier prefix is refused; one that names a prefix is not |
| `noPublicUrl` | error | The community has no `https` public URL, so a wallet cannot reach its request or response endpoints |

`noPublicUrl` makes every identity's `usable` false. A community-wide problem is reported once, in the response's own `problems`, not repeated on each identity.

## Authorization

The authority this task assumes is the community's **configuration administration**: the same authority installing or removing a verifier identity takes. The `proof` names the signer; the consumer reads that signer's standing at the time of the request (consumer rule 1). No other party learns anything from this task, and an applicant opening a wallet session learns only that one could not be opened ([`wallet-session/start`](../../../../join-requests/wallet-session/start/0.1/spec.md)), never why.

## Definitions

**Verifier identity** — an identity the community presents to wallets as an OpenID4VP verifier: a key that signs request objects, and the client identifier under which wallets know that key.

**`kind`** — the ecosystem: `swiyu` identities are DIDs registered in the swiyu Base Registry, with `clientId` `decentralized_identifier:<did>`; `eudi` identities are X.509 access certificates, with `clientId` `x509_hash:<hash of the leaf>`.

**`attestations`** — what every request object carries in `verifier_info`:

| `kind` | Meaning |
|---|---|
| `identityTrustStatement` | A swiyu Identity Trust Statement about the identity's DID |
| `protectedVerificationAuthorization` | A swiyu Protected Verification Authorization Trust Statement |
| `registrationCertificate` | An EUDI registration certificate, against which a wallet checks the claims requested |
| `other` | An attestation the consumer does not recognise, carried as installed |

**`isDefault`** — whether this is the identity a session that names no client identifier prefix uses: the configured `defaultIdentity` when one is configured and usable, otherwise — when none is configured — the first usable identity in the response's order. When a configured default is not usable, no identity is the default (`defaultIdentityNotInstalled`).

## Request

The administrator sends the request to the community, signed; the payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:5d2b8e14-7a3c-4f61-9b02-1e6c4a8f3d57",
  "type": "https://trusttasks.org/spec/vtc/oid4vp/identities/list/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-14T08:15:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-14T08:15:00Z",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsLoQ3MLvXa7YhFpDZBWzNTGdWHhPAn6V3iCtwiEgGcj7z6WQvkW4bRjKDsaTnN1xJyL"
  }
}
```

## Response

The community answers with its identities; the payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:9e4c1a73-2b5d-4e86-a0f7-3c8d6b1e5a24",
  "type": "https://trusttasks.org/spec/vtc/oid4vp/identities/list/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-14T08:15:01Z",
  "payload": {
    "identities": [
      {
        "id": "eudi",
        "kind": "eudi",
        "clientId": "x509_hash:Uvo3HtuIxuhC92rShpgqcT3YXwrqRxWEviRiA0OZszk",
        "custody": "agent",
        "usable": true,
        "isDefault": false,
        "certificate": {
          "subject": "CN=Example Community Verifier,O=Example Community,C=DE",
          "issuer": "CN=Example Access CA,O=Example Registrar,C=DE",
          "notBefore": "2026-07-01T00:00:00Z",
          "notAfter": "2026-10-31T00:00:00Z",
          "chainLength": 2
        },
        "attestations": [],
        "problems": [
          {
            "code": "certificateExpiresSoon",
            "severity": "warning",
            "detail": "the access certificate expires 2026-10-31; request a new one with `vtc oid4vp csr`"
          }
        ],
        "installedAt": "2026-07-01T09:30:00Z"
      },
      {
        "id": "swiyu",
        "kind": "swiyu",
        "clientId": "decentralized_identifier:did:webvh:QmVerifierScid:identifier-reg.example:api:v1:did:3f1c",
        "did": "did:webvh:QmVerifierScid:identifier-reg.example:api:v1:did:3f1c",
        "keyId": "did:webvh:QmVerifierScid:identifier-reg.example:api:v1:did:3f1c#verifier-key-1",
        "custody": "cached",
        "usable": true,
        "isDefault": true,
        "attestations": [
          {
            "kind": "identityTrustStatement",
            "expiresAt": "2027-04-01T00:00:00Z"
          }
        ],
        "problems": [],
        "installedAt": "2026-09-20T14:02:11Z"
      }
    ],
    "defaultIdentity": "swiyu",
    "problems": []
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries, per identity, what any wallet the community sends a request object to already sees — its client identifier, DID and verification method, its access certificate's subject and validity, and the attestations in `verifier_info` — together with two things a wallet does not see: **where the signing key is held**, and **what is misconfigured**. Both are operational detail that tells an attacker which part of the community's verifier to target and when it is weakest, which is why this task answers configuration administrators only (consumer rule 1).

It never carries key material, and never the identifier under which the community's agent holds a key (consumer rule 2): that identifier names the key to the custodian's signing interface, and an administrator who needs it reads it from the agent.

`detail` is maintainer-authored prose. It may name a command, a file or a date, and **MUST NOT** be parsed; a consumer decides from `code` and `severity`.

### Correlation

The identities described are public by design: the community presents them to every wallet it asks for a presentation, and a swiyu identity is registered in a public registry. Nothing in this task concerns an applicant or a holder.

Both parties declare `identifierScope: public`. The administrator's identifier is the one their entry in the community's access-control list names, and the consumer's authorization check (consumer rule 1) is a lookup of exactly that identifier: a pairwise one, never seen before, would hold no authority and be refused. The community answers under its public identifier because the administrator must recognise the party describing the verifier configuration as their own community; a pairwise community identifier would leave them unable to tell its answer from another party's.

### Retention

Transient. The response describes configuration at the time of the request and is discarded once shown. A console that polls it keeps only the latest answer.

### Consent/purpose

The purpose is an administrator keeping the community's verifier serviceable: seeing an expiring certificate or statement before wallets start refusing requests, and seeing why sessions are refused when they are. It confers no authority and changes nothing.
