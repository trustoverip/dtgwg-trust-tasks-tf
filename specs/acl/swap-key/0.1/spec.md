---
slug: acl/swap-key
version: "0.1"
title: ACL — Swap Key
summary: An ACL holder atomically replaces the VID bound to one of their own AclEntries with a new VID — preserving role, scopes, and label, and closing the old VID's access in the same transaction.
status: draft
targetFrameworkVersion: "0.1"
category: access-control
keywords:
  - acl
  - access-control
  - key-rotation
  - did-rotation
  - swap
  - recovery
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: ACL holder
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Swap-key is a high-trust mutation — the holder is changing which key controls future access to their resources. Without a verified proof the maintainer cannot tell the swap apart from a hostile takeover by an attacker who has captured a single access token.
errorCodes:
  - code: acl/swap-key:subject_not_found
    meaning: The `currentSubject` is not present in the ACL.
    retryable: false
  - code: acl/swap-key:subject_already_in_use
    meaning: The `newSubject` is already bound to a different AclEntry. The maintainer's policy decides whether to support "merge" semantics; the default is to refuse, leaving the operator to remove the existing entry first.
    retryable: false
  - code: acl/swap-key:link_proof_required
    meaning: The maintainer requires evidence that `newSubject` consents to taking over (see "Link proof"). The producer SHOULD retry with `linkProof` populated.
    retryable: false
  - code: acl/swap-key:link_proof_invalid
    meaning: The supplied `linkProof` failed verification.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum: ["signature_invalid", "nonce_mismatch", "subject_mismatch", "expired", "format_unsupported"]
  - code: acl/swap-key:not_holder
    meaning: The document's `issuer` is not the `currentSubject` and the maintainer's policy does not permit cross-subject swaps. This is the default policy — see "Administrative swap" for the exception.
    retryable: false
related:
  - acl/grant
  - acl/revoke
  - acl/change-role
  - acl/list
---

## Abstract

The **ACL — Swap Key** Trust Task atomically rebinds an AclEntry from one *subject* (VID) to another, preserving the entry's `role`, `scopes`, `label`, and `expiresAt`. The maintainer applies the swap as a single transaction: at no point do both `currentSubject` and `newSubject` hold access, and at no point is the entry missing.

This task is the canonical solution to *key rotation without a service window*. Existing patterns — grant the new subject, wait for caller-visible propagation, revoke the old — leave a transient over-privileged state in which a compromised old key still has access. Swap-key closes that window by treating the rotation as a single mutation.

