/**
 * Inbound-document orchestration for SPEC.md §7.2 items 4–8.
 *
 * Hand-written. Mirrors `consume.rs` in trust-tasks-rs, deliberately closely:
 * a TypeScript consumer and a Rust one must reach the same verdict on the same
 * document, or the two reference implementations disagree about what conforms.
 *
 * ```ts
 * import { consumeInbound, StaticTransport, AclGrant_v0_1 } from "@openvtc/trust-tasks";
 *
 * const outcome = await consumeInbound({
 *   transport,
 *   spec: AclGrant_v0_1.SPEC,
 *   proofPolicy: { kind: "verify", verify: myVerifier },
 *   doc,
 *   myVid: "did:web:maintainer.example",
 *   now: Date.now(),
 *   newErrorId: () => crypto.randomUUID(),
 *   handler: async (accepted, parties) =>
 *     respondWith(accepted, crypto.randomUUID(), buildResponse(parties)),
 * });
 *
 * switch (outcome.kind) {
 *   case "handled":    emit(outcome.response); break;
 *   case "rejected":   emit(outcome.error); break;
 *   case "suppressed": logSuppressed(); break;
 * }
 * ```
 *
 * Items 1 (framework schema), 2 (payload schema) and 3 (unknown `type`) are
 * *not* attempted here — they belong to the caller's parse and dispatch, and by
 * the time you hold a typed document they have already succeeded.
 */

import {
  enforceSpecPolicy,
  rejectWith,
  toErrorPayload,
  validateBasic,
  type ErrorResponse,
  type RejectReason,
  type SpecPolicy,
  type TrustTaskDocument,
} from "./document.js";
import {
  identityMismatchReason,
  reject,
  resolveParties,
  type ResolvedParties,
  type TransportHandler,
} from "./transport.js";

/** Verifies a document's Data Integrity proof (SPEC §4.7). */
export interface ProofVerifier {
  /** Resolve to `true` when the proof verifies against the in-band `issuer`. */
  verify<P>(doc: TrustTaskDocument<P>): Promise<boolean> | boolean;
}

/**
 * How {@link consumeInbound} treats a document's `proof` member (§7.2 item 7).
 *
 * The framework does not assume what integrity guarantees a consumer relies on.
 * Some verify Data Integrity proofs in-band; some have transport-layer
 * integrity (signed DIDComm, mTLS-bound HTTPS) and accept in-band proofs only
 * opportunistically; some have none. Making the choice explicit at the call
 * site is the point.
 *
 * `spec.isProofRequired` is consulted independently of the policy: a
 * specification that requires a proof rejects a proofless document whichever
 * policy is chosen.
 */
export type ProofPolicy =
  /**
   * Verify the proof when present. Failures map to `proofInvalid`. The safe
   * default for any consumer that honours in-band proofs.
   */
  | { kind: "verify"; verify: ProofVerifier }
  /**
   * Reject a document that carries an in-band proof, with `malformedRequest`.
   * For consumers with integrity from another layer that are deliberately not
   * verifying in-band proofs — silently dropping a producer-supplied proof
   * would mislead the producer about the guarantees of the exchange.
   */
  | { kind: "rejectIfPresent" }
  /**
   * SECURITY: accept any document, proof or not, without verifying. Only where
   * the transport already provides equivalent end-to-end integrity. This is the
   * explicit opt-out, and the name is deliberately uncomfortable to type.
   */
  | { kind: "acceptUnverified" };

/** The outcome of {@link consumeInbound}. */
export type ConsumeOutcome<R> =
  /** Every check passed and the caller's handler produced a response. */
  | { kind: "handled"; response: TrustTaskDocument<R> }
  /**
   * A framework check failed, or the handler refused. Either way the document
   * is already addressed per §8.1 — emit it over the transport.
   */
  | { kind: "rejected"; error: ErrorResponse }
  /**
   * §8.1: the rejection was `identityMismatch` and the transport authenticated
   * no sender, so no response may be emitted — one would be an oracle.
   *
   * Callers SHOULD log this: silent suppression is the spec rule, but
   * *invisible* suppression is an operational footgun.
   */
  | { kind: "suppressed"; reason: RejectReason };

/** Wire-safe message for the `rejectIfPresent` path. */
export const PROOF_NOT_ACCEPTED_BY_POLICY =
  "in-band proof not accepted by consumer policy (SPEC §7.2 item 7)";

