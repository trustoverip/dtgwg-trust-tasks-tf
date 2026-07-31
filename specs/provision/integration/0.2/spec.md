---
slug: provision/integration
version: "0.2"
wireCompatibleWith: "0.1"
title: Provision — Integration
summary: A relayer presents a VP-signed bootstrap request from an integration holder; the maintainer mints the integration's DIDs and admin credential from a registered DID template and returns the material in a sealed bundle the holder can open.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - provision
  - integration
  - bootstrap
  - did-template
  - sealed-bundle
  - admin-rotation
  - vc
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: integration holder
    requirement: REQUIRED
  - role: integration relayer
    requirement: OPTIONAL
    member: issuer
  - role: provisioning maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Provisioning mints DIDs, issues an authorization VC, and grants the resulting admin DID an ACL row at the maintainer — the equivalent of "create an account with admin powers." Two distinct proofs are involved (the relayer's transport-level credential authenticating the *caller*, and the holder's VP `DataIntegrityProof` authenticating *who the bundle belongs to*); both MUST be present and verified before the maintainer mints anything. See "Two-proof model" in the spec body.
sideEffects:
  level: mutating
  rationale: "Mints the integration's DIDs and admin credential from a template and returns a sealed bundle; the credential is revocable."
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: "Returns a sealed bundle carrying the minted admin credential and the integration's DID key material."
errorCodes:
  - code: provision/integration:invalidBootstrapRequest
    meaning: The presented VP failed structural validation (missing required field, malformed `holder`, unsupported cryptosuite, freshness window passed, signature does not verify, `verificationMethod` does not resolve under `holder`).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum:
            - "missing_type"
            - "holder_invalid"
            - "cryptosuite_unsupported"
            - "verification_method_mismatch"
            - "signature_invalid"
            - "expired"
            - "nonce_invalid"
            - "shape"
  - code: provision/integration:templateNotFound
    meaning: The integration or admin template named in the ask is not registered at the maintainer. Operator must upload it via the maintainer's template-management surface before retrying.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        templateName: { type: "string" }
        kind:
          type: "string"
          enum: ["integration", "admin"]
  - code: provision/integration:templateVarsInvalid
    meaning: The template's `requiredVars` are not satisfied by `template.vars`, or unknown vars were supplied. Producer SHOULD consult the template's declaration and retry with corrected bindings.
    retryable: false
  - code: provision/integration:contextNotFound
    meaning: "The requested `context` does not exist at the maintainer and `createContext` was either omitted or denied. When the caller has super-admin privileges they MAY retry with `createContext = true` to provision the context inline."
    retryable: false
  - code: provision/integration:contextRequired
    meaning: "`payload.context` was omitted and the maintainer could not infer a unique target context from the relayer's grant. The relayer either holds admin role in multiple contexts (rule #1 ambiguous) or is a super-admin and the maintainer has multiple contexts registered (rule #2 ambiguous). The relayer SHOULD retry with an explicit `context` value selected from `details.candidates`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        candidates:
          type: array
          minItems: 2
          items: { type: "string", minLength: 1 }
          description: "Contexts the maintainer considered as plausible targets. The relayer picks one and retries."
  - code: provision/integration:forbidden
    meaning: The authenticated caller is not authorised to provision into `context` (or to create it). Distinct from the framework's `unauthorized` — the caller was authenticated successfully but lacks the role.
    retryable: false
  - code: provision/integration:envelopeUnsupported
    meaning: The maintainer cannot emit a sealed bundle in any cipher envelope the holder's `did:key` supports. The wire shape pins HPKE/X25519-HKDF-SHA256/ChaCha20-Poly1305 as the only envelope today; the code reserves the slot for future envelope negotiation.
    retryable: false
  - code: provision/integration:assertionUnsupported
    meaning: The producer requested an `assertion` mode (e.g. `attested` for TEE deployments) that the maintainer does not support in its current configuration.
    retryable: false
related:
  - acl/grant
  - acl/swap-key
  - did-management/did/register
  - did-management/registry/admin-register
---

## Abstract

