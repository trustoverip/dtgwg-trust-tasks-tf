---
slug: vta/witness/request
version: "0.1"
title: "VTA Witness — Request"
summary: "A client asks its VTA to run a witnessed session for one of its principal's relationships — open the session with a named witness as the relationship DID, present the credentials it holds, verify the Verifiable Witness Credential returned, and store it with its evidence."
status: retired
supersededBy: vta/witness/open/0.1
targetFrameworkVersion: "0.6.0"
category: credentials
keywords:
  - witness
  - vwc
  - relationship
  - presentation
  - vta
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: The principal's VTA
    requirement: REQUIRED
    member: recipient
  - role: Witness
    requirement: REQUIRED
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA act outward in its principal's name: sign presentations as the
    principal's relationship DID — and, when asked, as its member DID — and disclose the
    credentials they carry to a third party the caller names. A request whose integrity
    depended on the transport would let a compromised channel choose the witness, and so the
    party the principal's credentials are shown to. The response carries the witness's own
    signed documents, which are evidence on their own proofs; the VTA's proof over them is
    recommended, not load-bearing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed
    request opens a second session and mints a second VWC; duplicate-execution protection
    can only be applied inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    The VTA opens a session at the witness, which mints a Verifiable Witness Credential on a
    valid submission, and stores that credential in the credential vault. A second execution
    is a second session and a second credential, not a no-op. Nothing is destroyed.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The VTA exercises the principal's relationship DID — and, under the member link, its member
    DID — to present the principal's own credentials to the witness. In: DIDs and vault handles;
    no credential travels in the request. Out to the caller: the vault handle of the VWC and the
    two evidence documents carrying it, which the principal's delegate is the right party to
    hold. The disclosure that matters happens to the witness, not to the caller, and is the
    subject of Security & Privacy.
retention:
  class: durable
  rationale: >-
    The VTA keeps the VWC with its evidence — the session document as sent and the submit
    response as received — for the useful life of the VWC, because a verifier cannot pair the
    credential with its session without both. It keeps neither presentation it built: they
    have no evidentiary role once the witness has answered.
errorCodes:
  - code: vta/witness/request:notFound
    meaning: "A named credential is not in the credential vault in a context the caller may act in, or is not active there."
    retryable: false
  - code: vta/witness/request:credentialMismatch
    meaning: "A named credential does not fit its role: the relationship credential is not a RelationshipCredential issued by relationshipDid naming counterpartyDid, or the membership credential is not a MembershipCredential naming the expected DID, or either is outside its validity period."
    retryable: false
  - code: vta/witness/request:noMatchingProtocol
    meaning: "The witness's DID document and relationshipDid's advertise no transport in common that the VTA speaks."
    retryable: false
  - code: vta/witness/request:witnessUnreachable
    meaning: "The witness's DID could not be resolved, or the witness did not answer within the VTA's bound."
    retryable: true
  - code: vta/witness/request:witnessRefused
    meaning: "The witness declined to open the session (witness/session:refused). Its code and message are carried in `details`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [code]
      properties:
        code:
          type: string
          minLength: 1
          maxLength: 256
        message:
          type: string
          maxLength: 1024
  - code: vta/witness/request:challengeMismatch
    meaning: "The witness refused the submission as not bound to the session it opened (witness/session/submit:challengeMismatch)."
    retryable: false
  - code: vta/witness/request:presentationInvalid
    meaning: "The witness refused the presentation (witness/session/submit:presentationInvalid) — commonly its policy asks for membership evidence that was not presented, or does not accept the evidence that was. Its code and message are carried in `details`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [code]
      properties:
        code:
          type: string
          minLength: 1
          maxLength: 256
        message:
          type: string
          maxLength: 1024
  - code: vta/witness/request:vwcInvalid
    meaning: "The witness answered, but its answer does not verify: an unproofed or wrongly signed response, a VWC not issued by the witness, a VWC citing another session, a VWC not naming the relationship credential presented, or a digest that does not match. Nothing is stored."
    retryable: false
