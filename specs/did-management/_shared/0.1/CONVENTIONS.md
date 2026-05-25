# did-management — category conventions

This document captures invariants that apply to **every** specification under the `did-management/` category. Individual specs reference this file rather than restating the same prose; conformance is anchored here.

## 1. Domain resolution

Every operation that creates, reads, or mutates a DID slot — or queries slots by listing or by domain — MAY carry an optional `payload.domain` field. The hosting service resolves the target hosting domain via the following ordered chain:

1. **Explicit on the wire.** If `payload.domain` is present and non-empty, the consumer uses it directly.
2. **Caller's ACL default.** If the calling party's ACL record carries a `DomainScope` with a default domain, that value is used.
3. **System default.** If the calling party's scope is `All` (no per-caller default), the host's system-default domain is used.
4. **Reject.** If none of the above resolves, the consumer responds with the framework's `malformed_request` carrying `details.reason: "no_default_domain"` so the caller knows to declare a target explicitly.

This chain is mandatory across the category. It enables a third-party operator (for example, a VTA managing DID lifecycle across multiple tenant domains on one shared `did-hosting-control` backplane) to direct every operation at the intended domain by passing `payload.domain` explicitly, without having to track per-caller defaults out-of-band.

## 2. Unknown-domain error

Every specification in this category that accepts a `payload.domain` MUST list the following extended error code in its `errorCodes` declaration:

```yaml
- code: did-management:unknown_domain
  meaning: The submitted `domain` is not a known hosting domain on this consumer, or is in the `disabled` state and the operation is not permitted under that status.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      domain: { type: string }
      activeDomains:
        type: array
        items: { type: string }
        description: "Optional. Hosts MAY return the list of active hosting domains to assist the caller. Hosts on privacy-sensitive deployments SHOULD omit this list."
```

The code is namespaced under the `did-management/_shared` pseudo-slug because it's the same condition across the family — consumers SHOULD recognise it uniformly.

## 3. Per-domain mnemonic disambiguation

For operations that identify a DID slot by `mnemonic`, the `(mnemonic, domain)` pair is the conceptual primary key. Consumers MAY implement a flat mnemonic namespace today (one `mnemonic` per host, regardless of domain), but the wire shape SHOULD always permit the caller to disambiguate by also passing `domain`. A consumer that ignores `domain` on a lookup MUST behave as if the domain matched the slot's recorded value; if the explicit `domain` is inconsistent with the slot's recorded one, the consumer MUST respond with `did-management:unknown_domain` rather than returning the wrong slot.

## 4. Audit fields

Every spec's `DidRecord` (and `DomainEntry` / `ServiceInstance` where applicable) carries `createdAt`, `updatedAt`, and — where the host supports it — `disabledAt`, `deletedAt`. These are written by the consumer (the host applies its own clock) and SHOULD be treated as authoritative by the producer; producers MUST NOT submit values for these fields.

## 5. Versioning

The category's first release is `0.1` across every constituent spec — versions evolve independently after that. A `MINOR` bump within a spec MUST remain wire-compatible with prior `MINOR`s of the same `MAJOR`. The category as a whole does not carry a version number; each spec stands on its own.

## 6. DID-method scoping and per-method extensions

This category is scoped to **hosted-DID methods** — DID methods whose resolution is anchored at an HTTP host the operator runs (`did:web`, `did:webvh`, and the planned `did:webs`, `did:webplus`). DID methods whose canonical state lives elsewhere (blockchain-anchored methods like `did:ion`, `did:ethr`, `did:cheqd`, `did:plc`; transport-only methods like `did:peer`, `did:key`) have fundamentally different operational concerns — provisioning is on-chain or in-protocol, there is no hosting domain to manage, and there is no log to publish via this surface — so they SHOULD register their own Trust Tasks under a different category (e.g. `did-anchored/...`) rather than overload this one.

Within the hosted-method scope, each operation declares the target method explicitly via `payload.method` (`"webvh"`, `"web"`, …). The consumer dispatches method-specific validation off that discriminator. The common fields modelled at the top of every payload — `path`, `domain`, `didData`, `force`, etc. — are deliberately small and method-agnostic.

**Method-specific metadata** lives in `payload.ext` under a reverse-DNS namespace that names the method:

```json
"ext": {
  "vnd.trusttasks.did-method-webvh": {
    "witnessUrls": ["https://witness1.example.com", "https://witness2.example.com"],
    "scid": "abc123",
    "updateKeyMultibase": "z6Mk..."
  }
}
```

This keeps the core wire shape stable across methods. A consumer that doesn't recognise the method's `ext` namespace MUST ignore it (per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member)) — the operation still proceeds against the common fields, and the method-specific consumer (e.g. a webvh-aware hosting service) reads the extension to validate its own invariants.

A method-specific extension shape MAY be published as a separate Trust Task spec or, more commonly, as a schema file under `did-management/_shared/0.1/did-method-extensions/<method>.schema.json` referenced by the spec's `ext` description. Both forms are conformant; the schema-file form keeps the published Trust Task catalogue compact.

The webvh method extension shape is sketched in [`did-method-extensions/webvh.schema.json`](did-method-extensions/webvh.schema.json) — a `draft` artefact that EVOLVES with the [`did:webvh` method spec](https://www.w3.org/TR/did-web-vh/).
