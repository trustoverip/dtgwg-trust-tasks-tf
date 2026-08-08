/**
 * The Trust Task document envelope (SPEC.md §4) and the framework-level checks
 * that operate on it.
 *
 * Hand-written. Mirrors `document.rs` in trust-tasks-rs.
 */

import type { StandardCode } from "./codes.js";

/** A W3C Data Integrity Proof object (SPEC.md §4.7). Opaque to the framework. */
export interface Proof {
  type: string;
  cryptosuite: string;
  verificationMethod: string;
  created: string;
  proofPurpose: string;
  proofValue: string;
  [k: string]: unknown;
}

/** A single Trust Task document, per SPEC.md §4.2. */
export interface TrustTaskDocument<P> {
  /** Globally unique to this instance (§4.3). */
  id: string;
  /** Correlates this document with others in the same exchange (§4.9). */
  threadId?: string;
  /** The Type URI identifying specification and version (§4.4). */
  type: string;
  /** VID of the party responsible for the content (§4.8). */
  issuer?: string;
  /** VID of the party expected to act on the document (§4.8). */
  recipient?: string;
  issuedAt?: string;
  expiresAt?: string;
  payload: P;
  "@context"?: unknown;
  proof?: Proof;
  /**
   * Unrecognized top-level members. §7.2 says a consumer SHOULD preserve but
   * MUST NOT act upon them, and §7.1 asks a forwarding producer to carry them
   * through.
   */
  [k: string]: unknown;
}

/** The `payload` of an error response (SPEC.md §8.2). */
export interface ErrorPayload {
  code: string;
  message?: string;
  retryable: boolean;
  retryAfter?: string;
  details?: Record<string, unknown>;
}

/** A `trust-task-error` document. */
export type ErrorResponse = TrustTaskDocument<ErrorPayload>;

/**
 * The per-specification declarations a consumer needs to apply SPEC §7.2 items
 * 5b, 7 and 8. Generated modules export this as `SPEC` / `RESPONSE_SPEC`.
 *
 * Structurally typed on purpose, so the 285 generated modules need no import
 * from this one.
 */
export interface SpecPolicy {
  readonly typeUri: string;
  /** §4.8.3 — opts out of the §4.8.2 audience-binding rule. */
  readonly isBearer: boolean;
  /** §7.3 item 8 — `proofRequirement.requirement` is REQUIRED. */
  readonly isProofRequired: boolean;
  /** §7.3 item 5 — the party filling `recipient` is REQUIRED. */
  readonly isRecipientRequired: boolean;
}

/** Why a consumer rejected a document, and the §8.3 code it maps to. */
export interface RejectReason {
  code: StandardCode;
  message: string;
  retryable: boolean;
  retryAfter?: string;
  details?: Record<string, unknown>;
}

/** The Type URI a consumer emits error responses under. */
export const TRUST_TASK_ERROR_TYPE_URI = "https://trusttasks.org/spec/trust-task-error/0.2";

/**
 * SPEC §7.2 items 4 and 5a — expiry and wrong-recipient.
 *
 * ⚠ This is *not* the full §7.2 check. Items 1–3 (framework schema, payload
 * schema, unknown `type`) belong to the caller's parse and dispatch; items 5b,
 * 7 and 8 need the specification's policy and live in
 * {@link enforceSpecPolicy}. {@link consumeInbound} bundles 4–8.
 *
 * @param now Milliseconds since the epoch, as from `Date.now()`.
 */
export function validateBasic<P>(
  doc: TrustTaskDocument<P>,
  now: number,
  myVid: string,
): RejectReason | null {
  if (doc.expiresAt !== undefined) {
    const expiresAt = Date.parse(doc.expiresAt);
    if (Number.isNaN(expiresAt)) {
      return {
        code: "malformedRequest",
        message: "expiresAt is not a valid RFC 3339 timestamp",
        retryable: false,
      };
    }
    // §4.2 / §7.2 item 4: inclusive bound — `now >= expiresAt` is expired.
    if (expiresAt <= now) {
      return {
        code: "expired",
        message: `document expired at ${doc.expiresAt}`,
        retryable: false,
      };
    }
  }

  if (doc.recipient !== undefined && doc.recipient !== myVid) {
    return {
      code: "wrongRecipient",
      message: "in-band recipient does not identify this consumer",
      retryable: false,
    };
  }

  return null;
}

