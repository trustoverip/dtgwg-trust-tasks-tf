# Design Note: Trust Ceremonies

| | |
|---|---|
| **Status** | Draft — proposed, not implemented |
| **Date** | 2026-08-09 |
| **Applies to** | Any interaction composed of more than one *Trust Task* between two or more parties |
| **Related** | `docs/adr/0001-naming-the-multi-task-flow-layer.md`, SPEC §2, §4.9–§4.9.2, §4.10 item 5, §6.1, §7.3 items 13–14, §8.6, §9.3, §11, `specs/vtc/join-requests/*`, `specs/vtc/members/{solicit-vmc,request-vmc,vmc}/0.1`, `specs/vtc/ceremonies/list/0.1`, `specs/audit/verify/0.1`, `specs/vta/credentials/issue/0.2` |

*This note is non-normative rationale. Unlike the other notes in this directory,
**there is no normative surface yet** — nothing here is specified, published, or
implemented. It proposes one, and takes the six decisions ADR 0001 deferred.*

*It is written before the implementation rather than after it, which is the
weaker position. §15 is therefore longer than it would be in a retrospective
note, and should be read as part of the proposal rather than as an appendix. A
design note that only records its wins is marketing; a prospective one that only
records its confidence is worse.*

## 1. Problem

Trust Tasks works where an interaction is one task and one response. Interactions
are now appearing that are not that shape: a governance decision needing several
endorsements, member onboarding spanning a witness and a registry, a credential
exchange with a consent step in the middle.

These are built today as several bilateral tasks correlated by `threadId`. That
part is right, and SPEC §2 already settles it:

> Exchanges involving more than two parties are modeled as multiple bilateral
> *Trust Tasks* linked by the framework's `threadId` member.

What is wrong is where the *composition* lives. Today it is in application code
at each party: which task follows which, who may perform each step, and what it
means for the whole thing to have completed. Three consequences follow.

- **It cannot be published.** A community cannot hand a counterparty a machine-
  readable description of an onboarding flow, so every integration is pairwise
  again — the exact problem §1 of SPEC set out to remove, displaced one level up.
- **It cannot be reviewed.** A governance body approving "how members are
  admitted" is approving prose plus somebody's implementation.
- **It cannot be proven.** Each step's document proves that step. Nothing proves
  the flow: not that these steps belong together, not that no step was dropped,
  not that it finished.

The third is the one that motivates the layer. The rest is convenience.

## 2. Three concerns, deliberately separable

"Orchestration" conflates three things with different design pressures. Keeping
them separate is the single most important structural decision here.

| | Concern | Nature | Status today |
|---|---|---|---|
| **1** | **Definition** — which tasks, which roles, what order | Static, published, auditable | Nothing (VTC has a local analogue) |
| **2** | **Coordination** — who drives, how a party learns what is next | Dynamic, per-run | Partial: `threadId`; `trust-task-next-step` reserved but unspecified |
| **3** | **Evidence** — what proves the *whole* flow happened | Artifact, retained | Nothing |

They must remain independently adoptable. A ceremony should be usable
**definition-only** — a community publishes the script, humans and existing task
implementations follow it, nothing new on the wire — and **evidence-only** — an
ad-hoc flow that nonetheless yields a receipt. If (1) is a precondition for (3),
adoption stalls behind a definition format nobody has written yet.

## 3. What already exists

Most of the mechanism is present and was never named as a layer.

| Piece | Status | Gives | Gap |
|---|---|---|---|
| `threadId` (§4.9) | Normative; no validation semantics | One exchange | Not unique, not enforceable |
| `parentThreadId` (§4.9.2) | Navigation only | One level of containment | §5.1 |
| §4.9.1 | Normative | How to cite an exchange as evidence | Only names one exchange |
| `trust-task-next-step` (§8.6) | **Reserved, unspecified** | Recipient-driven continuation | §8 |
| `trust-task-ok` (§8.6) | **Reserved, unspecified** | Success with receipt | — |
| `sideEffects` / `exposure` (§7.3.13–14) | Normative, per task | Risk class per step | Does not aggregate — §11 |
| `audit` hash chain | Shipped | Tamper-evidence pattern | Single-party, single log |
| `trust-task-discovery` (§11) | Shipped | "Which tasks do you accept?" | Not "which ceremonies, in which role?" |

§4.9.2 also concedes the gap in writing, which is worth quoting because it
scopes this note:

> The member records **one** level of containment. Reconstructing a deeper
> ancestry requires the intervening documents, and the framework defines no
> representation for a full chain; a specification needing one is better served
> by an explicit payload structure than by inferring it from thread metadata.

This layer is that explicit structure.

## 4. Vocabulary

