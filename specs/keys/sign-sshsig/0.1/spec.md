---
slug: keys/sign-sshsig
version: "0.1"
title: "Keys — Sign SSHSIG"
summary: "A producer obtains an SSHSIG signature — the format git uses for SSH commit and tag signing — over a message digest, from a key the custodian holds, without the private key leaving the custodian and without the custodian becoming a general signing oracle."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - keys
  - signing
  - sshsig
  - ssh
  - git
  - commit-signing
  - custody
parties:
  - role: Producer (the party wanting a signature)
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
    signature it obtains is a durable artefact — a signed commit is relied on by every later
    clone of the repository, by parties who never saw the exchange. Where the transport does
    not already authenticate the producer, the proof is the only thing standing between the
    key and anyone who can reach the custodian.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Signing acts with the key holder's authority and produces evidence that outlives the
    exchange. A request that cannot be placed in an acceptance window could be re-presented
    to obtain the same signature again; SPEC §7.2 item 11 can only absorb a duplicate inside
    a bounded window.
sideEffects:
  level: none
  rationale: >-
    No stored state changes. The custodian reads a key record, signs, and returns the
    signature. The signature is durable and externally verifiable, so the operation is not
    repeatable-without-consequence in the way a read is — but nothing at the custodian moves.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The custodian exercises the named key's private half, producing a signature that verifies
    as that key's identity. Nothing is disclosed to the producer beyond the signature. What the
    request carries in is a digest and a namespace — never the signed message — so the custodian
    learns that a key signed *something* under a namespace at a time, and nothing about what.
retention:
  class: transient
  rationale: >-
    Nothing about the request needs to survive the exchange except whatever audit line the
    deployment keeps. The digest is not a secret, but it is a stable identifier of the signed
    message, so an audit line that records it links the custodian's records to the artefact.
errorCodes:
  - code: keys:invalidArgument
    meaning: A payload member is well-formed against the schema but unusable for this request — the `messageHash` length does not match `hashAlgorithm`, or `algorithm` is incompatible with the key's `keyType`. See [category conventions](../../_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: keys/sign-sshsig:failedPrecondition
    meaning: The named key exists but its `status` is not `active`, so it cannot sign.
    retryable: false
related:
  - keys/sign
  - keys/export-secret
  - keys/show
---

## Abstract

The **Keys — Sign SSHSIG** Trust Task lets a *producer* obtain an [SSHSIG](https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig) signature from a key held by a *key custodian*. SSHSIG is the format `ssh-keygen -Y sign` produces and git verifies for SSH-signed commits and tags (`gpg.format = ssh`). The producer sends a digest of the message and the namespace; the custodian builds the SSHSIG signed-data structure itself, signs it, and returns the raw signature. The producer assembles the armoured signature from that and the public key it already has.

Two properties follow, and they are the reason this is a task of its own rather than a use of [`keys/sign`](../../sign/0.1/spec.md):

- **The private key never leaves the custodian.** A producer that previously had to take the key out ([`keys/export-secret`](../../export-secret/0.1/spec.md)) to sign a commit can sign without holding it, so a compromised producer can sign while its entitlement lasts but never keeps a key.
- **The custodian is not a general signing oracle.** It signs only bytes it has built, and every one begins with the SSHSIG magic preamble, so a signature obtained here verifies as an SSHSIG signature in the named namespace and as nothing else — not a JWS, not a Data Integrity proof, not an SSH authentication. A custodian can therefore grant this task to a producer it would never grant `keys/sign` to.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/sign-sshsig/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Compute `messageHash` as the `hashAlgorithm` digest of the exact message to be signed — for git, the bytes git passes to its `gpg.ssh.program` — and encode it as base64url without padding.
3. Assemble the SSHSIG blob from the returned `signature`, the key's public half, and the `namespace` and `hashAlgorithm` it sent, as `PROTOCOL.sshsig` defines. The producer obtains the public key independently (from the key holder's DID document, or [`keys/show`](../../show/0.1/spec.md)), and **SHOULD** verify the signature against it before using it.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Decide, from its own policy, whether **this producer may use this key for SSHSIG signing in this namespace** — and refuse with `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) where it may not. See [Authorization](#authorization).
3. Refuse a key whose `status` is not `active`, with `keys/sign-sshsig:failedPrecondition`.
4. Refuse with `keys:invalidArgument` an `algorithm` the key's `keyType` cannot perform, and a `messageHash` whose decoded length is not the output length of `hashAlgorithm` (32 bytes for `sha256`, 64 for `sha512`).
5. Build the signed data **itself**, exactly as below, and sign that. It **MUST NOT** sign any byte string supplied by the producer other than as the `H(message)` field of this structure.
6. Return the raw signature under the `#response` variant, and **never** the private key or any encoding of it.

