# Design Note: Delegated Trust-Task Execution

| | |
|---|---|
| **Status** | Implemented (reference implementation); normative surface is `task-consent/*` and `policy/_shared/0.3` |
| **Date** | 2026-07-15 |
| **Applies to** | Any *consumer* that executes Trust Tasks on a *producer*'s behalf and gates them on human approval |
| **Related** | `specs/task-consent/{request,decision}/0.1`, `specs/policy/_shared/0.3`, `specs/vta/webvh/dids/update/1.0`, SPEC §7.3 items 13–14, `bindings/push/0.1` |

*This note is non-normative rationale. The normative surface is the
`task-consent/*` payloads, the `requireConsent` decision in
`policy/_shared/0.3`, and the side-effect / exposure classifications in
SPEC §7.3.*

*It is written after the fact, against a working implementation, and against an
adversarial re-reading of that implementation. Where the original design was
wrong, or where the reference implementation does less than the design implies,
this note says so — in the body where it is load-bearing, and in §16 as a list.
A design note that only records its wins is marketing.*

## 1. Problem

A relying party — a web page, a mobile app — needs a *consumer* to perform an
action only the consumer can perform, because only the consumer holds the keys:
publish a `did:webvh` LogEntry, rotate an update key, issue a credential.

Doing this per-action does not scale, and each new arm is a fresh opportunity to
get the authorization wrong. We want one mechanism, used for every action, that
is correct by construction: the RP proposes, the consumer decides, and a human
approves anything that matters.

The requirement that shapes everything else:

> **What the human approves must be what executes.** Cryptographically, not by
> convention.

## 2. Roles and trust

| Role | Trust | Responsibilities |
|---|---|---|
| Relying party | none | Proposes `(typeUri, payload)`. Nothing else. |
| Consent surface (device) | enrolled, revocable | Relays proposals. Renders executor-authored consent. Signs decisions. |
| Executor (consumer) | full | Validates, evaluates policy, computes effects, routes approval, verifies consent, executes. |
| Mediator / push gateway | none | Transport. Contentless wake only. |

The executor is the only trusted component. The residual risk is a compromised
executor, which is total; everything else degrades gracefully.

## 3. The invariant

> No state-mutating Trust Task executes at the executor unless it has verified a
> single-use grant, produced by a signed decision from an approver the current
> policy names, whose `payloadDigest` equals the digest of the **exact payload it
> is about to execute**, against the **exact prior state** it used to compute the
> effects it showed the human — and unless policy and the approver set **still**
> permit it at the moment of execution.

**This is conditional, and the condition does not hold by default.** Policy
enforcement is opt-in (`policy.enforcement`, default false), and the
boot-installed baseline allows every task, destructive included. Out of the box
nothing requires consent. The invariant is a property of a deployment that has
turned enforcement on and authored a `requireConsent` policy — an
expand-before-contract stance, deliberate, but a configuration, not a law of the
system. §16 restates this because it is easy to read the rest of this note as
describing the shipped default. It does not.

## 4. The executor is the authority

### 4.1 The effects

A payload says what was *asked for*. Only the code about to run knows what will
*happen*, and it knows it only against state the requester cannot see.

The example that drove this, and which the reference implementation actually
computes: a `webvh/dids/update` whose payload adds one service endpoint **also
rotates the DID's update key and refreshes its pre-rotation commitments**. The
consequence lives in the handler's semantics, not the payload's shape. A consent
surface rendering a diff of the submitted payload shows a one-line endpoint
addition and silently hides a key rotation — and every signature over that
approval still verifies.

So `task-consent/request` carries an `effects[]` block, and:

> **`effects[]` MUST be produced by dry-running the real handler** — the same
> code path that will execute — against the executor's own prior state.

Not by a parallel implementation that *describes* the handler. A second
implementation drifts, and when it drifts the human is confidently misinformed
while every signature still verifies. This is `plan` and `apply` sharing one code
path, except that a human signs the digest of the plan and `apply` refuses to run
against a plan it did not produce.