Fixed by ADR 0001; restated so this note stands alone.

| Term | Names | Analogous to |
|---|---|---|
| **Ceremony** | The definition — published, versioned, resolvable | A *Trust Task specification* |
| **Enactment** | One run of a ceremony; globally unique, non-reusable | A *Trust Task document* |
| **Step** | One *Trust Task* exchange within an enactment | — |
| **Ceremony receipt** | The evidence artifact for a completed enactment | — |
| **Recorder** | The role a definition names as issuer of the receipt | — |

ADR §5 leaves open whether the definition is instead called a *choreography*.
This note spells the envelope member `ceremony` throughout; if the split
vocabulary is adopted, the member is `choreography` and every occurrence changes
with it. That substitution is the whole of the difference, which is itself an
argument that the sub-decision can be settled late — but not after 0.4 (§13).

## 5. Threading

Proposed: one new top-level member, added in framework 0.4.

```json
{
  "id": "urn:uuid:2c7f5e10-6a4b-4f8e-9d31-0b6a2f4c8e15",
  "type": "https://trusttasks.org/spec/webvh/witness/publish/0.1",
  "threadId": "urn:uuid:4a0e2b77-88c1-4d55-9f2a-6c3d1e5b7a92",
  "parentThreadId": "9b1d3f60-52a8-4c17-8e44-1d9c7b05f3ae",
  "ceremony": {
    "definition": "https://trusttasks.org/ceremony/vtc/member-onboarding/0.1",
    "enactment": "urn:uuid:8f21b0c4-7d3e-4a91-b5c2-1e6f0a9d4b83",
    "step": "witness-publish",
    "prev": [
      { "id": "urn:uuid:1d0a…", "digestMultibase": "zQmb…" },
      { "id": "urn:uuid:7c42…", "digestMultibase": "zQmW…" }
    ]
  },
  "issuer": "did:web:witness.example",
  "recipient": "did:web:host.example",
  "issuedAt": "2026-08-08T10:15:00Z",
  "payload": { "…": "…" }
}
```

An object rather than a bare `ceremonyId` string, because a bare string needs
siblings within a week: a verifier that knows only "these documents share an
identifier" cannot check the set against anything.

`prev` is a **set** of digests, not one. It degenerates to a chain when steps are
sequential and expresses a partial order when they are not — two endorsements
gathered concurrently, both named by the step that consumes them. A linear-only
`prev` would force definitions to serialize work that is genuinely parallel, and
serialized-for-the-format is the kind of lie that makes evidence wrong later.

`enactment` MUST be globally unique and non-reusable, on the same terms as `id`
(§4.3) and unlike `threadId`. It is what §4.9.1 citations name when the claim is
about the flow rather than about one exchange.

### 5.1 Why not `parentThreadId`

The obvious objection is that this already exists: model the ceremony as an
enclosing exchange and let each step set `parentThreadId` to its thread. It is
close, and it fails on three counts.

1. **It carries no normative semantics.** §4.9.2 states a consumer `MUST NOT`
   reject a document on the basis of `parentThreadId` alone. Ceremony membership
   needs to be checkable — "this is not the step I am at", "this predecessor
   digest does not match". Adding that would break the member's stated contract
   for every existing user of it.
2. **It records one level, and ceremonies nest.** §4.9.1's own worked example is
   a witnessing ceremony conducted inside a relationship exchange. Spend
   `parentThreadId` on ceremony membership and the nesting it was added for can
   no longer be expressed.
3. **It points at a `threadId`,** which §4.9 does not require to be unique or
   non-reusable. Evidence needs a stable anchor. §4.9.1 hit exactly this and
   mandated naming by document `id` instead; the same reasoning applies here.

There is also a modelling mismatch. Membership is not containment: an
enactment's steps are typically *siblings* — several top-level exchanges, none
nested inside another. That is a flat set, not a tree edge.

The four identifiers are orthogonal and should stay that way:

| Member | Names | Unique? | Enforceable? |
|---|---|:--:|:--:|
| `id` | One document | Yes (§4.3) | Yes |
| `threadId` | One exchange | No | No (§4.9) |
| `ceremony.enactment` | One flow | Yes | Yes |
| `parentThreadId` | Nesting between exchanges | n/a | No (§4.9.2) |

### 5.2 Why it must be a top-level member

It cannot live in `payload`: every task schema would have to opt in, and a
ceremony composed of tasks whose authors never anticipated it would be
impossible. That defeats the purpose — the point is to compose *existing* tasks.

It cannot live in `ext`: §4.5.1 requires reverse-DNS namespacing and item 6
forbids cross-specification semantics for any `ext` key. A *standard* ceremony
could not exist under those rules, only vendor ones.

