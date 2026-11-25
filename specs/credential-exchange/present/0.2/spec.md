---
slug: credential-exchange/present
version: "0.2"
title: Credential Exchange — Present
summary: Holder to verifier — an OID4VP vp_token, keyed by credential query id, disclosing exactly the consented claims, bound to the verifier's nonce and audience.
status: draft
targetFrameworkVersion: "0.6.0"
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
    identifierScope: pairwise
  - role: Verifier
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The `vp_token` already carries holder binding over the verifier's nonce and audience, and that remains the proof that establishes the presentation — an envelope proof does not strengthen it. The envelope proof is required for a different reason. Execution exercises the subject's own authority and the response discloses secret material the caller retains, so the exchange is one a third party may later be asked to rely on, which is the case §4.7.1 makes a MUST. Without a document proof the transaction as a whole is repudiable even though the presentation inside it is not, and an intermediary can relay a genuine `vp_token` under an envelope of its own.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A presentation carries the subject's credential material to a verifier under the subject's authority, with no response document to close the exchange. A presentation that cannot be dated is one a verifier can forward indefinitely.
sideEffects:
  level: none
  rationale: Presenting asserts; it does not mutate the verifier. Any record the verifier keeps is its own.
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: true
  rationale: "The body is a disclosure of the holder's own claims to a named audience. It is the point at which private attributes leave the wallet, and it cannot be undone. Read from the verifier's side the same member is an ingest: the `vp_token` carries attributes of an identifiable natural person into the verifier, which is what the verifier then has to protect and minimise."
retention:
  class: durable
  rationale: "Durable as a matter of fact rather than entitlement. The claims are in the verifier's hands and cannot be recalled, so the effect of this document outlives any exchange whatever the verifier's policy says; and where the verifier relies on the presentation it retains the `vp_token` itself as the evidence that the reliance was justified — deleting it leaves an audit with a decision and no basis for it. The stated `purpose` from the query is the only limit on that retention, and it is a limit on *reuse*, not a deletion schedule."
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

The **Credential Exchange — Present** Trust Task answers a [query](../../query/0.1/) with an [OID4VP](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) `vp_token` — a selectively-disclosed, holder-bound presentation of each matched credential.

## Changes from 0.1

