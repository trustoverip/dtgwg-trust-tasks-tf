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
  digest: string;
  retainUntil: number | undefined;
  response: unknown;
  completed: boolean;
}

/**
 * A bounded, in-process {@link ReplayGuard}: an LRU map from `id` to the
 * digest accepted under it, its retention deadline, and the response it
 * produced.
 *
 * **Suitable when** one process is the sole consumer for the `recipient` VID
 * it serves and losing the record on restart is acceptable.
 *
 * **Not suitable when** the consumer is replicated: two replicas hold separate
 * maps, so a document accepted by replica A is `fresh` at replica B and the
 * effect happens twice — the exact failure item 11 exists to prevent.
 * Replicated deployments MUST back the guard with a shared store.
 *
 * Eviction is by capacity as well as by `retainUntil`: a burst of distinct
 * documents can push an older record out before its deadline, and a replay
 * arriving after that would be accepted. Size the capacity above the number of
 * distinct documents the widest acceptance window can hold.
 *
 * `Map` preserves insertion order, which is what makes the LRU a delete-then-
 * reinsert rather than a second index.
 */
export class InMemoryReplayGuard implements ReplayGuard {
  readonly #entries = new Map<string, Entry>();
  readonly #capacity: number;

  /**
   * @param capacity Maximum records retained. Must be positive: a guard that
   * retains nothing answers `fresh` to everything, which is a silent total
   * defeat of item 11 rather than a visible misconfiguration.
   */
  constructor(capacity = 10_000) {
    if (!Number.isInteger(capacity) || capacity < 1) {
      throw new RangeError("InMemoryReplayGuard capacity must be a positive integer");
    }
    this.#capacity = capacity;
  }

  /** Number of records currently retained. For tests and metrics. */
  get size(): number {
    return this.#entries.size;
  }

  /** Drop every record whose `retainUntil` has passed. */
  purgeExpired(now: number): void {
    for (const [id, entry] of this.#entries) {
      if (entry.retainUntil !== undefined && entry.retainUntil <= now) this.#entries.delete(id);
    }
  }

  claim(id: string, digest: string, retainUntil: number | undefined, now: number): ReplayVerdict {
    const existing = this.#entries.get(id);

    // An expired record is treated as absent: the consumer would refuse the
    // document under §7.2 item 4 or the acceptance window anyway, so holding
    // the key would only manufacture a permanent `idConflict` for an `id`
    // nobody can use.
    if (existing !== undefined && existing.retainUntil !== undefined && existing.retainUntil <= now) {
      this.#entries.delete(id);
    } else if (existing !== undefined) {
      if (existing.digest !== digest) {
        // A conflicting document is not a *use* of the record, so it does not
        // refresh recency — otherwise a flood of conflicts could pin an entry
        // and evict live ones.
        return { kind: "conflict" };
      }
      this.#entries.delete(id);
      this.#entries.set(id, existing);
      return existing.response === undefined
        ? { kind: "duplicate", inFlight: !existing.completed }
        : { kind: "duplicate", priorResponse: existing.response, inFlight: !existing.completed };
    }

    this.#entries.set(id, { digest, retainUntil, response: undefined, completed: false });
    while (this.#entries.size > this.#capacity) {
      const oldest = this.#entries.keys().next();
      if (oldest.done === true) break;
      this.#entries.delete(oldest.value);
    }
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
      this.#entries.delete(id);
    }
  }
}
