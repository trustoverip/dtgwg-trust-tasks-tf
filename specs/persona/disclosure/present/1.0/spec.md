---
slug: persona/disclosure/present
version: "1.0"
title: Persona Disclosure — Present
summary: Produce the signed disclosure a preview described, consuming the preview so that a disclosure can never occur without the summary having been produced first.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, disclosure, presentation, audit]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Application
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: This is the operation that releases personal data to a third party. The audit record must name the key that asked for it, because a disclosure the holder later disputes is answerable only if the request is attributable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A disclosure is bound to a verifier challenge for freshness, and an agent that cannot place the request in time cannot enforce the preview's expiry.
sideEffects:
  level: mutating
  rationale: "Releases personal data to a third party, consumes the preview, and appends a permanent disclosure record. Not recoverable — a disclosed value cannot be un-disclosed."
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: true
  rationale: The agent signs as the holder's persona and emits an artifact containing the holder's personal data for release to a named verifier. It exercises the subject's own authority, which is why the preview it consumes exists.
errorCodes:
  - code: persona/disclosure/present:previewNotFound
    meaning: The previewId is unknown, already consumed, or expired. A producer previews again rather than retrying; the second preview is a second decision, which is the intent.
    retryable: false
  - code: persona/disclosure/present:staleClaim
    meaning: A claim in the preview could not be re-derived at signing time. The disclosure is refused whole rather than issued short, because a verifier receiving fewer claims than were approved cannot tell that from a holder who approved fewer.
    retryable: false
---

## Abstract

**Persona Disclosure — Present** produces the signed artifact.

It **consumes** the `previewId`, and that is the whole mechanism: the two calls
cannot be collapsed because the second requires a token only the first can mint.
There is no code path to a disclosure that did not first produce the summary a
human can be shown, and a maintainer cannot accidentally provide one.

A preview is **single-use**. A producer that wants to disclose twice previews
twice, which is correct rather than inconvenient — the second disclosure is a
second decision, and a token that could be replayed would let it ride the first
one.

By default the artifact is **ephemeral and carries no revocation status**.
Revoking something already read is meaningless, so status machinery would be
maintenance with no benefit. A holder who wants a counterparty to be able to
re-verify — and themselves to be able to revoke — asks for a durable credential
with `mint`, and takes on an artifact they must now maintain and someone else now
holds.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/disclosure/present/1.0`, populate
`contextId` and `previewId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Confine the caller to its own context.
2. Consume the preview, and **MUST NOT** honour a `previewId` twice.
3. Refuse an expired preview rather than re-deriving the disclosure from current state — the holder was shown one thing and re-deriving could disclose another.
4. Disclose exactly the claims the preview reported, at exactly the rungs it reported, through exactly the renderer it named.
5. Refuse the whole disclosure with `persona/disclosure/present:staleClaim` if any claim has ceased to be derivable since the preview, rather than issuing a shorter one. A verifier receiving fewer claims than were approved cannot distinguish that from a holder who approved fewer.
6. Bind the verifier's `challenge` into the artifact when supplied.
7. Append a disclosure record naming the verifier, the persona, the claim types, the purpose and the time — **before** returning the artifact, so a crash cannot release data that was never recorded.

## Authorization

**Context-scoped**, confined to the caller's own context, and — uniquely in this
family — the agent **acts as the subject**: it signs with the persona's key.

That is why the preview exists. Every other task either reads or arranges; this
one exercises the holder's own authority toward a third party.

## Request

`previewId` identifies the approved disclosure. `challenge` binds it to the
verifier's request so it cannot be replayed elsewhere. `mint` opts into a durable
credential.

## Response

The artifact, the subject it was issued under, and the identifier of the record
this disclosure wrote.

## Security & Privacy

### Data carried

The response carries **the holder's personal data in releasable form**, signed.
It is the most sensitive document in the family, and the only one whose contents
are intended to leave the holder's control.

A producer **MUST** relay the artifact to the verifier it was minted for and
**MUST NOT** retain it beyond that relay. The artifact is bound to a challenge,
so a retained copy is not usable elsewhere — but it remains a plaintext copy of
the holder's data outside the store that protected it.

### Correlation

The subject is **pairwise by default**, so two verifiers holding two artifacts
cannot join them by subject. What can still join them is a credential presented
at a linkable rung: an issuer signature is identical wherever it goes. That is
reported at preview and is the holder's decision to make, but a maintainer
**SHOULD** prefer an unlinkable rung whenever the credential supports one.

A minted durable credential is a lasting correlation handle by construction —
it is designed to be re-verified — which is the trade the holder accepts when
asking for one.

### Retention

The **disclosure record is permanent and append-only**, and is written before the
artifact is returned. That ordering is deliberate: a crash between signing and
recording would release data the holder could never afterwards discover they had
released.

The artifact itself is retained by the verifier under whatever terms the
relationship carries, and by nobody else. A maintainer **MUST NOT** retain a copy
of the artifact beyond what the disclosure record requires — the record is the
evidence, and a second copy of the payload is a second thing to lose.

### Consent/purpose

The purpose is release: the holder gives a named verifier specific claims for a
stated reason. This task exists as a separate call from the preview precisely so
that the release is distinguishable, in the record and in the code, from the
question of what a release would cost.

This specification does not require that a human approve — that is the
consumer's policy — but the token-consuming structure means the summary always
existed, and the disclosure record means the release is always answerable.