related:
  - witness/session
  - witness/session/submit
  - vault/credentials/receive
  - vault/credentials/get
  - vrc/relationships/issue
---

## Abstract

**Witnessing** is a third party observing a relationship exchange and attesting, in a Verifiable Witness Credential (VWC), that it took place ([`witness/session`](../../../../witness/session/0.1/spec.md), [`witness/session/submit`](../../../../witness/session/submit/0.1/spec.md)). The participating party opens a session with the witness under its relationship DID, presents its relationship credential bound to the session's challenge, and receives the VWC.

A principal whose keys and credentials are held by a VTA cannot do that from a client that holds neither. This task lets the client ask the VTA to be the participating party for it: open the session as the principal's relationship DID, build and sign the presentation from the credentials in its credential vault, submit it, verify what the witness returns, and store the VWC **with its evidence** so it can be presented later. The client names the witness, the relationship and the credentials; it never receives a key or a presentation.

The VTA is a **party** here, never a witness. A witness has value only as a third party independent of the exchange it attests.

## Status of this Document

This specification is **retired** per [SPEC §5.3](/SPEC.md#53-maturity-levels), superseded by the [`vta/witness/open`](../../open/0.1/spec.md), [`vta/witness/present`](../../present/0.1/spec.md) and [`vta/witness/accept`](../../accept/0.1/spec.md) tasks, whose shared rules are in the [witnessed-party conventions](../../../_shared/0.5/CONVENTIONS.md). No implementation shipped it.

It cannot be implemented as written. It has the VTA send both documents of the session to the witness as the relationship DID, but a witness checks that the transport-proven sender of each document is its `issuer` and replies to that sender, and the relationship DID's transport belongs to whoever holds the relationship — normally the client, not the VTA. The successor tasks keep the custody, the checks and the evidence this task specified, and have the client carry the documents. New implementations MUST use them.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. The VTA, in its dealings with the witness, is a conforming **participating party** of [`witness/session/0.1`](../../../../witness/session/0.1/spec.md) and [`witness/session/submit/0.1`](../../../../witness/session/submit/0.1/spec.md); every requirement those specifications place on that party is the VTA's.

A conforming **producer** (the client) **MUST**:

1. Name in `witness` a witness the principal has chosen to show these credentials to. The VTA cannot make that choice for it (see [Consent/purpose](#consentpurpose)).
2. Name `membership.memberDid` only when it intends the member DID to be disclosed to the witness.
3. Retry a request whose answer was lost by resending the **same document**, or under an idempotency mechanism its VTA offers — never as a new request, which is a second session and a second VWC (see [Retries and duplicates](#retries-and-duplicates)).

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
2. **Before contacting the witness**, establish everything it can locally, so that a request that cannot succeed discloses nothing to anyone:
   1. that it holds a signing key of `relationshipDid` — and, when `membership.memberDid` is present, of `memberDid` — in a context the caller may act in; otherwise `permissionDenied`;
   2. that `relationshipCredentialId` and `membership.credentialId` name active credentials in a context the caller may act in; otherwise `vta/witness/request:notFound`;
   3. that the relationship credential is a `RelationshipCredential` issued by `relationshipDid` whose subject is `counterpartyDid`, and the membership credential a `MembershipCredential` whose subject is `membership.memberDid` (or `relationshipDid` when `memberDid` is absent), each within its validity period; otherwise `vta/witness/request:credentialMismatch`;
   4. that `relationshipDid`, `counterpartyDid` and `witness` are three different DIDs, and that `membership.memberDid`, when present, is none of them; otherwise `malformedRequest`;
   5. that `witness` resolves, and that the witness's DID document and `relationshipDid`'s advertise a transport in common that the VTA speaks, matched on the service `type`; otherwise `vta/witness/request:witnessUnreachable` or `vta/witness/request:noMatchingProtocol`. The VTA **MUST NOT** fall back to a transport the witness does not advertise.
3. **Open the session.** Send a `witness/session/0.1` document to `witness` with `issuer` `relationshipDid`, `parties` `[relationshipDid, counterpartyDid]`, `threadId` equal to its own `id`, and `parentThreadId` equal to `relationshipThreadId`. Where the transport authenticates its sender, the sender **MUST** be `relationshipDid`: a witness checks the in-band `issuer` against it. A `witness/session:refused` answer is `vta/witness/request:witnessRefused`, carrying the witness's code and message in `details`.
4. **Check the session's answer**: a `#response` on that thread whose `issuer` is `witness`, under a proof that verifies for the witness's DID. Anything else is `vta/witness/request:vwcInvalid` — a challenge the VTA cannot attribute to the witness is one it **MUST NOT** sign over.
5. **Build the presentations**, itself, from the credentials it holds (see [Custody](#custody)):
   1. the **relationship presentation**: holder `relationshipDid`, carrying the relationship credential — and the membership credential, when `memberDid` is absent — bound to the session's `challenge` and `domain` under a proof by `relationshipDid` for `authentication`;
   2. when `membership.memberDid` is present, the **member presentation**: holder `memberDid`, carrying the membership credential, bound to the **same** `challenge` and `domain` under a proof by `memberDid` for `authentication`.
6. **Submit.** Send a `witness/session/submit/0.1` document on the session's thread, issued and signed by `relationshipDid`, with `parentThreadId` as in step 3, carrying the relationship presentation in `vp` and the member presentation where the witness accepts one (see [The member presentation's carriage](#the-member-presentations-carriage)). A `challengeMismatch` or `presentationInvalid` answer is `vta/witness/request:challengeMismatch` or `vta/witness/request:presentationInvalid`, carrying the witness's code and message in `details`.
7. **Verify the answer** as a verifier of the evidence would, and refuse anything that fails with `vta/witness/request:vwcInvalid`, storing nothing:
   1. it is a `#response` on the session's thread, issued by `witness`, under a proof that verifies for the witness's DID;
   2. the VWC's `issuer` is `witness` — the session document's `recipient` — and its proof verifies for the witness's `assertionMethod`;
   3. the VWC's `taskContext` is the session document's `id`, and its `taskDigestMultibase` the task digest of the session document as sent ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names));
   4. the VWC's `credentialSubject.id` is `relationshipDid`, and its `credentialSubject.object.digestMultibase` the digest of the relationship credential presented;
   5. `vwcDigestMultibase` is the digest of the VWC as received, proof included.
8. **Store** the VWC in the credential vault, in the context of `relationshipDid`'s key, under `storeAs` or a handle it derives, **together with** the session document as sent and the submit `#response` as received, each byte for byte. Only then answer, with the stored handle and the evidence.

The VTA **MUST NOT** return a key, a presentation, or the session's challenge, in any response, under any outcome.

## Authorization

**The authority is standing over the contexts the task acts in**: the VTA has recorded the caller as entitled to act in the context holding `relationshipDid`'s key, and in every context holding a credential or key the request names — `membership.memberDid`'s key included. Where that entitlement is expressed as a capability, the registered value is **`witnessRequest`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json).

It is not `sign`, and a VTA **MUST NOT** require `sign` for this task, nor satisfy a request for it under a grant of `sign` alone. The task signs only presentations the VTA builds itself, over credentials it holds, bound to a challenge the witness issued — never bytes the caller supplies — so a caller entitled to have the principal witnessed is not thereby a caller entitled to sign as the principal. Conversely, it reads two credentials from the vault and writes one, and `sign` confers neither. The write is narrow: only the VWC its own exchange produced, verified under Conformance step 7.

Verifying the document's `issuer`, transport identity or `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

The act is consequential beyond the VTA. It discloses the principal's credentials, in full, to a third party the caller names, and — under the member link — links the principal's relationship DID to its member DID at that party. A VTA's own policy may place a further gate on it; that gate is the VTA's to choose ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)), and this specification states only that the act is consequential.

Whether to witness the principal at all remains the **witness's** decision, under its own policy, and nothing here obliges it.

### Custody

The VTA speaks to the witness **as the principal's relationship DID**, and so signs as it:

1. It **MUST** sign the documents it sends to the witness, and the relationship presentation, with a key of `relationshipDid` it holds in a context the caller may act in, and **MUST NOT** sign them as itself. A witness checks that the session's `issuer` is in `parties`; a document issued by the VTA naming another party is one it must refuse.
2. It **MUST** sign the member presentation with a key of `memberDid` held on the same terms, and **MUST NOT** use that key for any other document of the exchange.
3. It **MUST NOT** sign or present over bytes the caller supplies. The caller names credentials by vault handle; the VTA reads them, checks them (Conformance step 2), and builds every presentation itself. A caller cannot make this task present a credential the vault does not hold, or sign anything but a presentation bound to the witness's challenge.

## Definitions

- **`witness`** — the witness's public DID.
- **`relationshipDid`** — the principal's relationship DID for this relationship; the participating party.
- **`counterpartyDid`** — the other party's relationship DID.
- **`relationshipThreadId`** — the `threadId` of the relationship exchange the session is nested in.
- **`relationshipCredentialId`** — the vault handle of the relationship credential to present.
- **`membership`** — OPTIONAL membership evidence: `credentialId`, the vault handle of a membership credential, and `memberDid`, the DID it names when that is not `relationshipDid`.
- **`storeAs`** — OPTIONAL vault handle for the VWC.
- **`credentialId`** (response) — the vault handle the VWC is stored under.
- **`witness`** (response) — the witness that issued it.
- **`vwcDigestMultibase`** (response) — the VWC's digest, as the witness returned it and the VTA checked it.
- **`evidence`** (response) — `session`, the `witness/session` document as sent, and `outcome`, the `witness/session/submit` `#response` as received, which carries the VWC.

### Membership evidence, and the member link

A witness may admit to witnessing only members of communities it serves. Its evidence is a membership credential (VMC), and a VMC names the member's own DID — not the pairwise relationship DID the party submits from. There are two ways to present one:

- **The VMC names `relationshipDid`.** Omit `memberDid`. The VMC rides in the relationship presentation beside the relationship credential.
- **The VMC names a member DID** (the common case — a relationship DID is pairwise, a membership is not). Name it in `memberDid`. The VTA builds a second presentation by that DID, bound to the same session, so the witness can see that whoever holds the relationship key also held the member key, in this one session.

The second form discloses `memberDid` to the witness and lets it link the pairwise relationship DID to the member's community identity. That is why it takes an explicit `memberDid`: the VTA could read the DID off the credential, and does not, so that no request discloses a member DID its caller did not name.

### The member presentation's carriage

[`witness/session/submit/0.1`](../../../../witness/session/submit/0.1/spec.md) defines one presentation, `vp`, and no member for a second. Until it does, a witness that accepts a member presentation defines its own carriage, and this specification does not: per [SPEC §4.5.1](/SPEC.md#451-the-ext-extension-member) the semantics of an `ext` namespace belong to its controller. A VTA **MUST NOT** send a member presentation to a witness whose carriage it does not know, and a witness that does not read one refuses the submission for want of membership, answered as `vta/witness/request:presentationInvalid`.

*Non-normative.* The OpenVTC witness reads it from the submit payload's `ext["org.openvtc.witness"].memberPresentation`.

### Retries and duplicates

The task is **not idempotent**: each execution is a session at the witness and a VWC in the vault. A retry layer therefore **MUST** treat it as needing duplicate protection, and it is safe to replay a cached answer, which carries no secret.

- A VTA **MUST** apply duplicate-execution protection to it ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) — the same document again is answered with the first reply — and **MAY** offer an idempotency mechanism that carries one operation across several documents.
- Within the exchange with the witness, a VTA that retries a document whose answer was lost **MUST** resend the same document, not a new one: a new `witness/session` is a second session, and a new submit on a spent session is `challengeMismatch`.
- The task spans two round trips to a remote party. A VTA **MAY** bound how long it waits and answer `vta/witness/request:witnessUnreachable` when the bound passes. A witness that answers after that may have minted a VWC the VTA never stored; it is inert — nobody holds it — and stays at the witness as a record only.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Witnessed as a member, through the member link

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000101",
  "type": "https://trusttasks.org/spec/vta/witness/request/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-10-15T09:00:00Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000001ff",
  "payload": {
    "witness": "did:webvh:QmWitness:witness.example.org:a",
    "relationshipDid": "did:key:z6MkrAliceForBobRelationshipKey333333333333333",
    "counterpartyDid": "did:key:z6MkrBobForAliceRelationshipKey444444444444444",
    "relationshipThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
    "relationshipCredentialId": "vrc-alice-to-bob",
    "membership": {
      "credentialId": "vmc-community-example",
      "memberDid": "did:webvh:QmAlice:alice.example.com"
    },
    "storeAs": "vwc-alice-to-bob"
  }
}
```

## Response

Once the VWC is verified and stored, the VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the vault handle, the witness, the VWC's digest, and the two evidence documents — the session document as sent, and the witness's submit `#response`, which carries the VWC. Failures use `trust-task-error`, not a `#response` document.

### Stored, with its evidence

```json
{
  "id": "urn:uuid:5b0f3c1e-0000-4000-8000-000000000102",
  "type": "https://trusttasks.org/spec/vta/witness/request/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-10-15T09:00:03Z",
  "threadId": "urn:uuid:5b0f3c1e-0000-4000-8000-0000000001ff",
  "payload": {
    "credentialId": "vwc-alice-to-bob",
    "witness": "did:webvh:QmWitness:witness.example.org:a",
    "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "evidence": {
      "session": {
        "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
        "type": "https://trusttasks.org/spec/witness/session/0.1#request",
        "issuer": "did:key:z6MkrAliceForBobRelationshipKey333333333333333",
        "recipient": "did:webvh:QmWitness:witness.example.org:a",
        "issuedAt": "2026-10-15T09:00:01Z",
        "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
        "parentThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
        "payload": {
          "parties": [
            "did:key:z6MkrAliceForBobRelationshipKey333333333333333",
            "did:key:z6MkrBobForAliceRelationshipKey444444444444444"
          ]
        }
      },
      "outcome": {
        "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000204",
        "type": "https://trusttasks.org/spec/witness/session/submit/0.1#response",
        "issuer": "did:webvh:QmWitness:witness.example.org:a",
        "recipient": "did:key:z6MkrAliceForBobRelationshipKey333333333333333",
        "issuedAt": "2026-10-15T09:00:02Z",
        "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
        "parentThreadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000a01",
        "payload": {
          "vwc": {
            "@context": [
              "https://www.w3.org/ns/credentials/v2",
              "https://registry.trustoverip.org/dtg/context/v1"
            ],
            "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"],
            "issuer": "did:webvh:QmWitness:witness.example.org:a",
            "issuerScope": "public",
            "validFrom": "2026-10-15T09:00:02Z",
            "credentialSubject": {
              "id": "did:key:z6MkrAliceForBobRelationshipKey333333333333333",
              "predicate": "https://registry.trustoverip.org/dtg/vsc/witnessed/1",
              "object": {
                "digestMultibase": "zQmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk"
              }
            },
            "taskContext": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
            "taskDigestMultibase": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
            "proof": {
              "type": "DataIntegrityProof",
              "cryptosuite": "eddsa-jcs-2022",
              "created": "2026-10-15T09:00:02Z",
              "verificationMethod": "did:webvh:QmWitness:witness.example.org:a#key-0",
              "proofPurpose": "assertionMethod",
              "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSbwUM6dcvcRnzsBGVX9AsF3EyLoJvxfsMEKmYYCsCS4rBFPKVJTF"
            }
          },
          "vwcDigestMultibase": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "created": "2026-10-15T09:00:02Z",
          "verificationMethod": "did:webvh:QmWitness:witness.example.org:a#key-0",
          "proofPurpose": "authentication",
          "proofValue": "z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEcqdoDwLWxaqA3Q1geV6"
        }
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries DIDs and vault handles. No credential travels in it, and no presentation: the VTA reads the credentials from its own vault and builds the presentations itself. A producer **SHOULD** use handles that are handles — `storeAs` included — and not descriptions of the relationship or its purpose, and **MUST NOT** put relationship context in `ext`: a VTA that is told what a relationship is for writes it into a record, and the witness specifications ask that a witness not be told at all.

The response carries the VWC's handle and digest and the two evidence documents. The session document is two relationship DIDs and the enclosing thread; the submit `#response` carries the VWC, a statement about the principal's relationship DID meant to be shown to verifiers. Neither presentation is returned, nor the session challenge.

**What the witness sees is the larger disclosure, and it happens on the principal's behalf.** [`witness/session/submit`](../../../../witness/session/submit/0.1/spec.md) types `vp` as an opaque object: every claim in the presented credentials reaches the witness in full and in the clear, and the witness must read all of it to verify it. Through this task the witness sees the relationship credential and, when `membership` is named, the membership credential — its issuer, its claims, its identifiers and any status entry. The VTA **SHOULD** present no credential beyond those the request names, and **SHOULD** present the narrowest derivation its credential format supports where the witness accepts one.

### Correlation

At the witness, the session pairs `relationshipDid` with `counterpartyDid`, by design: both parties name the same pair, which is how the witness knows its two sessions observe one exchange. The presentation adds what the credentials carry, and the membership evidence adds the community. **The member link discloses more**: the witness learns `memberDid`, the member's durable community identity, beside the pairwise relationship DID — *which member*, not only *which community*. A member DID seen by one witness across several relationships links them all. The VTA cannot prevent that; it can only refuse to disclose a member DID the caller did not name, which Conformance requires.

The witness is public by design (`identifierScope: public`): the session's recipient and the VWC's issuer, recognisable by both parties and by every verifier, so that a verifier can tell one witness attested both halves of an exchange. The cost is the witness's own privacy, and for the parties a single, nameable collection of the exchanges that witness observed. Choosing the witness is therefore the principal's substantive privacy decision, and this task leaves it to the caller rather than to the VTA.

To an observer of the VTA's outbound traffic, the exchange is the principal's relationship DID speaking to the witness, twice, close together. The member DID does not appear on the wire under the member link: its presentation travels inside the submission, which only the witness opens.

### Retention

The VTA keeps the VWC **with its evidence** — the session document as sent and the submit `#response` as received, byte for byte — for the useful life of the VWC. A holder presenting the VWC must ship both: a verifier cannot recompute the task digest without the session document, and cannot establish that the session completed without the witness's signed response ([`witness/session/submit`](../../../../witness/session/submit/0.1/spec.md), *This response is the outcome evidence*). Deleting either leaves a credential that still verifies on the witness's proof and can no longer be shown to belong to a session that happened.

The VTA keeps **neither presentation**. Once the witness has answered they have discharged their function and have no evidentiary role: the VWC is bound to the session document and to the relationship credential, never to the presentation. The session challenge is likewise spent. Beyond the stored credential and its evidence, the VTA keeps its audit record of the operation — the caller, the context, the witness and the time — and a duplicate-execution record for the acceptance window.

### Consent/purpose

The principal's credentials are disclosed to the witness for one purpose: so that a witness the principal chose can attest that the principal's relationship DID took part in this exchange. The member DID, when named, is disclosed so that the witness can judge the principal eligible under its policy, and for nothing else. The VWC is stored so that the principal can later show a verifier the exchange was witnessed.

Choosing a witness is choosing who sees these credentials, and only the principal can make that choice. The VTA acts on the caller's choice; it does not select a witness, substitute one, or present to any party other than the one named. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