**Reality check.** In the reference implementation, exactly one handler has a
dry-run: `webvh/dids/update`. Every other consent-gated task sends an empty
`effects[]`, and the surface renders "consequences could not be determined"
(§9). That is the fail-safe, and it is correct as far as it goes, but §4.1
describes the norm the design is built toward, not the current breadth. A
`vault/delete` with `force: true` — an irreversible, unrecoverable hard delete —
is today approved against a blank effects list. The dry-run is the exception, not
the rule; widening it is the substance of making this design real, and it is not
done.

### 4.2 The classification

The same argument applies to *whether a task is destructive*.

If `sideEffects` were read from the registry, the registry would decide which
operations require human approval. That makes it a consent kill-switch, and a
downgradeable one: publish `…/update/1.1` declaring `sideEffects: none` and
consent evaporates for every consumer resolving by URI.

> **Registry metadata is for rendering. The executing code is for policy.**

SPEC §7.3 items 13–14 say this normatively: the declared class is *descriptive*,
a specification **MUST NOT** declare that a task requires consent, and a consumer
that gates on the class **MUST** determine it from the handler it is about to
invoke — treating an absent or unresolvable declaration as no weaker than
`mutating` / `discloses: secret` / `actsAsSubject: true`.

In the reference implementation the dispatch macro will not compile a handler
without a classification, and the value fed to the policy engine comes from that
compiled table. The floor for an unclassified URI is the most restrictive value.
No component consumes the served `registry.json` at all — the class is never
fetched — so a registry compromise is not even a cosmetic attack on this path.

## 5. Dry-running is harder than it looks

This section did not exist in the original design, and it is the most important
thing implementation taught us.

**A dry-run that mutates state can introduce the very deception it exists to
prevent.**

In the reference implementation, `webvh` update keys are derived from a BIP-32
path counter. Deriving a key *allocates* an index — it writes and fsyncs. A plan
that derived keys the way the real run does would therefore consume an index, and
the real run, taking the *next* one, would install **a different key than the
plan reported**. The approver would have authorized a rotation to a key that
never existed, and every signature over that approval would still verify.

The function's own doc comment claimed it was pure. It was not. This was found by
reading the code that runs, rather than the code's description of itself — which
is precisely the argument of §4.1, arriving uninvited.

The fix has three parts, and all three are load-bearing:

1. **Peek, don't allocate.** Predict the derivation without consuming it, sharing
   the derivation itself with the allocating path so the two cannot disagree
   about the key at a given index.
2. **Peek once, contiguously; allocate once, contiguously.** Two separate
   allocations read a counter the concurrent world can move between them, so the
   auth key and the pre-rotation keys — which the plan peeked as adjacent — could
   end up non-adjacent. Plan and execute each take the whole block in one step
   and split it the same way.
3. **Pin what you peeked, and assert it at the allocation.** A peek reserves
   nothing. The counter is pinned into the plan, and the allocation itself
   refuses (under the same lock that allocates) if the counter has moved. This
   holds even inside a single execution, in the window the pre-dispatch re-check
   cannot reach. An earlier version pinned the counter but re-checked it only in
   the gate, leaving that inner window open; it is now closed.

The general rule, for anyone implementing a planner:

> **A plan must be a read-only prediction, and every implicit allocation on its
> path is a correctness bug, not just an efficiency one.** Enumerate what your
> handler consumes — counters, nonces, sequence numbers, leases — before you
> assume a dry-run is side-effect free.

## 6. The binding chain

Five checks. Only the last survives a compromised consent surface.

### 6.1 Two digests, not one

The original design specified one digest,
`multihash(JCS(payload) ‖ challenge)` — salted, so it is not a confirmation
oracle. The salt is right and the single digest is not.

