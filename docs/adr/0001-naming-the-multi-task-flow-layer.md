# ADR 0001: Naming the multi-task flow layer

| | |
|---|---|
| **Status** | Proposed |
| **Date** | 2026-08-09 |
| **Applies to** | The prospective framework layer that composes several *Trust Tasks* into one flow |
| **Related** | `docs/design-notes/trust-ceremonies.md` (the design deferred below), SPEC §2 (bilateral model), §4.9–§4.9.2 (threading), §6.1 (slug reservation), §8.6 (reserved response types), §9.3 (`/binding/` subtree), `specs/vtc/ceremonies/list/0.1` |

*This ADR decides a **name and a vocabulary**. It does not decide the design of
the layer, which is deferred to a design note. It is recorded separately, and
first, because the name is load-bearing on the design — see §3 — and because two
of its consequences are expensive to reverse once framework 0.4 ships (§8).*

## 1. Context

Trust Tasks works well where an interaction is one task and one response. That
is most of the registry today: 27 spec families, nearly all of them a single
request/response exchange between two parties.

Interactions are now appearing that are not that shape — governance decisions
requiring several endorsements, member onboarding that spans a witness and a
registry, credential exchanges with a consent step in the middle. These are
built today as several bilateral tasks correlated by `threadId`, with the
knowledge of how they compose held in application code at each party.

The framework already anticipated this. §2 commits the shape:

> Exchanges involving more than two parties are modeled as multiple bilateral
> *Trust Tasks* linked by the framework's `threadId` member.

So the mechanism is settled. What is missing is a **name, a definition format,
and an evidence artifact** for the collection. Several pieces exist already but
were never named as a layer:

| Piece | Status | Gives |
|---|---|---|
| `threadId` (§4.9) | Normative; no validation semantics | One exchange |
| `parentThreadId` (§4.9.2) | Navigation only; `MUST NOT` reject on | One level of containment |
| §4.9.1 | Normative | How to cite an exchange as evidence |
| `trust-task-next-step` (§8.6) | **Reserved, unspecified** | Recipient-driven continuation |
| `trust-task-ok` (§8.6) | **Reserved, unspecified** | Success with receipt |
| `sideEffects` / `exposure` (§7.3.13–14) | Normative, per task | Risk class per step |
| `audit` hash chain | Shipped | A working tamper-evidence pattern |
| `vtc/ceremonies/list/0.1` | Shipped | A declarative flow manifest, invented locally |

That last row matters: the VTC namespace independently reached for the word
*ceremony* to describe a declarative, multi-step governance decision. The
concept is already in the codebase under that name.

## 2. Decision

Call the layer **Trust Ceremony**, with a three-part vocabulary:

| Term | Names | Analogous to |
|---|---|---|
| **Ceremony** | The definition — published, versioned, resolvable | A *Trust Task specification* |
| **Enactment** | One run of a ceremony, globally unique and non-reusable | A *Trust Task document* |
| **Ceremony receipt** | The evidence artifact for a completed enactment | — |

Two namespace reservations follow, and should land ahead of the design work
(§8):

1. Extend the §6.1 slug reservation from `^trust-task($|-|/)` to
   `^trust-(task|ceremony)($|-|/)`.
2. Reserve `https://trusttasks.org/ceremony/<slug>/<MAJOR.MINOR>` as a third
   subtree, structurally disjoint from `/spec/` and `/binding/`, on the §9.3
   pattern — a document whose `type` is rooted there is `malformedRequest`.

## 3. Why: the name is a design constraint

The layer has three separable concerns, and conflating them is the known failure
mode:

1. **Definition** — which tasks, which roles, what order. Static, auditable.
2. **Coordination** — who drives, how a party learns what is next. Dynamic.
3. **Evidence** — what proves the *whole flow* happened, as distinct from N
   documents each proving its own step.

The primary risk to this layer is not picking the wrong data model. It is
feature creep in concern (1) — loops, timers, parallel gateways, a data-mapping
language — until the definition is Turing-complete and therefore no longer
readable by the governance body that has to approve it. The acceptance criterion
we want to hold is that **a ceremony definition is readable end-to-end in one
sitting.**

A name is the cheapest available defence. "Ceremony" is understood to mean a
small number of consequential steps with named roles; nobody asks a ceremony for
a `while` loop. Names that carry no such limit — *workflow*, *orchestration*,
*flow* — invite exactly the scope we are trying to refuse, and the request
arrives as a reasonable feature ask rather than as a violation of anything
written down.

