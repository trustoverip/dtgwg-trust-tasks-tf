---
slug: vtc/members/personhood/assert
version: "0.2"
title: VTC Members Personhood — Assert
summary: A member asserts personhood by presenting a Verifiable Presentation that satisfies the community's personhood policy, with the outcome evidence of any Verifiable Witness Credential it presents; re-issues their credentials with the personhood flag set.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - members
  - personhood
  - assert
  - outcome-evidence
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The carried Verifiable Presentation remains the personhood evidence, gated by its own proof over the challenge — the framework proof is not that gate and does not replace it. It is required because execution acts with the subject's authority to set a personhood flag and re-issue their credentials, so the request must be attributable to the party that made it. Replay of a captured VP under a different envelope, and repudiation of the assertion afterwards, are the threats addressed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The assertion is made under the member's own authority and becomes the basis of how the community treats them. A replayed assertion re-asserts personhood at a moment the member did not choose and may no longer stand behind.
sideEffects:
  level: mutating
  rationale: "Sets the member's personhood flag and re-issues their credentials; reversible via personhood/revoke."
subjectPath: /did
exposure:
  discloses: secret
  ingests: personal
  actsAsSubject: true
  rationale: "The asserting member is the subject — they present a Verifiable Presentation held as themselves, exercising their own authority over their own personhood state. In: the presentation, whose credentials are about the member, and the outcome evidence of any Verifiable Witness Credential in it — whole Trust Task documents carrying the witnessing session's identifiers. Out: the member's re-issued membership and role credentials, which the member retains."
retention:
  class: durable
  rationale: "The community's decision is a record: the personhood flag, the re-issued credentials, and the audit of what was assessed. Neither the presentation nor the outcome evidence is needed to keep that decision, and the Security & Privacy section says what a community keeps of them."
errorCodes:
  - code: vtc/members/personhood/assert:notFound
    meaning: No member with the supplied did exists.
    retryable: false
  - code: vtc/members/personhood/assert:challengeExpired
    meaning: The presentation's challenge is unknown or expired. Open a new personhood/challenge.
    retryable: false
  - code: vtc/members/personhood/assert:presentationInvalid
    meaning: The Verifiable Presentation failed verification, its holder did not match did, or it did not satisfy the community's active personhood policy — including where the policy rests on a Verifiable Witness Credential that counts for nothing because it has no paired outcome evidence.
    retryable: false
related:
  - vtc/members/personhood/challenge
  - vtc/members/personhood/revoke
  - witness/session
  - witness/session/submit
  - vta/witness/accept
---

## Abstract

The **VTC Members Personhood — Assert** Trust Task asserts that the member `did` is a person. The member submits a W3C Verifiable Presentation whose `holder` is `did`, whose `proof.challenge` is the `challengeId` from a prior [`vtc/members/personhood/challenge`](../../challenge/0.1/spec.md), and which carries at least one credential satisfying the community's active personhood policy. That credential is either an identity credential from a provider the community lists in `acceptedIdvps` — any W3C VC meeting its identity-proofing requirements — or, where the community checked the member's identity in person itself, the `vetted/1` statement (`https://registry.trustoverip.org/dtg/vsc/vetted/1`) it issued under its own DID through [`vtc/endorsements/issue`](../../../../endorsements/issue/0.1/spec.md) to record that check, whose `object.value.community` is this community. On success the community sets the personhood flag and re-issues the member's membership (`vmc`) and role (`roleVac`) credentials.