§4.2 already permits additional top-level members, so this is a framework minor
bump and not a breaking change to the document model. It is, however, a breaking
change to the *libraries* — see §13.

There is a security reason as well, and it is the stronger one. A Data Integrity
proof covers the document with `proof` excluded (§4.7), so a top-level `ceremony`
member is **signed**. Both `enactment` and `definition` are therefore bound to
the step by its issuer. Carried as transport metadata or as an unsigned sidecar,
a step could be lifted into a different enactment, or reinterpreted under a
different definition that gives its `step` name another meaning. §10 develops
this.

## 6. The definition

A ceremony definition is a published document at
`https://trusttasks.org/ceremony/<slug>/<MAJOR.MINOR>`, versioned on the §5
scheme, resolvable under content negotiation like a *Type URI*.

### 6.1 Contents

- **Roles** — named participants (`witness`, `registry`, `applicant`), with the
  VID schemes each accepts. Roles are bound to actual VIDs at enactment, not in
  the definition.
- **Steps** — each with a stable `step` name, the *Type URI* it enacts, the role
  that issues it, the role that receives it, whether it is required or optional,
  and its `prev` step names.
- **Evidence level** — one of §7's four, plus the role acting as recorder where
  applicable.
- **Completion** — which steps must have occurred for the enactment to be
  complete. Not necessarily "all of them": a definition with optional steps needs
  to say what suffices.
- **Compensation** — per step, whether it is compensatable and by which task
  (§9). Descriptive, on the §7.3.13 pattern.

The role/step split is what makes a definition publishable. `member-onboarding`
says *a witness publishes*, not *`did:web:witness.example` publishes*; the
binding happens per enactment, so one definition serves every community.

### 6.2 What it deliberately cannot express

No loops. No timers as first-class control flow. No data transformation or
mapping language. No conditional expressions over payload contents. No runtime
state machine.

