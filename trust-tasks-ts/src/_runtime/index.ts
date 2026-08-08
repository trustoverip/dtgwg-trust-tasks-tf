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
  PROOF_NOT_ACCEPTED_BY_POLICY,
  consumeInbound,
  refuse,
  type ConsumeOptions,
  type ConsumeOutcome,
  type ProofPolicy,
  type ProofVerifier,
} from "./consume.js";