The **Provision — Integration** Trust Task is the canonical bootstrap surface for *every* new integration of a maintainer (a VTA in the canonical deployment): mediators, DID-hosting controls, DID-hosting daemons, DID-hosting servers, application identities, mobile companions, AI-agent services, and any future integration kind. The maintainer renders one (or two) DID templates the operator has already registered, mints the DIDs and private keys server-side, issues a short-lived W3C *Verifiable Credential* authorising the resulting admin DID against a named *context*, and ships the whole bundle back HPKE-sealed to the integration's ephemeral `holder` `did:key`.

The maintainer always mints the keys. Holders never present long-lived private material on the wire; the only key they bring is the **ephemeral `did:key`** that signs the bootstrap VP and opens the sealed bundle, used once and discarded.

Two distinct flows are expressed as the `ask` tagged union on the bootstrap request:

* **`templateBootstrap`** — mints an *integration* DID from `template`, optionally mints a *long-term admin* DID from `adminTemplate` ("admin rollover" — the resulting admin DID replaces the ephemeral `holder` in the maintainer's ACL in the same transaction), and returns both plus the rendered DID document, the `did.jsonl` log (for `did:webvh` templates), template-declared side outputs, and the issued authorization VC.
* **`adminRotation`** — mints **only** a long-term admin DID from `adminTemplate`. No integration template is rendered, no integration DID exists. Used when the holder brings (or will mint elsewhere) its own integration-side DIDs and only needs an admin credential at this maintainer.

Both variants share the same envelope, the same proof model, and the same sealed-bundle transport.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

This task is the *canonical migration target* of the private FPN-namespaced `provision-integration/1.0` protocol (URI `https://firstperson.network/protocols/provision-integration/1.0`) that earlier VTA deployments speak. Maintainers MAY accept both URIs during a deprecation window; new consumers MUST emit the canonical Trust Task URI.

## Two-proof model

Two cryptographic proofs MUST be present and MUST agree on `holder` identity before the maintainer mints anything:

1. **The transport-level credential** — DIDComm authcrypt `from` (DIDComm transport) or Bearer JWT subject (REST transport) — authenticates the *relayer* (the party making the call). The relayer is ACL-checked at the maintainer; it MUST hold admin role in the target `context`. The relayer is **not** required to equal the bootstrap holder; the framework explicitly supports air-gap onboarding flows where a trusted relayer hands off a sealed bundle to an offline holder who then opens it independently.

2. **The bootstrap VP's `DataIntegrityProof`** — covers every field of `request` (the VP) including `ask`, `nonce`, and `validUntil`. The proof's `verificationMethod` MUST resolve under the VP's `holder` DID. This proof authenticates the *holder* — the party the bundle is sealed *for*. An attacker who steals the relayer's bearer token cannot mint a new bundle for a holder they don't control because they cannot forge the VP signature; a relayer cannot redirect a bundle to a different holder because the bundle is HPKE-sealed to the VP's `holder` DID.

When the relayer and holder are the **same** party (the common case — `pnm bootstrap provision-integration`), the same DID may appear at both layers. The framework does not require them to differ; it requires them to be *separately verifiable*.

The maintainer MUST emit `unauthorized` (framework) for transport-level auth failures and `provision/integration:forbidden` (this spec) for cases where the caller authenticated successfully but lacks the role in `context`. CLIs distinguish the two so operators receive accurate "your token expired" vs "your role is wrong" hints.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the integration holder or its relayer) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/provision/integration/0.2`, with the relayer as `issuer` and the maintainer as `recipient`. Carry a transport-level proof per the framework's transport bindings (DIDComm authcrypt or REST bearer).
2. Populate `payload.request` with a VP-framed `BootstrapRequest` signed by the holder's ephemeral `did:key` (Ed25519). The VP MUST:
   * declare `@context` containing both `https://www.w3.org/ns/credentials/v2` and `https://openvtc.org/contexts/bootstrap-v1`;
   * declare `type` containing both `VerifiablePresentation` and `BootstrapRequest`;
   * carry an `id` URN-shaped (the `urn:uuid:<v4>` convention is RECOMMENDED);
   * carry a `holder` that decodes as a valid `did:key:z6Mk…` Ed25519 DID;
   * carry a `nonce` of 16 random bytes encoded as base64url-no-pad — this value becomes the sealed-bundle `bundleId`;
   * carry a `validUntil` RFC 3339 UTC timestamp; the maintainer enforces ±5min skew;
   * carry an `ask` per "Ask variants" below;
   * carry a `proof` of cryptosuite `eddsa-jcs-2022`, `proofPurpose: "authentication"`, whose `verificationMethod` resolves under `holder`. The proof MUST sign the JCS canonicalisation of the VP with `proof` removed.