> **Amended by [§13.4](#134-the-finding-62s-no-loops-rule-is-too-strong).** The
> first real flow instrumented needs bounded repetition — read "no loops" as
> "no *unbounded* loops" throughout this section.

This is the constraint the ADR named the layer to protect, so it is worth
stating the criterion plainly:

> **A ceremony definition must be readable end-to-end, by the governance body
> that has to approve it, in one sitting.**

The moment the definition language is Turing-complete it stops being auditable,
and an unauditable definition is worse than none — it looks like governance and
is not. Every one of the excluded features arrives as a reasonable request, and
each is individually defensible; the discipline has to be structural rather than
case-by-case, which is what a written acceptance criterion is for.

Branching is the hard case, because some is genuinely needed — a step that
happens only if an endorsement was declined. The proposal is that branching is
expressible **only** as step optionality plus `prev` structure, never as a
predicate over payload contents. A verifier can then check a completed enactment
against the definition using only document metadata, without interpreting any
task's payload semantics. That property is worth more than the expressiveness it
costs, and it degrades gracefully: a flow needing real predicates decomposes into
two ceremonies, and the outer one records which was enacted.

### 6.3 Borrow the projection, not the name

ADR §4 rejected *choreography* as a name while recommending its machinery. The
specific thing worth taking is from multiparty session types (Honda, Yoshida):
a global description is **well-formed** only if it can be *projected* onto each
participant as a local behaviour that participant can actually execute.

Concretely, definition validation should check that every step's issuing role
can know it is its turn — from a document it received, not from out-of-band
state. A definition where the registry is expected to act on a step it never
observes is unimplementable, and this is checkable statically, at publication,
by the registry build. That is a cheap check that catches a class of definition
bug which would otherwise surface as a hung enactment in production.

## 7. Evidence

The core of the note. Four levels, and the recommendation is not to pick one
globally.

### 7.1 The levels

| Level | Mechanism | Detects | Cost |
|---|---|---|---|
| `collected` | Verify N step documents, check `enactment`, check the set against the definition | Forgery of any step | None beyond today |
| `chained` | Each step commits to predecessors via `ceremony.prev` | + drop, reorder, insertion, duplication | Digest handed forward |
| `receipt` | A recorder issues one artifact enumerating the steps | + a portable, single-artifact claim | A named recorder role |
| `countersigned` | Every participant signs the transcript | + recorder omission of a whole enactment | Full round of signatures |

### 7.2 `collected`

The relying party gathers the step documents, verifies each `proof` under §4.7,
checks each `ceremony.enactment` matches, and checks the resulting set satisfies
the definition's completion rule.

No new cryptography, and it is genuinely sufficient for low-stakes flows. Its
limit is that it **cannot prove absence**: not that no additional step occurred,
not that the enactment did not later abort. It also inherits the availability of
all N documents — a verifier who was not a participant must be handed every one.

### 7.3 `chained`

Each step's `ceremony.prev` carries the digests of its predecessor documents.
This is the `audit` pattern (`prevHash` / `entryHash`) lifted from one party's
log to a multi-party flow, and the failure kinds `audit/verify` already
distinguishes — `tamperedEntry`, `brokenLink` — carry over directly. Reuse that
vocabulary rather than minting a second one.

Two properties worth being explicit about:

**Privacy is preserved.** Step 3's issuer needs its predecessor's *digest*, not
its content. The prior party (or the recorder) hands the digest forward. Since a
step document contains a globally unique, non-reusable `id` (§4.3), the digest is
not guessable from the document's structure, so it discloses nothing about the
payload — which matters when the steps of one ceremony have genuinely different
audiences.

**It is what makes a receipt trustworthy.** See below.

### 7.4 `receipt`, and the recorder's deliberately weak claim

A `trust-ceremony-receipt` is a *Trust Task document* enumerating each step by
`(step, type, issuer, id, digest)`, issued by the role the definition names as
recorder.

The crux is what the recorder is claiming. Not "these things happened" — the
recorder mostly did not witness them, and a claim that strong would make it a
trusted third party, which the framework's posture rejects. The recorder attests
**completeness and ordering only**. The content of each step is attested by that
step's own issuer, via that step's own `proof`, verifiable independently by
anyone holding the receipt.

This makes the receipt useful without making the recorder trusted:

- A **forged step** is impossible — it would need the step issuer's key.
- A **fabricated enactment** is impossible for the same reason.
- An **omitted intermediate step** is caught by the chain: under `chained`, the
  successor commits to the omitted step's digest, so the receipt does not
  verify. This is why `receipt` **implies** `chained`; without it, the recorder's
  ordering claim rests on nothing.

What a malicious recorder *can* still do is refuse to issue a receipt, or claim
an enactment is incomplete when it is not. Both are availability failures, not
integrity failures, and both are visible. §7.5 is the answer where they matter.

Because a receipt is by construction retained and relied upon by parties outside
the exchange, §4.7.1 makes `proof` **REQUIRED** on it, and §4.8.2's audience
binding then requires an in-band `recipient` — unless the receipt is declared a
bearer specification under §4.8.3, which is the more likely reading given a
receipt's whole purpose is to be shown to unspecified verifiers. That is a real
decision, not a formality: bearer means any holder is a legitimate audience, and
a receipt naming the participants of a private governance ceremony may not want
that. **Proposal: the receipt is non-bearer by default, and a definition may
declare its receipts bearer.**

### 7.5 `countersigned`

Every participant signs the transcript. This closes recorder misbehaviour
entirely and is the right level for irreversible or authority-shifting flows —
key ceremonies, governance decisions that transfer control. It is far too heavy
as a default, needing a coordination round after the work is done.

### 7.6 Declared per definition, on the framework's existing pattern

The evidence level is declared by the ceremony definition, not fixed by the
framework. This is the same shape the framework already committed to for
`proofRequirement` (§7.3.8), `sideEffects` (item 13) and `exposure` (item 14):
declare at the boundary, let the specification state its own threat model.

The §4.7.1 rule carries over unchanged — a definition **MAY** strengthen the
default for its steps and **MUST NOT** weaken it. A ceremony cannot declare
`collected` and thereby relieve a step whose own specification requires a proof.

### 7.7 Digests: reuse the W3C conventions, do not mint a format

Every digest in this design — `prev` links, receipt step enumeration — uses:

> **A multibase-encoded multihash over the RFC 8785 (JCS) canonicalization of
> the referenced document, carried in a member named `digestMultibase`.**

Nothing here is new. Each part is already used either by W3C or by this
registry, and picking anything else would be minting a private format for a
solved problem.

- **`digestMultibase`** is the W3C Verifiable Credentials Data Model 2.0
  property for exactly this job — a digest naming a referenced resource, as used
  by `relatedResource`. Reusing the property name means a receipt's step
  enumeration is structurally a `relatedResource` list, which VC tooling already
  understands.
- **Multihash** carries the algorithm in-band, so the digest is self-describing
  and the format survives an algorithm change. A bare `sha-256:` prefix or raw
  hex hard-codes SHA-256 into the wire format, and the sibling design note in
  this directory already records that hard-coding as a mistake worth not
  repeating: *"State the aspiration; do not claim the capability."* Multihash is
  how the aspiration becomes real rather than stated.
- **Multibase** makes the encoding self-describing too, so a verifier never has
  to guess base58 versus base64url from context.
- **JCS (RFC 8785)** is already the registry's canonicalization: it is what
  `task-consent`'s `payloadDigest` is defined over, what `vta/credentials/issue`
  uses for `policyHash`, and what the `eddsa-jcs-2022` cryptosuite requires. A
  digest with no declared canonicalization is not reproducible, which is the
  defect in several fields today (§7.8).
- **`did:webvh`** — a method this registry has specs for — already derives its
  SCID and entry hashes as multibase-multihash. A ceremony chain over documents
  that frequently *are* webvh log entries should not use a second convention.

`prev` entries carry `{id, digestMultibase}` rather than a bare digest, because
the `id` is what §4.9.1 makes globally unique and non-reusable, and a verifier
that cannot locate the predecessor cannot check the digest against anything.

### 7.8 The registry is not consistent about this today

Surveying every digest-shaped field in `specs/` finds 18, across four
incompatible conventions:

| Convention | Fields | Example |
|---|---|---|
| Multibase-multihash over JCS — **correct** | 2 | `vta/credentials/issue/{0.1,0.2}` `policyHash` |
| "Multihash", encoding unstated | 4 | `chat/message` `digest`, `consent/request` `firstMessageDigest`, `policy/_shared/0.3` `payloadDigest` |
| Lowercase hex, SHA-256 hard-coded | 2 | `provision/integration/{0.1,0.2}` `digest` (`^[0-9a-f]{64}$`) |
| Unspecified — no encoding, no pattern | 6 | `audit/_shared` `prevHash` / `entryHash`, `task-consent/*` `payloadDigest` |

`audit` is the one that matters most here, because §7.3 proposes reusing its
chain vocabulary: its schema constrains nothing while `audit/verify`'s prose
says "Hex `entryHash`", so implementations emit hex and the schema does not say
so.

Converging these is worth doing and is **out of scope for this note**, because
it is not a docs change. Moving `audit` and `provision/integration` off hex
changes the wire format, which SPEC §5.2 makes a breaking change requiring a new
version folder rather than an in-place edit, followed by regenerated bindings and
library bumps. It also needs per-slug CODEOWNERS review. It should be its own
change, sequenced before Stage 2 so the ceremony layer is not the last thing
built on an inconsistent base.

Two fields must be **excluded** from any such sweep: `scid` / `new_scid` /
`newScid` and `nextKeyHashes` are `did:webvh`-defined values. SPEC §4.10 item 5
requires externally-owned values to be carried verbatim, so their format belongs
to the DID method, not to this registry.

## 8. Coordination, and the smallest useful increment

`trust-task-next-step` is already reserved (§8.6) and unspecified. Specifying it
needs no definition format, no envelope member, and no framework version bump:
a recipient answers "understood, but I need this next", naming a *Type URI* and
carrying the `threadId` forward.

That is sequential, recipient-driven continuation — and a large share of what is
being asked for today is exactly that, with no definition and no receipt. It
should be built first, on its own, for two reasons beyond cost: it is the
cheapest way to find out what the real flows look like before fixing a definition
format around guesses, and it is the one part of this note that is unblocked
regardless of how ADR §5 closes.

Its relationship to the rest is compositional, not preparatory. `next-step`
drives an enactment where the definition leaves ordering to the recipient;
`ceremony.step` names where the enactment has got to. Neither requires the other.

## 9. Abort, and the state no step describes

§7.3.13 classifies side effects **per task**. A ceremony that runs three
`mutating` steps and then never receives its fourth leaves the system in a state
no single declaration covers. `trust-task-error` reports one document's failure;
nothing says "this enactment is dead, and here is what already took effect."

Proposed: a `trust-ceremony-abort` document naming the enactment and enumerating
the steps that committed, issued by any participant that determines the flow
cannot complete, with the definition marking each step compensatable or not.

Deliberately **not** proposed: a compensation engine, or saga-style automatic
rollback. ADR §4 rejected *saga* as a name precisely to avoid promising this.
The goal is to make the wreckage legible — a party inspecting an aborted
enactment should be able to see what took effect and what did not, and decide.
Automating that decision requires understanding each task's semantics, which is
exactly what the framework has never claimed to do.

Timeout is the common trigger and needs a home. `expiresAt` (§4.2) governs one
document; there is no equivalent for an enactment. The minimal answer is that a
definition declares a maximum enactment duration and any participant may abort
past it. Whether that is enough is genuinely unclear (§15).

## 10. Security

**Ceremony membership is a claim, not a fact.** A party receiving a step is being
*told* by that step's issuer that it belongs to enactment X of definition Y. The
receiver can resolve the definition, check its own step matches, and check `prev`
digests against documents it holds. It cannot verify the enactment exists as
described, or that other steps went as claimed. This is the same posture as
discovery (§11.4): advisory, and narrowing what a party chooses to do rather than
binding it.

The consequence must be normative:

> A consumer **MUST NOT** grant authority on the basis of ceremony membership
> alone.

Without this, `ceremony` becomes a confused-deputy vector — "you are in the
onboarding ceremony, so perform this step" is an unauthenticated assertion by
whoever composed the document. Every step still runs the full §7.2 pipeline and
every authorization decision still rests on `issuer`, proof, and local policy.
Membership is context, never permission.

**Replay across enactments** is prevented by `ceremony` being a signed top-level
member (§5.2): a step naming enactment X cannot be lifted into enactment Y
without invalidating its proof. This is the security argument for the member's
placement, and it fails entirely if the data is carried as transport metadata.

**Reinterpretation under a different definition** is prevented by the same
mechanism, since `definition` is signed alongside `enactment`. A step named
`approve` means what `member-onboarding/0.1` says it means, and cannot be
relocated to a definition where `approve` carries other weight.

**Enactment identifiers are correlation handles.** An `enactment` value appearing
across steps with different audiences links those audiences together — by
construction, since that is the point. Where a ceremony's steps have genuinely
disjoint audiences, that linkage is a disclosure. There is no proposal here that
fixes it; per-party pairwise enactment identifiers would break the shared anchor
the evidence model depends on. Named in §15 rather than solved.

**The definition is a fingerprint.** Publishing which ceremonies a deployment
enacts discloses its configuration, on the same terms §11.5 already describes for
discovery, and the same mitigation applies: authenticate before answering, or
answer partially.

## 11. Aggregate side effects and exposure do not compose

A ceremony's aggregate risk class is **not** the maximum of its steps', and this
is the open problem most likely to be got wrong quietly.

For `sideEffects` the maximum is defensible: a flow containing a `destructive`
step is destructive. For `exposure` it is not. Three steps each disclosing
`metadata` to three *different* parties may aggregate to `secret`, because the
correlation across recipients is itself the disclosure — precisely the property
that makes a shared `enactment` identifier useful (§10). Taking `max()` would
report `metadata` and be wrong in a way no individual step's declaration is.

`actsAsSubject` compounds similarly: a ceremony in which the subject's authority
is exercised by several parties in sequence is a different proposition from any
one of those steps.

**Proposal: framework 0.4 declares this unsolved rather than shipping a
composition rule.** A definition may declare its own aggregate `exposure`, as a
statement by its author on the same descriptive-not-prescriptive terms as
§7.3.14, and the framework defines no derivation from the steps. Shipping a
`max()` rule would be worse than shipping nothing, because it would be relied on.

## 12. Discovery

§11's `supportedTypes` answers "which tasks do you accept?". The ceremony
question is "which ceremonies do you participate in, and in which roles?" — a
party may implement every task of a definition and still decline to act as its
recorder.

The natural extension keeps §11's shape and its advisory status:

```json
{
  "supportedCeremonies": [
    {
      "definition": "https://trusttasks.org/ceremony/vtc/member-onboarding/0.1",
      "roles": ["witness", "recorder"]
    }
  ]
}
```

Stage 3. It is useful only once definitions exist and are being enacted across
organizational boundaries.

## 13. Worked example: VTC member onboarding

Instrumenting a real flow was meant to illustrate the design. It also falsified
part of it (§13.4) and found a gap in the registry (§13.5), which is the reason
this section is placed before §15 rather than in an appendix.

### 13.1 The flow as it exists today

Admitting a member to a Verifiable Trust Community is already a multi-task,
multi-party flow built entirely from shipped specs:

| # | Task | Issuer → Recipient | Side effects |
|---|---|---|---|
| 1 | `vtc/join-requests/manifest/0.1` | applicant → community | `none` |
| 2 | `vtc/join-requests/submit/0.1` | applicant → community | `mutating` |
| 3 | `vtc/join-requests/submit-receipt/0.1` | community → applicant | `none` |
| 4 | `vtc/join-requests/status/0.1` | applicant → community | `none` |
| 5 | `vtc/join-requests/decide/0.1` | administrator → community | `mutating` |
| 6 | `vtc/members/solicit-vmc/0.1` | administrator → community | `mutating` |
| 7 | `vtc/members/request-vmc/0.1` | community → member | `none` |
| 8 | `vtc/members/vmc/0.1` | member → community | `mutating` |

Steps 6–8 are the reciprocal-membership exchange. `solicit-vmc` states the
decomposition explicitly:

> Three tasks, three party pairs. This one is administrator → community. It is
> **not** the request that reaches the member, and it does not carry a
> credential.

That is SPEC §2's bilateral rule applied by hand, and it is exactly the shape a
ceremony describes.

### 13.2 The layer is already being hand-rolled

`solicit-vmc` returns a `threadId` in its **payload**, documented as:

> The returned `threadId` is how a caller correlates the eventual
> `vtc/members/vmc` delivery with this solicitation.

A correlation handle spanning three party pairs, carried in a task payload
because the framework offers nowhere else to put it. This is §1's claim
demonstrated rather than asserted, and it inherits precisely the weakness §5.1
identifies: `threadId` is not required to be unique or non-reusable (§4.9), so
the anchor the whole correlation rests on is the one identifier the framework
declines to constrain.

Under this design that value becomes `ceremony.enactment` — unique,
non-reusable, signed, and carried on the envelope rather than in one task's
payload, where every other step can see it.

`vtc/members/vmc`'s optional `requestId` is the same pattern: it exists to carry
"the join-ceremony close" — the registry's own words — because the step needs to
say which enactment it completes.

### 13.3 What the definition would look like

Roles: `applicant`, `community`, `administrator`, `member` (the applicant, after
step 5). Sketch, not schema:

```
ceremony vtc/member-onboarding/0.1
  evidence: receipt, recorder = community

  discover     manifest        applicant → community    optional
  apply        submit          applicant → community    prev: []
  acknowledge  submit-receipt  community → applicant    prev: [apply]
  decide       decide          administrator → community prev: [apply]
  solicit      solicit-vmc     administrator → community prev: [decide]
  ask          request-vmc     community → member       prev: [solicit]
  reciprocate  vmc             member → community       prev: [ask]

  complete when: decide ∧ (decision = rejected ∨ reciprocate)
```

Three things this makes visible that prose does not:

- **`discover` is genuinely optional** and has no `prev`. An applicant who
  already knows the criteria skips it, and the receipt is still complete.
- **`acknowledge` and `decide` both depend only on `apply`,** so they are
  concurrent. A linear `prev` would have forced a false ordering — the partial
  order of §5 is doing real work on the first flow tested.
- **The same Type URI can appear as two steps.** `vmc` is `reciprocate` here
  (carrying `requestId`), and is also sent unsolicited at renewal time with no
  ceremony at all. That is why `step` is a name in the definition rather than the
  Type URI: the retirement of `join-requests/accept` into `vmc` collapsed two
  tasks into one whose meaning depends on context, and only a step name can
  distinguish them.

### 13.4 The finding: §6.2's no-loops rule is too strong

`vtc/join-requests/status` can return `deferred`, with `needs` naming what the
applicant must supply and a `presentationDefinition` describing it. The applicant
supplies more evidence; the community may defer again.

That is unbounded repetition, and §6.2 forbids loops outright. The first real
flow tested breaks the rule as written.

The rule was over-tightened rather than wrong. Three ways out:

1. **Bounded repetition** — the definition declares a maximum round count, so
   the loop unrolls to a finite graph. "At most three rounds of supplementary
   evidence" is still readable in one sitting and still statically checkable.
2. **Nested enactment** — each deferral round is its own small ceremony, named
   once as an optional step in the outer definition. Nesting already exists via
   `parentThreadId`.
3. **Exclude it** — the deferral cycle sits outside the ceremony. Cheapest, and
   it loses the evidence of what was supplied, which is the part a governance
   body would most want.

**Recommend (1),** as a narrow amendment to §6.2: bounded repetition with a
declared constant, never an open `while`. It keeps the auditability criterion —
a reader can still enumerate every path — while admitting a construct the domain
plainly needs. §6.2 should be read as amended by this section.

### 13.5 The gap: `deferred` has no exit

Writing the definition surfaced something the prose does not: **no task supplies
the additional evidence.** `status` reports `needs`, but `submit` takes no
`requestId` — it opens a *new* request — and the registry defines no
`join-requests/supplement`. `deferred` is a reachable state with no defined way
out.

This is a defect in the VTC specs, not in the ceremony design, and it is worth
reporting on its own. But note *how* it surfaced: a ceremony definition cannot be
written without naming the step that exits every non-terminal state, so
authoring one is a **reachability check over the registry**. That is a stronger
argument for the definition format than anything in §6, and it was not one this
note anticipated.

Whether it is fixed by adding a supplement task or by having `submit` accept a
`requestId` is for the `vtc/join-requests` owners. Either resolves §13.4 the same
way.

### 13.6 A second, smaller example

`vtc/members/personhood/{challenge,assert}` is a two-step challenge/response:
the community issues a single-use `challengeId`, the member embeds it as
`proof.challenge` in a Verifiable Presentation. `vtc/members/{rotate-challenge,
rotate}` has the same shape, as does `auth/passkey/enroll/{start,finish}`.

These need none of this layer. Two steps, one party pair, correlation already
carried by the challenge, and evidence needs are met by the documents
themselves. A ceremony definition would add ceremony and subtract nothing.

Worth stating because the registry's own prose already calls all three
"ceremonies" — the word is in shipped specs describing WebAuthn enrolment,
personhood assertion, and DID rotation, which is independent support for ADR
0001's naming. **The framework sense proposed here is narrower than that
colloquial use.** A challenge/response pair is a ceremony in the WebAuthn sense
and not one in this note's sense, and §6 of ADR 0001 needs a third row saying so,
or implementers will reasonably expect `personhood/assert` to grow a `ceremony`
member.

## 14. Staging and library impact

| Stage | Contents | Framework | Blocked by |
|---|---|---|---|
| **0** | Specify `trust-task-next-step` (§8) | none | nothing |
| **1** | §6.1 slug reservation; `/ceremony/` subtree | 0.4 | nothing |
| **1a** | Converge digest fields on multibase-multihash (§7.8) | — | nothing |
| **2** | `ceremony` envelope member (§5) | 0.4 | ADR §5 |
| **3** | Definition format; `trust-ceremony-receipt`; evidence levels | — | Stages 1a, 2 |
| **4** | Abort (§9); discovery (§12); `countersigned` | — | Stage 3 |

Stage 1a is independent of the ceremony layer and useful without it, but should
precede Stage 3 so the chain and receipt are not the last thing built on an
inconsistent digest base. It is the only stage that touches shipped specs: two
of its fields (`audit`, `provision/integration`) change wire format and so need
new version folders under §5.2, with regenerated bindings and library bumps
following.

Stage 1 should land ahead of everything, including the design: `trust-ceremony-*`
is registrable by any contributor under the current `^trust-task($|-|/)`
reservation, and the fix does not depend on any decision here.

Stage 2 is the expensive one. Adding a top-level envelope member changes what
consumers compile against, which per `CLAUDE.md` is a leading-component bump:
`trust-tasks-rs` 0.3.0 → 0.4.0, plus the six crates pinning
`trust-tasks-rs = "0.3"` (`-https`, `-didcomm`, `-didcomm-v1`, `-proof`, `-tsp`,
`-capability-client`), released in `publish.yml`'s dependency order, with
`@openvtc/trust-tasks` kept in step. `cargo check` fails immediately on a missed
requirement, so the trap is discovering it mid-release rather than planning for
it. This is also the deadline for ADR §5: the member is spelled `ceremony` or
`choreography`, and changing it afterwards pays this cost twice.

## 15. What is not true yet

Everything above is a proposal. Nothing is specified, published, or implemented,
and the note has not been tested against a working flow — which is the reverse of
how the delegated-execution note in this directory was written, and a weaker
position. Specifically:

- **No definition format is written.** §6 lists what one contains, not its
  schema.
- **The branching assumption was tested once, and partly failed.** §6.2's claim
  that branching reduces to optionality plus `prev` survived VTC member
  onboarding for ordering, but its blanket prohibition on loops did not: the
  `deferred` supplementary-evidence cycle needs bounded repetition (§13.4).
  §6.2 stands as amended by §13.4 and has not been re-tested against any other
  flow. One counterexample found one flaw; a second flow may find another, and
  `vtc/ceremonies`' remaining governance decisions are the obvious next test.
- **Aggregate exposure is unsolved** (§11), and proposed to ship declared-unsolved.
- **Enactment identifiers correlate across audiences** (§10) with no proposal.
- **Timeout has no real home** (§9). "The definition declares a duration" is the
  minimum, not obviously enough, and interacts with `expiresAt` in ways not
  worked through.
- **Partial-order verification is unspecified.** Checking a completed enactment
  against a definition with optional steps and a DAG `prev` is a graph-matching
  problem, and no algorithm is given. It is likely simple; it is not written.
- **The recorder is an availability single point.** §7.4 shows it cannot forge
  or omit, but it can decline to issue. `countersigned` is the escape hatch, and
  it is heavy. Flows that need integrity *and* availability have no cheap answer.
- **Nesting and enactment membership interact untested.** A ceremony conducted
  as a step of another ceremony has both a `parentThreadId` and two enactment
  identifiers in scope. §4.9.1's citation rule should extend cleanly — name the
  innermost — but this has not been worked through against an example.
- **No implementation exists**, so none of the digest, chaining, or projection
  claims have met a wire. The delegated-execution note's closing lesson applies
  in advance: for a system whose correctness is a property of how components meet
  over a wire, the only tests that count are the ones that use the wire.