The salt matters: an unsalted digest over a low-entropy payload ("deactivate
`did:webvh:abc…`" has essentially one canonical form) lets anyone who observes it
in transit guess the operation and hash to check. A compromised, subpoenaed or
retrospectively-decrypted mediator turns the digest into a record of what you
did.

But the salted digest **cannot be the storage key**. The executor must recompute
the digest on a re-submit *before it knows the challenge* — the challenge lives on
the pending record it is trying to find. Keying storage by the salted value is
circular.

So there are two:

| | Salted? | Where it lives |
|---|---|---|
| **Internal digest** | no | Keys the pending request and the grant. **Never leaves the process.** |
| **Wire digest** | yes | The only one an approver sees, signs, or matches. |

An index maps the wire digest back to the internal one, because an approver's
decision can only carry the digest it was shown. The oracle closes without
breaking the reject → approve → re-submit loop.

### 6.2 The digest MUST bind the type URI

Also not in the original design, and found as a live bypass in the reference
implementation.

If the digest covers only the payload, two tasks whose payloads canonicalize
identically share a digest. `{"did": …, "contextId": …}` is a plausible payload
for an update, a key rotation **and** a deactivate. An approval collected for the
benign one authorizes the destructive one — and because the approver sees only an
opaque digest, nothing downstream can catch the substitution.

Bind the type URI, length-prefixed under a domain-separation tag so the
URI/payload boundary cannot be shifted:

```
SHA-256( DOMAIN ‖ len(typeUri) ‖ typeUri ‖ len(JCS(payload)) ‖ JCS(payload) [‖ challenge] )
```

### 6.3 State pinning, and what does *not* go on the wire

`effects[]` are computed against a specific prior state, pinned into the consent
request (`StatePin`) and asserted at execution. A human in the loop makes the
window minutes wide, so a lost update is a real risk, not a theoretical one.

The executor may also hold **internal** preconditions — the derivation counter of
§5, for instance. Those deliberately **do not** go on the wire. The approver
cannot verify them (they trust the executor; that is the whole trust model), and
putting a derivation counter in front of a consent surface only invites it to
render a number it cannot interpret. The wire pin carries what a human needs; the
executor keeps its own.

### 6.4 Closed payloads, single-use grant

Payload schemas are `additionalProperties: false` and validated against the
published schema *before* anything reads the payload — before the class is
derived, before policy is evaluated, before the dry-run — **for every task the
executor has a published schema for**. The reference implementation validates via
a compiled Type-URI → schema table; where no schema is published (many tasks
today), it dispatches unvalidated and logs, unless
`policy.require_payload_schema` is set. So "nothing rides along outside the
rendered effects" holds where a schema exists, and is a best-effort default
elsewhere. The `ext` slot is admitted by the closed schema (SPEC §4.5.1) so the
relay can still stamp an origin (§10).

This is not academic. A caller once sent `expectedVersionId` where the handler's
type expected `expected_version_id`; serde matched no field, nothing rejected the
unknown member, and the optimistic-concurrency precondition silently never
applied. A closed, validated schema rejects that; a permissive one drops it
without a word.

The **grant** is single-use: it is consumed at execution, and one grant
authorizes exactly one execution. (The original design said "the challenge is
consumed at execution"; that was imprecise. The challenge lives on the pending
record, which is deleted when the decision arrives; what execution consumes is the
grant. A decision is separately prevented from replaying by pending-deletion and
per-approver idempotency, and the executor's own retry by envelope-id dedup.)

### 6.5 Cross-device matching

For `destructive` tasks the requesting surface and the approving device each
display a short prefix of the wire digest, and the user **matches** them.

Checks 6.1–6.4 assume an honest device and defeat a hostile *page*. Only this one
defeats a hostile *device*, because only it moves the comparison into the human's
head, across two screens an attacker would have to control both of. It is the
reason approval routing must be able to target a device other than the requester,
and the reason a tap-to-approve UI throws the whole property away.

A spinner is something you wait out. A code is something you check. This is why
`webvh/dids/update` being classified `destructive` matters concretely: the
approving device demands a *typed* match only for `destructive` tasks, so an
under-classification would have quietly downgraded the flagship flow to a tap.

## 7. Time of check, time of use

Policy is evaluated when the consent request is minted. The task executes after a
human has looked at it — minutes later.

The gate runs on **every** submit, including the re-submit that consumes the
grant, so policy is re-evaluated. The *data* is covered by the state pin. And the
**approver set** is re-checked at grant consumption: the approvers who signed must
still be members of the set the current policy names, and still meet its
threshold, so a set edited or a threshold raised during the window applies.

One honest limit: this re-check is against the policy's *named approver set*
(configuration), not against a live device-enrolment registry. Disabling a device
via `device/disable` does not remove it from an approver set, so a disabled
device's already-signed decision still verifies unless an operator also edits the
set. "Currently-enrolled approver" in §3 should be read as "member of the set the
current policy names" — which is weaker, and §16 says so.

## 8. Two documents, not a version bump

The original design proposed generalizing `consent/*` to a `2.0`. That was wrong,
and the implementation says so.

`consent/*` `1.0` gates an inbound *messaging conversation* for an AI agent: its
subject is `{platform, conversationRef, kind, agent}` and its scope is
`receive | converse`. It cannot express "approve this task payload", and
subsuming both would require a polymorphic subject that neither family benefits
from. A MAJOR bump also implies the same task evolved, which is untrue.

Hence `task-consent/{request,decision}/0.1` — a distinct family, additive,
leaving `consent/*` untouched. They start at `0.1`, not `1.0`, because they were
unproven when published, and they did in fact change.

A related versioning rule, learned the same way: a task the executor **already
dispatches** takes the version already on the wire. `vta/webvh/dids/update/1.0`
was dispatched long before it was specified; its spec is `1.0`, because a `0.1`
spec would describe a URI nobody calls. New families the author mints start at
`0.1`; retro-specced tasks take what ships.

## 9. Effects: an open kind, a required summary

`Effect.kind` is an **open string** (grammar-constrained, but not a closed enum).
Handlers evolve faster than any schema, and an executor must be able to describe a
consequence the schema does not name.

`Effect.summary` is **REQUIRED**, human-facing, executor-authored, and is the one
member a surface is *obliged* to render. That is what makes the open kind safe: a
surface that meets an unrecognised kind still shows something truthful, rather
than silently dropping it — which would misinform the human exactly where the
design is weakest.

Two consequences worth stating:

- **`before`/`after` carry no null-vs-absent distinction.** JSON Schema and its
  code generators map an explicit `null` and an absent member to the same thing.
  The schema says so, rather than promising a distinction it cannot keep.
- **"No effects" and "effects unknown" MUST NOT render alike.** The wire does not
  distinguish them — an executor that determined a task is inert sends the same
  empty `effects[]` as one with no dry-run — so a surface treats *both* as
  "could not be determined" and says so. That is fail-safe: the ambiguous case is
  rendered as the more alarming one, never the more reassuring. It is also, today,
  the common case (§4.1).

## 10. The relay

```ts
window.vtaWallet.requestTask({ type, payload }): Promise<RequestTaskOutcome>
```

The page supplies a Type URI and a payload. **That is all it supplies.**

The device mints the envelope — `id`, `issuedAt`, `issuer`, `recipient` — inside
its own trust boundary, and stamps the runtime-attested origin.

> **A consent surface MUST NOT counter-sign an envelope authored by the relying
> party.**

Counter-signing means attesting to a document it did not write: the RP chooses the
issuer, the recipient, the expiry, the id — every field the executor subsequently
trusts *because the device signed it*. The device becomes a notary for claims it
never checked.

The origin rides in `payload.ext`, which puts it **inside the digest the approver
signs** — so the site shown to the human is bound to the payload that executes and
cannot be swapped afterwards. It is worth being honest about the ceiling: this
tells an approver which site asked *according to their own device*, which is a
weaker claim than a cryptographic binding to the site. It is still worth having,
because the alternative is telling them nothing.

A `requireConsent` refusal is **a result, not an error**. It arrives as a rejected
Trust Task, so every transport layer beneath the relay treats it as an error and
the natural thing is to let it propagate. That is a quiet catastrophe: the refusal
carries the signed consent requests an approver must render and the digest the
page must display. The relay recognises it by its extended error *code*
(`auth:consent_required`, not a `reason` member) and returns it as a discriminated
outcome the caller cannot reach the result of without handling. Getting this wrong
fails silently — the refusal just looks like an ordinary error and the flow dies
without a sound — which is exactly how it shipped once, and was fixed.

## 11. Origin provenance

An extension's `sender.id === runtime.id` check proves a message came from the
extension. It says nothing about which *page* it came from: every content script,
the popup, the options page, the consent window and the offscreen document all
pass it. A body-supplied origin is therefore a claim by an untrusted party about
itself.

The original design called the fix "one line: use `sender.origin`". **It is not
one line.** An extension-internal sender reports `chrome-extension://<id>` — a
perfectly real origin that sails through any check asking merely "is an origin
present?". A page-facing message must *also* have a tab behind it, and one that
does not must be refused.

And it must be wired to *every* page-facing method. It shipped applied to the
per-method surface but not to the generic relay — the one method whose origin ends
up inside a signed digest — so for a window the relay read its origin from the
message body after all. The set of page-facing methods is now derived from a
single source, and adding a method without registering it is a compile error. The
lesson: a security check that is a hand-maintained allowlist will drift off the
thing it is supposed to cover.

## 12. Origin trust is not capability trust

The sharpest lesson from building the relay.

Under a per-method surface, a "remember this site" grant meant *"this site may
call `vaultList()` and `proxyLogin()`"* — because those were the only things it
could call. Under a **generic** relay the identical grant means *"this site may
ask my agent to do anything at all"*, and a tick made once on a `vaultList()`
prompt would silently authorize a DID deactivation a year later.

**The set of things the grant covers grew, without the user ever being asked
about the new members.**

So the relay prompts on every call and offers no "remember", and the
holder-key-signing method (`signTrustTask`) — which signs an *arbitrary* envelope
— does the same, after a regression in which it briefly short-circuited on a
remembered origin. That regression is instructive: the ergonomic helper
(`gatedConsent`) that is right for a login is wrong for a method that signs
anything, and choosing it by reflex reintroduced the exact bug this section names.

This every-time prompt is a stopgap, not an answer. Enforcement defaults off, so
on a default deployment that prompt is the only thing between an arbitrary page
and an arbitrary task. The real answer is **scoped grants**: `(origin, subject,
typeGlob, expiry)`, rememberable only for tasks the executor classifies
`sideEffects: none`. Not yet built. It is the largest outstanding gap.

## 13. Consent fatigue

Designs like this die to habituation, not to cryptography.

- **The push follows the question, not the submit.** The reject is idempotent, so
  a requester can retry without invalidating an approval in flight. Push on every
  re-submit and that property becomes a weapon: an RP can ring an approver's phone
  as fast as it can retry a task it knows will be rejected. A push fires only when
  a pending request is newly raised. (Honest scope: suppression keys on the
  internal digest, so a payload varied by one byte gets a fresh push. It stops a
  *replay* flood, not a *distinct-payload* flood.)
- **De-duplicate replays before prompting.** A mediator re-delivers un-acked
  messages on every reconnect. Without this, one pending consent pops a fresh
  prompt each time — training the user to dismiss it.
- **Reset per-origin budgets on denial, not on approval.** (Not yet built.)
- **A destructive operation requires a match, not a tap.** A tap is a reflex; a
  comparison is an act of attention.
- **Silence is never assent.** A closed window, a timeout, a dropped message: all
  denials. The decision is an explicit enum on the wire rather than a boolean, so
  that a missing or falsy value cannot decode as approval.

## 14. Privacy

- **The digest was a correlator.** Salting with the per-request challenge closes
  it (§6.1) at zero cost. Both screens are shown the same executor-computed value
  (the approving device explicitly does *not* recompute it — it holds no payload),
  so the salt costs nothing in usability.
- **Contentless is not metadata-free.** `push/wake` carries no payload, which is
  right. The gateway nonetheless learns that *this handle* was woken at *this
  time*, at *this rate*; correlate against an RP's logs and you have a timing
  channel. Rotating wake handles per approver-set helps; batching and cover
  traffic are the real fixes and are correspondingly annoying.
- **Authentication identity ≠ subject identity.** The DID being *managed* is
  public — that is what `did:webvh` is for. The DID you *log in as* need not be
  reused across relying parties. Default to pairwise.
- **The audit log is a dossier.** A signed, payload-bound, timestamped record of
  every approval is exactly what an operator wants, exactly what makes a user
  coercible, and exactly what makes the audit store worth stealing. There is a
  genuine tension between the non-repudiation this design sells as a benefit and
  the user's interest in deniability. Minimum posture: encrypt at rest under a key
  requiring user unlock, cap retention, make it user-inspectable and
  user-deletable.
- **Bundle the registry.** Fetching payload schemas on demand tells the registry
  host which task types each user encounters. The reference implementation bundles
  its schemas (compiled in), which is the right posture; a fetch-at-decision-time
  consumer would leak.

## 15. Threat model

| Threat | Defence |
|---|---|
| Malicious / XSS'd RP page | Executor-side policy; consent rendered from executor-authored `effects[]`, never RP prose; digest binding |
| Compromised consent surface | `excludeRequester` approver sets; cross-device digest **matching**; policy + approver-set re-checked at execution |
| Payload swap after approval | `payloadDigest` echoed in the decision, re-derived at execution |
| Approval for a different task | Digest binds the **type URI** (§6.2) |
| Stale-state approval / lost update | State pin asserted at execution; stale pendings retired |
| Dry-run desynchronised from the run | Planner peeks; keys allocated as one pinned contiguous block, refused if the counter moved (§5) |
| Replay of a decision | Pending deleted on decision receipt; per-approver idempotency; single-use grant at execution |
| Revoked *device* with an in-flight approval | *Partial.* The named approver **set** is re-checked; device-enrolment revocation is not wired to it (§7, §16) |
| Registry compromise / version downgrade | Class from the compiled handler; registry never consumed |
| Confused deputy | Origin from the runtime's attestation, plus a tab assertion, on **every** page-facing method incl. the relay (§11) |
| Hidden payload fields | Closed schema, validated before diffing, **where a schema is published** (§6.4) |
| Digest as confirmation oracle | Wire digest salted with the challenge (§6.1) |
| Consent fatigue / prompt spam | Push follows the question; replays de-duplicated; match, not tap (§13) |
| Scope creep of a remembered grant | Relay prompts every time; scoped grants not built (§12) |
| **Task with no requireConsent policy** | *None.* Enforcement is opt-in and the default baseline allows everything (§3) |

## 16. What is not true yet

Stated plainly. Several of these were claimed as done by an earlier draft of this
note and were not; the note is only trustworthy if it lists them.

- **Enforcement is off by default** (§3). The invariant holds only where an
  operator has enabled the PDP and authored a `requireConsent` policy. The shipped
  baseline allows every task.
- **One handler has a dry-run** (§4.1). `webvh/dids/update`. Every other
  consent-gated task, including `vault/delete --force`, is approved against an
  empty `effects[]` rendered as "consequences unknown". Widening this is the main
  body of remaining work.
- **Approver sets are configuration, not enrolled-device records** (§7). Device
  revocation does not reach an in-flight approval unless an operator also edits the
  set. There is no approver-eligibility tier and no task surface to list or revoke
  approvers.
- **Scoped grants** (§12) — the relay's every-time prompt is correct and will
  fatigue people.
- **Budgets and escalating friction** (§13) are specified but not built.
- **Payload validation is best-effort by coverage** (§6.4). ~60 dispatched task
  URIs have no published schema and dispatch unvalidated by default.
- **Recovery is unsolved.** If the sole approver is lost, destructive operations
  become impossible — or a recovery path exists, and it is now the weakest link.
  M-of-N approver sets help. The rule to write down:

  > **The recovery path must satisfy at least the consent requirements of the
  > strongest operation it can recover.**

  Most systems quietly violate this. It is where they get broken.
- **Cryptographic agility.** Nothing in the *protocol* fixes a suite, but the
  reference implementation hardcodes Ed25519 / `eddsa-jcs-2022` and SHA-256 end to
  end. "Post-quantum update keys are a configuration choice" is true of the design
  and false of this implementation. State the aspiration; do not claim the
  capability.

## 17. What the reference implementation demonstrated

Two things worth keeping.

**The trust boundary was drawn in the right place.** A `did:webvh` hosting
service, asked to accept a LogEntry signed by a key it does not hold and submitted
by a session that does not control it, **needed no changes at all**. It verifies
the proof chain on whatever log it is handed, and it never cared who submitted.
The work was in not crossing that boundary.

**The claim is now tested, not asserted.** An end-to-end test drives a real update
over real HTTP: propose, dry-run, refuse, sign a decision, re-submit, execute, and
then *read the DID log* to confirm the update keys in force are the ones the
approver was shown. "What the human approves is what executes" is checked against
the committed chain, not against a mock. Writing that test found three integration
bugs no unit test could — which is the note's final lesson: **for a system whose
correctness is a property of how components meet over a wire, the only tests that
count are the ones that use the wire.**