Secondary reasons:

- **Established meaning.** In security literature a *ceremony* is a protocol
  extended to include human participants (Ellison; and in practice root-KSK and
  CA key ceremonies). These flows have humans in them — consent, approval,
  witnessing — which the alternatives do not connote.
- **It implies witnessed evidence.** Ceremonies are attested by design. Concern
  (3) sits naturally under the word; under *choreography* or *flow* it does not.
- **It is decentralized.** A ceremony has no conductor. Participants know their
  parts from the rite. That matches §1.1 Goal 1 and the bilateral model of §2.
- **It is already in use here,** with the VTC namespace's agreement to share it
  (§6).

## 4. Options considered

Collision counts are literal-substring matches across `SPEC.md`, `specs/`, and
`bindings/`, taken 2026-08-09.

| Candidate | Decentralized | Limits scope | Human-in-loop | Evidence | Prior art | Collisions |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| **Ceremony** *(chosen)* | ✓ | ✓✓ | ✓✓ | ✓✓ | ✓✓ | shared, §6 |
| **Choreography** | ✓✓ | ✗ | ✗ | ✗ | ✓✓ / WS-CDL | 0 |
| **Rite** | ✓ | ✓✓ | ✓ | ✓ | ~ | — |
| **Procedure** | ~ | ✓ | ✓ | ✓ | ✓ | 0 |
| **Assembly** | ✓ | ~ | ✓ | ~ | ~ | 4 |
| **Flow** | ✗ | ✗ | ✗ | ✗ | overloaded | 103 |
| **Sequence** | ✗ | ✓ | ✗ | ✗ | — | 90 |
| **Saga** | ✗ | ✗ | ✗ | ✗ | ✓ | 0 |
| **Dance** | ✓ | ✓ | ✓ | ✗ | informal | — |
| **Session** | — | — | — | — | — | **799** |
| **Score** | — | — | — | — | — | see below |

**Choreography** was the strongest alternative and is the technically more
precise word: *choreography vs orchestration* is exactly the decentralized-vs-
centralized distinction we are asserting, and the multiparty-session-types
literature (Honda, Yoshida) supplies a formal treatment — a *global type* for
the definition, and **endpoint projection** for deriving each party's local
behaviour from it. Rejected on three grounds: the last specification to claim
the word, W3C WS-CDL (2005), is a well-known failure and the association is
live; the word carries no implication of limited scope, which is the property we
most need (WS-CDL had loops, parallelism and exception handling precisely
because nothing in the name said no); and it speaks only to concern (2),
covering neither humans nor evidence. **We should still borrow its machinery —
endpoint projection and well-formedness checking — in the design note, without
borrowing its name.**

