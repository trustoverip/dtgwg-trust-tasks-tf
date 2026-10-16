---
slug: keys/sign-jws
version: "0.1"
title: "Keys — Sign JWS"
summary: "A producer obtains a compact JWS — a signed OpenID4VP request object, a proof-of-possession JWT — from a key the custodian holds, with the custodian building the protected header and signing input itself, so the key never leaves and the custodian signs nothing but JWS."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - keys
  - signing
  - jws
  - jwt
  - jose
  - openid4vp
  - custody
parties:
  - role: Producer (the party wanting a JWS)
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
    JWS it obtains is a bearer artefact: a wallet, a registry or any other verifier accepts it
    on the strength of the key alone, without ever seeing this exchange. Where the transport
    does not already authenticate the producer, the proof is the only thing standing between
    the key and anyone who can reach the custodian.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Signing acts with the key holder's authority and produces evidence that outlives the
    exchange. A request that cannot be placed in an acceptance window could be re-presented
    to obtain the same JWS again; SPEC §7.2 item 11 can only absorb a duplicate inside a
    bounded window.
sideEffects:
  level: none
  rationale: >-
    No stored state changes. The custodian reads a key record, signs, and returns the JWS.
    The JWS is durable and verifiable by anyone holding the public key, so the operation is
    not repeatable-without-consequence in the way a read is — but nothing at the custodian
    moves.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The custodian exercises the named key's private half, producing a JWS that verifies as
    that key's identity. Nothing is disclosed to the producer beyond the JWS. What the request
    carries in is a header and a payload the custodian serialises but does not interpret — an
    OpenID4VP request object carries a nonce, a response endpoint and the query a wallet will
    be shown — so `metadata` is the most this specification can honestly claim about it, as
    for `keys/sign`; a producer whose payload is personal is responsible for the transport
    that carries it.
retention:
  class: transient
  rationale: >-
    Nothing about the request needs to survive the exchange except the audit line. That line
    records a digest of the signing input and never the payload, so it can show which JWS a
    key made without becoming a store of what the JWS said.
maxDocumentBytes:
  request: 65536
  response: 131072
  rationale: >-
    A request object with a DCQL query and an `x5c` chain of two or three certificates fits
    in a few KiB; 64 KiB leaves room for a larger query or chain without inviting the
    custodian to sign bulk content. The response carries the header and payload again,
    base64url-encoded (a third larger), plus a signature of up to 3,309 bytes (ML-DSA-65),
    so it is bounded at twice the request.
errorCodes:
  - code: keys:notFound
    meaning: No key record on this custodian carries the named `keyId`. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys:invalidArgument
    meaning: A header member is well-formed against the schema but unusable — a JOSE-registered member whose value is not of the type RFC 7515 defines for it, or an `x5c` whose first certificate does not carry the named key's public key. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys/sign-jws:failedPrecondition
    meaning: The named key exists but cannot sign a JWS in its current state — its `status` is not `active`, or the custodian has bound it to a protocol other than JWS.
    retryable: false
  - code: keys/sign-jws:keyTypeUnsupported
    meaning: The named key's `keyType` has no JOSE signature algorithm this custodian implements. An `x25519` key always draws this code.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [keyType]
      properties:
        keyType:
          type: string
  - code: keys/sign-jws:headerMemberForbidden
    meaning: The `header` names a member the custodian sets or refuses itself — `alg`, `crit`, `b64`, `jwk`, `jku` or `x5u` — or one its policy refuses for this key.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member]
      properties:
        member:
          type: string
  - code: keys/sign-jws:payloadInvalid
    meaning: The payload cannot be signed as given — `payloadBase64Url` is not the canonical unpadded base64url encoding of any byte string, or `payload` holds a value RFC 8785 cannot serialise exactly (a number outside IEEE 754 double precision, a lone surrogate).
    retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
  - keys/sign
  - keys/sign-sshsig
  - keys/sign-csr
  - keys/show
  - keys/create
---

## Abstract

