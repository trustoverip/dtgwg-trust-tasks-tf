---
slug: vtc/join-requests/manifest
version: "0.2"
title: VTC Join-Requests — Manifest
summary: Discover a community's join criteria before applying — including, per criterion, the identity-vetting evidence it requires and a digest naming that exact version of the requirements.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - discovery
  - vetting
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: Join criteria are pre-membership discovery information; a proof is not required to read them, though it is recommended so a community can rate-limit or attribute discovery. What an applicant later relies on is the requirementsDigest, which is recomputable from the criterion itself and so needs no signature to be checked.
sideEffects:
  level: none
  rationale: "Reads the community's published join criteria; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response describes what the community asks of applicants — presentation-definitions, vetting counts and methods, a governance link. None of it is data about a member, applicant, or natural person. The request carries nothing."
retention:
  class: transient
  rationale: The request has no members at all, so there is nothing for a community to keep from it beyond whatever attribution the optional proof carries. This is the one task in the family that leaves no trace of an applicant, which is what makes it usable before deciding whether to apply.
errorCodes: []
related:
  - vtc/join-requests/submit
  - vtc/endorsement-types/register
---

## Abstract

The **VTC Join-Requests — Manifest** Trust Task returns a community's join criteria so a prospective applicant knows what to present before presenting anything. Each criterion names a presentation-definition the applicant must satisfy. A criterion MAY also carry a **`vetting`** requirements object — how many identity-vetting statements the community needs, by which methods, from whom — and then MUST carry a **`requirementsDigest`** naming that exact version of the criterion.