export interface ConsumeOptions<P, R> {
  transport: TransportHandler;
  /** The generated module's `SPEC` (request) or `RESPONSE_SPEC` (response). */
  spec: SpecPolicy;
  proofPolicy: ProofPolicy;
  doc: TrustTaskDocument<P>;
  /** This consumer's own VID, for the §7.2 item 5 recipient check. */
  myVid: string;
  /** Milliseconds since the epoch, as from `Date.now()`. */
  now: number;
  /** Invoked at most once, only when a rejection needs an error-response id. */
  newErrorId: () => string;
  /** Business handler, called only once every framework check has passed. */
  handler: (
    doc: TrustTaskDocument<P>,
    parties: ResolvedParties,
  ) => Promise<TrustTaskDocument<R> | ErrorResponse> | TrustTaskDocument<R> | ErrorResponse;
  /** Override the response `issuedAt` clock, for deterministic tests. */
  clock?: () => string;
}

/**
 * Run SPEC §7.2 items 4–8 against `doc`, then either call the handler or build
 * the routed error response per §8.1.
 *
 * ⚠ A handler-returned {@link ErrorResponse} is passed through verbatim — the
 * framework does not re-apply §8.1 routing to it. A handler rejecting for
 * identity-style reasons MUST address the response itself (see `reject` in
 * ./transport.js); `rejectWith` copies `doc.issuer` into `recipient`, which is
 * safe for ordinary refusals but not for one that contests that identity.
 */
export async function consumeInbound<P, R>(
  opts: ConsumeOptions<P, R>,
): Promise<ConsumeOutcome<R>> {
  const { transport, spec, proofPolicy, doc, myVid, now, newErrorId, handler, clock } = opts;

  const route = (reason: RejectReason): ConsumeOutcome<R> => {
    const error = reject(transport, doc, newErrorId(), reason, clock);
    return error === undefined ? { kind: "suppressed", reason } : { kind: "rejected", error };
  };

  // §7.2 items 4 + 5a — expiry and wrong-recipient.
  const basic = validateBasic(doc, now, myVid);
  if (basic !== null) return route(basic);

  // §7.2 item 6 — in-band vs transport-derived identity cross-check.
  const resolved = resolveParties(transport, doc);
  if ("error" in resolved) return route(identityMismatchReason(resolved.error));

  // §7.2 item 7 clause B — the consumer's chosen proof policy.
  if (doc.proof !== undefined) {
    switch (proofPolicy.kind) {
      case "verify": {
        let ok: boolean;
        try {
          ok = await proofPolicy.verify.verify(doc);
        } catch (e) {
          ok = false;
        }
        if (!ok) {
          return route({
            code: "proofInvalid",
            message: "proof verification failed",
            retryable: false,
          });
        }
        break;
      }
      case "rejectIfPresent":
        return route({
          code: "malformedRequest",
          message: PROOF_NOT_ACCEPTED_BY_POLICY,
          retryable: false,
        });
      case "acceptUnverified":
        break;
    }
  }

  // §7.2 items 5b + 7 clause A + 8 — the policy-driven checks, in one place so
  // this pipeline and any binding-specific one cannot diverge on the check set.
  const policy = enforceSpecPolicy(doc, spec);
  if (policy !== null) return route(policy);

  const result = await handler(doc, resolved.parties);
  return isErrorResponse(result)
    ? { kind: "rejected", error: result }
    : { kind: "handled", response: result as TrustTaskDocument<R> };
}

/**
 * Whether a handler returned an error response rather than a success response.
 *
 * Keys off the `trust-task-error` Type URI rather than payload shape: §8 makes
 * the type the discriminator, and a success payload could coincidentally carry
 * a `code` member.
 */
function isErrorResponse(doc: TrustTaskDocument<unknown>): doc is ErrorResponse {
  return doc.type.split("#")[0]!.includes("/trust-task-error/");
}

/**
 * Build a handler-side refusal addressed to the original producer.
 *
 * Convenience over `rejectWith` + `toErrorPayload` for the common case where a
 * handler refuses for a business reason. Not safe for refusals that contest the
 * in-band identity — see the warning on {@link consumeInbound}.
 */
export function refuse<P>(
  request: TrustTaskDocument<P>,
  id: string,
  reason: RejectReason,
  clock?: () => string,
): ErrorResponse {
  return rejectWith(request, id, toErrorPayload(reason), clock);
}
