---
slug: keys/sign-csr
version: "0.1"
title: "Keys — Sign CSR"
summary: "A producer obtains a PKCS#10 certificate signing request for a key the custodian holds — so a certificate authority can certify a key that never leaves the custodian — with the custodian building the request from structured subject attributes and the key's own public key."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - keys
  - signing
  - csr
  - pkcs10
  - x509
  - certificate
  - eudi
  - custody
parties:
  - role: Producer (the party wanting a certificate request)
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request asks a custodian to exercise a private key on the producer's say-so, and the
    certificate request it obtains is the proof of possession a certificate authority relies
    on to bind an identity to that key — a binding every relying party on the certificate
    then trusts. Where the transport does not already authenticate the producer, the proof is
    the only thing standing between the key and anyone who can reach the custodian.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Signing acts with the key holder's authority and produces evidence that outlives the
    exchange: a CA may act on a certificate request long after it was made. A request that
    cannot be placed in an acceptance window could be re-presented to obtain further
    certificate requests for the key; SPEC §7.2 item 11 can only absorb a duplicate inside a
    bounded window.
sideEffects:
  level: none
  rationale: >-
    No stored state changes. The custodian reads a key record, builds and signs the request,
    and returns it. The request is durable — a CA may certify the key on the strength of it —
    so the operation is not repeatable-without-consequence in the way a read is, but nothing
    at the custodian moves.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: true
  rationale: >-
    The custodian exercises the named key's private half, producing a signature that proves
    possession of that key to a certificate authority. Nothing is disclosed to the producer
    beyond the request and the public key it already could read. What the request carries in
    is the identity to be certified — an organisation's name and registration identifier for
    a relying-party access certificate, but a `CN` or `emailAddress` can as easily name a
    natural person — so the request is declared as carrying personal data.
retention:
  class: transient
  rationale: >-
    Nothing about the request needs to survive the exchange except the audit line. That line
    keeps the subject and alternative names, because they are what the key was made to
    vouch for and the one thing an operator needs to answer "who asked for a certificate for
    this key, and in whose name".
errorCodes:
  - code: keys:notFound
    meaning: No key record on this custodian carries the named `keyId`. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys/sign-csr:failedPrecondition
    meaning: The named key exists but cannot sign a certificate request in its current state — its `status` is not `active`, or the custodian has bound it to a protocol that does not use certificates.
    retryable: false
  - code: keys/sign-csr:keyTypeUnsupported
    meaning: The named key's `keyType` has no certificate-request signature this specification defines — anything but `p256` and `ed25519`.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [keyType]
      properties:
        keyType:
          type: string
  - code: keys/sign-csr:subjectInvalid
    meaning: A `subject` attribute or a `subjectAltNames` entry is well-formed against the schema but breaks a rule of this specification — a value outside its attribute's character set or length, or a control character.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member]
      properties:
        member:
          type: string
          enum: [subject, subjectAltNames]
        index:
          type: integer
          minimum: 0
        reason:
          type: string
          maxLength: 256
related:
  - keys/sign-jws
  - keys/sign
  - keys/sign-sshsig
  - keys/show
  - keys/create
---

## Abstract