3. **MAY** populate `payload.context` with the maintainer's context identifier the integration is to land in. Producers that don't track the maintainer's context layout (typically wallet-class consumers — browser plugins, mobile companions) SHOULD omit this and let the maintainer infer per "Context inference" below. Producers targeting a specific operational context (typically integration-class consumers — mediators, did-hosting hosts) SHOULD send it explicitly.
4. **MAY** include `payload.assertion` to request a non-default sealed-bundle assertion mode. The wire enum is `"didSigned"` (default — Ed25519 signature over the bundle's domain-bound digest, verified by the holder out-of-band against the producer's published key) and `"pinnedOnly"` (the holder pins the bundle's SHA-256 digest as the sole integrity anchor; for dev/test only).
5. **MAY** include `payload.vcValiditySeconds` to request a non-default validity window for the issued authorization VC. The maintainer's policy decides the floor and ceiling; values outside that range MAY be silently clamped or rejected with `provision/integration:invalidBootstrapRequest` (`reason: "shape"`).
6. **MAY** include `payload.createContext: true` to provision the target context inline if it does not exist. The maintainer accepts this **only** when the caller has super-admin role; context-admin callers MUST receive `provision/integration:forbidden` against a missing context.
7. Persist the holder's Ed25519 seed in operator-private storage (RECOMMENDED file mode `0600` on POSIX, ACL-restricted on Windows). The same seed derives the X25519 receiver key the sealed bundle is opened with.

A conforming **consumer** (the provisioning maintainer) **MUST**:

1. Validate the *Trust Task document* per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements). Verify the transport-level proof; refuse with `unauthorized` if it fails.
2. Validate the VP structurally before any signature work: required fields present, `type` array contains the two reserved entries, `holder` decodes as `did:key`, `validUntil` parses as RFC 3339 UTC and falls within the maintainer's freshness window. Failures emit `provision/integration:invalidBootstrapRequest` with a `details.reason` from the enum.
3. Verify the VP's `proof`:
   * cryptosuite MUST equal `eddsa-jcs-2022`;
   * `verificationMethod` MUST split on `#` into a DID that equals `holder`;
   * the proof MUST verify against the JCS canonicalisation of the VP with `proof` removed, using the public key derived from `holder`.
4. Validate the `ask` variant against the maintainer's template registry:
   * for `templateBootstrap`: `template.name` MUST be registered with `kind` ≠ `"admin"`; when `adminTemplate` is present, `adminTemplate.name` MUST be registered with `kind == "admin"`. Missing → `provision/integration:templateNotFound`.
   * for `adminRotation`: `adminTemplate.name` MUST be registered with `kind == "admin"`. Missing → `provision/integration:templateNotFound`.
   * For every named template, validate `vars` against the template's declared `requiredVars` / `optionalVars`. Missing or unknown vars → `provision/integration:templateVarsInvalid`.
5. Resolve the target context: use `payload.context` when present; otherwise infer per "Context inference" above and emit `provision/integration:contextRequired` if no unique target can be determined. Authorise the relayer in the resolved context. If the context does not exist and `payload.createContext` is `true`, the relayer MUST be super-admin; otherwise emit `provision/integration:contextNotFound`. If the relayer is authenticated but lacks the required role, emit `provision/integration:forbidden`.
6. Mint the keys server-side:
   * for `templateBootstrap`: render the integration template; mint Ed25519 (signing) and X25519 (key-agreement) keypairs for every verification method the template declares; persist private halves in the maintainer's keystore; allocate a DID identifier per the template's method (`did:webvh` writes a `did.jsonl` log; `did:key` derives from the signing pubkey). When `adminTemplate` is present, additionally mint the admin DID + keys.
   * for `adminRotation`: render the admin template only.
