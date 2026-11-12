/**
 * Duplicate-execution protection — SPEC §7.2 item 11, §8.4, §10.1.
 *
 * Mirrors `replay.rs` in trust-tasks-rs, check for check.
 *
 * # The rule
 *
 * §7.2 item 11 is normative and unconditional for a *consequential Trust
 * Task*: once a consumer has accepted a document with a given `id` for
 * execution, receiving that same document again **MUST NOT** cause the
 * consequential effect a second time, and receiving a *different* document
 * under the same `id` **MUST** be rejected with `idConflict`.
 *
 * §8.4 is the same mechanism from the producer's end: a retry is a bit-for-bit
 * identical resend, and it is safe *precisely because* item 11 obliges the
 * consumer to absorb it. Every transport binding in this repo delegates replay
 * defence to the consumer — `bindings/https/0.2` §5 says "Freshness / replay:
 * None", and the DIDComm and TSP bindings say the same — so if the consumer
 * does not do it, nobody does, and an ordinary mediator redelivery grants an
 * ACL entry twice by accident. §10.1: "The rule deliberately does not
 * distinguish a hostile replay from a legitimate transport retry, because at
 * the document layer the two are indistinguishable."
 *
 * # The key
 *
 * The `id` **alone** — "Transport request identifiers, transport message
 * identifiers, and execution handles **MUST NOT** substitute" — plus a digest
 * of the canonical serialization, because "an `id` alone cannot distinguish
 * the retry it must absorb from the conflict it must reject".
 *
 * {@link documentDigest} hashes {@link canonicalJson} of the whole document
 * rather than the octets as received: a re-indented body or a member order
 * chosen by an intermediary would otherwise make a legitimate §8.4 retry look
 * like a *different* document.
 *
 * The digest covers the **entire** document, `proof` included. That differs
 * deliberately from the §4.9.3 *task digest*, computed over
 * `JCS(document ∖ proof)`, and the spec spells the distinction out: "Item 11
 * and §8.4 ask *which serialization arrived*, so a re-signed `proof` over
 * identical content makes a different document — that is the `idConflict`
 * case, and the distinction is the whole point of the rule."
 */

import { canonicalJson, sha256Hex } from "./canonical.js";
import type { TrustTaskDocument } from "./document.js";

/**
 * The content identity of a document, per SPEC §7.2's keying paragraph:
 * SHA-256 over the canonical serialization of the whole document, as hex.
 *
 * Consumer-local. Never on the wire, and **not** the §4.9.3 task digest.
 */
export function documentDigest<P>(doc: TrustTaskDocument<P>): string {
  return sha256Hex(canonicalJson(doc));
}

/** What a {@link ReplayGuard} says about a document offered for execution. */
export type ReplayVerdict =
  /** Not seen before. The caller may execute. */
  | { kind: "fresh" }
  /**
   * Already accepted under the *same* digest — a §8.4 retry, or a replay. The
   * caller **MUST NOT** execute again.
   */
  | {
      kind: "duplicate";
      /** The response the first execution produced, where one was retained. */
      priorResponse?: unknown;
      /** Whether that first execution is still running. */
      inFlight: boolean;
    }
  /**
   * Already accepted under a *different* digest. §7.2 item 11 requires
   * `idConflict`, and requires that this not be treated as a retry.
   */
  | { kind: "conflict" };

/**
 * The consumer-side record that makes SPEC §7.2 item 11 true.
 *
 * An interface, so a deployment can back it with Redis, Postgres, or anything
 * that survives a process restart. {@link InMemoryReplayGuard} is the default
 * and is correct for a single-process consumer; it is **not** correct behind a
 * load balancer, where two replicas would each accept the same document once.
 */
export interface ReplayGuard {
  /**
   * Claim `id` for execution on behalf of a document with identity `digest`.
   *
   * `retainUntil` is the instant past which the record may be dropped —
   * {@link recordExpiry}, which SPEC §7.2 makes the same instant as the end of
   * the consumer's willingness to execute the document. An implementation
   * SHOULD treat a record whose `retainUntil` has passed as absent, so the key
   * is released rather than conflicting forever with a document nobody would
   * execute.
   *
   * Implementations **MUST** make claim-and-record atomic with respect to
   * concurrent calls: two simultaneous deliveries of the same document must
   * not both receive `fresh`. That is the whole guarantee.
   *
   * Throwing means the record could not be consulted. {@link consumeInbound}
   * fails closed on that, mapping it to `unavailable` with `retryable: true` —
   * a consumer that cannot establish whether a document is a duplicate has not
   * satisfied item 11, and executing anyway is the double execution the rule
   * forbids.
   */
  claim(
    id: string,
    digest: string,
    retainUntil: number | undefined,
    now: number,
  ): Promise<ReplayVerdict> | ReplayVerdict;

