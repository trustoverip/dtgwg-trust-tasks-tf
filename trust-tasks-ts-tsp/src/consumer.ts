import {
  type ConsumeOutcome,
  type ErrorResponse,
  type FreshnessPolicy,
  type PayloadValidator,
  type ProofVerifier,
  type ReplayGuard,
  type ResolvedParties,
  type SpecPolicy,
  type TrustTaskDocument,
  CONSEQUENTIAL_FRESHNESS,
  InMemoryReplayGuard,
  consumeInbound,
} from "@openvtc/trust-tasks";
import {
  packTrustTask,
  unpackTrustTask,
  advertisedSender,
  newUrnUuid,
  TspEnvelopeError,
  TspTransport,
  type PackKeys,
  type UnpackKeys,
  type Unpacked,
} from "./envelope.js";

/** The public keys of a sender VID, resolved out of band — the Go/TS TSP
 * libraries do no DID resolution, so the caller supplies these. */
export interface SenderPublicKeys {
  /** X25519 public key (HPKE-Auth sender verification). */
  encryptionKey: Uint8Array;
  /** Ed25519 public key (message signature verification). */
  signingKey: Uint8Array;
}

export interface TspConsumerConfig {
  /** The identity messages are addressed to, and replies are sealed from. */
  recipientVid: string;
  /** The recipient's X25519 private key — decrypts inbound and authenticates
   * sealed replies. */
  decryptionKey: Uint8Array;
  /** The recipient's Ed25519 private key — signs sealed replies. */
  signingKey: Uint8Array;
  /** Resolve a sender VID to its public keys. Called after the allowlist check
   * and before the message is opened. */
  resolveSender: (vid: string) => SenderPublicKeys | Promise<SenderPublicKeys>;
  /** When set, only these sender VIDs are opened; checked before resolution. */
  allowedSenders?: ReadonlySet<string>;
  /** The §7.2 item 11 record. Defaults to an in-process guard; wrong for a
   * replicated consumer, which needs a shared store. */
  guard?: ReplayGuard;
  /** Keep the record. Defaults to true. */
  replayProtection?: boolean;
  /** Acceptance window, and so the record's retention. Defaults to
   * `CONSEQUENTIAL_FRESHNESS`. */
  freshness?: FreshnessPolicy;
  /** Verifies in-band proofs. Absent refuses a proof-bearing document with
   * `malformedRequest`. TSP authenticates the sender end-to-end, so a proof may
   * be omitted (§5.3) unless the spec requires one. */
  proofVerifier?: ProofVerifier;
  /** Validates payloads against their schema (§7.2 item 2). */
  payloadValidator?: PayloadValidator;
  now?: () => number;
  newErrorId?: () => string;
}

/** What {@link TspConsumer.receive} made of one TSP message. */
export interface Received<R> {
  outcome: ConsumeOutcome<R>;
  transport: TspTransport;
  /** The document to seal back, as JSON bytes, or undefined when nothing is to
   * be sent: a fire-and-forget success, a suppressed identityMismatch (§8.1), or
   * a duplicate with no retained response (§7.1). Seal it with
   * {@link TspConsumer.packReply}. */
  reply?: Uint8Array;
}

const encoder = new TextEncoder();

/**
 * The guarded inbound path for the TSP binding: resolve the sender, open the
 * sealed message, then run SPEC §7.2 with the item-11 duplicate-execution record
 * and freshness bound on by default.
 *
 * §7 records that TSP data messages "do not inherently prevent replay", and
 * under §5's routed and nested carriage an intermediary may re-send the sealed
 * inner message — so an ordinary re-forward executes a consequential task twice
 * unless the consumer keeps the record, which is keyed on the document `id`,
 * never the TSP envelope (resealed with fresh material on every send).
 *
 * One consumer per process — the guard is the record. Refusals are made by
 * returning an `ErrorResponse` from the handler; a handler that throws is a
 * programming error and propagates.
 */
export class TspConsumer {
  private readonly guard: ReplayGuard;
  private readonly replayProtection: boolean;
  private readonly freshness: FreshnessPolicy;
  private readonly now: () => number;
  private readonly newErrorId: () => string;