7. Issue a `VtaAuthorizationCredential` (W3C VC, JSON-LD, signed with the maintainer's assertionMethod key) whose `credentialSubject` is the long-term admin DID (when `adminTemplate` was used) or the ephemeral `holder` (when `templateBootstrap` had no `adminTemplate` — the "no admin rollover" path). The VC's `validFrom` is `now`; `validUntil` is `now + vcValiditySeconds` capped to the maintainer's policy. The VC has no `credentialStatus` — revocation is ACL removal, not status change.
8. Bind an `AclEntry` for the long-term admin DID (admin role) in `context`, in the same transaction as bundle assembly. When `adminTemplate` was used the entry is created against the freshly-minted admin DID; the maintainer MUST NOT leave the ephemeral `holder` with admin role after a successful provisioning — see [`acl/swap-key/0.1`](../../../acl/swap-key/0.1/) for the equivalent ad-hoc rotation.
9. Assemble the sealed payload per [§"Sealed bundle"](#sealed-bundle). HPKE-encrypt to the X25519 derivation of `holder`'s Ed25519 pubkey. Wrap in OpenPGP-style ASCII armor with `Bundle-Id` and `Digest-Algo` headers. Compute the SHA-256 of the *armored ciphertext* and emit it as `summary.digest` (lowercase hex).
10. Emit `payload.summary` with audit-grade metadata: `clientDid`, `adminDid`, `adminRolledOver`, optional `integrationDid` / `templateName` / `templateKind` / `adminTemplateName`, `bundleIdHex` (the VP nonce as hex — MUST match the `bundleId` armor header), `secretCount`, `outputCount`, `webvhServerId` when the integration's DID was published to a registered hosting server, and `contextCreated` when `createContext: true` actually provisioned the context.
11. Audit-log the provisioning with `{ relayer, holder, context, templateName, adminTemplateName, adminDid, bundleIdHex, outcome }`. The log entry MUST persist even if the response delivery later fails — the keys have been minted and the ACL bound; that's an audit event regardless of whether the relayer received the bundle.
12. **MUST NOT** include any private key material outside the sealed `bundle`. The wire `summary` is non-secret metadata only.

A consumer **MAY** support additional `assertion` modes (e.g. `attested` carrying a Nitro attestation quote for TEE deployments). When a producer requests an unsupported mode, emit `provision/integration:assertionUnsupported`.

## Ask variants

The `ask` member of the VP body is a discriminated union, tagged on `type`:

### `templateBootstrap`

```json
{
  "type": "templateBootstrap",
  "contextHint": "default",
  "template": {
    "name": "didcomm-mediator",
    "vars": { "MEDIATOR_DOMAIN": "mediator.example.com" }
  },
  "adminTemplate": {
    "name": "vta-admin",
    "vars": {}
  },
  "note": "first-boot mediator for tenant alpha"
}
```

* `template` (REQUIRED) — the integration template to render. `kind` MUST NOT be `"admin"`.
* `adminTemplate` (OPTIONAL) — when present, the maintainer mints a fresh long-term admin DID, binds the authorization VC + ACL row to it, and *rolls over* the holder from the ephemeral `did:key` in a single transaction. When absent (legacy / pre-rollover behaviour) the VC subject and ACL subject are the ephemeral `holder`, which the operator is expected to swap out via [`acl/swap-key/0.1`](../../../acl/swap-key/0.1/) before steady-state operation.
* `contextHint` (OPTIONAL) — hint for the integration's context. The wire `payload.context` is authoritative.
* `note` (OPTIONAL) — free-form operator note carried into the audit log.

### `adminRotation`

```json
{
  "type": "adminRotation",
  "contextHint": "default",
  "adminTemplate": {
    "name": "vta-admin",
    "vars": {}
  },
  "note": "wallet onboarding"
}
```

* `adminTemplate` (REQUIRED) — admin-DID template. `kind` MUST equal `"admin"`.
* `contextHint`, `note` as above.

The `adminRotation` variant is what a holder that needs *only* an admin identity at this maintainer requests — e.g. a companion browser plugin or mobile wallet whose integration-side identity is irrelevant to the maintainer. The reply carries an `AdminRotationPayload` (see "Sealed bundle"), not a `TemplateBootstrapPayload`.

## Context inference

When the producer omits `payload.context`, the maintainer infers the target context using the following rules in order:

1. **Single-context grant.** If the relayer's ACL entry scopes to exactly one context, use that context. This is the common case for an integration-scoped admin (e.g. `pnm acl create --did <eph> --role admin --contexts ctx_x`): the operator already named the bucket on the wire when they granted the ephemeral, and the maintainer respects that scoping.

2. **Single-context maintainer.** If the relayer is a super-admin (Admin role with unrestricted `allowed_contexts`) and the maintainer has exactly one context registered, use that context. Covers the typical single-VTA, single-context, single-admin deployment where the wallet's ephemeral was granted with `pnm acl create --did <eph> --role admin` (no `--contexts` flag, producing a super-admin grant).

3. **Ambiguous — refuse.** Any other state — multi-context relayer, super-admin against a multi-context maintainer — emits `provision/integration:contextRequired` with `details.candidates` listing the contexts the maintainer considered. The relayer picks one and retries with an explicit `context`.

The inference is opportunistic, not authoritative — when the producer DOES send a `context`, the maintainer uses it verbatim and inference does not run. Producers MAY send `context` even when inference would succeed, e.g. to make audit logs unambiguous; the wire shape supports both modes interchangeably.

Maintainers that wish to expose a configured "primary" context (e.g. TEE deployments that pin `admin_context_id` at boot) MAY treat that as a fallback for case (2) above when multiple contexts are registered — the produced wire-shape is identical from the consumer's perspective. This MUST be documented in the maintainer's operator guide; the spec does not pin which approach the maintainer takes.

## Payload

`payload.request` (REQUIRED) — VP-framed bootstrap request. Full shape under `$defs.BootstrapRequest` of [`payload.schema.json`](payload.schema.json).

`payload.context` (OPTIONAL) — maintainer's context identifier. When absent, the maintainer infers per "Context inference" above. Producers that don't know the maintainer's context layout SHOULD omit; producers targeting a specific operational context SHOULD send.

`payload.assertion` (OPTIONAL) — `"didSigned"` (default) or `"pinnedOnly"`.

`payload.vcValiditySeconds` (OPTIONAL) — caller-preferred VC validity window. Capped server-side.

`payload.createContext` (OPTIONAL, super-admin only) — provision the target context inline if it does not exist.

`payload.ext` (OPTIONAL) — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Sealed bundle

The `bundle` member of the response is an OpenPGP-style ASCII-armored ciphertext. Framing:

```
-----BEGIN VTA SEALED BUNDLE-----
Bundle-Id: <hex>
Digest-Algo: sha-256
Chunk: 0/N

<base64 ciphertext lines, CRC24 checksum on final line>
-----END VTA SEALED BUNDLE-----
```

Cipher: HPKE base mode, X25519-HKDF-SHA256 KEM, HKDF-SHA256 KDF, ChaCha20-Poly1305 AEAD. Info string `vta-sealed-transfer/v1` (domain-bound — future envelope formats use a fresh info string, not a version parameter). AAD covers the armor headers.

Cleartext after HPKE open + CBOR decode is `SealedPayloadV1`, a tagged union whose relevant variants are:

### `templateBootstrap` variant

```cbor
{
  "type": "template_bootstrap",
  "authorization": <VC JSON>,
  "secrets": {
    "did:webvh:host/path": {
      "did": "...",
      "signing_key": { "key_id": "...", "public_key_multibase": "...", "private_key_multibase": "..." },
      "ka_key":      { "key_id": "...", "public_key_multibase": "...", "private_key_multibase": "..." }
    }
  },
  "config": {
    "template_name": "...",
    "template_kind": "...",
    "did_document": <DID doc JSON>,
    "outputs": [ <TemplateOutput>... ],
    "vta_url": "https://vta.example",
    "vta_trust": { "vta_did": "...", "vta_did_document": <JSON>, "vta_did_log": "..." }
  }
}
```

### `adminRotation` variant

```cbor
{
  "type": "admin_rotation",
  "authorization": <VC JSON>,
  "admin": {
    "did": "did:key:z6Mk…",
    "signing_key": { ... },
    "ka_key":      { ... }
  },
  "vta_url": "https://vta.example",
  "vta_trust": { "vta_did": "...", "vta_did_document": <JSON>, "vta_did_log": null }
}
```

Holders that need only the long-term DID + private key (the wallet case) extract `admin.did` and `admin.signing_key.private_key_multibase` (plus `admin.ka_key.private_key_multibase` for DIDComm) and discard everything else. Holders that need to operate as an integration (mediator, did-hosting host) consume the full `templateBootstrap` payload including `config.did_document` and `config.outputs[]`.

Sealed-bundle internal fields are `snake_case` (preserving the existing CBOR-level wire shape, which downstream tools depend on for cross-language opens). Trust-Task wire-level fields outside the sealed envelope are `camelCase` per the framework convention.

## Producer assertion

Independent of HPKE confidentiality, the producer asserts authorship of the bundle out-of-band:

* `didSigned` (default) — Ed25519 signature over `DID_SIGNED_DOMAIN_TAG || client_x25519_pub || bundle_id` where `DID_SIGNED_DOMAIN_TAG = b"vta-sealed-transfer/v1\0"`. Verified by the holder using the maintainer's published Ed25519 key.
* `pinnedOnly` — no signature; the holder is expected to have pinned `summary.digest` out-of-band before opening. Dev/test deployments only.

Maintainers MAY support `attested` as an additional mode carrying a Nitro attestation quote bound to the holder's X25519 pubkey + `bundle_id` + producer's Ed25519 pubkey; this is implementation-defined and out of scope of this version of the spec.

## Examples

### TemplateBootstrap with admin rollover (mediator first-boot)

Request:

```json
{
  "id": "prov-01HZX2…",
  "type": "https://trusttasks.org/spec/provision/integration/0.2",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-26T13:00:00Z",
  "payload": {
    "request": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://openvtc.org/contexts/bootstrap-v1"
      ],
      "type": ["VerifiablePresentation", "BootstrapRequest"],
      "id": "urn:uuid:9a8c4b…",
      "holder": "did:key:z6MkpTH…",
      "nonce": "RmFrZU5vbmNlVmFsdWVYWFg",
      "validUntil": "2026-05-26T13:15:00Z",
      "label": "tenant-alpha-mediator",
      "ask": {
        "type": "templateBootstrap",
        "contextHint": "default",
        "template": {
          "name": "didcomm-mediator",
          "vars": { "MEDIATOR_DOMAIN": "mediator.example.com" }
        },
        "adminTemplate": {
          "name": "vta-admin",
          "vars": {}
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkpTH…#z6MkpTH…",
        "created": "2026-05-26T13:00:00Z",
        "proofPurpose": "authentication",
        "proofValue": "z3kg…"
      }
    },
    "context": "default",
    "assertion": "didSigned"
  },
  "proof": { "…": "…" }
}
```

Response:

```json
{
  "id": "prov-resp-01HZX2…",
  "type": "https://trusttasks.org/spec/provision/integration/0.2#response",
  "threadId": "prov-01HZX2…",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-05-26T13:00:01Z",
  "payload": {
    "bundle": "-----BEGIN VTA SEALED BUNDLE-----\nBundle-Id: 4665ab…\nDigest-Algo: sha-256\nChunk: 0/1\n\nU3VwZXJTZWNyZXRDaXBoZXJ0ZXh0\n=Q1JD\n-----END VTA SEALED BUNDLE-----",
    "digest": "9a8c4b6d2e1f…",
    "summary": {
      "clientDid": "did:key:z6MkpTH…",
      "adminDid": "did:key:z6MkbXq…",
      "adminRolledOver": true,
      "integrationDid": "did:webvh:mediator.example.com",
      "templateName": "didcomm-mediator",
      "templateKind": "mediator",
      "adminTemplateName": "vta-admin",
      "bundleIdHex": "4665ab…",
      "secretCount": 1,
      "outputCount": 2,
      "contextCreated": false
    }
  }
}
```

### AdminRotation (browser-plugin onboarding)

Wallet-class consumer; `context` omitted so the maintainer infers per "Context inference" — typically rule (1) when the ephemeral was granted with `--contexts` or rule (2) for a super-admin grant against a single-context maintainer:

```json
{
  "request": {
    "@context": ["https://www.w3.org/ns/credentials/v2", "https://openvtc.org/contexts/bootstrap-v1"],
    "type": ["VerifiablePresentation", "BootstrapRequest"],
    "id": "urn:uuid:0e1f2a…",
    "holder": "did:key:z6Mkve…",
    "nonce": "QnJvd3Nlck5vbmNlVlhYWFhY",
    "validUntil": "2026-05-26T13:15:00Z",
    "ask": {
      "type": "adminRotation",
      "adminTemplate": { "name": "vta-admin", "vars": {} },
      "note": "companion: brave / glenn-mbp"
    },
    "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "verificationMethod": "did:key:z6Mkve…#z6Mkve…", "created": "2026-05-26T13:00:00Z", "proofPurpose": "authentication", "proofValue": "z6ab…" }
  }
}
```

Response summary:

```json
{
  "clientDid": "did:key:z6Mkve…",
  "adminDid": "did:key:z6Mk9rT…",
  "adminRolledOver": true,
  "adminTemplateName": "vta-admin",
  "bundleIdHex": "5be8c1…",
  "secretCount": 0,
  "outputCount": 0,
  "contextCreated": false
}
```

`integrationDid`, `templateName`, `templateKind` are absent — no integration was rendered.

### Failure: template not registered

```json
{
  "id": "prov-resp-err-01HZX3…",
  "type": "https://trusttasks.org/spec/trust-task-error/0.2",
  "threadId": "prov-01HZX2…",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-05-26T13:00:01Z",
  "payload": {
    "code": "provision/integration:templateNotFound",
    "message": "template 'mediator-custom' is not registered at this maintainer",
    "details": { "templateName": "mediator-custom", "kind": "integration" }
  }
}
```

## Security and privacy considerations

* **Idempotency.** The maintainer SHOULD treat the VP's `nonce` as a one-shot replay anchor: a second `provision/integration` whose `nonce` matches a previously-completed provisioning MUST be refused. The 16-byte random nonce makes collisions cryptographically infeasible.
* **Bundle digest leakage.** `summary.digest` is the SHA-256 of the armored ciphertext. It is not secret on its own, but maintainers MAY decline to publish it in any surface less authenticated than the response itself to prevent an unauthenticated party from confirming "a bundle with this digest was produced." Treat it like a session identifier.
* **VP cryptosuite agility.** This version pins `eddsa-jcs-2022`. A future minor version MAY extend the allowlist; producers and consumers MUST NOT silently accept other suites, and producers MUST NOT degrade in response to an unsupported-suite error — there is no negotiation path.
* **Air-gap relayer.** When relayer ≠ holder, the relayer cannot decrypt the bundle (no holder X25519 private key) and the VP cannot be forged without the holder's signing key. The threat model assumes the relayer can read but not modify the request body; it relies on the holder verifying `summary.digest` against the bundle they receive out-of-band.
* **VC validity floor.** Maintainers SHOULD reject `vcValiditySeconds` below their minimum useful operating window (RECOMMENDED: 60s) — a near-zero VC validity creates noisy auditing without operational value.
* **ACL race.** The maintainer MUST bind the resulting ACL row in the same transaction as bundle assembly. A partial provisioning that mints keys without binding the ACL row leaves the holder with no way to authenticate at the maintainer; the maintainer MUST roll the mint back rather than ship a half-broken bundle.

## Migration from `firstperson.network/protocols/provision-integration/1.0`

Existing VTA deployments speak the private FPN protocol at the URI `https://firstperson.network/protocols/provision-integration/1.0/provision-integration` (request) and `…/provision-integration-result` (reply). Maintainers SHOULD accept both URIs during a deprecation window so older PNM/CNM CLIs continue to work; new producers MUST emit the canonical Trust Task URI.

Wire shape is **identical in semantics** but the canonical Trust Task spec is `camelCase` outside the sealed envelope; the FPN-private DIDComm body used `snake_case` for response fields (`client_did`, `admin_did`, `bundle_id_hex`, etc.). Maintainers SHOULD emit both shapes during the migration window — older consumers expect `snake_case`. The sealed bundle's CBOR-level fields remain `snake_case` so existing openers continue to work without modification.

## References

* [`acl/swap-key/0.1`](../../../acl/swap-key/0.1/) — the equivalent ad-hoc rotation when the holder already has an admin entry and wants to swap to a new DID without re-provisioning.
* [`acl/grant/0.1`](../../../acl/grant/0.1/) — the post-provisioning surface for granting additional roles to other DIDs in the same context.
* [`did-management/did/register/0.1`](../../../did-management/did/register/0.1/) — the DID-lifecycle surface the rendered integration DID will be operated through.
* W3C VC Data Model 2.0 §6.1 — VPs MAY omit `verifiableCredential`; the `BootstrapRequest` is a self-attested VP under this clause.
* W3C Data Integrity 1.0 + `eddsa-jcs-2022` cryptosuite — the VP's proof format.