The **Keys — Sign CSR** Trust Task lets a *producer* obtain a [PKCS#10](https://www.rfc-editor.org/rfc/rfc2986) certificate signing request for a key held by a *key custodian*. The producer names the subject as a list of typed attributes and, optionally, the subject alternative names; the custodian builds the request from those and the key's own public key, signs it with the key, and returns it as PEM. The producer takes it to a certificate authority, which certifies the key without the key ever leaving the custodian.

The case that motivates it is a community server acting as an [OpenID4VP](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) verifier for EUDI wallets. Those wallets identify a verifier by an X.509 *access certificate*, issued by a registrar's CA for the verifier's own key; the verifier then signs its request objects with that key ([`keys/sign-jws`](../../sign-jws/0.1/spec.md)) and carries the certificate in `x5c`. The CA needs a certificate request signed by the key it certifies — a proof of possession — and [`keys/sign`](../../sign/0.1/spec.md) cannot produce one: a custodian that frames its general oracle's payloads under its own signing domain makes sure nothing signed there verifies as anything else, a certificate request included.

So the custodian builds the request itself. Two properties follow:

- **The private key never leaves the custodian**, before or after certification.
- **The custodian signs only what it built.** The signed bytes are a DER `CertificationRequestInfo` whose public key is the key's own and whose subject is assembled from a closed set of attribute types. The producer cannot hand in a DER blob, a distinguished-name string or an extension, so a signature obtained here is a certificate request for this key and nothing else.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/sign-csr/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Give `subject` in encoded order — element 0 is the first RDN of the DER `RDNSequence`, conventionally `C` — which is the reverse of the RFC 4514 string a tool prints.
3. Confirm, before submitting the request to a CA, that `publicKey` in the response is the public key of the key it meant to certify.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether **this producer may obtain a certificate request for this key** — and refuse with `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) where it may not. See [Authorization](#authorization).
3. Refuse a `keyId` it holds no record for with `keys:notFound`, and a key whose `status` is not `active` with `keys/sign-csr:failedPrecondition`.
4. Refuse with `keys/sign-csr:keyTypeUnsupported` a key whose `keyType` is not `p256` or `ed25519`.
5. Check every `subject` attribute against [Subject attributes](#subject-attributes) and every `subjectAltNames` entry against the schema and the rules below, and refuse with `keys/sign-csr:subjectInvalid` — naming the member and, for an array, the index — any that breaks one. It **MUST NOT** normalise, trim, re-case or otherwise repair a value to make it pass: a certificate names what the request named, and a repaired name is a name the producer did not ask for.
6. Build the request **itself**, exactly as in [The certificate request](#the-certificate-request), and sign the DER encoding of its `CertificationRequestInfo`. It **MUST NOT** sign any other byte string, and **MUST NOT** place in the request anything the producer supplied other than the subject attributes and alternative names this specification defines.
7. Return the request as PEM, and the key's public half as `KeyRecord.publicKey` carries it, under the `#response` variant — and **never** the private key or any encoding of it.

A custodian **SHOULD** record each request — producer, key, subject, alternative names, and the SHA-256 digest of the DER `CertificationRequestInfo` — in an audit trail. A certificate request is the evidence on which a CA binds an identity to the key; a custodian that cannot say afterwards who asked for one, and in whose name, cannot tell a legitimate certificate for its key from one obtained by a compromised producer.

### Subject attributes

Each element of `subject` becomes one single-valued RDN. The set of types is closed, because each one's encoding is fixed here; a type a CA needs that is not listed is a change to this specification, not something to pass through by OID.

| `type` | Attribute (OID) | Encoded as | Value rule |
|---|---|---|---|
| `C` | countryName (2.5.4.6) | PrintableString | Exactly two upper-case letters: an ISO 3166-1 alpha-2 code |
| `ST` | stateOrProvinceName (2.5.4.8) | UTF8String | 1–128 characters |
| `L` | localityName (2.5.4.7) | UTF8String | 1–128 characters |
| `O` | organizationName (2.5.4.10) | UTF8String | 1–64 characters |
| `OU` | organizationalUnitName (2.5.4.11) | UTF8String | 1–64 characters |
| `organizationIdentifier` | organizationIdentifier (2.5.4.97) | UTF8String | 1–128 characters; for an EU legal person, the ETSI EN 319 412-1 form (`VATDE-…`, `LEIXG-…`) |
| `CN` | commonName (2.5.4.3) | UTF8String | 1–64 characters |
| `serialNumber` | serialNumber (2.5.4.5) | PrintableString | 1–64 characters from the PrintableString set (`A–Z a–z 0–9`, space, `' ( ) + , - . / : = ?`) |
| `emailAddress` | emailAddress (1.2.840.113549.1.9.1) | IA5String | 1–255 characters; an ASCII mailbox, as `subjectAltNames.email` |

Lengths are counted in Unicode characters, as RFC 5280's upper bounds are; `organizationIdentifier` has no X.520 bound, and 128 is this specification's. No value may contain a control character (U+0000–U+001F, U+007F–U+009F) or be anything but well-formed Unicode. An attribute type may repeat — two `OU`s are legitimate — and order is kept exactly as given.

`emailAddress` in a distinguished name is legacy ([RFC 5280 §4.1.2.6](https://www.rfc-editor.org/rfc/rfc5280#section-4.1.2.6)); a producer **SHOULD** use `subjectAltNames.email` instead and carry it in the DN only for a CA that requires it there.

### The certificate request

The custodian builds a `CertificationRequest` ([RFC 2986 §4](https://www.rfc-editor.org/rfc/rfc2986#section-4)) in DER:

- **`version`** — `0`.
- **`subject`** — the `RDNSequence` built from `subject`, in the order given, each RDN a `SET` of one `AttributeTypeAndValue` encoded as the table above says.
- **`subjectPKInfo`** — the named key's own public key, never one the producer supplied:
  - `p256`: algorithm `id-ecPublicKey` (1.2.840.10045.2.1) with parameters `namedCurve secp256r1` (1.2.840.10045.3.1.7), and the uncompressed point ([RFC 5480 §2](https://www.rfc-editor.org/rfc/rfc5480#section-2));
  - `ed25519`: algorithm `id-Ed25519` (1.3.101.112) with parameters absent, and the 32-byte public key ([RFC 8410 §4](https://www.rfc-editor.org/rfc/rfc8410#section-4)).
- **`attributes`** — empty when `subjectAltNames` is absent. Otherwise exactly one attribute, `extensionRequest` (1.2.840.113549.1.9.14, [RFC 2985 §5.4.2](https://www.rfc-editor.org/rfc/rfc2985#section-5.4.2)), whose one value is an `Extensions` holding one non-critical `subjectAltName` extension (2.5.29.17): the `dns` entries as `dNSName`, then the `uri` entries as `uniformResourceIdentifier`, then the `email` entries as `rfc822Name`, each list in the order given.
- **`signatureAlgorithm`** and **`signature`** — over the DER encoding of the `CertificationRequestInfo`, exactly:
  - `p256`: `ecdsa-with-SHA256` (1.2.840.10045.4.3.2) with parameters absent ([RFC 5758 §3.2](https://www.rfc-editor.org/rfc/rfc5758#section-3.2)); the signature is the DER `Ecdsa-Sig-Value` `SEQUENCE { r INTEGER, s INTEGER }` ([RFC 3279 §2.2.3](https://www.rfc-editor.org/rfc/rfc3279#section-2.2.3)) — not the fixed-width JOSE form;
  - `ed25519`: `id-Ed25519` with parameters absent, and the 64-byte Ed25519 signature ([RFC 8410 §6](https://www.rfc-editor.org/rfc/rfc8410#section-6)).

The result is armoured as [RFC 7468](https://www.rfc-editor.org/rfc/rfc7468) `CERTIFICATE REQUEST` text: base64 in 64-character lines, LF line endings.

**No other extension is requested.** Key usage, extended key usage, basic constraints, certificate policies, QC statements and every other certificate extension come from the CA, under its own policy — for an EUDI access certificate, the registrar's certificate profile. A requested extension is a claim about what the key may do, which the custodian cannot evaluate and the CA would override anyway. The subject alternative name is the exception because it is part of the identity being certified, not a statement about the key's use: OpenID4VP's `x509_san_dns` client identifier, for instance, is matched against the certificate's `dNSName`, and many CAs take that name from the request. A future version may add an extension a named CA profile requires the request to carry; this one found none.

**Key types.** `p256` is what the EUDI access-certificate CAs this task is written for issue for. `ed25519` is included because [RFC 8410](https://www.rfc-editor.org/rfc/rfc8410) defines it and some CAs accept it. ML-DSA, secp256k1 and `x25519` are refused: the last signs nothing, and the others are left to a later version once a CA this task serves asks for them.

## Authorization

The authority is **standing over the key's scope**: the custodian has recorded this producer as entitled to act on the scope the key belongs to, and the key is one the producer may name. A producer without it is refused with the framework's `permissionDenied`.

Where that entitlement is expressed as a device capability, the registered value is **`signCsr`** (`sign-csr` in the `0.1` casing) — see `Capability` in [`device/_shared`](../../../device/_shared/0.2/device-binding.schema.json). A custodian **MUST NOT** require `sign` for this task — certifying a key should not need the general oracle — and granting `signCsr` confers no other capability. Whether `sign` also authorizes it follows the same reasoning as [`keys/sign-jws`](../../sign-jws/0.1/spec.md#authorization): a custodian that signs `keys/sign` payloads verbatim **MAY** accept `sign`, since its `sign` holders could build the request themselves; a custodian that frames `keys/sign` payloads under its own signing domain has made `sign` a grant that cannot yield a certificate request, and **SHOULD NOT** accept it here.

The authority this task needs is the authority to have the key *certified*, which is more than the authority to use it: a certificate binds the key to a name every relying party will trust. A custodian **MAY** therefore restrict:

- **the keys** — to those provisioned for a protocol that uses certificates (see [Correlation](#correlation));
- **the subject** — for instance to the organisation name and identifier the deployment was registered under, so that a producer cannot obtain a request naming someone else.

Neither restriction is something the producer can satisfy by presenting more identity: verifying the `issuer`, the transport identity or the `proof` establishes *who* asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may. Nor does the custodian vouch for the subject: a certificate request proves possession of the key and *asserts* a name, and verifying that the name belongs to the requester is the CA's job.

## Definitions

* **Producer.** The party requesting the certificate request; identified by `issuer`. In the motivating case, a community server obtaining a relying-party access certificate.
* **Key custodian.** The party holding the private key and performing the signing; identified by `recipient`.
* **`keyId`.** The key to be certified, as described by [`keys/show`](../../show/0.1/spec.md).
* **`subject`.** The distinguished name to request, as an ordered list of `{type, value}` attributes from the closed set above.
* **`subjectAltNames`.** Optional DNS names, URIs and email addresses to request as the certificate's subject alternative names.
* **`csr`.** The signed PKCS#10 request, as PEM.
* **`publicKey`.** The key's public half, multibase-encoded as `KeyRecord.publicKey` carries it.

## Request

The producer sends the request to the custodian; its payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A request for an EUDI relying-party access certificate

A community, registered as a legal person, asks for a request for its P-256 verifier key, naming itself by country, organisation, LEI-based organisation identifier and common name, and its verifier's host name and URL as alternative names.

```json
{
  "id": "urn:uuid:a4c1e2f3-5b6d-4e7f-8091-a2b3c4d5e601",
  "type": "https://trusttasks.org/spec/keys/sign-csr/0.1",
  "issuer": "did:example:community",
  "recipient": "did:example:community-custodian",
  "issuedAt": "2026-10-11T10:00:00Z",
  "payload": {
    "keyId": "eudi-access-p256",
    "subject": [
      { "type": "C", "value": "DE" },
      { "type": "O", "value": "Example Community e.V." },
      { "type": "organizationIdentifier", "value": "LEIXG-5299000EXAMPLE000001" },
      { "type": "CN", "value": "Example Community Verifier" }
    ],
    "subjectAltNames": {
      "dns": ["verifier.community.example"],
      "uri": ["https://verifier.community.example"]
    }
  }
}
```

## Response

The custodian answers with the PEM request and the key's public half; the payload validates against the sub-schema reachable via `$anchor: "response"`. `keyId` is echoed so a response separated from its request is still attributable. Failures (`permissionDenied`, `keys:notFound`, and this specification's own codes) use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### The certificate request

```json
{
  "id": "urn:uuid:a4c1e2f3-5b6d-4e7f-8091-a2b3c4d5e602",
  "type": "https://trusttasks.org/spec/keys/sign-csr/0.1#response",
  "threadId": "urn:uuid:a4c1e2f3-5b6d-4e7f-8091-a2b3c4d5e601",
  "issuer": "did:example:community-custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-11T10:00:01Z",
  "payload": {
    "keyId": "eudi-access-p256",
    "csr": "-----BEGIN CERTIFICATE REQUEST-----\nMIIBjzCCATYCAQAweDELMAkGA1UEBhMCREUxHzAdBgNVBAoMFkV4YW1wbGUgQ29t\nbXVuaXR5IGUuVi4xIzAhBgNVBGEMGkxFSVhHLTUyOTkwMDBFWEFNUExFMDAwMDAx\nMSMwIQYDVQQDDBpFeGFtcGxlIENvbW11bml0eSBWZXJpZmllcjBZMBMGByqGSM49\nAgEGCCqGSM49AwEHA0IABPeQCDiwlBbsbMalmpZyIBiXUMwKLUsPdS9cdMUqT5BK\n82pnE50hQM83H375pOf/nBtaNQboR0m18jFARbmBbD+gXDBaBgkqhkiG9w0BCQ4x\nTTBLMEkGA1UdEQRCMECCGnZlcmlmaWVyLmNvbW11bml0eS5leGFtcGxlhiJodHRw\nczovL3ZlcmlmaWVyLmNvbW11bml0eS5leGFtcGxlMAoGCCqGSM49BAMCA0cAMEQC\nIEiJ000jPd1gG5qAM+WaNYAc+RpApmiSAn7Mvw+i3ODfAiBwV/5C5DQzAAmXbxxr\nFmoRrcONsMPAjGP1XarGPxBspA==\n-----END CERTIFICATE REQUEST-----\n",
    "publicKey": "zDnaezKfW1F36Bc3ranfFMiPJXexoqUeHSYLeQwGns4npGdh3"
  }
}
```

The example is real: `openssl req -verify` accepts its self-signature, and it decodes to the subject `C=DE, O=Example Community e.V., organizationIdentifier=LEIXG-5299000EXAMPLE000001, CN=Example Community Verifier` with `countryName` as a PrintableString and the rest as UTF8String, a P-256 public key matching `publicKey`, and one `extensionRequest` carrying the two alternative names. ECDSA signatures are randomised, so a custodian building the same request produces different signature bytes that verify equally; everything before the signature is fixed by the request.

## Security & Privacy

### Data carried

The request carries a key identifier, a subject and optional alternative names; the response carries the certificate request, the key's public half and the echoed `keyId`. No key material travels. The subject is caller-supplied identity data, and it does not stay with the custodian: it goes into the request, to the CA, into the CA's records and, once issued, into a certificate shown to every relying party — every wallet the verifier presents to, in the motivating case. For an organisation that is the point. For a natural person a `CN` or `emailAddress` is personal data published by design, and a producer **SHOULD** put into `subject` only what the CA's profile requires.

The structure is what limits the custodian's part. It encodes typed attributes from a closed set and alternative names of three kinds; it never accepts a raw DER structure, a DN string to parse, an extension or a public key. So the custodian asserts nothing it did not build: the only things in the signed request are the key's own public key and the name the producer asked for, in an encoding this specification fixes.

### Correlation

A certificate request names its key and its subject together; that join is the request's purpose. Every certificate later issued for the key is linked to every signature the key makes, and the subject travels with all of them. A key that is certified under a name **SHOULD** be used for nothing that should not carry that name.

That is also the cross-protocol boundary. The signed bytes are a DER `CertificationRequestInfo`, beginning `0x30`, which is not a JWS signing input, an SSHSIG structure or a payload framed by `keys/sign`, so no signature obtained here can be replayed as one of those. But a certificate makes a key usable by every relying party that trusts the CA, so certifying a key that was provisioned for something else — a DID verification key, a commit-signing key — attaches a name to signatures that were never meant to carry one. A custodian **SHOULD** bind each key to the protocol it signs for, and **SHOULD** issue certificate requests only for keys bound to a protocol that uses certificates, such as JWS with `x5c` ([`keys/sign-jws`](../../sign-jws/0.1/spec.md)); a key so bound **MAY** also be used here, since a certificate request is how such a key comes to be certified.

At the custodian, the sequence of requests — which keys, under which names, how often — is a record of what the producer is getting certified.

### Retention

Transient at the custodian; the operation stores nothing. The audit line [Conformance](#conformance) recommends keeps the subject and alternative names, unlike the audit line of [`keys/sign-jws`](../../sign-jws/0.1/spec.md), which keeps only a digest. The difference is deliberate: a JWS payload is the producer's working data, while the subject of a certificate request is the very name the custodian's key is being made to vouch for, and the one thing an operator needs to answer a question about a certificate for its key. It is also not secret — it will appear in the certificate. The custodian **SHOULD** keep that line as long as certificates for the key may be valid.

### Consent/purpose

The purpose is to have a key the producer is entitled to certified under a name, by a CA that will verify the name. The subject data is collected for that and for the custodian's audit of it, and for nothing else. Whether a particular certificate request also needs a human's approval at the custodian is the custodian's policy, not something this specification declares; whether the name is the requester's to claim is the CA's determination.
