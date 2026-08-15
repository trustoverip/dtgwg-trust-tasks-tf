# Design Note: Task Control and Corrigibility

| | |
|---|---|
| **Status** | Proposed — decisions recorded, no normative text written yet |
| **Date** | 2026-08-15 |
| **Applies to** | Any *producer* that needs previously requested work to stop, and any *consumer* that executes work which outlives the exchange that requested it |
| **Related** | [#204](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/204), SPEC §7.2 items 10–12, §8, §4.11.4, §6.1, `specs/trust-task-next-step/0.1` |

*This note is non-normative. It records the decisions taken for a task-control
mechanism and the reasoning behind each, so that the normative text can be
written — and reviewed — against a settled design rather than an argument
conducted inside a diff.*

*It is written **before** implementation, which is the opposite of
[`delegated-trust-task-execution.md`](delegated-trust-task-execution.md). That
is deliberate: the mechanism has more open choices than it has hard
constraints, and the cheapest place to be wrong is here.*

## 1. Problem

The framework can express what should happen next. It cannot express that
previously requested work should stop.

§4.9.2 gives correlation, §4.11 gives ceremonies, §8.6's `trust-task-next-step`
lets a blocked task stay open. None of them lets a *producer* withdraw a request
it has already made. For short synchronous exchanges that gap does not matter —
the work is done before anyone could change their mind. For long-running,
asynchronous, or agentic execution it is a **corrigibility** gap: an agent can
be told to start and has no defined way to be told to stop.

Transport-level cancellation is not a substitute. Dropping an HTTP request,
cancelling a queue delivery, or closing a session terminates *that delivery*. It
says nothing about a document that has already been accepted, queued, forwarded,
or handed to a worker — and a *consumer* that treats a dropped connection as a
withdrawal will stop work the *producer* still wants.

## 2. What is already solved, and by what

The single most useful finding in this design is how little of it is new. Three
rules merged during the #202–#206 batch do most of the work:

| Concern | Already handled by |
|---|---|
| The race between a control operation and an irreversible effect | **§7.2 item 12** — re-evaluate every required condition immediately before each irreversible effect |
| Reporting that effects landed before the stop took hold | **§7.2 item 12**'s partial-execution disposition |
| A cancelled task being re-delivered and re-executed | **§7.2 item 11** — duplicate-execution protection keyed on `id` |
| Who may authorize the operation | **§7.2 item 10** — identity and proof are not authorization |

**Item 12 is the important one.** It already requires a *consumer* to
re-evaluate, immediately before each irreversible or externally visible effect,
every condition its policy and the *Trust Task specification* require. A
received cancellation **is such a condition**. So the mechanism is:

> the control document arrives → the *consumer* records it → at the next item 12
> checkpoint the condition "not cancelled" fails → the *consumer* **MUST NOT**
> perform the subsequent effect → it reports partial execution distinguishably,
> as item 12 already requires

There is no new race protocol to design. The checkpoint exists, it is normative,
and it is already placed where it needs to be. What #204 adds is a document
type, an authorization rule, and a set of dispositions — not a distributed
algorithm.

## 3. The two directions are not symmetric

An early instinct is to treat "producer cancels" and "recipient cancels" as one
mechanism with two senders. They are not, and conflating them would invent a
second document type for no gain.

**Producer → consumer** is genuinely new: *stop the work I asked for*. This is
what the control document is for.

**Consumer → producer** is already specified. A *consumer* that stops on its own
initiative — operator intervention, policy, capacity, a compliance hold — is
refusing or abandoning, which §8 error responses already cover, and §7.2 item 12
already tells it to report partial execution distinguishably. It needs no
control document, only a code that says what happened (§7).

This asymmetry is also what makes §5's authorization rule tolerable. A recipient
operator never needs to *send* a cancellation, so restricting who may send one
does not disenfranchise them.

## 4. Scope

**In:** `cancel`, `suspend`, `resume`.

**Deferred:** `supersede`. It is substantially cancel-plus-a-new-document, and
it overlaps `threadId` correlation and `trust-task-next-step`. Nothing is lost
by waiting for a flow that actually needs it, and specifying it now would mean
guessing at semantics no implementation is asking for.

The document is therefore `trust-task-control` with an `operation` member rather
than a narrow `trust-task-cancel`, so the third and any later operation arrive
without a new slug.

**Note the versioning wrinkle.** `operation` is a *discriminating* field, so
adding a value is not the clean `MINOR` that §5.2 grants enum additions on
non-discriminating fields. A *consumer* at the older version fails validation
rather than misbehaving, which is a safe failure — but it is a failure, and the
version arithmetic should be decided deliberately when `supersede` lands.

## 5. Authorization: only the initiator

**Decision: only the original initiator may cancel, suspend, or resume a task.**

The control document's `issuer` **MUST** be the same party as the target
document's `issuer`. Where the target carried no in-band `issuer`, "same party"
means the same authenticated identity under §4.8.1's precedence.

This is stricter than the framework's usual posture — §7.2 item 10 leaves
authorization to the *consumer*'s policy — and the strictness is affordable
precisely because of §3: an operator who wants work stopped does not send a
control document, it stops and emits `cancelled`. The two paths cover the two
parties, and neither needs to reach into the other's.

Two consequences fall out for free:

* **Ceremony membership authorizes nothing.** §4.11.4 already says so, so being
  a step of the same enactment does not let one party cancel another's task.
* **`proof` is REQUIRED on a control document**, with audience binding. A forged
  cancellation is a denial-of-service against someone else's work, and the
  document is worthless as evidence of withdrawal if it cannot be attributed.

## 6. Identifying the target

The control document **MUST** name the target document's `id`. It **SHOULD**
also carry the target's `type`, and **SHOULD** carry the same `threadId` as the
target so the two correlate in one exchange.

`threadId`, `parentThreadId`, and `ceremony` membership **MUST NOT** identify
the target on their own. More than one *Trust Task document* can exist in a
single exchange or enactment, and a control operation that names only the
exchange is ambiguous exactly when it matters — in a flow busy enough to have
several tasks in flight.

Carrying the `type` is defence in depth: it lets a *consumer* detect a control
document aimed at an `id` it holds under a different specification, rather than
acting on a coincidence.

## 7. Cancellation does not roll anything back

**Decision: cancellation prevents future effects. It never undoes past ones.**

This was the most consequential decision taken, and it was taken against the
intuition that a cancelled task should leave no trace. Three reasons:

**The framework's own taxonomy forbids it.** §7.3 item 13 defines `destructive`
as "irreversible or authority-shifting." A rollback requirement would oblige
*consumers* to undo effects the framework has already classified as impossible
to undo.

**Retained rollback state is frequently a security defect.** To roll back a key
rotation, a *consumer* would have to retain the superseded private key — which
defeats the rotation. To roll back a disclosure it would have to un-know a
secret. The state needed to reverse a `destructive` task is often precisely the
material the task existed to destroy.

**Compensation is task-specific and is itself a Trust Task.** The undo for an
issuance is a revocation, with its own authority requirements, its own audit
trail, and its own party. The framework cannot define that generically, and
should not pretend to.

The registry reached this conclusion before the framework did. Two specs already
carry the formula:

> "Not compensatable by this exchange; revocation is the issuer's own act."
> — `vrc/relationships/issue/0.1`
>
> "Not compensatable by this exchange; revocation is the witness's own act."
> — `witness/session/submit/0.1`

**What the framework owes instead is information.** The cancel response
**MUST** report which effects were created before the stop took hold, so the
*producer* can decide whether to invoke a compensating task. A *producer* that
knows only "cancelled" and not "cancelled, having already issued the credential"
cannot act correctly.

A machine-readable `sideEffects.compensatedBy` naming the compensating task was
considered and **deferred**. #218 established what a new front-matter field
costs: a `spec.meta.schema.json` change, a build-validation change, and an audit
across the whole registry. The prose convention above already exists in two
specs; promote it to a field when an agent actually needs to read it.

## 8. Dispositions: three outcomes, and who decided

A cancel has three possible outcomes and they are **not** interchangeable:

1. **Nothing landed.** The task stopped before any irreversible or externally
   visible effect. A clean cancellation.
2. **Something landed.** Effects occurred before the stop took hold. The
   *consumer* **MUST NOT** report this as a clean cancellation; the response
   names what was created.
3. **Already completed.** The task finished before the control document
   arrived. Not a cancellation at all — the *producer* is told it completed,
   and compensation, if it wants any, is its own act (§7).

**The control document gets a real response.** This is the one place the
"fire-and-forget" posture of §9 does not apply, and the reason is outcome 2:
what landed is exactly the information that determines whether the *producer*
must compensate. A fire-and-forget cancellation withholds the only fact that
makes the mechanism useful.

**Who decided must always be recoverable.** A `#response` to a control document
means *you asked, and here is what happened*. The `cancelled` error code (§10)
means *I stopped, on my own initiative*. If both used the same signal, no party
— and no auditor reading the retained documents afterwards — could tell a
withdrawal from a refusal. That distinction also drives behaviour: a producer
whose own cancellation succeeded should not retry, while a producer whose task
was stopped by the recipient may well want to.

## 9. Suspension, resumption, and expiry

**A suspended task is not "under way."** It returns to a pre-execution state,
and this single definition resolves the interaction with `expiresAt` without any
new rule:

* §7.2 item 4 forbids *beginning* work on an expired document.
* Resuming is beginning work.
* Therefore a suspension that outlives `expiresAt` cannot be resumed, and the
  *producer* issues a fresh document if it still wants the work.

That is the desired behaviour, obtained for free, and — importantly — **without
contradicting §7.2 item 12**, which forbids abandoning execution merely because
`expiresAt` passed. Execution under way is protected; a suspension is not
execution under way.

**A suspension carries no producer-set timer.** A "resume automatically after X"
field was considered and rejected twice over. It repeats the mistake §7.2 item
12 was written to avoid — a *producer* setting a deadline it cannot calibrate,
because it does not know how long the *consumer*'s work takes — and it is a
resource-exhaustion vector: a party that can pin *consumer* state for an
interval of its own choosing can pin a great deal of it.

**The *consumer* bounds its own retention.** Explicit-resume-only does not
remove the exhaustion risk, it relocates it: a suspension held "until resume" is
held forever if resume never comes. So *consumer* policy caps how long a
suspension is retained, with `expiresAt` bounding it where present. The
*producer* proposes; the *consumer* disposes.

The recipient-side "not now, come back later" cases are **already** covered and
should not be rebuilt here: `unavailable` with `retryAfter` closes the task and
invites a resend, and `trust-task-next-step` leaves it open and blocked.
`retryAfter` is an RFC 3339 *instant*, not a duration — which is the right
choice on store-and-forward transports, where a duration is ambiguous about when
its countdown began.

## 10. The `cancelled` error code

**Decision: a new standard code, `cancelled`**, for a *consumer* that stopped a
task on its own initiative.

Named for what happened, not for who caused it. An operator is only one of
several reasons — policy, capacity, a compliance hold, a business rule — and
`operatorCancelled` would have us minting `policyCancelled` next. The reason
goes in the payload, not the code.

* **`message`** carries free text for a human operator.
* **`details`** carries the machine-readable `reason`, **and** whether effects
  had already landed — the same fact outcome 2 of §8 requires, defined once.
* Default `retryable` is `false`. A deliberate stop is not a transient fault,
  and §11's rule that a cancelled task is terminal makes a retry meaningless.

**This is cheaper than it would have been a week ago.** `StandardCode` became
`#[non_exhaustive]` in trust-tasks-rs 0.7.0 (#223), so adding a standard code is
no longer a breaking change for downstream `match` expressions. It still needs a
`trust-task-error/0.5` — the code enum lives in that payload schema, so a
document carrying a code the declared version does not list will not validate —
and both SDKs pointed at it. That is the cheap half of what #223 cost.

## 11. Cancellation is terminal

**Decision: a cancelled task cannot be resumed, retried, or re-cancelled.** If
the work is still wanted, the *producer* issues a **new** *Trust Task document*
with a fresh `id`.

This aligns with §8.4, which already draws the same line for error responses: a
retry is the bit-for-bit identical document, and anything else is a new task.
Reviving a cancelled task would give a document two contradictory lifecycle
states and leave every auditor of the retained documents guessing which won.

**The item 11 record survives cancellation.** A cancelled task keeps its
duplicate-execution entry for the remainder of the *consumer*'s acceptance
window, so that a re-delivery of the original document after cancellation is
absorbed rather than executed. This is reuse, not new machinery: the *consumer*
is already retaining a per-`id` record with a digest for exactly that window.

## 12. Ordering and out-of-order arrival

A control document can arrive **before** the task it names. On asynchronous and
store-and-forward transports this is ordinary, not exceptional.

**Decision: the *consumer* records a tombstone**, and a later-arriving document
whose `id` matches is refused rather than executed. This reuses §7.2 item 11's
per-`id` record, so the storage obligation is one the *consumer* already carries,
and it expires on the same acceptance-window bound (§7.2 item 11's "bounding the
record").

A control document naming an `id` the *consumer* has never seen and whose window
has since lapsed has nothing to match, and is reported as such.

## 13. Notifications are a courtesy, and silence means nothing

Both parties emit when a task is cancelled or lapses, so the other side can
clean up. These notifications are **fire-and-forget**: no response is expected,
and none is required.

The rule that makes them safe to specify:

> **Neither party may infer the state of a task from the absence of a
> notification.**

A notification may be lost, dropped by an intermediary, or never sent by a
*consumer* that does not implement the mechanism. A *producer* that reads
silence as "still running" waits forever; one that reads it as "safely dead" and
reissues can cause the second consequential effect §7.2 item 11 exists to
prevent. Silence carries no information, and the specification must say so
rather than leave implementers to discover it.

This is also why §8's cancel response is exempt: it is a reply to a request, not
an unsolicited courtesy, and its content is load-bearing.

## 14. Support is optional, and that is a real limit

A *consumer* that does not implement task control returns `unsupportedType`
against the control document. Cancellation is therefore **best-effort**, and a
*producer* cannot rely on it.

A per-specification "is this cancellable" declaration was considered and
**rejected for v1**. Discovery (§11) already lets a *consumer* advertise the
*Type URIs* it dispatches, and `trust-task-control` is simply one more; adding a
§7.3 declaration would duplicate that at greater cost, in the same way §7.3 item
15 costs review attention no build check can supply.

## 15. What this note does not settle

* **The payload shape.** Sketched by §6 and §8 but not specified; the registry
  entry does that, on the `trust-task-next-step/0.1` model.
* **The response type.** Whether the control document defines its own
  `#response` or waits for `trust-task-ok`, which remains reserved and
  unspecified (§8.6). Recommendation: define its own, and do not block on a
  reserved slug nobody has scheduled.
* **`supersede`,** deferred entirely (§4).
* **Whether `operation` values beyond `cancel`/`suspend`/`resume` justify a
  `MAJOR`** given the discriminator problem in §4.
* **Ceremony-level control** — cancelling an *enactment* rather than a task.
  Out of scope, and probably a ceremony-layer concern rather than a task-layer
  one.

## 16. Where this design may be wrong

*Recorded now, so a later reader can check these rather than rediscover them.*

**"Only the initiator" may prove too strict.** It rests on §3's claim that a
recipient operator never needs to send a control document. That holds while the
two parties are the only participants. It is less obviously true where a
*consumer* executes on behalf of a third party — a mandate holder, a supervising
principal — who may have a legitimate interest in stopping work it did not
itself initiate. §7.2 item 10 would let a *consumer* honour such a party as a
matter of policy; this note forecloses that at the framework level. If that
proves wrong, the fix is to relax the rule to a floor ("the initiator is
authorized by default") rather than to redesign anything.

**Suspension may not earn its place.** Cancel has a clear safety story. Suspend
and resume bring a state machine, a retention obligation, and an exhaustion
vector that §9 bounds by policy rather than by construction. If no flow adopts
them, they are surface we did not need.

**The tombstone conflates two states.** "Cancelled before arrival" and
"cancelled after acceptance" both occupy the item 11 record, and a *consumer*
that does not distinguish them will report the same disposition for a task it
never saw and one it stopped. The dispositions of §8 assume it can tell.