Swap-key does NOT issue or revoke tokens itself. The maintainer's policy decides whether existing sessions bound to `currentSubject` are revoked, kept, or migrated — see the conformance section. Producers that want to elevate to a new key without dropping in-flight access on the old key MUST hold separate sessions on each VID until the swap commits.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the ACL holder) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/swap-key/0.1`, with itself (the holder) as `issuer` and the ACL maintainer as `recipient`.
2. Populate `payload.currentSubject` with the VID currently bound in the ACL.
3. Populate `payload.newSubject` with the VID to bind after the swap. `newSubject` MUST differ from `currentSubject`.
4. The `issuer` of the document MUST equal `currentSubject` (the default policy: only the existing holder may swap their own entry). Maintainers whose policy permits administrative swap MAY accept other issuers — see "Administrative swap" below.
5. Include a `proof` per [SPEC.md §4.7](../../../../SPEC.md#47-proof). The proof's `verificationMethod` MUST resolve via `currentSubject`'s DID document.
6. **MAY** include `payload.linkProof` — see "Link proof" below.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Verify the document's `issuer` equals `payload.currentSubject` (unless administrative swap is configured — see below).
3. Look up `payload.currentSubject` in the ACL. Absent → `acl/swap-key:subject_not_found`.
4. Verify `payload.newSubject` is not already an AclEntry subject. Present → `acl/swap-key:subject_already_in_use`.
5. Apply the consumer's link-proof policy:
   - If link-proof is REQUIRED for `newSubject`'s VID scheme and `payload.linkProof` is absent → `acl/swap-key:link_proof_required`.
   - When present, verify it per the consumer's policy. Failure → `acl/swap-key:link_proof_invalid` with `details.reason`.
6. Apply the swap as a single transaction: rebind the existing AclEntry's `subject` to `payload.newSubject` and remove `payload.currentSubject` from any subject indexes. The entry's `role`, `scopes`, `label`, `expiresAt` MUST be preserved verbatim. The entry's `createdAt`/`createdBy` MUST NOT be modified.
7. Revoke any persistent authentication artifacts bound to `currentSubject` (refresh tokens, server-side sessions, signed bearer tokens issued to that VID). The maintainer's policy decides whether short-lived access tokens are also revoked — RECOMMENDED: yes; RATIONALE: if the holder rotated because the old key was compromised, leaving a 15-minute access-token window open is the gap that prompted the rotation in the first place.
8. Return a `#response` document carrying the rebound `AclEntry` and the echoed `previousSubject`. Operate retries idempotently: a swap whose `currentSubject` is no longer present MAY succeed without effect if the maintainer can verify the same `newSubject` is already the bound entry (i.e. a prior swap with identical parameters succeeded).

A consumer **MAY** also accept swap requests from an *administrative* issuer whose `subject` differs from `currentSubject`. Maintainers whose policy supports this **MUST** ensure the administrator's authority to swap others' entries is logged and audit-traceable. The framework does not specify the policy itself — that's a trust-framework concern.

## Definitions

* **ACL holder.** The party whose AclEntry is being swapped; default policy: equals `issuer` and `payload.currentSubject`.
* **ACL maintainer.** The party storing and enforcing the ACL; identified by `recipient`.
* **currentSubject.** The VID being swapped out.
* **newSubject.** The VID being swapped in.
* **Link proof.** Optional evidence that `newSubject` consents to the swap. See below.

## Link proof

Some maintainer policies require explicit evidence that `newSubject` consents to take over. This protects against an attacker compromising `currentSubject` and rebinding their access to a VID under attacker control — in that scenario, the attacker controls `currentSubject` and can sign a swap, but cannot mint a `linkProof` valid for `newSubject` (which is the legitimate holder's new key).

The framework deliberately does not pin a single `linkProof` format. Common shapes:

1. **Trust Task envelope.** `linkProof` is itself an `acl/swap-key/0.1` (or a dedicated wire-form) document signed by `newSubject`, echoing `currentSubject` + `newSubject` and carrying a maintainer-issued nonce. The maintainer verifies the inner document's proof with `newSubject`'s key.
2. **VP/VC.** A Verifiable Presentation signed by `newSubject` carrying a maintainer-issued challenge. Useful when `newSubject`'s key system is already in the W3C VC ecosystem.
3. **Plain JWS.** A JWS over `{ currentSubject, newSubject, nonce, exp }`, signed by `newSubject`.

Maintainers documenting their policy MUST specify which formats they accept and how the nonce / challenge is established (push from maintainer, holder-supplied, or derived from the maintainer's tip-of-chain state). When the holder is online, the maintainer SHOULD issue the nonce via a sibling Trust Task (`auth/challenge/0.1` is a fine choice) so the wire shape is consistent across the family.

## Payload

`payload.currentSubject` (REQUIRED) — VID being swapped out.

`payload.newSubject` (REQUIRED) — VID being swapped in; MUST differ from `currentSubject`.

`payload.linkProof` (optional, REQUIRED by some maintainer policies) — see above.

`payload.reason` (optional) — human-readable rationale.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

The full JSON Schema is in [`payload.schema.json`](payload.schema.json).

## Examples

### Standard self-swap (no link proof required by maintainer policy)

```json
{
  "id": "swap-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/acl/swap-key/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-23T15:00:00Z",
  "payload": {
    "currentSubject": "did:web:alice.example",
    "newSubject": "did:peer:2.Ez6LSc…",
    "reason": "key-rotation"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T15:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Swap with link proof (maintainer requires newSubject consent)

```json
{
  "id": "swap-2345-6789-01bc-def234567890",
  "type": "https://trusttasks.org/spec/acl/swap-key/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-23T15:00:00Z",
  "payload": {
    "currentSubject": "did:web:alice.example",
    "newSubject": "did:peer:2.Ez6LSc…",
    "linkProof": {
      "type": "DataIntegrityProof",
      "cryptosuite": "eddsa-jcs-2022",
      "verificationMethod": "did:peer:2.Ez6LSc…#key-1",
      "created": "2026-05-23T15:00:00Z",
      "proofPurpose": "assertionMethod",
      "proofValue": "z6ab…",
      "challenge": "TWFpbnRhaW5lck5vbmNlVmFsdWU"
    },
    "reason": "hardware-token-replacement"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/swap-key/0.1#response`. Payload: `{ entry, previousSubject }`.

### Successful swap

```json
{
  "id": "swap-resp-3456-7890-12cd-ef3456789012",
  "type": "https://trusttasks.org/spec/acl/swap-key/0.1#response",
  "threadId": "swap-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T15:00:01Z",
  "payload": {
    "previousSubject": "did:web:alice.example",
    "entry": {
      "subject": "did:peer:2.Ez6LSc…",
      "role": "admin",
      "label": "Alice — primary admin",
      "createdAt": "2026-05-16T10:00:01Z",
      "createdBy": "did:web:org.example"
    }
  }
}
```

## Security & Privacy

**Atomicity.** The maintainer MUST commit the swap as a single transaction. A partial state (entry under the new subject but the old subject not yet removed, or vice versa) leaves either a brief over-privileged window or a brief unauthenticated window — both are policy-violating. Maintainers implementing on top of non-transactional stores MUST use a write-ahead marker or equivalent.

**Compromised-key window.** If `currentSubject`'s key was compromised before the swap, the attacker could sign their own swap pointing to a key they control. The `linkProof` requirement is the only defense; maintainers handling high-value ACLs SHOULD require it unconditionally. Maintainers operating in lower-stakes environments MAY accept proof-of-currentSubject alone (the default policy).

**Token revocation.** The conformance section is firm: tokens bound to `currentSubject` MUST be revoked. The `recommended-but-not-required` framing on access tokens specifically (because their natural expiry is minutes) MAY be relaxed only when the maintainer's audit policy demands a complete cutover trace.

**Concurrent swaps.** A swap and a concurrent administrative `acl/revoke` for the same entry, or two concurrent swaps with different `newSubject` values, MUST serialize. Maintainers MUST ensure one wins and the others fail with `subject_not_found` or `subject_already_in_use`.

**Replay.** The framework's general guidance on document replay applies: maintainers SHOULD reject documents whose `issuedAt` is far in the past or in the future, and SHOULD include the document's id in their idempotency window so a retry of the same document is observed as the same swap.

**Audit.** A successful swap MUST be logged with both `currentSubject` and `newSubject` so a future incident investigation can reconstruct the rotation chain. Administrative swaps MUST additionally log the administrator's VID.

The optional `ext` extension is part of the producer's signed surface; producers MUST NOT place data in `ext` that they would not be comfortable signing. The `linkProof` is similarly signed (transitively) and MUST be sized to fit comfortably within the maintainer's document-size policy.
