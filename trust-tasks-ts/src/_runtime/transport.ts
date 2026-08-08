/**
 * The integration point between the transport-agnostic document model and a
 * concrete transport binding (HTTPS + mTLS, DIDComm, TSP, an in-memory test
 * loopback, …).
 *
 * Hand-written. Mirrors `transport.rs` in trust-tasks-rs. The contract encodes
 * SPEC.md §4.8.1 and §9.2:
 *
 * - In-band `issuer` / `recipient` are authoritative when present.
 * - Transport-derived identity fills in absent members.
 * - When both are present they MUST agree; a mismatch is a validation failure
 *   (`identityMismatch`, §8.3).
 */

import {
  rejectWithRecipient,
  toErrorPayload,
  type ErrorResponse,
  type RejectReason,
  type TrustTaskDocument,
} from "./document.js";

/**
 * Party identity after §4.8.1 precedence — the values a consumer applies for
 * every subsequent framework rule referencing the issuer or recipient.
 */
export interface ResolvedParties {
  issuer?: string;
  recipient?: string;
}

/**
 * What the transport authenticated about an inbound message.
 *
 * A field is absent when the transport does not authenticate that party — a
 * plain HTTPS handler with no client certificate leaves `issuer` unset.
 * Returning an entirely empty context is valid; the framework then relies on
 * the in-band members and any `proof` they carry.
 */
export interface TransportContext {
  issuer?: string;
  recipient?: string;
}

/** Raised when in-band and transport-derived identity disagree (§7.2 item 6). */
export interface ConsistencyError {
  party: "issuer" | "recipient";
  inBand: string;
  transport: string;
}

/** A transport binding's plug-in for the framework. */
export interface TransportHandler {
  /**
   * Stable identifier for this binding, for logs and audit (§9.1, §9.2).
   */
  bindingUri(): string;

  /** Identities the transport authenticated for the message under consideration. */
  deriveParties(): TransportContext;
}

/**
 * Apply §4.8.1 precedence to produce the final {@link ResolvedParties}.
 *
 * Returns a {@link ConsistencyError} when an in-band member is present and
 * disagrees with the transport-derived value for the same party. Callers
 * translate that into an `identityMismatch` error response (§8.3) — or let
 * {@link consumeInbound} do it.
 */
export function resolveParties<P>(
  handler: TransportHandler,
  doc: TrustTaskDocument<P>,
): { parties: ResolvedParties } | { error: ConsistencyError } {
  const ctx = handler.deriveParties();

  for (const party of ["issuer", "recipient"] as const) {
    const inBand = doc[party];
    const transport = ctx[party];
    if (inBand !== undefined && transport !== undefined && inBand !== transport) {
      return { error: { party, inBand, transport } };
    }
  }

  const parties: ResolvedParties = {};
  const issuer = doc.issuer ?? ctx.issuer;
  const recipient = doc.recipient ?? ctx.recipient;
  if (issuer !== undefined) parties.issuer = issuer;
  if (recipient !== undefined) parties.recipient = recipient;
  return { parties };
}

/**
 * Build the error response for `doc`, applying the §8.1 routing rules.
 *
 * Returns `undefined` when the rejection is `identityMismatch` and the
 * transport authenticated no sender. §8.1 is explicit that the consumer SHOULD
 * NOT emit a response in that case: the in-band `issuer` is by definition the
 * contested identity, so addressing it would be an oracle, and on any transport
 * that signs error responses it would compel the consumer to emit a signed
 * document about a party that did not take part in the exchange.
 */
export function reject<P>(
  handler: TransportHandler,
  doc: TrustTaskDocument<P>,
  id: string,
  reason: RejectReason,
  now?: () => string,
): ErrorResponse | undefined {
  let recipient: string | undefined;
  if (reason.code === "identityMismatch") {
    recipient = handler.deriveParties().issuer;
    if (recipient === undefined) return undefined;
  } else {
    recipient = doc.issuer;
  }
  return rejectWithRecipient(doc, id, toErrorPayload(reason), recipient, now);
}

/**
 * The `identityMismatch` reason for a {@link ConsistencyError}.
 *
 * §8.1 additionally requires the wire `message` to be sanitized: naming the
 * consumer's expected transport-authenticated identity, or echoing the
 * contested in-band value, leaks identity information to a possibly hostile
 * sender. The standard wire form is the code alone with a non-identifying
 * message — so the mismatched values are deliberately *not* included here.
 * Log them locally instead.
 */
export function identityMismatchReason(_error: ConsistencyError): RejectReason {
  return {
    code: "identityMismatch",
    message: "identityMismatch: in-band identity does not match transport-derived identity",
    retryable: false,
  };
}

/**
 * A handler for transports that authenticate nothing — an unauthenticated
 * HTTP POST, a public queue, paper. Party identity comes entirely from the
 * in-band members and whatever `proof` they carry.
 */
export class UnauthenticatedTransport implements TransportHandler {
  constructor(private readonly uri = "urn:trust-tasks:transport:unauthenticated") {}
  bindingUri(): string {
    return this.uri;
  }
  deriveParties(): TransportContext {
    return {};
  }
}

/** A fixed-identity handler, for tests and for transports resolved out-of-band. */
export class StaticTransport implements TransportHandler {
  constructor(
    private readonly ctx: TransportContext,
    private readonly uri = "urn:trust-tasks:transport:static",
  ) {}
  bindingUri(): string {
    return this.uri;
  }
  deriveParties(): TransportContext {
    return { ...this.ctx };
  }
}