  /**
   * {@link claim}, told which party the document is attributed to: the issuer
   * SPEC §4.8.1 resolved for it, `undefined` when there is none.
   *
   * Optional. `consumeInbound` calls it in place of `claim` where present, so a
   * guard can share its capacity out between parties —
   * {@link InMemoryReplayGuard} does, so that one party cannot hold every slot.
   */
  claimFor?(
    issuer: string | undefined,
    id: string,
    digest: string,
    retainUntil: number | undefined,
    now: number,
  ): Promise<ReplayVerdict> | ReplayVerdict;

  /**
   * Attach the response a completed execution produced, so a later duplicate
   * can be answered with it per §7.2 (*Disposition of a duplicate*) rather
   * than merely absorbed in silence.
   *
   * Optional: a guard that records nothing still satisfies item 11 — the
   * effect does not happen twice — and is the right shape for a
   * fire-and-forget specification, which has no response to return.
   */
  recordResponse?(id: string, response: unknown): Promise<void> | void;

  /**
   * Release a claim whose execution is not to stand — for example a refusal
   * the consumer marked `retryable`, where §8.4 has just invited the producer
   * to re-send the same bytes. Without this, that invited retry would come
   * back as an absorbed duplicate carrying the same failure forever.
   */
  release?(id: string, digest: string): Promise<void> | void;
}

/** How a consumer applies SPEC §7.2 item 11 in {@link consumeInbound}. */
export type ReplayPolicy =
  /** Apply item 11 using this guard. Correct for any consequential spec. */
  | { kind: "guard"; guard: ReplayGuard }
  /**
   * Keep no duplicate-execution record. Conformant **only** where the task is
   * not consequential (§2), or where the specification "explicitly declares
   * repeated execution safe and intended" — a property of the operation, not
   * of the consumer's convenience.
   */
  | { kind: "notConsequential" };

interface Entry {
  issuer: string | undefined;
  digest: string;
  retainUntil: number | undefined;
  response: unknown;
  completed: boolean;
}

/**
 * The capacity of `new InMemoryReplayGuard()`: 100 000 live records.
 *
 * Enough for the consequential policy's five-minute acceptance window at ~330
 * distinct documents per second. Documents carrying an `expiresAt` hold their
 * record until then (up to the expiry horizon, 30 days by default), so a
 * consumer accepting them should size for that window instead. Mirrors Rust's
 * `DEFAULT_REPLAY_CAPACITY`.
 */
export const DEFAULT_REPLAY_CAPACITY = 100_000;

/**
 * Thrown by {@link InMemoryReplayGuard.claim} when every slot holds a live
 * record, or (`issuerShare`) when the document's issuer holds its share.
 */
export class ReplayGuardFullError extends Error {
  readonly issuerShare: boolean;
  constructor(size: number, issuerShare = false) {
    super(
      issuerShare
        ? `in-memory replay guard: this issuer holds its share of ${size} records`
        : `in-memory replay guard is full: ${size} records are inside their retention window`,
    );
    this.name = "ReplayGuardFullError";
    this.issuerShare = issuerShare;
  }
}

/**
 * A bounded, in-process {@link ReplayGuard}: a map from `id` to the digest
 * accepted under it, its retention deadline, and the response it produced.
 *
 * **Suitable when** one process is the sole consumer for the `recipient` VID
 * it serves and losing the record on restart is acceptable.
 *
 * **Not suitable when** the consumer is replicated: two replicas hold separate
 * maps, so a document accepted by replica A is `fresh` at replica B and the
 * effect happens twice — the exact failure item 11 exists to prevent.
 * Replicated deployments MUST back the guard with a shared store.
 *
 * **When it is full.** A record is dropped only once its `retainUntil` has
 * passed — never to make room. When a new `id` arrives and every slot holds a
 * record still inside its retention window, `claim` throws
 * {@link ReplayGuardFullError}, which `consumeInbound` answers `unavailable`
 * (retryable): the guard refuses to execute a document it could not then
 * remember, rather than forgetting one it promised to. Records whose window
 * has closed are reclaimed first. The capacity is therefore a ceiling on
 * distinct documents *per retention window*; see
 * {@link DEFAULT_REPLAY_CAPACITY} for sizing. A document carrying an
 * `expiresAt` but no `issuedAt` is not bounded by the expiry horizon (it runs
 * from `issuedAt`), so under a policy that does not require `issuedAt` its
 * record lives until its `expiresAt`.
 *
 * **One party's share.** So that one party cannot fill the guard and hold
 * every slot for the length of the expiry horizon, no issuer may hold more than
 * a share of the capacity — a tenth by default, rounded up (the second
 * constructor argument changes it). A party at its share is refused
 * `unavailable` for its *own* new documents; others are unaffected. The issuer
 * is the one `consumeInbound` resolved (SPEC §4.8.1) and passes to
 * {@link claimFor}; documents with no resolved issuer, and direct calls to
 * {@link claim}, share one allowance. The share is per identifier: where anyone
 * can mint identifiers, a party using many is limited only by the capacity.
 *
 * **Memory.** At most `capacity` records, each its `id`, a digest and the
 * response cached for duplicates: roughly `capacity` × (largest `id` + largest
 * response) at worst.
 */
