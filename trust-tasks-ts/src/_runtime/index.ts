/**
 * The Trust Tasks framework runtime for TypeScript — the SPEC.md §7.2 consumer
 * pipeline, the document envelope, and the transport seam.
 *
 * Hand-written, unlike everything else under `src/`, which is generated from
 * the spec registry. Kept in the same package so a consumer gets the types and
 * the checks that make them meaningful from one install, as trust-tasks-rs
 * does.
 *
 * The entry point is {@link consumeInbound}.
 */

export {
  TRUST_TASK_ERROR_TYPE_URI,
  enforceAudienceBinding,
  enforceSpecPolicy,
  rejectWith,
  rejectWithRecipient,
  respondWith,
  toErrorPayload,
  validateBasic,
  type ErrorPayload,
  type ErrorResponse,
  type InResponseTo,
  type Proof,
  type RejectReason,
  type SpecPolicy,
  type TrustTaskDocument,
} from "./document.js";

export {
  STANDARD_CODES,
  extendedCode,
  familyCode,
  isStandardCode,
  normalizeCode,
  slugFromTypeUri,
  type StandardCode,
} from "./codes.js";

export {
  StaticTransport,
  UnauthenticatedTransport,
  identityMismatchReason,
  reject,
  resolveParties,
  type ConsistencyError,
  type ResolvedParties,
  type TransportContext,
  type TransportHandler,
} from "./transport.js";

export {
  ID_CONFLICT_WIRE_MESSAGE,
  PROOF_INVALID_WIRE_MESSAGE,
  PROOF_NOT_ACCEPTED_BY_POLICY,
  REPLAY_RECORD_UNAVAILABLE,
  consequentialChecks,
  consumeInbound,
  notConsequentialChecks,
  refuse,
  type ConsumeChecks,
  type ConsumeOptions,
  type ConsumeOutcome,
  type PayloadPolicy,
  type PayloadValidator,
  type ProofPolicy,
  type ProofVerifier,
} from "./consume.js";

export { canonicalJson, sha256Hex } from "./canonical.js";

export {
  CONSEQUENTIAL_FRESHNESS,
  DEFAULT_FRESHNESS,
  DEFAULT_MAX_AGE_MS,
  DEFAULT_SKEW_MS,
  EXPIRY_NOT_AFTER_ISSUANCE,
  FUTURE_ISSUED_AT,
  ISSUED_AT_REQUIRED,
  STALE_WIRE_MESSAGE,
  recordExpiry,
  validateFreshness,
  type FreshnessPolicy,
} from "./freshness.js";

export {
  InMemoryReplayGuard,
  documentDigest,
  type ReplayGuard,
  type ReplayPolicy,
  type ReplayVerdict,
} from "./replay.js";