This version settles the two readings [issue #1358](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/1358) found in `0.1`.

- **Token shape.** `vp_token` is the OID4VP 1.0 response to a DCQL query ([OID4VP §8.1](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.1)): a JSON object keyed by credential query `id`, each value an array of one or more presentations. `0.1` defined a bare string or a bare Data Integrity presentation object, and its schema could not tell a keyed object from a presentation. A bare SD-JWT-VC string is still accepted, for compatibility with `0.1` producers; a bare Data Integrity presentation object is not.
- **Where a Data Integrity presentation carries the nonce and audience.** `0.1` said "the proof's `nonce`", which was a wording slip for the proof's `challenge`, never a separate placement. A consumer now accepts the verifier's nonce and audience either as the holder proof's `challenge` and `domain` ([OID4VP §14.1.2](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-14.1.2)) or as `nonce` and `domain` members of the presentation the holder proof signs; where both are present the values must agree. A producer should use the proof's members. [`vtc/directory/card/present`](../../../vtc/directory/card/present/0.1/) states the same rule.

The schema narrows what `0.1` accepted as an object, so this is a new version rather than an in-place edit, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## The `vp_token`

`vp_token` takes one of two shapes:

- **Keyed (canonical).** A JSON object whose every member name is the `id` of a credential query in the query's `dcql_query`, and whose value is an array of one or more presentations answering that credential query — as in [OID4VP §8.1](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.1). A producer **MUST** use this shape. A consumer **MUST** refuse a key that names no credential query of the query being answered.
- **Bare string (compatibility).** A JSON string holding a single SD-JWT-VC presentation, the shape `0.1` producers send. A consumer **SHOULD** accept it, as an answer to the query's only credential query; a producer **MUST NOT** send it.

Within the array each presentation is a JSON **string** or a JSON **object**, and a consumer selects its verification path from the value's type:

- **String** — an SD-JWT-VC presentation: exactly the consented disclosures, plus a mandatory key-binding JWT whose `nonce` is the verifier's nonce and whose `aud` is the verifier's identifier.
- **Object** — a W3C Verifiable Presentation secured with a Data Integrity proof by its holder, bound as [below](#holder-binding-is-mandatory).

## Format-agnostic, with an honest asymmetry

The two presentation formats are not equivalent in what they can withhold, and a specification that glossed over it would be misleading. Plain `eddsa-jcs-2022` has **no claim-level selective disclosure**: presenting a credential that way discloses all of it. A holder MUST therefore refuse to present on that path unless the credential's claims are a subset of what was consented to. The alternative — disclosing more than was consented to because the format could not do better — is the failure this rule exists to prevent, and it is silent unless checked for.

## Holder binding is mandatory

Every presentation binds the verifier's `nonce` (freshness) and audience — the verifier's identifier, which is this document's `recipient`.

- **SD-JWT-VC.** The key-binding JWT carries `nonce` equal to the verifier's nonce and `aud` equal to the audience.
- **Data Integrity.** The nonce and the audience have two accepted placements:
  - the holder proof's `challenge` (the nonce) and `domain` (the audience), as [OID4VP §14.1.2](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-14.1.2) places them; or
  - `nonce` and `domain` members of the presentation itself, which the holder proof signs.

  A consumer **MUST** accept either placement, for each of the two values independently. Where a value is stated in both places, the two **MUST** agree, and a consumer **MUST** refuse the presentation when they do not. A consumer **MUST** refuse a presentation that states the value in neither place. A presentation carrying more than one holder proof (a proof set) is refused when any proof states a `challenge` or `domain` that disagrees with another proof or with the presentation's own member. A producer **SHOULD** use the proof's `challenge` and `domain`, so that a verifier following OID4VP alone accepts it; during the transition it **MAY** state the same values on the presentation as well. Either way the value is signed: the proof's `challenge` and `domain` are proof options, covered by the proof, and the presentation's members are covered as part of the signed document. A value outside the signed material binds nothing, and a consumer **MUST NOT** read one from there.

  The proof's own `nonce` member ([VC Data Integrity §2.1](https://www.w3.org/TR/vc-data-integrity/#proofs)) is a value the proof creator supplies, not the verifier's nonce, and is not a placement for it.

Both bindings are load-bearing and neither substitutes for the other. Without the nonce a presentation can be replayed later; without the audience a presentation captured by one verifier can be forwarded to another and accepted. A consumer MUST verify both, for every presentation in the `vp_token`, and reject with `staleNonce` or `audienceMismatch` accordingly.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

Producer: send a keyed `vp_token` answering the query's credential queries by `id`. Disclose exactly the consented claims and no more. Bind the verifier's nonce and audience in every presentation, for Data Integrity preferably in the holder proof's `challenge` and `domain`. On a format without claim-level selective disclosure, present only if the whole credential is within the consented set — otherwise refuse. Reply on the query's thread; for a deferred query, present on the original thread once approval lands.

Consumer: accept the keyed shape, and the bare SD-JWT-VC string for compatibility. Verify holder binding, freshness and audience — in either Data Integrity placement, refusing values that disagree — before reading any claim. Verify the underlying credential's issuer signature and status. A presentation that verifies cryptographically may still be a credential you should not accept — signature validity is not the same question as issuer trust.

## Examples

### SD-JWT-VC

The verifier `did:example:verifier` queried with credential query `id` `pid` and nonce `n-0S6_WzA2Mj`. The key-binding JWT (abbreviated) carries that `nonce` and `aud` `did:example:verifier`.

```json
{
  "id": "urn:uuid:1f0c2a7e-6c1d-4f43-9b6e-3d2a5c7e0b01",
  "type": "https://trusttasks.org/spec/credential-exchange/present/0.2",
  "issuer": "did:example:holder-pairwise-7",
  "recipient": "did:example:verifier",
  "issuedAt": "2026-11-24T09:00:00Z",
  "threadId": "urn:uuid:1f0c2a7e-6c1d-4f43-9b6e-3d2a5c7e0b00",
  "payload": {
    "vp_token": {
      "pid": [
        "eyJhbGciOiJFUzI1NiIsInR5cCI6ImRjK3NkLWp3dCJ9.example-issuer-signed-jwt~WyJzYWx0IiwiZ2l2ZW5fbmFtZSIsIkVyaWthIl0~eyJhbGciOiJFUzI1NiIsInR5cCI6ImtiK2p3dCJ9.example-kb-jwt"
      ]
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:holder-pairwise-7#key-1",
    "created": "2026-11-24T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Data Integrity

The same query answered with a Data Integrity presentation for credential query `id` `membership`. The holder proof carries the verifier's nonce as `challenge` and its identifier as `domain`.

```json
{
  "id": "urn:uuid:1f0c2a7e-6c1d-4f43-9b6e-3d2a5c7e0b02",
  "type": "https://trusttasks.org/spec/credential-exchange/present/0.2",
  "issuer": "did:example:holder-pairwise-7",
  "recipient": "did:example:verifier",
  "issuedAt": "2026-11-24T09:00:00Z",
  "threadId": "urn:uuid:1f0c2a7e-6c1d-4f43-9b6e-3d2a5c7e0b00",
  "payload": {
    "vp_token": {
      "membership": [
        {
          "@context": [
            "https://www.w3.org/ns/credentials/v2"
          ],
          "type": [
            "VerifiablePresentation"
          ],
          "holder": "did:example:holder-pairwise-7",
          "verifiableCredential": [
            {
              "@context": [
                "https://www.w3.org/ns/credentials/v2"
              ],
              "type": [
                "VerifiableCredential",
                "MembershipCredential"
              ],
              "issuer": "did:example:community",
              "validFrom": "2026-01-01T00:00:00Z",
              "credentialSubject": {
                "id": "did:example:holder-pairwise-7",
                "memberOf": "did:example:community"
              },
              "proof": {
                "type": "DataIntegrityProof",
                "cryptosuite": "eddsa-jcs-2022",
                "verificationMethod": "did:example:community#key-1",
                "created": "2026-01-01T00:00:00Z",
                "proofPurpose": "assertionMethod",
                "proofValue": "z4ExampleIssuerProofValueNotReal"
              }
            }
          ],
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "verificationMethod": "did:example:holder-pairwise-7#key-1",
            "created": "2026-11-24T09:00:00Z",
            "proofPurpose": "authentication",
            "challenge": "n-0S6_WzA2Mj",
            "domain": "did:example:verifier",
            "proofValue": "z5ExampleHolderProofValueNotReal"
          }
        }
      ]
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:holder-pairwise-7#key-1",
    "created": "2026-11-24T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

A presentation that also states `"nonce": "n-0S6_WzA2Mj"` and `"domain": "did:example:verifier"` beside `holder` is accepted too; one whose members disagree with the proof's `challenge` or `domain` is refused.

## Authorization

The authorization evidence this task presupposes is **holder binding** — that the presenting party controls the credential it presents — together with the audience and freshness bindings the verifier checks before reading any claim.

The specification already states the principle this section names: *a presentation that verifies cryptographically may still be a credential you should not accept — signature validity is not the same question as issuer trust.* Holder binding establishes who is presenting; whether the underlying credential's issuer is one the verifier trusts, and whether the disclosure serves a purpose it will honour, are separate decisions the verifier makes under its own policy.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

### Data carried

One member, and it is the disclosure itself. `vp_token` carries the holder's own claims
to a named audience, one array of presentations per credential query answered: each an
SD-JWT-VC presentation consisting of exactly the consented disclosures plus a key-binding
JWT over the verifier's nonce and audience, or a W3C Data-Integrity VP whose holder proof
signs the same nonce and audience. This is the
moment private attributes leave the wallet, and every other task in this family exists
to make sure it happens only when it should.

The two shapes are not equivalent in what they can withhold, and the asymmetry is a
data-minimisation problem rather than a formatting one. Plain `eddsa-jcs-2022` has no
claim-level selective disclosure: presenting a credential that way discloses all of it,
including claims the query never asked for and the approver never saw. A holder
**MUST** refuse to present on that path unless the credential's claims are a subset of
what was consented to. Nothing in the wire format signals the over-disclosure — the
presentation verifies perfectly — so this check is the only thing standing between a
consented request and a silent release of everything the credential happens to contain.

A holder **MUST NOT** put anything in `ext` alongside a presentation. The verifier is
about to store this document as evidence, and material that rides in beside the
`vp_token` is retained on the strength of a consent decision that was made about the
claims, not about it.

### Correlation

The verifier's identifier is bound into the presentation and is therefore load-bearing:
the key-binding JWT's audience, or the Data-Integrity `domain` the holder proof signs, is what stops a
presentation captured by one verifier being forwarded to another and accepted. That
binding only works if the holder and the verifier mean the same identifier, and if the
verifier can be recognised as the same party the query came from and the trust list
names — which is why the Verifier party declares `identifierScope: public`. A pairwise
verifier identifier would leave audience binding checking a value with no external
meaning.

The Holder declares `identifierScope: pairwise` for the opposite reason, and it is the
strongest privacy lever available in this task. Holder binding proves control of a key;
it does not require that key, or the DID naming it, be the same one shown to the last
verifier. A holder that reuses one identifier across verifiers hands every one of them
a join key, and colluding verifiers can then assemble a disclosure history the holder
never consented to as a whole. Pairwise identifiers make that join unavailable — but
only up to the credential itself: an underlying credential whose `credentialSubject.id`
or `cnf` is stable across presentations reintroduces the linkage inside the very token
the pairwise envelope was protecting, and the holder cannot fix that here. It has to be
fixed at issuance.

What remains joinable regardless is the disclosure itself. Claims are correlating data
by nature — a date of birth and a postcode narrow a population sharply — so the subset
rule above is a correlation control as much as a minimisation one.

### Retention

The verifier holds what it received, permanently, whatever anyone's policy says: there
is no revocation of a disclosure. `retention.class` is `durable` because that is the
honest description of the effect, not because the specification grants an entitlement.
Where a verifier relies on the presentation it will keep the `vp_token` as the evidence
that the reliance was justified; deleting it leaves an audit trail with a decision and
no basis for it.

The stated `purpose` carried by the [query](../../query/0.1/) is the limit that applies,
and it limits *reuse* rather than setting a deletion date. A verifier **MUST NOT**
retain more than its stated purpose requires, and **MUST NOT** repurpose what it holds:
the holder consented to a disclosure for a reason, and the purpose binding is the only
record of what that reason was. A verifier that needed only to check a predicate — over
eighteen, resident in this jurisdiction — **SHOULD** record the predicate's outcome and
not the claims that established it.

### Consent/purpose

The basis for this document is a decision already taken: either the holder's policy
pre-trusted the verifier and auto-consented, or a human answered
[`pending/approve`](../../pending/approve/0.1/). By the time a `present` exists the
consent question is settled, which is why this task has no consent machinery of its own
and why it carries no `purpose` member — the purpose lives on the query it answers, and
duplicating it here would create a second, editable copy of the thing the decision was
made against.

The rule that follows is the subset rule: the holder discloses exactly the consented
claims and no more. A presentation that verifies cryptographically may still be one a
verifier should not accept, and equally a presentation the holder can technically mint
may be one it should not send. Whether a verifier's stated purpose justifies the claims
it asked for is a judgement made before this task runs, at the holder's policy or in
front of a person; this specification describes what moves once that judgement is made
and takes no position on how it should be reached.