The **Keys — Sign JWS** Trust Task lets a *producer* obtain a [JSON Web Signature](https://www.rfc-editor.org/rfc/rfc7515) in compact serialisation from a key held by a *key custodian*. The producer sends the protected header members it wants and the payload; the custodian chooses `alg` from the key, serialises the header and payload itself, signs the JWS signing input, and returns the compact JWS.

The case that motivates it is a community server acting as an [OpenID4VP](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html) verifier. A wallet will only present to a verifier whose authorization request arrives as a signed request object ([RFC 9101](https://www.rfc-editor.org/rfc/rfc9101), `typ: oauth-authz-req+jwt`), and a trust registry may ask the verifier to prove possession of its key with a JWT. The verifier's key is one the custodian holds and never exports. [`keys/sign`](../../sign/0.1/spec.md) cannot stand in: a custodian that hardens its general oracle frames every payload under its own signing domain, precisely so that nothing it signs there verifies as a protocol signature — and a JWS is a protocol signature. This task is the narrow way through.

Three properties follow, and they are the reason this is a task of its own:

- **The private key never leaves the custodian.** The producer — here the verifier — signs request objects without holding the key, so a compromised producer can sign while its entitlement lasts but never keeps a key.
- **The custodian builds the bytes.** It writes the header, including the one member that decides how the signature is checked (`alg`), and it serialises both halves. Every byte string it signs is a JWS signing input — ASCII, beginning with the encoding of a JSON object — so a signature obtained here verifies as a JWS and as nothing else: not an SSHSIG signature, not a certificate request, not a payload framed by `keys/sign`.
- **It is still an oracle, for JWS.** Within JOSE, the producer chooses the payload and most of the header, so it can obtain any JWS the key could make. What keeps that safe is a key that is used for one JOSE purpose and nothing else — which the custodian can enforce, and which [Security & Privacy](#security--privacy) sets out.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/sign-jws/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Supply exactly one of `payload` (a JSON object) and `payloadBase64Url` (bytes, base64url-encoded without padding).
3. Use the returned `jws` as it is. The custodian serialises the header and payload itself, so the JSON inside the JWS is not byte-for-byte the JSON the producer sent; a producer that re-assembles a JWS around the signature from its own serialisation produces one that does not verify.

A producer **SHOULD** include `typ` in `header` and **SHOULD** verify the returned JWS against the key's public half (from [`keys/show`](../../show/0.1/spec.md) or the key holder's DID document) before using it.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether **this producer may obtain a JWS from this key** — and refuse with `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) where it may not. See [Authorization](#authorization).
3. Refuse a `keyId` it holds no record for with `keys:notFound`, and a key whose `status` is not `active` with `keys/sign-jws:failedPrecondition`.
4. Choose `alg` from the key's `keyType` alone, by the table below, and refuse with `keys/sign-jws:keyTypeUnsupported` a key type the table does not list or the custodian does not implement.
5. Refuse with `keys/sign-jws:headerMemberForbidden`, naming the member in `details.member`, a `header` that contains any of the members in [Forbidden header members](#forbidden-header-members).
6. Refuse with `keys:invalidArgument` a `header` member that RFC 7515 registers (`typ`, `cty`, `kid`, `x5c`, `x5t`, `x5t#S256`) whose value is not of the type RFC 7515 gives it, and an `x5c` whose first certificate's subject public key is not the named key's public key. A JWS whose header names a certificate for another key verifies against nothing — or, worse, against a verifier that trusted the chain without checking the signature.
7. Build the signing input **itself**, exactly as in [The signing input](#the-signing-input), and sign that. It **MUST NOT** sign any byte string other than a JWS signing input it has built, and **MUST NOT** prefix, frame or otherwise transform it — a signature over anything else does not verify as a JWS.
8. Return the compact JWS and the `alg` it chose under the `#response` variant, and **never** the private key or any encoding of it.

### Choosing `alg`

| `keyType` | `alg` | Signature |
|---|---|---|
| `ed25519` | `EdDSA` | Ed25519 over the signing input, 64 bytes ([RFC 8037](https://www.rfc-editor.org/rfc/rfc8037)) |
| `p256` | `ES256` | ECDSA P-256 with SHA-256, as the 64-byte `r \|\| s` of [RFC 7518 §3.4](https://www.rfc-editor.org/rfc/rfc7518#section-3.4) — never DER |
| a secp256k1 key | `ES256K` | ECDSA secp256k1 with SHA-256, 64-byte `r \|\| s` ([RFC 8812 §3.2](https://www.rfc-editor.org/rfc/rfc8812#section-3.2)) |
| `mldsa44` | `ML-DSA-44` | ML-DSA-44 as its JOSE registration defines it |
| `mldsa65` | `ML-DSA-65` | ML-DSA-65 as its JOSE registration defines it |
| `x25519` | — | A key-agreement key signs nothing: always `keys/sign-jws:keyTypeUnsupported` |

The ML-DSA rows use the JOSE identifiers [`keys/_shared` `SignAlgorithm`](../../_shared/0.1/sign-algorithm.schema.json) already carries for `keys/sign`. `keys/_shared` `KeyType` does not yet name a secp256k1 key; the `ES256K` row says what a custodian that holds one does, so that registering the key type does not need a new version of this specification.

The producer cannot name an algorithm. A key has one algorithm, and the custodian knows the key's type; a member that let the producer say otherwise could only be checked against what the custodian was going to do anyway, or not checked — which is how algorithm-substitution attacks on JOSE begin ([RFC 8725 §2.1](https://www.rfc-editor.org/rfc/rfc8725#section-2.1)).

### Forbidden header members

Each of these is refused rather than ignored or overwritten. Overwriting would sign a header the producer did not write; ignoring would sign one it did not get.

- **`alg`** — set by the custodian from the key, above. A producer-chosen `alg` is the lever of every algorithm-confusion attack (`none`, or `HS256` keyed with the public key); there is no value the producer could send that the custodian should prefer to its own.
- **`crit`** — it tells every verifier that the listed extensions **MUST** be understood and processed ([RFC 7515 §4.1.11](https://www.rfc-editor.org/rfc/rfc7515#section-4.1.11)). The custodian processes no extension, so signing `crit` would put its key behind processing obligations it never evaluated.
- **`b64`** — `b64: false` ([RFC 7797](https://www.rfc-editor.org/rfc/rfc7797)) changes the signing input to the raw payload. The custodian signs the encoded form only, so the header would describe a signing input other than the one signed; and an unencoded payload is producer-chosen bytes inside the signing input, which is the general oracle again.
- **`jwk`** — an embedded public key that a verifier may use to check the signature. A verifier that takes its key from the token trusts whatever key the token names, so a caller-chosen `jwk` makes the custodian's signature carry a key-trust instruction it did not check. A producer names the key with `kid` (a DID URL, for instance) or certifies it with `x5c`, which the custodian does check.
- **`jku`** and **`x5u`** — URLs a verifier dereferences for the key or the certificate chain. Signing one makes the custodian vouch for a location the producer controls, whose content can change after the signature is made, and points every verifier that follows it at a host the producer chose. `x5c` carries the chain inline instead, where the custodian can check its first certificate.

A custodian **MAY** refuse further members by policy — for instance any `typ` but the ones a key was provisioned for — with the same code.

### The signing input

The custodian builds the JWS as [RFC 7515 §7.1](https://www.rfc-editor.org/rfc/rfc7515#section-7.1) defines, with these choices fixed so that the bytes are a function of the request alone:

1. **Header.** The object is the producer's `header` (or `{}` when absent) with `alg` added. It is serialised with the JSON Canonicalization Scheme ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)) — UTF-8, no insignificant whitespace, members in sorted order — and encoded as BASE64URL without padding.
2. **Payload.** For `payload`, the object is serialised with RFC 8785 and encoded as BASE64URL without padding. For `payloadBase64Url`, the custodian decodes the string, refuses with `keys/sign-jws:payloadInvalid` one that is not the canonical encoding of the decoded bytes (its unused trailing bits are not zero, or its length leaves a single dangling character), and otherwise uses the string unchanged.
3. **Signing input.** `ASCII(BASE64URL(header) || '.' || BASE64URL(payload))`, signed with the key as the table above says. No domain tag, length prefix or other framing is added: the JWS signing input is itself the domain separation, since it is ASCII and, its header being a JSON object, always begins `eyJ`.
4. **Result.** `BASE64URL(header) || '.' || BASE64URL(payload) || '.' || BASE64URL(signature)`.

RFC 8785 is chosen for determinism, not because verifiers need it — a JWS verifier works on the bytes it receives. Determinism means a producer can compute the exact signing input before it sends the request, and an auditor can recompute the digest in the custodian's audit line from the request alone. Its one cost is I-JSON: a number RFC 8785 cannot represent exactly (an integer beyond 2⁵³, say) is refused as `payloadInvalid` rather than rounded, since a rounded claim is a different claim. A producer that needs such a value sends it as a string, or sends the whole payload as `payloadBase64Url`.

The custodian adds no claim to the payload and checks none. Whether the JWS carries `iat`, `exp`, `nonce`, `aud` or `jti` is the producer's and its profile's business; the custodian cannot know what a given profile requires.

### Audit

A custodian **SHOULD** record each request — producer, key, `alg`, the header's `typ` where present, and the SHA-256 digest of the signing input — in an audit trail, and **MUST NOT** write the payload into it. The JWS is durable and bearer; a custodian that cannot say afterwards who asked for it has lost the only record that would distinguish authorized use from compromise. The digest is enough to match a JWS seen later to the request that produced it, and the payload — which for a request object names a response endpoint and the query a wallet was shown — is the producer's record to keep, not the custodian's.

## Authorization

The authority is **standing over the key's scope**: the custodian has recorded this producer as entitled to act on the scope the key belongs to, and the key is one the producer may name. A producer without it is refused with the framework's `permissionDenied`.

Where that entitlement is expressed as a device capability, the registered value is **`signJws`** (`sign-jws` in the `0.1` casing) — see `Capability` in [`device/_shared`](../../../device/_shared/0.2/device-binding.schema.json). It is deliberately narrower than `sign`, and a custodian **MUST NOT** require `sign` for this task: that would force every JWS signer to hold the general oracle, which is the grant this task exists to avoid. Granting `signJws` confers no other capability.

Whether `sign` **also** authorizes this task depends on what `sign` means at the custodian, and the answer is not the one [`keys/sign-sshsig`](../../sign-sshsig/0.1/spec.md) gives:

- A custodian that signs `keys/sign` payloads verbatim, as that specification describes, has made `sign` strictly wider than `signJws` — a `sign` holder can obtain the same JWS by building the signing input itself — and **MAY** accept `sign` in its place.
- A custodian that frames `keys/sign` payloads under a signing domain of its own has made `sign` a grant that **cannot** yield a JWS. Accepting `sign` here would silently widen every existing `sign` grant into the one capability the framing was introduced to withhold. Such a custodian **SHOULD NOT** accept `sign` for this task, and **SHOULD** require `signJws`.

Entitlement is necessary and not sufficient. A custodian **MAY** further restrict:

- **the key's purpose** — see [Correlation](#correlation): a key bound to JWS signing is refused by every other signing task, and a key provisioned for one `typ` can be refused any other;
- **the rate** — a quota per key bounds what a compromised producer can obtain before its entitlement is withdrawn.

Neither restriction is something the producer can satisfy by presenting more identity: verifying the `issuer`, the transport identity or the `proof` establishes *who* asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

* **Producer.** The party requesting the JWS; identified by `issuer`. In the motivating case, a community server acting as an OpenID4VP verifier.
* **Key custodian.** The party holding the private key and performing the signing; identified by `recipient`.
* **`keyId`.** The key to sign with, as described by [`keys/show`](../../show/0.1/spec.md). Chosen by the producer from the keys it is entitled to; the key's type decides `alg`.
* **`header`.** The protected header members the producer wants, other than `alg`. Names and values belong to JOSE and to the profile the JWS is for — `typ: oauth-authz-req+jwt` and `kid` or `x5c` for an OpenID4VP request object, `profile_version` where the swiyu profile asks for it — and are carried verbatim.
* **`payload`.** The JWS payload as a JSON object; for a JWT, its claims set.
* **`payloadBase64Url`.** The JWS payload as bytes, for a payload that is not a JSON object.
* **Signing input.** `ASCII(BASE64URL(header) || '.' || BASE64URL(payload))`, the bytes the signature covers ([RFC 7515 §2](https://www.rfc-editor.org/rfc/rfc7515#section-2)).

## Request

The producer sends the request to the custodian; its payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An OpenID4VP request object, signed with a P-256 key

A community server asks its custodian to sign the request object a wallet will fetch. The server names its verifier key and the `typ` and `kid` the swiyu profile requires; the claims are the authorization request.

```json
{
  "id": "urn:uuid:3b9d6c1e-8f42-4a7d-b5e0-9c1f2a3d4e01",
  "type": "https://trusttasks.org/spec/keys/sign-jws/0.1",
  "issuer": "did:example:community",
  "recipient": "did:example:community-custodian",
  "issuedAt": "2026-10-11T09:00:00Z",
  "payload": {
    "keyId": "verifier-p256",
    "header": {
      "typ": "oauth-authz-req+jwt",
      "kid": "did:example:community-verifier#key-1"
    },
    "payload": {
      "aud": "https://self-issued.me/v2",
      "client_id": "decentralized_identifier:did:example:community-verifier",
      "response_type": "vp_token",
      "response_mode": "direct_post.jwt",
      "response_uri": "https://community.example/v1/oid4vp/response/7d1e4c2a",
      "nonce": "q8vTt3yN5mQ0aW2f",
      "state": "7d1e4c2a",
      "dcql_query": {
        "credentials": [
          {
            "id": "membership",
            "format": "dc+sd-jwt",
            "meta": { "vct_values": ["https://credentials.example/membership"] }
          }
        ]
      },
      "iat": 1791709200,
      "exp": 1791709500
    }
  }
}
```

## Response

The custodian answers with the compact JWS; the payload validates against the sub-schema reachable via `$anchor: "response"`. `keyId` is echoed so a response separated from its request is still attributable, and `alg` says what the custodian chose. Failures (`permissionDenied`, `keys:notFound`, `keys:invalidArgument`, and this specification's own codes) use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### The signed request object

```json
{
  "id": "urn:uuid:3b9d6c1e-8f42-4a7d-b5e0-9c1f2a3d4e02",
  "type": "https://trusttasks.org/spec/keys/sign-jws/0.1#response",
  "threadId": "urn:uuid:3b9d6c1e-8f42-4a7d-b5e0-9c1f2a3d4e01",
  "issuer": "did:example:community-custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-11T09:00:01Z",
  "payload": {
    "keyId": "verifier-p256",
    "alg": "ES256",
    "jws": "eyJhbGciOiJFUzI1NiIsImtpZCI6ImRpZDpleGFtcGxlOmNvbW11bml0eS12ZXJpZmllciNrZXktMSIsInR5cCI6Im9hdXRoLWF1dGh6LXJlcStqd3QifQ.eyJhdWQiOiJodHRwczovL3NlbGYtaXNzdWVkLm1lL3YyIiwiY2xpZW50X2lkIjoiZGVjZW50cmFsaXplZF9pZGVudGlmaWVyOmRpZDpleGFtcGxlOmNvbW11bml0eS12ZXJpZmllciIsImRjcWxfcXVlcnkiOnsiY3JlZGVudGlhbHMiOlt7ImZvcm1hdCI6ImRjK3NkLWp3dCIsImlkIjoibWVtYmVyc2hpcCIsIm1ldGEiOnsidmN0X3ZhbHVlcyI6WyJodHRwczovL2NyZWRlbnRpYWxzLmV4YW1wbGUvbWVtYmVyc2hpcCJdfX1dfSwiZXhwIjoxNzkxNzA5NTAwLCJpYXQiOjE3OTE3MDkyMDAsIm5vbmNlIjoicTh2VHQzeU41bVEwYVcyZiIsInJlc3BvbnNlX21vZGUiOiJkaXJlY3RfcG9zdC5qd3QiLCJyZXNwb25zZV90eXBlIjoidnBfdG9rZW4iLCJyZXNwb25zZV91cmkiOiJodHRwczovL2NvbW11bml0eS5leGFtcGxlL3YxL29pZDR2cC9yZXNwb25zZS83ZDFlNGMyYSIsInN0YXRlIjoiN2QxZTRjMmEifQ.9w2it71b0W3qKXW-vPPTvwCxJo7tGZgH5Fqn9oSFJMgaaYKlt3GS1GQAbXKfOYl51eZ3a8FJzoRwXctc4oiUvg"
  }
}
```

The example is real and checkable. The header segment decodes to the RFC 8785 serialisation `{"alg":"ES256","kid":"did:example:community-verifier#key-1","typ":"oauth-authz-req+jwt"}` — the producer's two members with `alg` added, in sorted order — and the payload segment to the claims in sorted order. The signature verifies under the P-256 public key with JWK coordinates `x: Lm0fFbVxwiCVFr4qjgHf0iqtfr71uDa7xjtuQ4aZ6GI`, `y: x0li057zSpas3S_T7KfmpGiVZz-iLWP8wLVyd1PyJQA` (multibase `zDnaeTZCJom8WYgWfqp7ubMUwHK9ayrv31SSBJACRDDFiazq7`). The SHA-256 of the signing input, which is what the custodian's audit line records, is `e0c236e3a90999a1ab54699f308c0ad0468405da185731762396cd733e031257`. ECDSA signatures are randomised, so a custodian signing the same request produces a different third segment that verifies equally; the first two segments are fixed.

### A refused header

A producer that sent the same request with `"alg": "none"` in `header` is refused before anything is signed:

```json
{
  "id": "urn:uuid:3b9d6c1e-8f42-4a7d-b5e0-9c1f2a3d4e04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:3b9d6c1e-8f42-4a7d-b5e0-9c1f2a3d4e03",
  "issuer": "did:example:community-custodian",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-11T09:00:02Z",
  "payload": {
    "code": "keys/sign-jws:headerMemberForbidden",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/keys/sign-jws/0.1",
      "id": "urn:uuid:3b9d6c1e-8f42-4a7d-b5e0-9c1f2a3d4e03"
    },
    "message": "the custodian sets alg from the key",
    "retryable": false,
    "details": { "member": "alg" }
  }
}
```

## Security & Privacy

### Data carried

The request carries a key identifier, the header members the producer wants and a payload; the response carries the compact JWS, the chosen `alg` and the echoed `keyId`. No key material travels in either direction. The payload is whatever the producer is signing — for a request object, a nonce, a state value, a response endpoint and the DCQL query that tells a wallet which credentials and claims are wanted — and the custodian reads it only to serialise it. It applies no policy to the claims, writes none of them to its audit line, and **SHOULD NOT** keep the payload once the JWS is returned. The header is smaller and more telling: `typ` says what kind of JWS this is, and `kid` or `x5c` names the identity it will be checked as. That is what lets a custodian hold a key to one purpose.

The smallest conforming request is a key and a payload. A producer **SHOULD NOT** put members into `header` that its profile does not ask for: every one is signed, published to every verifier, and becomes part of what the key has vouched for.

### Correlation

A JWS verifies under the key's public half forever, and every JWS one key makes is linked to every other; for a verifier identity that is the intent — wallets recognise the verifier by its key — and not a side effect. The custodian-side trace is the sequence of requests: which keys a producer names, how often and when, which for a verifier is a record of how many presentation sessions it opened. The digest in the audit line is a stable identifier of one JWS, and anyone who later holds the JWS can match it to the line.

The cross-protocol risk is the one to design for. Within JOSE, a JWS is a JWS: a verifier of ID tokens, of VC-JWTs, of OpenID4VCI proofs or of request objects checks a signature by a key, and the custodian here will sign any payload the producer sends. A key used for two JOSE purposes therefore lets anyone able to obtain one kind of JWS present it as the other, wherever a verifier does not check `typ` ([RFC 8725 §3.11](https://www.rfc-editor.org/rfc/rfc8725#section-3.11)). So:

- A custodian **SHOULD** bind each key to the protocol it signs for, and **SHOULD** refuse with `keys/sign-jws:failedPrecondition` a key it has bound to anything else — and, from the other side, refuse a key used here when it is named in [`keys/sign`](../../sign/0.1/spec.md), [`keys/sign-sshsig`](../../sign-sshsig/0.1/spec.md) or any other signing task. The format boundary already holds by construction (no JWS signing input is an SSHSIG structure, a DER certificate request or a domain-framed payload); the binding is what stops one key from speaking two protocols at all.
- The one exception is [`keys/sign-csr`](../../sign-csr/0.1/spec.md): a certificate request is how a JWS key comes to be certified — an EUDI access certificate is exactly that — and its signed bytes are a DER structure the custodian builds, beginning `0x30`, which no JWS signing input can be. A key bound to JWS **MAY** also sign certificate requests for itself.
- A custodian **SHOULD** let a key be restricted to the `typ` values it was provisioned for, and a producer **SHOULD** always send `typ`, so that verifiers checking explicit types can tell the custodian's JWS kinds apart.

Replay has two layers. A replayed *request* is bounded by the REQUIRED `issuedAt` and the consumer's duplicate handling ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)); at worst it yields a second signature over the same signing input. A replayed *JWS* is the verifier's problem and the payload's: freshness and audience claims (`iat`, `exp`, `nonce`, `aud`) are the producer's to include, because the custodian neither adds nor checks claims.

### Retention

Transient at the custodian. The operation stores nothing; the audit line is the only thing that survives it. That line **SHOULD** hold producer, key, `alg`, `typ`, signing-input digest and time, and **MUST NOT** hold the payload, as [Audit](#audit) requires — enough to answer "did this key sign this JWS, and at whose request" after a compromise, without the custodian becoming a second store of every authorization request its producer ever made.

### Consent/purpose

The purpose is exercise of an existing custody relationship: a producer already entitled over a key asks the custodian to sign a JWS with it. Unlike `keys/sign`, part of the purpose is visible — the header says what kind of JWS this is and which identity will be checked — so a deployment can limit a key to one purpose structurally, by binding it to JWS and to the `typ` values it was provisioned for. The payload's purpose is the producer's to govern; for an OpenID4VP request object, the wallet user is the one asked to consent to what it requests. Whether a particular signature also needs a human's approval at the custodian is the custodian's policy, not something this specification declares.
