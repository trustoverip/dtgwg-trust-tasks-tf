/**
 * Inbound-document orchestration for SPEC.md §7.2 item 2 and items 4–8.
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
 *   // ajv, or whatever validator you already run — see PayloadPolicy.
 *   payloadPolicy: { kind: "validate", validate: myValidator },
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
 *   case "accepted":   break; // fire-and-forget: nothing to emit
 * }
 * ```
 *
 * Items 1 (framework schema) and 3 (unknown `type`) are *not* attempted here —
 * they belong to the caller's parse and dispatch.
 *
 * Item 2 (payload schema) used to be on that list, on the same reasoning: "by
 * the time you hold a typed document it has already succeeded." That was never
 * true in TypeScript. `TrustTaskDocument<P>` is a compile-time type and erases
 * to nothing; a `JSON.parse` result cast to it has been checked by no one. The
 * package also shipped no schema a caller could have validated against even if
 * they had wanted to, so the step was delegated to a caller who had no way to
 * perform it — and every REQUIRED payload member was optional in practice.
 *
 * The schema now travels on the generated `SPEC` / `RESPONSE_SPEC`, and
 * `payloadPolicy` decides what to do with it. See {@link PayloadPolicy}.
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

/** Evaluates a payload against its schema (SPEC §7.2 item 2). */
export interface PayloadValidator {
  /**
   * Check `payload` against `schema` — the value from
   * {@link SpecPolicy.payloadSchema}, a JSON Schema 2020-12 document with all
   * cross-file `$ref`s already inlined, so no resolver is needed.
   *
   * Return `true` (or `{ ok: true }`) to accept. Return `false`, or
   * `{ ok: false, errors }`, to reject — the errors land in the
   * `malformedRequest` message, so a caller can see what failed.
   */
  validate(
    schema: unknown,
    payload: unknown,
  ): boolean | { ok: boolean; errors?: readonly string[] };
}

/**
 * How {@link consumeInbound} performs SPEC §7.2 item 2 — payload-schema
 * validation.
 *
 * This package bundles no JSON Schema implementation, for the same reason it
 * bundles no cryptosuite: the engine, its draft support and its resource
 * limits are the consumer's choice, and a zero-dependency package is worth
 * keeping. So the schema ships with the generated module and the validator
 * comes from you — wire up ajv, or whatever you already run.
 *
 * **`acceptUnvalidated` is a real choice, not a formality.** TypeScript types
 * are erased at runtime, so nothing else in this pipeline looks at the payload
 * at all: with no validator, a document whose payload is missing every
 * REQUIRED member reaches your handler indistinguishable from a conforming
 * one. That is why the policy is a required option rather than an optional
 * one that defaults to skipping.
 */
export type PayloadPolicy =
  /** Validate against `spec.payloadSchema`. Failures map to `malformedRequest`. */
  | { kind: "validate"; validate: PayloadValidator }
  /**
   * Skip item 2 entirely. Appropriate only where something upstream — an API
   * gateway, a schema-validating transport — has already performed it on the
   * same bytes.
   */
  | { kind: "acceptUnvalidated" };

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
  | { kind: "suppressed"; reason: RejectReason }
  /**
   * Every check passed and the handler completed without producing a
   * document — a fire-and-forget task.
   *
   * SPEC §4.4.1: a specification that defines no success response is one whose
   * consumers **MUST NOT** emit a `#response`-variant document. Such a handler
   * has nothing to return, and before 0.9.0 it had nowhere to say so: the
   * return type demanded a document or an error, and returning neither threw a
   * bare `TypeError` out of the pipeline. Emit nothing.
   */
  | { kind: "accepted" };

/** Wire-safe message for the `rejectIfPresent` path. */
export const PROOF_NOT_ACCEPTED_BY_POLICY =
  "in-band proof not accepted by consumer policy (SPEC §7.2 item 7)";

export interface ConsumeOptions<P, R> {
  transport: TransportHandler;
  /** The generated module's `SPEC` (request) or `RESPONSE_SPEC` (response). */
  spec: SpecPolicy;
  proofPolicy: ProofPolicy;
  /** §7.2 item 2. Required — see {@link PayloadPolicy} for why. */
  payloadPolicy: PayloadPolicy;
  doc: TrustTaskDocument<P>;
  /** This consumer's own VID, for the §7.2 item 5 recipient check. */
  myVid: string;
  /** Milliseconds since the epoch, as from `Date.now()`. */
  now: number;
  /** Invoked at most once, only when a rejection needs an error-response id. */
  newErrorId: () => string;
  /**
   * Business handler, called only once every framework check has passed.
   *
   * Return a success response, an {@link ErrorResponse} to refuse, or nothing
   * at all for a specification that defines no success response (§4.4.1) —
   * which yields the `accepted` outcome.
   */
  handler: (
    doc: TrustTaskDocument<P>,
    parties: ResolvedParties,
  ) =>
    | Promise<TrustTaskDocument<R> | ErrorResponse | void>
    | TrustTaskDocument<R>
    | ErrorResponse
    | void;
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
  const { transport, spec, proofPolicy, payloadPolicy, doc, myVid, now, newErrorId, handler, clock } =
    opts;

  const route = (reason: RejectReason): ConsumeOutcome<R> => {
    const error = reject(transport, doc, newErrorId(), reason, clock);
    return error === undefined ? { kind: "suppressed", reason } : { kind: "rejected", error };
  };

  // A JavaScript caller gets no compile-time check that the option is present,
  // and the old behaviour for an absent policy — skip item 2 silently — is
  // exactly the defect this argument exists to remove. Say so instead of
  // failing on a property read three lines down.
  if (payloadPolicy === undefined) {
    throw new TypeError(
      "consumeInbound: `payloadPolicy` is required as of 0.9.0 (SPEC §7.2 item 2). " +
        'Pass { kind: "validate", validate } to check the payload against ' +
        "`spec.payloadSchema`, or { kind: \"acceptUnvalidated\" } to state that you " +
        "are deliberately not checking it.",
    );
  }

  // §7.2 item 2 — payload schema. Runs first, in the spec's own order, and
  // before anything that reasons about what the payload means: a payload that
  // is not the shape the specification declares should be refused as malformed
  // rather than interpreted.
  if (payloadPolicy.kind === "validate" && spec.payloadSchema !== undefined) {
    let verdict: boolean | { ok: boolean; errors?: readonly string[] };
    try {
      verdict = payloadPolicy.validate.validate(spec.payloadSchema, doc.payload);
    } catch (e) {
      // A validator that throws has not accepted the document. Treating an
      // exception as a pass would make a broken validator indistinguishable
      // from a passing one — the failure mode this policy exists to remove.
      verdict = { ok: false, errors: [e instanceof Error ? e.message : String(e)] };
    }
    const ok = typeof verdict === "boolean" ? verdict : verdict.ok;
    if (!ok) {
      const errors = typeof verdict === "boolean" ? undefined : verdict.errors;
      return route({
        code: "malformedRequest",
        message:
          errors !== undefined && errors.length > 0
            ? `payload does not conform to its schema (SPEC §7.2 item 2): ${errors.join("; ")}`
            : "payload does not conform to its schema (SPEC §7.2 item 2)",
        retryable: false,
      });
    }
  }

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
  if (result === undefined || result === null) return { kind: "accepted" };
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