  constructor(private readonly config: TspConsumerConfig) {
    this.guard = config.guard ?? new InMemoryReplayGuard();
    this.replayProtection = config.replayProtection ?? true;
    this.freshness = config.freshness ?? CONSEQUENTIAL_FRESHNESS;
    this.now = config.now ?? Date.now;
    this.newErrorId = config.newErrorId ?? newUrnUuid;
  }

  /** Resolve the sender, open `wire`, and run it through §7.2 for `spec`. Throws
   * {@link TspEnvelopeError} for a message that never reached the pipeline (§4). */
  async receive<P, R>(
    wire: Uint8Array,
    spec: SpecPolicy,
    handler: (doc: TrustTaskDocument<P>, parties: ResolvedParties) => Promise<TrustTaskDocument<R> | ErrorResponse | void> | TrustTaskDocument<R> | ErrorResponse | void,
  ): Promise<Received<R>> {
    const named = advertisedSender(wire);
    if (named === undefined) {
      throw new TspEnvelopeError("notForThisReceiver", "not a TSP message this consumer can read");
    }
    if (this.config.allowedSenders && !this.config.allowedSenders.has(named)) {
      throw new TspEnvelopeError("notForThisReceiver", `sender ${named} is not on this consumer's allowlist`);
    }
    let senderKeys: SenderPublicKeys;
    try {
      senderKeys = await this.config.resolveSender(named);
    } catch (cause) {
      throw new TspEnvelopeError("notForThisReceiver", `cannot resolve sender ${named}: ${String(cause)}`, undefined, { cause });
    }
    const keys: UnpackKeys = {
      receiverDecryptionKey: this.config.decryptionKey,
      senderEncryptionKey: senderKeys.encryptionKey,
      senderSigningKey: senderKeys.signingKey,
    };
    const unpacked = await unpackTrustTask<P>(wire, { recipientVid: this.config.recipientVid, keys });
    return this.consume<P, R>(unpacked, spec, handler);
  }

  /** {@link receive} for a document already unpacked. */
  async consume<P, R>(
    unpacked: Unpacked<P>,
    spec: SpecPolicy,
    handler: (doc: TrustTaskDocument<P>, parties: ResolvedParties) => Promise<TrustTaskDocument<R> | ErrorResponse | void> | TrustTaskDocument<R> | ErrorResponse | void,
  ): Promise<Received<R>> {
    const transport = unpacked.transport;
    const doc = unpacked.document as unknown as TrustTaskDocument<P>;

    const outcome = await consumeInbound<P, R>({
      transport,
      spec,
      proofPolicy: this.config.proofVerifier
        ? { kind: "verify", verify: this.config.proofVerifier }
        : { kind: "rejectIfPresent" },
      payloadPolicy: this.config.payloadValidator
        ? { kind: "validate", validate: this.config.payloadValidator }
        : { kind: "acceptUnvalidated" },
      checks: {
        freshness: this.freshness,
        replay: this.replayProtection
          ? { kind: "guard", guard: this.guard }
          : { kind: "notConsequential" },
      },
      doc,
      myVid: transport.local ?? "",
      now: this.now(),
      newErrorId: this.newErrorId,
      handler,
    });

    let reply: Uint8Array | undefined;
    switch (outcome.kind) {
      case "handled":
        reply = encoder.encode(JSON.stringify(outcome.response));
        break;
      case "rejected":
        reply = encoder.encode(JSON.stringify(outcome.error));
        break;
      case "duplicate":
        reply = outcome.priorResponse === undefined ? undefined : encoder.encode(JSON.stringify(outcome.priorResponse));
        break;
      default:
        reply = undefined; // accepted, suppressed
    }
    return { outcome, transport, reply };
  }

  /** Seal a reply back to the authenticated sender (§6). `reply` is the bytes
   * from {@link Received.reply}. */
  async packReply(received: Received<unknown>): Promise<Uint8Array> {
    const peer = received.transport.peer;
    if (received.reply === undefined || peer === undefined) {
      throw new Error("nothing to reply with, or nobody to reply to");
    }
    const senderKeys = await this.config.resolveSender(peer);
    const keys: PackKeys = {
      senderSigningKey: this.config.signingKey,
      senderEncryptionKey: this.config.decryptionKey,
      receiverEncryptionKey: senderKeys.encryptionKey,
    };
    const document = JSON.parse(new TextDecoder().decode(received.reply)) as Record<string, unknown>;
    return packTrustTask(document, { senderVid: this.config.recipientVid, receiverVid: peer, keys });
  }
}