/**
 * SPEC §7.2 item 8 — a proof-bearing document on a non-bearer specification
 * must carry an in-band `recipient`, so the proof binds the audience as well as
 * the content (§4.8.2).
 */
export function enforceAudienceBinding<P>(
  doc: TrustTaskDocument<P>,
  spec: SpecPolicy,
): RejectReason | null {
  if (doc.proof !== undefined && doc.recipient === undefined && !spec.isBearer) {
    return {
      code: "malformedRequest",
      message:
        "proof present with no in-band recipient on a non-bearer specification " +
        "(SPEC §4.8.2 audience binding)",
      retryable: false,
    };
  }
  return null;
}

/**
 * The policy-driven subset of SPEC §7.2 — items 5b, 7 clause A, and 8.
 *
 * Single source of truth for the flag-driven checks, so a binding-specific
 * pipeline and {@link consumeInbound} cannot diverge on the check set. Ordering
 * matches trust-tasks-rs `enforce_spec_policy`: recipient, then proof, then
 * audience binding.
 */
export function enforceSpecPolicy<P>(
  doc: TrustTaskDocument<P>,
  spec: SpecPolicy,
): RejectReason | null {
  if (doc.recipient === undefined && spec.isRecipientRequired) {
    return {
      code: "malformedRequest",
      message:
        "specification declares recipient REQUIRED but the document carries no in-band recipient",
      retryable: false,
    };
  }
  if (doc.proof === undefined && spec.isProofRequired) {
    return {
      code: "proofRequired",
      message: "specification declares proof REQUIRED but the document carries none",
      retryable: false,
    };
  }
  return enforceAudienceBinding(doc, spec);
}

/**
 * Build the error response for `request`, addressed to an explicit `recipient`.
 *
 * ⚠ Prefer {@link rejectWith} for ordinary refusals. This form exists for
 * `identityMismatch`, where §8.1 forbids addressing the contested in-band
 * issuer — see `TransportHandler.reject` in ./transport.js, which applies that rule.
 */
export function rejectWithRecipient<P>(
  request: TrustTaskDocument<P>,
  id: string,
  payload: ErrorPayload,
  recipient: string | undefined,
  now: () => string = () => new Date().toISOString(),
): ErrorResponse {
  return {
    id,
    // §4.9: continue the thread, falling back to the request's own id.
    threadId: request.threadId ?? request.id,
    type: TRUST_TASK_ERROR_TYPE_URI,
    issuer: request.recipient,
    recipient,
    issuedAt: now(),
    payload,
  };
}

/**
 * Build the error response for `request`, addressed to its original producer.
 *
 * ⚠ Not safe under a rejection that contests the in-band identity: it copies
 * `request.issuer` into `recipient`, which under `identityMismatch` is the
 * contested value §8.1 says MUST NOT be addressed. Use
 * `TransportHandler.reject` (./transport.js) for those.
 */
export function rejectWith<P>(
  request: TrustTaskDocument<P>,
  id: string,
  payload: ErrorPayload,
  now?: () => string,
): ErrorResponse {
  return rejectWithRecipient(request, id, payload, request.issuer, now);
}

/** Turn a {@link RejectReason} into the §8.2 payload it maps to. */
export function toErrorPayload(reason: RejectReason): ErrorPayload {
  const payload: ErrorPayload = {
    code: reason.code,
    message: reason.message,
    retryable: reason.retryable,
  };
  if (reason.retryAfter !== undefined) payload.retryAfter = reason.retryAfter;
  if (reason.details !== undefined) payload.details = reason.details;
  return payload;
}

/**
 * Build the success-response document for `request`, per SPEC §4.4.1 — the
 * request's Type URI with the `#response` fragment, the parties swapped, and
 * the thread continued.
 */
export function respondWith<P, R>(
  request: TrustTaskDocument<P>,
  id: string,
  payload: R,
  now: () => string = () => new Date().toISOString(),
): TrustTaskDocument<R> {
  const bare = request.type.split("#")[0]!;
  return {
    id,
    threadId: request.threadId ?? request.id,
    type: `${bare}#response`,
    issuer: request.recipient,
    recipient: request.issuer,
    issuedAt: now(),
    payload,
  };
}
