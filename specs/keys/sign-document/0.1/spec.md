---
slug: keys/sign-document
version: "0.1"
title: "Keys — Sign Document"
summary: "A producer obtains a Data Integrity proof (eddsa-jcs-2022) over a JSON document from a key the custodian holds — a credential issued as a group or community DID — with the custodian building the proof and signing only as the DID the document names."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - keys
  - signing
  - data-integrity
  - verifiable-credentials
  - eddsa-jcs-2022
  - issuance
  - custody
parties:
  - role: Producer (the party wanting a proof)
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request asks a custodian to exercise a private key on the producer's say-so, and what
    it obtains is a credential or other document that every verifier will accept as the
    signer DID's own assertion, without ever seeing this exchange. Where the transport does
    not already authenticate the producer, the proof is the only thing standing between the
    key and anyone who can reach the custodian.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Signing acts with the signer DID's authority and produces evidence that outlives the
    exchange. A request that cannot be placed in an acceptance window could be re-presented
    to obtain the same document signed again; SPEC §7.2 item 11 can only absorb a duplicate
    inside a bounded window.
sideEffects:
  level: none
  rationale: >-
    No stored state changes. The custodian reads a key record and the signer DID's document,
    signs, and returns the secured document. The proof is durable and verifiable by anyone
    who resolves the DID, so the operation is not repeatable-without-consequence in the way a
    read is — but nothing at the custodian moves.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: true
  rationale: >-
    The custodian exercises the named key's private half, producing a proof every verifier
    reads as the signer DID's own assertion over the document's content — a credential that
    DID issued, or a document it authenticated as. Nothing is disclosed to the producer beyond
    the proof. What the request carries in is the whole document, and the documents this task
    exists for are credentials about people: a membership credential names its member's DID
    and whatever claims the community makes about them, so the request is declared as
    carrying personal data.
retention:
  class: transient
  rationale: >-
    Nothing about the request needs to survive the exchange except the audit line. That line
    records the document's `id` and `type`, the signer and a digest of the hash data, never
    the document, so it can show which proof a key made without becoming a second store of
    every credential the producer ever issued.
maxDocumentBytes:
  request: 65536
  response: 131072
  rationale: >-
    A credential with a dozen claims and two contexts fits in a few KiB; 64 KiB leaves room
    for a larger credential or a Trust Task document carrying a presentation, without
    inviting the custodian to sign bulk content. The response carries the document again
    with its proof, plus the proof on its own, so it is bounded at twice the request.
errorCodes:
  - code: keys:notFound
    meaning: No key record on this custodian carries the named `keyId` that the producer may name — whether the custodian holds no such key or holds it outside the producer's scope, which this code does not distinguish. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys:invalidArgument
    meaning: A request member is well-formed against the schema but unusable — a `created` outside the custodian's acceptance window around its own clock. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys/sign-document:failedPrecondition
    meaning: The named key exists but cannot sign a document in its current state — its `status` is not `active`, the custodian has bound it to a protocol other than Data Integrity, or the custodian knows of no DID whose document lists it as a verification method.
    retryable: false
  - code: keys/sign-document:keyTypeUnsupported
    meaning: The named key's `keyType` has no cryptosuite this specification defines — anything but `ed25519`.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [keyType]
      properties:
        keyType:
          type: string
  - code: keys/sign-document:proofPresent
    meaning: The document already carries a `proof` member. This task makes a document's only proof; it neither strips an existing proof nor adds to a proof set or chain.
    retryable: false
  - code: keys/sign-document:documentInvalid
    meaning: The document cannot be signed as given — it names its signer in neither `issuer` nor `controller` (`noSigner`), names it in a form this specification does not admit (`signerMalformed`), or holds a value RFC 8785 cannot serialise without changing it, such as a number outside IEEE 754 double precision or a lone surrogate (`notCanonicalizable`).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [reason]
      properties:
        reason:
          type: string
          enum: [noSigner, signerMalformed, notCanonicalizable]
  - code: keys/sign-document:issuerMismatch
    meaning: The document's `issuer` (or `issuer.id`), or its `controller`, names a DID of which the named key is not a verification method. The custodian signs only as the DID the document names, and only with that DID's own key.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member]
      properties:
        member:
          type: string
          enum: [issuer, controller]
  - code: keys/sign-document:purposeNotAuthorized
    meaning: The named key is a verification method of the signer DID, but that DID's document does not list it under the requested `proofPurpose`.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [proofPurpose]
      properties:
        proofPurpose:
          type: string
          enum: [assertionMethod, authentication]