A community's policy may also count **Verifiable Witness Credentials** (VWCs, `https://registry.trustoverip.org/dtg/vsc/witnessed/1`): a witness's attestation that the member took part in a witnessed relationship exchange. A VWC cites the session that produced it, and counts only with the **outcome evidence** of that session ([SPEC §4.9.4](/SPEC.md#494-evidence-that-a-cited-exchange-completed)), which this version carries in `outcomeEvidence`.

**The presented credential is evidence; personhood is the community's decision.** A `vetted/1` statement establishes only that its issuer carried out the check it describes — the predicate's profile says expressly that it does not establish personhood — and a VWC only that a witness observed an exchange. The community weighs the evidence under its policy and records its decision on the member's VMC, and a verifier relies on that VMC, not on the evidence behind it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## What changed from 0.1

`outcomeEvidence` is new, and OPTIONAL. Version 0.1 had nowhere to carry the outcome evidence a VWC's citation needs, so a VWC could not be counted under it at all; implementations carried the evidence, before this version, in a vendor extension (`ext["org.openvtc.witness"].outcomeEvidence`), which another community implementation would not read. Everything else is as in 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the member):

1. Supplies `did` and the `presentation`. The presentation's holder **MUST** equal `did` and its `proof.challenge` **MUST** be the paired `challengeId`.
2. For each VWC in the presentation it wants counted, carries one `outcomeEvidence` entry: the `witness/session` document the VWC's `taskContext` names and the `witness/session/submit` `#response` that delivered the VWC, each exactly as the member's agent stored it ([`vta/witness/accept`](../../../../../vta/witness/accept/0.2/spec.md) returns both). It **MUST NOT** re-serialise either: the task digest and the witness's proof are taken over their members.
3. Carries no entry for a session whose VWC is not in the presentation, and presents only the VWCs the member selected for this assertion (Security & Privacy).

A conforming **consumer** (the community):

1. Resolves the member (`notFound` if absent), and verifies the presentation: an unknown or expired challenge is `challengeExpired`, and a failed verification or a holder that is not `did` is `presentationInvalid`.
2. Counts a VWC in the presentation toward its policy **only** when all of these hold, and otherwise treats it as counting for nothing:
   1. its issuer is a witness the community's own policy trusts — how a community names its trusted witnesses is its configuration, and an empty or unreadable list trusts none;
   2. an `outcomeEvidence` entry pairs with it by the four checks of [SPEC §4.9.4](/SPEC.md#494-evidence-that-a-cited-exchange-completed), taking the VWC's `taskContext` and `taskDigestMultibase` as the citation, and the terminal document's type as the response the **initiating document's own specification** declares — `witness/session/submit/0.1#response` for a `witness/session/0.1` session, `witness/session/submit/0.2#response` for `witness/session/0.2`;
   3. that pairing meets the witnessing checks of [`witness/session/submit`](../../../../../witness/session/submit/0.2/spec.md) (*This response is the outcome evidence*): the session document's `recipient` and the response's `issuer` are the VWC's issuer, the session was opened by the VWC's subject, the response's `vwcDigestMultibase` is the digest of the VWC it delivers, and that VWC is the one presented.
3. Ignores an `outcomeEvidence` entry that pairs with no VWC in the presentation. It is not evidence of anything the member asserted.
4. When the policy is not satisfied by what counts, returns `presentationInvalid` and changes nothing. Otherwise sets personhood, re-issues the credentials, and returns `{ did, personhood: true, vmc, roleVac }`. Audits the assertion.

A community **MUST NOT** let its policy count a VWC the host checks above refused: the checks decide what is evidence, and the policy weighs only evidence.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorization evidence this task presupposes is the **presentation**, whose holder **MUST** equal the `did` being asserted and whose `proof.challenge` **MUST** be the paired `challengeId`.

Both halves are load-bearing and neither substitutes for the other. Holder equality establishes that the assertion is about the party making it; the challenge binding establishes that this presentation was made for this exchange, and is what stops one captured and replayed into another. A consumer that checks only the first accepts replays; one that checks only the second accepts an assertion made on someone else's behalf.

`outcomeEvidence` authorizes nothing. It is evidence about a credential in the presentation, attributed by the witness's proof on the documents themselves, and it carries weight only through the presentation it accompanies.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Definitions

- **`did`** — the member asserting personhood; the presentation's holder.
- **`presentation`** — the W3C Verifiable Presentation carrying the personhood evidence.
- **`outcomeEvidence`** — OPTIONAL. At most 16 entries, each the witness family's shared `OutcomeEvidence` ([`outcome-evidence.schema.json`](../../../../../witness/_shared/0.1/outcome-evidence.schema.json)): `session`, the `witness/session` document a VWC cites, and `outcome`, the `witness/session/submit` `#response` that delivered it.

## Request

The member sends the request to the community. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Personhood resting on a witnessed relationship

The presentation and the evidence documents are abbreviated.

```json
{
  "id": "urn:uuid:7c1d2e3f-0000-4000-8000-000000000301",
  "type": "https://trusttasks.org/spec/vtc/members/personhood/assert/0.2",
  "issuer": "did:webvh:QmAlice:alice.example.com",
  "recipient": "did:webvh:QmCommunity:community.example.org",
  "issuedAt": "2026-10-16T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmAlice:alice.example.com",
    "presentation": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "type": ["VerifiablePresentation"],
      "holder": "did:webvh:QmAlice:alice.example.com",
      "verifiableCredential": [
        { "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"] }
      ],
      "proof": {
        "type": "DataIntegrityProof",
        "proofPurpose": "authentication",
        "challenge": "urn:uuid:7c1d2e3f-0000-4000-8000-0000000002aa"
      }
    },
    "outcomeEvidence": [
      {
        "session": {
          "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201",
          "type": "https://trusttasks.org/spec/witness/session/0.2#request",
          "issuer": "did:peer:2.Ez6LSAliceForBobRelationship.Vz6MkAliceForBobRelationship",
          "recipient": "did:webvh:QmWitness:witness.example.org:a"
        },
        "outcome": {
          "id": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000204",
          "type": "https://trusttasks.org/spec/witness/session/submit/0.2#response",
          "issuer": "did:webvh:QmWitness:witness.example.org:a",
          "threadId": "urn:uuid:3a2b1c0d-0000-4000-8000-000000000201"
        }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-16T09:00:00Z",
    "verificationMethod": "did:webvh:QmAlice:alice.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSbwUM6dcvcRnzsBGVX9AsF3EyLoJvxfsMEKmYYCsCS4rBFPKVJTF"
  }
}
```

## Response

The community answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json), unchanged from 0.1. Failures use `trust-task-error`, not a `#response` document.

## Security & Privacy

### Data carried

**The presentation is the gate.** Personhood is proven by the VP over the single-use challenge, not by the framework proof. The holder-match and challenge binding together stop one member asserting personhood on another's behalf or replaying an old presentation.

`outcomeEvidence` carries whole Trust Task documents. Presenting outcome evidence discloses both documents of each witnessing session in full ([SPEC §13](/SPEC.md#13-privacy-considerations) item 6): the session's `id` and thread, the enclosing relationship exchange's thread, both relationship DIDs of the witnessed exchange — the counterparty's included — and the witness's DID, besides the VWC itself.

### Correlation

A VWC and its evidence join the member's identity to a pairwise relationship DID, and to the counterparty's, at the community. Pairwise relationship DIDs exist so that nothing outside a relationship can link it to the member's other activity; every VWC a member presents here, with its evidence, gives the community that link for one relationship. Two members presenting VWCs from the two sessions of one witnessed exchange give it the link between them, since both sessions name the same `parties` pair.

**A member presents only the VWCs it selects for this assertion.** A producer **MUST NOT** add VWCs, or their evidence, that the member has not chosen to present for this assertion, and **SHOULD** present the fewest that satisfy the community's policy. A client acting for the member presents none by default, and presents every one it holds only where the member has authorised exactly that for this one assertion. Presenting a VWC is a disclosure of a relationship, and the choice of which relationships to disclose is the member's.

### Retention

The community keeps its decision — the personhood flag, the re-issued credentials and the audit of the assertion — and **SHOULD NOT** keep the presentation or the outcome evidence once it has decided: neither is needed to keep the decision, and both carry the correlators above. An audit record **SHOULD** name what was assessed (the VWC's issuer and digest, for instance) rather than copy the documents.

### Consent/purpose

The member discloses its personhood evidence, and the relationships behind any VWC in it, for one purpose: so the community can decide its personhood under its published policy. That disclosure does not extend to building a relationship graph of the community's members, or to answering third-party questions about who relates to whom. This section is descriptive: per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 it does not declare whether consent, approval or a step-up is required.