**Dance** has genuine informal lineage ("the OAuth dance", "the TLS handshake
dance"), but it is always informal — nobody published the OAuth Dance
Specification. These documents are read by auditors and governance bodies, where
"the KYC dance receipt" does not survive review. It remains available as
colloquial usage, which needs no ratification.

**Saga** is the closest existing term of art for the abort-and-compensate
problem this layer will have to address. Rejected because a saga *is* the
compensating-transaction pattern; adopting the name would promise rollback
semantics we have explicitly decided not to build an engine for. Cite it in the
abort section; do not name the layer for it.

**Sequence** implies linear ordering. The design anticipates a partial order
(a step committing to a *set* of predecessors), so the name would be actively
misleading.

**Session** is unusable: 799 collisions, including the shipped `auth/sessions`
family.

**Score** is recorded here as a trap, because the musical metaphor is otherwise
apt — score as definition, performance as enactment — and it will be proposed.
"Trust Score" collides with credit and reputation scoring, the most loaded
phrase in the identity space. The misreading is not merely confusing.

## 5. Runner-up, and the one sub-decision left open

The same word does not obviously win for all three terms. A ceremony is an
*occasion*; the written form of one is a rite or an order of service, so
"Ceremony definition" is already slightly strained. That argues for a split
vocabulary:

> **Choreography** (the definition) → **Ceremony** (one enactment of it) →
> **Receipt** (the evidence)

This is more precise than either word used alone, reads correctly in prose —
*"the onboarding choreography, enacted as ceremony `urn:uuid:8f21…`, produced
this receipt"* — and dissolves the VTC question with no retconning at all:
`vtc/ceremonies/list` returns what a community can *enact*, and the published
artifact it points at is a choreography. Both words would be used in their exact
senses.

It costs two nouns to teach instead of one, and leaves no single name for the
layer: asked what we are adding to Trust Tasks, the answer is "choreographies",
which sheds the human and evidence connotations that motivated §3.

**This ADR proposes the single name and records the split as a live
alternative.** It is the one sub-decision that should be settled before framework
0.4, because the envelope member is spelled `ceremony` or `choreography` and
renaming it later is the §8 cost.

## 6. Relationship to `vtc/ceremonies/*`

The word is already used, at `specs/vtc/ceremonies/list/0.1`, for a community's
governance-decision manifests. Reuse is deliberate and agreed. The two are
related as general to particular, not as homonyms:

| | Framework **Ceremony** | `vtc/ceremonies/*` |
|---|---|---|
| Scope | Cross-party, cross-transport | Inside one VTC community |
| Describes | Which Trust Tasks, in what order, by which roles | Which governance decision, and the fields an operator UI renders |
| Audience | Both parties, and a third-party verifier | An operator UI |
| Identified by | `https://trusttasks.org/ceremony/<slug>/<M.m>` | A `purpose` string, local to the maintainer |
| Produces | A receipt relied on outside the exchange | A policy decision in VTC's audit log |

A VTC ceremony is a local, single-party, UI-shaped description of one decision.
A framework Ceremony is a published, multi-party, wire-shaped description of an
exchange. VTC's `nature` and `fields` members have no framework analogue and
should not acquire one — form rendering is VTC's business.

The upgrade path requires no rename: `vtc/ceremonies/list` comes to mean "the
ceremonies this community can enact", with each manifest optionally carrying a
`definition` pointing at a `/ceremony/` URI. Any framework specification MUST
carry the §2 disambiguation table so the distinction is not left to inference.

## 7. Consequences

**Positive.**

- The scope limit of §3 is expressible as a first-class argument in review
  ("that is not ceremony-shaped") rather than as a matter of taste.
- The three concerns get distinct names, so a ceremony can be adopted
  definition-only (published script, humans follow it) or evidence-only
  (ad-hoc flow that still yields a receipt).
- VTC's existing usage is validated rather than deprecated.

**Negative, and accepted.**

- "Ceremony" reads as heavyweight and may deter use for the three-step flows
  that are the common case. Mitigated by tiered evidence levels in the design
  note — a low-stakes ceremony should not pay key-ceremony costs.
- We give up the precision of *choreography* and its formal literature as a
  naming lineage, keeping only its machinery.
- Two senses of one word now exist in the repo. The §6 table is mandatory
  wherever both appear, and that is a standing documentation obligation.
- `Enactment` is an unfamiliar term of art and will need defining in SPEC §2.

**Not decided here.** The definition format; the threading model and whether a
`ceremony` envelope member is added; evidence levels and who issues a receipt;
abort and compensation semantics; whether aggregate `sideEffects` / `exposure`
is derivable from a ceremony's steps (we believe it is not); the discovery
extension. All deferred to a design note.

## 8. Why this is recorded before the design

Two consequences are expensive to reverse, and both are cheaper to settle now
than to relitigate under the design.

**The slug namespace is squattable today.** §6.1 reserves `^trust-task($|-|/)`
and nothing else. `trust-ceremony-*` is currently registrable by any
contributor. The reservation in §2 item 1 should land whatever the layer is
eventually called, and does not depend on the rest of the design.

**The envelope member name is a workspace event.** If the layer contributes a
top-level member — the design note's likely direction, since it can live neither
in `payload` (every task schema would have to opt in) nor in `ext` (§4.5.1 item
6 forbids cross-specification `ext` semantics, so no *standard* ceremony could
exist) — then it is spelled `ceremony` or `choreography` in framework 0.4.
Renaming it afterwards changes what consumers compile against, which per
`CLAUDE.md` is a leading-component bump: `trust-tasks-rs` 0.3.0 → 0.4.0, plus
the six crates pinning `trust-tasks-rs = "0.3"` (`-https`, `-didcomm`,
`-didcomm-v1`, `-proof`, `-tsp`, `-capability-client`), released in
`publish.yml`'s dependency order, with `@openvtc/trust-tasks` kept in step.

Note that the first increment of the layer — specifying the already-reserved
`trust-task-next-step` (§8.6) — needs neither reservation nor envelope member,
and so is unblocked by this ADR either way.