related:
  - keys/sign-jws
  - keys/sign-csr
  - keys/sign
  - keys/derive-and-sign-document
  - keys/show
---

## Abstract

The **Keys — Sign Document** Trust Task lets a *producer* obtain a [W3C Data Integrity](https://www.w3.org/TR/vc-data-integrity/) proof over a JSON document from a key held by a *key custodian*. The producer names the key and sends the document; the custodian checks that the document names, as its `issuer` or `controller`, the DID the key belongs to, that the DID's document lists the key under the requested proof purpose, and then canonicalises the document, builds the proof and returns the document secured with it. The cryptosuite is [`eddsa-jcs-2022`](https://www.w3.org/TR/vc-di-eddsa/#eddsa-jcs-2022).

The case that motivates it is a community server issuing credentials. A community acts as its own DID, and as the DID of each group it runs; the keys behind those DIDs are held by the community's custodian and never leave it. The community issues membership and role credentials as itself, and group credentials as each group, so it needs Data Integrity proofs made with keys it does not hold. [`keys/sign`](../../sign/0.1/spec.md) cannot stand in: a custodian that hardens its general oracle frames every payload under its own signing domain, precisely so that nothing it signs there verifies as a protocol signature — and a Data Integrity proof is one. [`keys/derive-and-sign-document`](../../derive-and-sign-document/0.1/spec.md) cannot either: it signs as a key derived at a path, with no key record and no DID document behind it, which is not what a community's or a group's DID is.

So this task is the narrow way through, a sibling of [`keys/sign-jws`](../../sign-jws/0.1/spec.md) and [`keys/sign-csr`](../../sign-csr/0.1/spec.md). Three properties make it one:

- **The private key never leaves the custodian.** The producer issues credentials without holding the issuer's key, so a compromised producer can issue while its entitlement lasts but never keeps a key.
- **The custodian builds the bytes.** It canonicalises the document, writes the proof configuration — including the cryptosuite and the verification method, which decide how the proof is checked — and hashes both as the cryptosuite defines. The producer never supplies signing input, so a signature obtained here is a Data Integrity proof and nothing else: not a JWS, not a certificate request, not a payload framed by `keys/sign`.
- **The document names its signer, and the custodian holds it to that.** A key signs here only as the DID the document itself says issued or controls it, and only for a purpose that DID's document grants the key. A producer entitled to a group's key cannot use it to sign a credential claiming to come from the community, nor exercise a key the DID published for one purpose under another.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/sign-document/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Send a `document` with no `proof`, naming the DID it is to be signed as in `issuer` (a string, or an object whose `id` is the DID) or in `controller` (a string, or an array of that one DID).
3. Use the returned `document` as it is. The proof covers the RFC 8785 serialisation of every member the producer sent; a producer that adds, removes or re-orders a member's content after signing holds a document that does not verify.

A producer **SHOULD** verify the returned document against the signer DID's document before relying on it or handing it on.

A conforming **consumer** (the key custodian) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether **this producer may obtain document proofs at all** — and refuse with `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) where it may not. See [Authorization](#authorization).
3. Refuse with `keys/sign-document:proofPresent` a `document` that carries a `proof` member, whatever its value. See [The issuer binding](#the-issuer-binding) for why it is refused rather than stripped.
4. Refuse with `keys/sign-document:documentInvalid` a `document` that names its signer in neither `issuer` nor `controller` (`details.reason` `noSigner`); whose `issuer` is neither a DID nor an object whose `id` is a DID, or whose `controller` is neither a DID nor an array of exactly one DID (`signerMalformed` — a DID URL with a path, query or fragment is not a DID); or that RFC 8785 cannot serialise exactly (`notCanonicalizable`). It **MUST NOT** round, re-encode or otherwise repair a value to make it pass: a repaired credential is a credential the producer did not write. The schema leaves `document` an open object so that it is carried whole, which is why these rules are the custodian's to check.
5. Refuse with `keys:notFound` a `keyId` it holds no record for **and** one it holds outside the scope this producer may act in. The two **MUST** be indistinguishable — same code, same message, no difference in timing a custodian can avoid — so that the task cannot be used to learn which keys exist in another's scope.
6. Refuse with `keys/sign-document:failedPrecondition` a key whose `status` is not `active`, one it has bound to a protocol other than Data Integrity, and one it knows of no DID document listing as a verification method.
7. Refuse with `keys/sign-document:keyTypeUnsupported` a key whose `keyType` is not `ed25519`. See [Cryptosuite](#cryptosuite).
8. Apply [The issuer binding](#the-issuer-binding), refusing with `keys/sign-document:issuerMismatch` — naming the member in `details.member` — a document whose `issuer` or `controller` names any DID of which the key is not a verification method.
9. Refuse with `keys/sign-document:purposeNotAuthorized` a key that the signer DID's document does not list under `proofPurpose` (absent: `assertionMethod`).
10. Refuse with `keys:invalidArgument` a `created` outside its acceptance window around its own clock. That window **SHOULD** be no wider than five minutes either side.
11. Build the proof **itself**, exactly as in [The proof](#the-proof), and sign that hash data. It **MUST NOT** sign any byte string other than `eddsa-jcs-2022` hash data it has computed from the request's document and the proof configuration it built, and **MUST NOT** prefix, frame or otherwise transform it — a signature over anything else does not verify as a Data Integrity proof.
12. Return the secured document, the proof on its own, the signer DID and the echoed `keyId` under the `#response` variant, and **never** the private key or any encoding of it.

The order puts every check that needs nothing but the request before the key is looked up, and every check that reads the key's DID document after it, so that no refusal says anything about a key the producer could not name.

### The issuer binding

The custodian signs only as a DID the document names, and only with that DID's own key. Concretely, with *K* the named key:

- **The signer DID** is the DID the document names: `issuer` when it is a string, `issuer.id` when `issuer` is an object, and otherwise `controller` (or its one element). When both `issuer` and `controller` are present, both **MUST** name the same DID; a document whose two members disagree is refused with `issuerMismatch` naming `controller`.
- *K* **MUST** be a verification method of the signer DID: the signer DID's document lists, in `verificationMethod` or embedded in a verification relationship, a method whose public key is *K*'s. Otherwise the request is refused with `issuerMismatch` naming the member that named the DID.
- The comparison is between DIDs as strings, exactly. The custodian **MUST NOT** normalise case, percent-encoding or method-specific forms, or treat two DIDs as equal because they resolve to related documents: the DID a verifier sees in `issuer` is the DID the proof's `verificationMethod` must belong to, character for character.
- The signer DID's document is one the custodian **MUST** obtain without the producer's help — the document it publishes itself, for a DID whose log or document it maintains, or one it resolves. It **MUST NOT** take a DID document, a verification method or a key-to-DID association from the request.

**A document with neither `issuer` nor `controller` is not signed.** It is refused as `documentInvalid` (`noSigner`) before any key is looked up. This task secures credentials and other documents that say who signs them; a document that does not cannot be checked against its signer, and signing it would make the custodian a Data Integrity oracle over arbitrary JSON for any key the producer can name — the very thing this task is narrower than. A presentation, whose signer is its `holder`, is likewise out of scope: presenting is a holder's act with tasks of its own.

**A document that already carries `proof` is not signed either.** [`keys/derive-and-sign-document`](../../derive-and-sign-document/0.1/spec.md) strips an existing proof; this task refuses it instead, for two reasons. Stripping silently discards somebody's signature — the failure that specification itself warns about. And extending one into a proof set or chain would mean signing a document whose existing proof the custodian has not verified, under an `issuer` that proof may have been made for: a set's members are each an assertion about the same document, and a custodian adding to a set it has not checked lends its key to whatever the first proof claims. A later version may define proof sets with that verification as a precondition.

### Proof purpose

`proofPurpose` is `assertionMethod` (the default) or `authentication`, carried verbatim into the proof. `assertionMethod` is what an issuer signs a credential under. `authentication` is for a document the DID presents as itself — a Trust Task document the DID sends, an authentication response — and is where `challenge` and `domain` ordinarily appear.

`capabilityInvocation` and `capabilityDelegation` are excluded, and the schema refuses them. Those relationships carry a DID's authority over its own document and over capabilities it delegates; a proof made under them through a task whose purpose is document signing would let a producer entitled to issue as a DID act as its controller. `keyAgreement` keys sign nothing.

### Cryptosuite

This version defines one cryptosuite, `eddsa-jcs-2022`, and so one key type, `ed25519`. The producer cannot name a cryptosuite: as with `alg` in [`keys/sign-jws`](../../sign-jws/0.1/spec.md#choosing-alg), the key decides it, and a member that let the producer say otherwise could only be checked against what the custodian was going to do anyway, or not checked.

`ecdsa-jcs-2019` (P-256) is **deliberately not included**. The P-256 keys custodians hold exist for protocols whose wire fixes the curve — a verifier identity whose request objects a wallet accepts only as ES256, an EUDI access certificate, the holder key a foreign SD-JWT-VC or mdoc credential binds to — and each is bound to that protocol: signing JWS, a certificate request, a key-binding JWT or a device authentication, and the presentation or receipt proofs that holding the credential needs, through the tasks that build them. None of them is the key of a DID that issues credentials. Admitting P-256 here would give every such key a second protocol, which is the cross-protocol use [`keys/sign-jws`](../../sign-jws/0.1/spec.md#correlation) tells a custodian to refuse, and would buy nothing the issuers this task serves need. A version that adds `ecdsa-jcs-2019` for an issuer that genuinely holds a P-256 assertion key is an additive change; this one found none.

### The proof

The custodian secures the document as [`eddsa-jcs-2022`](https://www.w3.org/TR/vc-di-eddsa/#eddsa-jcs-2022) defines, with these choices fixed:

1. **Proof configuration.** An object with `type` `DataIntegrityProof`, `cryptosuite` `eddsa-jcs-2022`, `created` (the request's, or the custodian's clock as an XML Schema `dateTimeStamp` in UTC), `verificationMethod` (the DID URL of *K*'s verification method in the signer DID's document — chosen by the custodian; where the document lists *K* under more than one identifier, the first listed under `proofPurpose`), `proofPurpose`, and `challenge` and `domain` exactly as the request gives them, when it gives them. When the document has `@context`, the configuration's `@context` is that value, as the cryptosuite requires.
2. **Hash data.** `SHA-256(JCS(proof configuration)) || SHA-256(JCS(document))`, where JCS is [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) and the document is the request's `document` as received.
3. **Signature.** Ed25519 over the 64-byte hash data, with no domain tag, length prefix or other framing: the cryptosuite's hash data is itself the domain separation.
4. **Result.** The proof configuration with `proofValue` added — the signature, base58btc multibase-encoded — attached to the document as `proof`.

The custodian adds no member to the document and checks none of its claims: whether a credential carries `validFrom`, `validUntil`, `credentialStatus` or a subject is the producer's and its credential profile's business. It does read the document's `issuer` and `controller`, because those say whose key may sign it.

### Audit

A custodian **SHOULD** record each request — producer, key, signer DID, verification method, `proofPurpose`, the document's `id` and `type` where present, and the SHA-256 digest of the hash data — in an audit trail, and **MUST NOT** write the rest of the document into it. A secured credential is durable and verifiable by anyone; a custodian that cannot say afterwards who asked for it has lost the only record that distinguishes authorized issuance from compromise. The `id`, the `type` and the digest are enough to match a credential seen later to the request that produced it; the claims — about the members of a community — are the producer's record to keep, not the custodian's.

## Authorization

The authority is **standing over the key's scope**: the custodian has recorded this producer as entitled to act on the scope the key belongs to, and the key is one the producer may name. A producer without it is refused with the framework's `permissionDenied`, or, where it holds the capability but not over this key, with the `keys:notFound` of [Conformance](#conformance) item 3.

Where that entitlement is expressed as a device capability, the registered value is **`signDocument`** (`sign-document` in the `0.1` casing) — see `Capability` in [`device/_shared`](../../../device/_shared/0.2/device-binding.schema.json). Granting `signDocument` confers no other capability. A custodian:

- **MUST NOT** require `sign` for this task. Issuing credentials should not need the general oracle, and requiring it would force every issuer to hold the grant this task exists to avoid.
- **MUST NOT** accept `sign` in place of `signDocument`. A custodian that frames `keys/sign` payloads under its own signing domain has made `sign` a grant that cannot yield a Data Integrity proof, so accepting it here would silently widen every existing `sign` grant into the one capability the framing was introduced to withhold — the reasoning [`keys/sign-jws`](../../sign-jws/0.1/spec.md#authorization) and [`keys/sign-csr`](../../sign-csr/0.1/spec.md#authorization) give. And unlike those tasks, this one adds checks a `sign` holder never faced: the issuer binding and the proof-purpose check. Even at a custodian that signs `keys/sign` payloads verbatim, a `sign` grant was given without either, so it is not evidence that its holder may issue as a DID. An operator who wants a producer to issue grants `signDocument`, and can withdraw it without withdrawing anything else.

Entitlement is necessary and not sufficient. The issuer binding and the purpose check are not authorization — they constrain *what* an entitled producer can obtain, and a producer cannot satisfy them by presenting more identity. A custodian **MAY** further restrict:

- **the key's purpose** — see [Correlation](#correlation): a key bound to Data Integrity is refused by every other signing task;
- **the document** — for instance to the credential types a key was provisioned to issue, by its `type`;
- **the rate** — a quota per key bounds what a compromised producer can issue before its entitlement is withdrawn.

Verifying the `issuer`, the transport identity or the `proof` of the request establishes *who* asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may. Nor does the custodian vouch for the document's claims: the proof says the signer DID asserted them, and whether the DID should have is the producer's responsibility and, after it, the verifier's judgement.

## Definitions

* **Producer.** The party requesting the proof; identified by `issuer`. In the motivating case, a community server issuing credentials as itself and as its groups.
* **Key custodian.** The party holding the private key and performing the signing; identified by `recipient`.
* **`keyId`.** The key to sign with, as described by [`keys/show`](../../show/0.1/spec.md). Chosen by the producer from the keys it is entitled to.
* **`document`.** The JSON document to secure, without a `proof`.
* **Signer DID.** The DID the document names as its `issuer` or `controller`, and the DID the proof's verification method belongs to.
* **Hash data.** The 64 bytes `eddsa-jcs-2022` signs: the SHA-256 of the canonical proof configuration followed by the SHA-256 of the canonical document.

## Request

The producer sends the request to the custodian; its payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A membership credential, issued as the community

A community server asks its custodian to secure a DTG membership credential it is issuing to a member, signed with the community's own key. The credential's `issuer` is the community's DID, which lists `community-ed25519` as `#key-1` under `assertionMethod`.

```json
{
  "id": "urn:uuid:4d2e8f1a-6b3c-4e5d-9f70-8a1b2c3d4e01",
  "type": "https://trusttasks.org/spec/keys/sign-document/0.1",
  "issuer": "did:example:community-server",
  "recipient": "did:example:community-custodian",
  "issuedAt": "2026-10-18T09:00:00Z",
  "payload": {
    "keyId": "community-ed25519",
    "document": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://firstperson.network/credentials/dtg/v1"
      ],
      "id": "urn:uuid:7c0e9a44-2b1d-4f6e-9a35-1d2c3b4a5e60",
      "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
      "issuer": "did:example:community",
      "validFrom": "2026-10-18T09:00:00Z",
      "validUntil": "2027-10-18T09:00:00Z",
      "credentialSubject": { "id": "did:example:member" }
    },
    "created": "2026-10-18T09:00:01Z"
  }
}
```

A group credential is the same request with the group's key and the group's DID as `issuer`. The issuer binding is what keeps the two apart: the group's key named against a document whose `issuer` is the community is `issuerMismatch`, whatever else the producer is entitled to.

## Response

The custodian answers with the secured document; the payload validates against the sub-schema reachable via `$anchor: "response"`. `document` is the request's document with `proof` added, `proof` is that proof on its own, `signerDid` is the DID it was signed as, and `keyId` is echoed so a response separated from its request is still attributable. Failures (`permissionDenied`, `keys:notFound`, `keys:invalidArgument`, and this specification's own codes) use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### The secured credential

```json
{
  "id": "urn:uuid:4d2e8f1a-6b3c-4e5d-9f70-8a1b2c3d4e02",
  "type": "https://trusttasks.org/spec/keys/sign-document/0.1#response",
  "threadId": "urn:uuid:4d2e8f1a-6b3c-4e5d-9f70-8a1b2c3d4e01",
  "issuer": "did:example:community-custodian",
  "recipient": "did:example:community-server",
  "issuedAt": "2026-10-18T09:00:01Z",
  "payload": {
    "keyId": "community-ed25519",
    "signerDid": "did:example:community",
    "document": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://firstperson.network/credentials/dtg/v1"
      ],
      "id": "urn:uuid:7c0e9a44-2b1d-4f6e-9a35-1d2c3b4a5e60",
      "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
      "issuer": "did:example:community",
      "validFrom": "2026-10-18T09:00:00Z",
      "validUntil": "2027-10-18T09:00:00Z",
      "credentialSubject": { "id": "did:example:member" },
      "proof": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://firstperson.network/credentials/dtg/v1"
        ],
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-10-18T09:00:01Z",
        "verificationMethod": "did:example:community#key-1",
        "proofPurpose": "assertionMethod",
        "proofValue": "z5e9cErCFXraDE8Cs3a46rSRaReyv68UtZz7JmzDUCcnCbFbEfXaUWW1RkAomKidanRrLDWhQqraAEq5XtuwK9zQo"
      }
    },
    "proof": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://firstperson.network/credentials/dtg/v1"
      ],
      "type": "DataIntegrityProof",
      "cryptosuite": "eddsa-jcs-2022",
      "created": "2026-10-18T09:00:01Z",
      "verificationMethod": "did:example:community#key-1",
      "proofPurpose": "assertionMethod",
      "proofValue": "z5e9cErCFXraDE8Cs3a46rSRaReyv68UtZz7JmzDUCcnCbFbEfXaUWW1RkAomKidanRrLDWhQqraAEq5XtuwK9zQo"
    }
  }
}
```

The example is real and checkable. The proof verifies under the Ed25519 public key `z6MkgAnvkP45uNxwCKeNdt6wrYkEjpYX4f7Nrd8MQqFL8Fbn`, the key `did:example:community#key-1` stands for here. Its hash data is the SHA-256 of the RFC 8785 serialisation of the proof configuration (the proof without `proofValue`) followed by the SHA-256 of the credential's, and the SHA-256 of that hash data — what the custodian's audit line records — is `f41709f4624d40e85db4e275ccb0fce4a4c606a73e7900bb153b4284cf9de214`. Ed25519 is deterministic, so a custodian signing the same request with the same `created` produces the same `proofValue`.

### A credential claiming another issuer

The same producer names the group's key against a credential whose `issuer` is the community. It is refused before anything is signed:

```json
{
  "id": "urn:uuid:4d2e8f1a-6b3c-4e5d-9f70-8a1b2c3d4e04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:4d2e8f1a-6b3c-4e5d-9f70-8a1b2c3d4e03",
  "issuer": "did:example:community-custodian",
  "recipient": "did:example:community-server",
  "issuedAt": "2026-10-18T09:00:02Z",
  "payload": {
    "code": "keys/sign-document:issuerMismatch",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/keys/sign-document/0.1",
      "id": "urn:uuid:4d2e8f1a-6b3c-4e5d-9f70-8a1b2c3d4e03"
    },
    "message": "the key is not a verification method of the document's issuer",
    "retryable": false,
    "details": { "member": "issuer" }
  }
}
```

## Security & Privacy

### Data carried

The request carries a key identifier, a document and the proof options; the response carries the document again with its proof, the proof, the signer DID and the echoed `keyId`. No key material travels in either direction. The document is whatever the producer is securing, and for the credentials this task exists for it is personal data: a membership credential names its member's DID and whatever the community asserts about them. The custodian reads the document to canonicalise it and reads two of its members, `issuer` and `controller`, to decide whose key may sign it. It applies no policy to the claims, writes none of them to its audit line, and **SHOULD NOT** keep the document once the response is sent.

The smallest conforming request is a key and a document naming its signer. A producer **SHOULD NOT** put into the document claims its credential profile does not ask for: every member is signed, and becomes part of what the signer DID has asserted to every verifier that ever sees the credential. `challenge` and `domain` are carried into the proof verbatim and are visible to every verifier too.

### Correlation

A proof verifies under the signer DID forever, and every proof one DID makes is linked to every other; for an issuer that is the intent — verifiers recognise the community by its DID — and not a side effect. The credential itself correlates its subject with its issuer wherever it is presented; that is the credential's property, and this task neither adds to it nor can remove it. The custodian-side trace is the sequence of requests: which keys a producer names, how often, and which credential types, which for a community is a record of how many credentials of each kind it issued and when. The digest and the document `id` in the audit line identify one credential, and anyone who later holds the credential can match it to the line.

**Why this is not a general oracle.** The custodian builds every byte it signs: hash data computed by the cryptosuite from a document it canonicalised and a proof configuration it wrote, with a verification method and a cryptosuite it chose. A producer cannot obtain a signature over bytes of its choosing — the input is two SHA-256 digests, of which the first always covers a configuration naming `DataIntegrityProof` and `eddsa-jcs-2022` — so a signature obtained here is a Data Integrity proof and verifies as nothing the other signing tasks produce: not a JWS signing input (ASCII, beginning `eyJ`), not a DER certificate request (beginning `0x30`), not an SSHSIG structure, not a domain-framed `keys/sign` payload. Within Data Integrity, the issuer binding confines a key to the documents that claim its own DID, and the purpose check to the relationships that DID published it for.

**Why the issuer binding.** A Data Integrity verifier checks that the proof's verification method belongs to the DID the document names; it cannot know which keys a custodian holds or which producer asked. Without the binding, a producer entitled to one DID's key could obtain a proof over a document claiming another DID as issuer, and whether that fails depends on every verifier doing the controller check correctly. With it, the custodian refuses the mismatch before signing, where it is cheap and certain, so a producer entitled to a group's key can issue only as that group. The binding also stops a producer from using an issuer's key to sign a document that names no issuer at all, which a careless verifier might accept on the strength of the key alone.

The cross-protocol risk is the one to design for. A custodian **SHOULD** bind each key to the protocol it signs for, **SHOULD** refuse with `keys/sign-document:failedPrecondition` a key it has bound to anything else, and, from the other side, refuse a key used here when it is named in [`keys/sign`](../../sign/0.1/spec.md), [`keys/sign-jws`](../../sign-jws/0.1/spec.md) or any other signing task. The format boundary already holds by construction; the binding is what stops one key from speaking two protocols at all.

Replay has two layers. A replayed *request* is bounded by the REQUIRED `issuedAt` and the consumer's duplicate handling ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)); at worst it yields a second proof over the same document. A replayed *document* is the verifier's problem and the document's: validity periods, status and, for an authentication proof, `challenge` and `domain` are the producer's to include, because the custodian neither adds nor checks them.

### Retention

Transient at the custodian. The operation stores nothing; the audit line is the only thing that survives it. That line **SHOULD** hold producer, key, signer DID, verification method, proof purpose, the document's `id` and `type`, the hash-data digest and time, and **MUST NOT** hold the document's other members, as [Audit](#audit) requires — enough to answer "did this key issue this credential, and at whose request" after a compromise, without the custodian becoming a second store of every credential, and every member's claims, the producer ever issued. It **SHOULD** keep that line as long as credentials it signed may be valid.

### Consent/purpose

The purpose is exercise of an existing custody relationship: a producer already entitled over a key asks the custodian to secure a document as the DID that key belongs to. Part of the purpose is visible to the custodian — the document's `type` says what kind of credential this is and its `issuer` which identity asserts it — so a deployment can limit a key to issuing particular credential types. The claims are collected by the producer for its own purpose, which the credential profile defines, and pass through the custodian only to be signed. Whether a particular issuance also needs a human's approval at the custodian is the custodian's policy, not something this specification declares.