The applicant then gathers what is required and submits via [`vtc/join-requests/submit`](../../submit/0.2/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### Changes from 0.1

`0.1` could say *which credentials* an applicant must present, because that is what a presentation-definition expresses. It could not say what a community that admits people on **peer identity vetting** needs: statements from *distinct* eligible vetters, *at least one* of them in person, *none* from family. A presentation-definition can ask for "credentials of type `EndorsementCredential`". It cannot count issuers, cap relationships, or require that the issuer holds a role. `0.2` adds those as an optional `vetting` object beside the presentation-definition, not inside it.

Because vetting takes days or weeks, an applicant starts under one version of the requirements and may finish under another. `0.2` therefore adds `requirementsDigest`, which names one version of a criterion so the applicant, the vetters and the community can all refer to the same one.

The change is additive. A `0.1` response is a valid `0.2` response once `communityDid` is a DID, and a criterion without `vetting` is unchanged.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **applicant** (`issuer`) sends the request with no parameters.

A conforming **community** (`recipient`):

1. Returns `communityDid` and `criteria`. A community **MAY** tailor the criteria to the caller, but the default is its public join policy.
2. For every criterion carrying `vetting`, **MUST** carry `requirementsDigest`, computed as defined below over the criterion exactly as returned.
3. **MUST NOT** publish a `vetting` object it cannot evaluate as written: `minStatements` at least 1, `acceptedMethods` non-empty, every method in `minByMethod` also in `acceptedMethods`, and every duration in the form the schema permits.
4. **MUST** evaluate a submission that cites a `requirementsDigest` under the criterion version that digest names, while that version is within its `requirementsGrace`, and **MUST** record which version governed the decision. Outside the grace window, or where no grace is declared, the current version governs.
5. **MUST NOT** apply to a vetting statement any constraint the `vetting` object does not state. Absent members mean no constraint of that kind; this specification defines no default count, method floor, age limit, or documentation, and a community that relies on one publishes it.

A conforming **applicant**:

1. **MUST** ignore members of `vetting` it does not recognise, and **MUST** treat a `vetting` object that fails item 3 above as unsatisfiable rather than guess at its meaning. A client's reading of the requirements is advisory in any case: the community's decision is authoritative, and some of what it evaluates — current vetter eligibility, for one — is visible only to the community.
2. On starting an application under a criterion carrying `vetting`, **SHOULD** record the `requirementsDigest` at that moment, cite it in every request it makes to a vetter and at submission, and recompute it from the criterion before relying on it.

### Computing `requirementsDigest`

```
requirementsDigest = multibase( multihash( SHA-256( JCS( criterion ∖ requirementsDigest ) ) ) )
```

`criterion ∖ requirementsDigest` is the criterion object with its `requirementsDigest` member removed and nothing else changed — members of `vetting` a reader does not recognise included. `JCS` is [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) serialized as UTF-8. The value is a `DigestMultibase`. SHA-256 and base58btc (`z`) are **RECOMMENDED**, and a consumer compares decoded multihash bytes, not encoded strings. The digest covers `description` and `presentationDefinition` as well as `vetting`, because a change to any of them changes what the applicant was told to do.

## Authorization

This task is not consequential ([SPEC §2](/SPEC.md#2-terminology)): it changes nothing and discloses only the community's own published policy. It presupposes no authorization evidence beyond the community's own policy on who may read its criteria. A community that requires a `proof` does so to rate-limit or attribute discovery, not to authorize it; see Security & Privacy.

## Definitions

**Criterion** — one way of satisfying the join policy: an `id`, an optional human `description`, a `presentationDefinition`, and optionally `vetting` and `requirementsDigest`.

**Vetting requirements object (`vetting`)** — what identity-vetting evidence the criterion needs beyond what the presentation-definition can express. Every number in it is community policy. The object is open: members defined by a later version of its shape are ignored by readers that do not know them.

| Member | Meaning |
|---|---|
| `version` | Version of this object's shape — `0.1` for the members below |
| `statementType` | The endorsement type URI a counted statement carries, registered via [`vtc/endorsement-types/register`](../../../endorsement-types/register/0.1/spec.md) |
| `minStatements` | Statements needed, counting each vetter once however many DIDs they hold |
| `minByMethod` | Per-method floors within `minStatements`, keyed by method |
| `acceptedMethods` | Methods whose statements count at all: `inPerson`, `video`, `priorAcquaintance` |
| `acceptedDocumentClasses` | Optional, and absent by default. Absent: each vetter decides what documentation they accept, including none. Present: a statement that relied on none of the listed documentation does not count |
| `requiredClaims` | Claim types the Vetting Card must carry and a counted statement must list as verified; the identity commitment is computed over these |
| `optionalClaims` | Claim types an applicant may add; never affect counting |
| `maxStatementAge` | A statement older than this at decision time does not count |
| `eligibleVetters.role` | The community role a statement's issuer must hold for it to count |
| `independence` | Caps per declared vetter–applicant relationship (`none`, `communityColleague`, `sameEmployer`, `family`, `otherPersonal`); whether all statements must carry the same identity commitment |
| `invitation` | Whether an invitation credential must (`required`) or may (`optional`) accompany the statements, or plays no part (`none`) |
| `decisionSla` | How long after submission the community undertakes to decide |
| `requirementsGrace` | How long an application started under an earlier digest is evaluated under that version |
| `governanceFrameworkUrl` | Where the vetting governance, including the attestation text vetters sign, is published |

Durations are ISO 8601 in weeks, days, hours, minutes and seconds (`P120D`, `P2W`, `PT15M`). Years and months are not accepted: their length depends on the calendar, and an age limit that means different things on different days is not a limit.

**`requirementsDigest`** — the digest defined under Conformance. It names a version of a criterion, not a community policy as a whole.

## Request

The applicant sends the request to the community; the payload is empty. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Reading the criteria

```json
{
  "id": "urn:uuid:5b0f3c1e-7d2a-4c8e-9f41-2a6b8d0e1c01",
  "type": "https://trusttasks.org/spec/vtc/join-requests/manifest/0.2",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-12T08:00:00Z",
  "payload": {}
}
```

## Response

The community returns its criteria, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). This task defines no extended error codes; failures use `trust-task-error` with the framework's standard codes.

### A vetted criterion beside an invitation criterion

The first criterion needs two vetting statements, at least one in person, none from family. The numbers are this community's choice. The second criterion is a plain invitation path with no vetting, and so carries no digest. The `requirementsDigest` shown is the real value for the first criterion as printed.

```json
{
  "id": "urn:uuid:5b0f3c1e-7d2a-4c8e-9f41-2a6b8d0e1c02",
  "type": "https://trusttasks.org/spec/vtc/join-requests/manifest/0.2#response",
  "threadId": "urn:uuid:5b0f3c1e-7d2a-4c8e-9f41-2a6b8d0e1c01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-12T08:00:01Z",
  "payload": {
    "communityDid": "did:webvh:QmVtcScid:kernel-vtc.example",
    "criteria": [
      {
        "id": "kernel-developer",
        "description": "Two kernel vetters must confirm who you are. At least one must meet you in person.",
        "presentationDefinition": {
          "credentials": [
            {
              "id": "vetting",
              "format": "ldp_vc",
              "multiple": true,
              "meta": { "type_values": [["EndorsementCredential"]] }
            }
          ]
        },
        "vetting": {
          "version": "0.1",
          "statementType": "https://firstperson.network/endorsements/identity-vetting/0.1",
          "minStatements": 2,
          "minByMethod": { "inPerson": 1 },
          "acceptedMethods": ["inPerson", "video", "priorAcquaintance"],
          "requiredClaims": ["name.legal"],
          "optionalClaims": ["account.handle", "url.homepage"],
          "maxStatementAge": "P120D",
          "eligibleVetters": { "role": "vetter" },
          "independence": {
            "maxByDeclaredRelationship": { "family": 0, "sameEmployer": 1 },
            "requireConsistentIdentityCommitment": true
          },
          "invitation": "optional",
          "decisionSla": "P14D",
          "requirementsGrace": "P30D",
          "governanceFrameworkUrl": "https://kernel-vtc.example/governance#vetting"
        },
        "requirementsDigest": "zQmYZQN9M169SXXg1sZdpNCDAjoVajkrPLaFhJ6A4mecQfC"
      },
      {
        "id": "invited",
        "description": "Present an invitation issued by this community.",
        "presentationDefinition": {
          "credentials": [
            {
              "id": "invitation",
              "format": "ldp_vc",
              "meta": { "type_values": [["InvitationCredential"]] }
            }
          ]
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request payload has no members. That remains the substance of this task:
`manifest` is the only document in the join family a prospective applicant can send
without disclosing anything about themselves, and `0.2` extends what it lets them
learn before deciding. Someone can now find out that a community requires two
people to check their identity, one of them face to face, and walk away having
handed over nothing. Discovering the same thing by applying would have cost them a
join DID, a Vetting Card and several introductions.

The response describes the community's policy and nothing else. The `vetting`
object names methods, counts, claim *types* and a governance link; it never carries
a claim value, a vetter's identity, or anything about any applicant. In particular
it does not list vetters. How an applicant finds one is a separate question, with a
different privacy answer, and folding a vetter list into a document anyone can
fetch anonymously would publish exactly the social graph peer vetting is designed
to keep private. A community **MUST NOT** use the open `vetting` object to carry
vetter identities, applicant data, or anything else about a person.

`acceptedDocumentClasses` is absent by default, and communities **SHOULD** leave it
absent unless their governance genuinely needs a floor. Stating a floor pushes every
vetter towards the same documents. It also excludes applicants who do not hold one
of the listed classes, and a vetter who knows someone well enough to attest without a
document is often the better evidence.

### Correlation

The community maintainer declares `identifierScope: public` for the reason `0.1`
gave. Discovery only works if a stranger can name the community: an applicant finds
`communityDid` in a directory or governance document, asks it what to present, and
later addresses the same identifier at [`submit`](../../submit/0.2/spec.md) and
names it to every vetter. A pairwise community identifier would break that chain at
its first link. The cost, a fixed community identifier any observer can name, is
accepted openly.

`requirementsDigest` adds a correlation handle that deserves a precise statement.
An applicant echoes it to each vetter and at submission, so it links those
documents to one criterion version. That is its purpose, and it links nothing a
`communityDid` does not already link, **provided every caller receives the same
digest**. A community that tailors criteria per caller produces per-caller digests,
and the digest then becomes a tag the community can read back at submission to
learn which discovery request led to which application — and, where a proof was
required at discovery, which DID asked. A community **SHOULD NOT** tailor a
criterion carrying `vetting`. An applicant can detect tailoring by comparing digests
with another prospective applicant, or with a second anonymous read, and **SHOULD**
treat a digest it has not seen served anonymously with suspicion.

The applicant declares `pairwise`, and an unproofed read discloses no applicant
identifier at all. A community that requires a proof converts an anonymous read into
an attributable one, and thereby learns who is considering applying. That is a larger
population than its applicants, and one that never chose to be observed.

### Retention

Transient on the community's side: there is nothing in the request to retain. Where a
community requires a proof, what it may keep is that a particular DID asked, at a
particular time. That is attribution metadata, and it **SHOULD** be aged out on a
rate-limiting schedule, not an adjudication one.

The applicant has a real reason to keep the response. The criterion version its
`requirementsDigest` names is the evidence of what the applicant was told to gather.
If requirements change mid-application, it is the only thing that lets the applicant
show they were owed evaluation under the earlier version within `requirementsGrace`.
An applicant **SHOULD** retain the criterion alongside the application until the
decision.

The community, for its part, **MUST** retain every criterion version whose digest a
pending or decided application cites, because a digest it can no longer resolve is a
decision record it can no longer explain.

### Consent/purpose

The purpose is pre-application discovery, and it serves data minimisation directly.
An applicant who knows the requirements gathers exactly the vetting they need. One who
does not either over-gathers — more vetters, more copies of their legal name on more
devices — or learns what was missing only after submitting, disclosing more at each
attempt.

Nothing about a caller is collected here, so there is no reuse question about
applicant data. The reuse question that does arise is the community's own: a proof
required for rate-limiting yields a record of interest, and using it for anything
other than rate-limiting is a purpose the caller never addressed. Whether to require
a proof at all is consumer policy, and per
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification takes
no position on it.