export class InMemoryReplayGuard implements ReplayGuard {
  readonly #entries = new Map<string, Entry>();
  readonly #capacity: number;
  readonly #perIssuer: number;
  /** Live records per issuer (`undefined`: no resolved issuer). */
  readonly #held = new Map<string | undefined, number>();
  /**
   * No record's `retainUntil` is earlier than this (it may be earlier than every
   * record's, after a removal). Lets a full guard skip the reclaiming scan when
   * nothing can have expired yet.
   */
  #earliestExpiry = Number.POSITIVE_INFINITY;

  /**
   * @param capacity Maximum live records retained. Must be positive: a guard
   * that retains nothing answers `fresh` to everything, which is a silent total
   * defeat of item 11 rather than a visible misconfiguration.
   * @param perIssuerLimit Most live records one issuer may hold; defaults to a
   * tenth of `capacity`, rounded up, and is clamped to `1..capacity`.
   */
  constructor(capacity = DEFAULT_REPLAY_CAPACITY, perIssuerLimit?: number) {
    if (!Number.isInteger(capacity) || capacity < 1) {
      throw new RangeError("InMemoryReplayGuard capacity must be a positive integer");
    }
    this.#capacity = capacity;
    const limit = perIssuerLimit ?? Math.ceil(capacity / 10);
    this.#perIssuer = Math.min(capacity, Math.max(1, Math.floor(limit)));
  }

  #delete(id: string): void {
    const entry = this.#entries.get(id);
    if (entry === undefined) return;
    this.#entries.delete(id);
    const held = (this.#held.get(entry.issuer) ?? 1) - 1;
    if (held <= 0) this.#held.delete(entry.issuer);
    else this.#held.set(entry.issuer, held);
  }

  /** Number of records currently retained. For tests and metrics. */
  get size(): number {
    return this.#entries.size;
  }

  /** Drop every record whose `retainUntil` has passed. */
  purgeExpired(now: number): void {
    let earliest = Number.POSITIVE_INFINITY;
    for (const [id, entry] of this.#entries) {
      if (entry.retainUntil === undefined) continue;
      if (entry.retainUntil <= now) this.#delete(id);
      else if (entry.retainUntil < earliest) earliest = entry.retainUntil;
    }
    this.#earliestExpiry = earliest;
  }

  claim(id: string, digest: string, retainUntil: number | undefined, now: number): ReplayVerdict {
    return this.claimFor(undefined, id, digest, retainUntil, now);
  }

  claimFor(
    issuer: string | undefined,
    id: string,
    digest: string,
    retainUntil: number | undefined,
    now: number,
  ): ReplayVerdict {
    const existing = this.#entries.get(id);

    // An expired record is treated as absent: the consumer would refuse the
    // document under §7.2 item 4 or the acceptance window anyway, so holding
    // the key would only manufacture a permanent `idConflict` for an `id`
    // nobody can use.
    if (existing !== undefined && existing.retainUntil !== undefined && existing.retainUntil <= now) {
      this.#delete(id);
    } else if (existing !== undefined) {
      if (existing.digest !== digest) return { kind: "conflict" };
      return existing.response === undefined
        ? { kind: "duplicate", inFlight: !existing.completed }
        : { kind: "duplicate", priorResponse: existing.response, inFlight: !existing.completed };
    }

    // Full: reclaim the records whose window has closed, and if every slot
    // still holds a live record, refuse rather than drop one. A record dropped
    // inside its window is a replay accepted later.
    // Likewise for one issuer's share, so that one party cannot hold every slot
    // and refuse everyone else.
    const held = () => this.#held.get(issuer) ?? 0;
    if (this.#entries.size >= this.#capacity || held() >= this.#perIssuer) {
      if (this.#earliestExpiry <= now) this.purgeExpired(now);
      if (this.#entries.size >= this.#capacity) throw new ReplayGuardFullError(this.#entries.size);
      if (held() >= this.#perIssuer) throw new ReplayGuardFullError(held(), true);
    }

    this.#entries.set(id, { issuer, digest, retainUntil, response: undefined, completed: false });
    this.#held.set(issuer, held() + 1);
    if (retainUntil !== undefined && retainUntil < this.#earliestExpiry) this.#earliestExpiry = retainUntil;
    return { kind: "fresh" };
  }

  recordResponse(id: string, response: unknown): void {
    const entry = this.#entries.get(id);
    if (entry === undefined) return;
    entry.response = response;
    entry.completed = true;
  }

  release(id: string, digest: string): void {
    const entry = this.#entries.get(id);
    // Only release the claim this digest made, and only while it is
    // unfinished: a concurrent arrival that legitimately holds the key must
    // not have it taken away by another document's cleanup.
    if (entry !== undefined && entry.digest === digest && !entry.completed) {
      this.#delete(id);
    }
  }
}