### The signed data

The custodian signs this structure, with `string` encoded as in [RFC 4251 §5](https://www.rfc-editor.org/rfc/rfc4251#section-5) (a `uint32` big-endian length followed by that many bytes):

```text
byte[6]   MAGIC_PREAMBLE   "SSHSIG"
string    namespace        payload.namespace, as ASCII
string    reserved         empty
string    hash_algorithm   payload.hashAlgorithm, as ASCII
string    H(message)       the decoded bytes of payload.messageHash
```

For `EdDSA` the signature is Ed25519 over those bytes. For `ES256` it is ECDSA P-256 with SHA-256 over those bytes — the hash `ecdsa-sha2-nistp256` prescribes — returned in the 64-byte JOSE `r || s` form.

A custodian **SHOULD** record each request — producer, key, namespace, and the digest — in an audit trail. The signature is durable; a custodian that cannot say afterwards who asked for it has lost the only record that would distinguish authorized use from compromise.

## Authorization

The authority is **standing over the key's scope**: the custodian has recorded this producer as entitled to act on the scope the key belongs to. A producer without it is refused with the framework's `permissionDenied`.

Where that entitlement is expressed as a device capability, the registered value is **`signSshsig`** (`sign-sshsig` in the `0.1` casing) — see `Capability` in [`device/_shared`](../../../device/_shared/0.2/device-binding.schema.json). It is deliberately narrower than `sign`. A producer granted `sign` can obtain a signature over any bytes, and so over anything the key could ever be asked to vouch for; a producer granted `signSshsig` can obtain only SSHSIG signatures. A custodian **MUST NOT** require `sign` for this task — that would force every git signer to hold the general oracle, which is the grant this task exists to avoid — and granting `signSshsig` confers no other capability. A custodian **MAY** accept `sign` in place of `signSshsig`, since the general grant already confers strictly more.

Entitlement is necessary and not sufficient. A custodian **MAY** further restrict:

- **the namespace** — a key provisioned for commit signing has no business signing under `file`, and a deployment that says so keeps a stolen credential from producing signatures for anything but commits;
- **the rate** — a daily quota per key or scope bounds what a compromised producer can obtain before its entitlement is withdrawn.

Neither restriction is something the producer can satisfy by presenting more identity: verifying the `issuer`, the transport identity or the `proof` establishes *who* asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

* **Producer.** The party requesting the signature; identified by `issuer`. For git, the program configured as `gpg.ssh.program`.
* **Key custodian.** The party holding the private key and performing the signing; identified by `recipient`.
* **`keyId`.** The key to sign with, as described by [`keys/show`](../../show/0.1/spec.md). Chosen by the producer from the keys it is entitled to.
* **`algorithm`.** The JOSE identifier of the signature algorithm — `EdDSA` (`ssh-ed25519`) or `ES256` (`ecdsa-sha2-nistp256`). Chosen by the producer to match the key.
* **`namespace`.** The SSHSIG namespace. Chosen by the producer (`git` for commits and tags); the custodian may refuse one its policy does not allow.
* **`hashAlgorithm`.** `sha256` or `sha512`, the digest applied to the message. git uses `sha512`.
* **`messageHash`.** The digest itself. The message never travels.

## Request

The producer sends the request to the custodian; its payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Signing a git commit with an Ed25519 key

```json
{
  "id": "urn:uuid:5f0c2a7e-3d41-4b8e-9a6f-1c2d3e4f5a61",
  "type": "https://trusttasks.org/spec/keys/sign-sshsig/0.1",
  "issuer": "did:example:alice-laptop",
  "recipient": "did:example:alice-custodian",
  "issuedAt": "2026-10-05T09:00:00Z",
  "payload": {
    "keyId": "did:example:alice#key-1",
    "algorithm": "EdDSA",
    "namespace": "git",
    "hashAlgorithm": "sha512",
    "messageHash": "ZNK6UR4HwIpHXYznMreGQrxQYoYkXyEqA9lXghSB8x0qCebX7kzmrrWkvpd-TTFuFZ7HD5DBVT80O19QxSt8FA"
  }
}
```

## Response

The custodian answers with the raw signature; the payload validates against the sub-schema reachable via `$anchor: "response"`. `keyId` and `algorithm` are echoed so a response separated from its request is still attributable. Failures (`permissionDenied`, `keys/sign-sshsig:failedPrecondition`, `keys:invalidArgument`) use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### The signature over the commit

```json
{
  "id": "urn:uuid:5f0c2a7e-3d41-4b8e-9a6f-1c2d3e4f5a62",
  "type": "https://trusttasks.org/spec/keys/sign-sshsig/0.1#response",
  "threadId": "urn:uuid:5f0c2a7e-3d41-4b8e-9a6f-1c2d3e4f5a61",
  "issuer": "did:example:alice-custodian",
  "recipient": "did:example:alice-laptop",
  "issuedAt": "2026-10-05T09:00:01Z",
  "payload": {
    "keyId": "did:example:alice#key-1",
    "algorithm": "EdDSA",
    "signature": "qQkWdYdoux5DnKSW0G1nIGQMrGC2L1RJkfuq9nrNuuAlU8hFazUPwKWlS3A68WWHs0DQxVbPl2GXjPUJdSIPBQ"
  }
}
```

The producer wraps that signature as `string "ssh-ed25519" || string signature`, places it with the public key, namespace and hash algorithm in the SSHSIG blob, and armours the blob between `-----BEGIN SSH SIGNATURE-----` lines — the output `ssh-keygen -Y sign` would have produced.

## Security & Privacy

### Data carried

The request carries a key identifier, an algorithm, a namespace and a digest; the response carries a signature and the echoes. No key material travels in either direction, and the message being signed — a commit object, naming its author, tree and message — does not travel at all. That is the smallest conforming request: every member is required and none is padding.

The difference from [`keys/sign`](../../sign/0.1/spec.md) is in what the custodian can know. There, it signs bytes it cannot interpret and so can apply no policy to them. Here, it knows exactly what it is signing — an SSHSIG statement, in a named namespace, over some message — and can refuse a namespace, rate-limit a key, and write a meaningful audit line, all without learning what the message was.

### Correlation

A signature verifies under the key's public half forever, and every signature made under one key is linked to every other; this is the purpose of commit signing, not a side effect of it. A key used for commit signing **SHOULD NOT** be used for anything else, which this task helps enforce: its signatures cannot be repurposed outside SSHSIG.

The digest is a stable identifier of the signed message. Anyone holding the message can recompute it, so an audit trail that records digests can be joined to the public repository it signed into — which is what makes it useful for incident response, and why it is also a record of the producer's activity. The sequence of requests — which keys, how often, when — is likewise a behavioural trace of the producer, available to the custodian without any message content.

### Retention

Transient at the custodian. Nothing is stored by the operation itself. A deployment that keeps an audit line **SHOULD** keep producer, key, namespace, digest and time, and nothing else; that is enough to answer "who signed this commit" after a compromise, and keeping more would be keeping data the task never needed.

### Consent/purpose

The purpose is exercise of an existing custody relationship: a producer already entitled over a key asks the custodian to sign a statement with it. Unlike `keys/sign`, the purpose is partly visible in the request — the namespace states what kind of statement this is — so a deployment can limit a key to one purpose structurally, by refusing the namespaces it was not provisioned for. Whether a particular signature also needs a human's approval is the custodian's policy, not something this specification declares.
