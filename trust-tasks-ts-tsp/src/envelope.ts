import type { TransportContext, TransportHandler } from "@openvtc/trust-tasks";
import { pack, unpack, decodeEnvelope, type PackKeys, type UnpackKeys } from "@openvtc/vti-tsp-js";

/** The binding's stable identifier (`bindings/tsp/0.1` §1). Never on the wire. */
export const BINDING_URI = "https://trusttasks.org/binding/tsp/0.1";

/** The `type` member of the envelope object a TSP message carries (§1, §2). A
 * TSP VID is a framework VID, so — unlike DIDComm — no identifier is transformed. */
export const ENVELOPE_TYPE = "https://trusttasks.org/binding/tsp/0.1/envelope";

/** The keys needed to seal a message to a recipient (from `@openvtc/vti-tsp-js`). */
export type { PackKeys, UnpackKeys };

/** The {@link TransportHandler} for one TSP exchange: `peer` is the VID TSP
 * authenticated as the sender, `local` the VID that opened the message (§3). */
export class TspTransport implements TransportHandler {
  constructor(
    readonly local: string | undefined,
    readonly peer: string | undefined,
  ) {}

  bindingUri(): string {
    return BINDING_URI;
  }

  deriveParties(): TransportContext {
    return { issuer: this.peer, recipient: this.local };
  }
}

/** Why a TSP message was refused before a document could be produced. TSP has no
 * anonymous sender (§2), so none of these can be answered with a
 * `trust-task-error` — there is nothing authenticated to reply about (§4). */
export type TspFailureKind =
  | "notForThisReceiver"
  | "notFinalRecipient"
  | "wrongEnvelopeType"
  | "invalidBody";

/** A TSP message refused by {@link unpackTrustTask}. */
export class TspEnvelopeError extends Error {
  constructor(
    readonly failure: TspFailureKind,
    detail: string,
    /** The authenticated sender, where authentication got that far. */
    readonly sender?: string,
    options?: { cause?: unknown },
  ) {
    super(`tsp: ${failure}: ${detail}`, options);
    this.name = "TspEnvelopeError";
  }
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

interface Envelope {
  type: string;
  document: Record<string, unknown>;
}

/** A Trust Task document taken out of a TSP message. */
export interface Unpacked<P> {
  /** The document, exactly as the sender serialised it — what a proof over it
   * was computed on. */
  document: TrustTaskLike<P>;
  /** The authenticated sender and the opening recipient. */
  transport: TspTransport;
}

/** A parsed Trust Task document. Structural: the core's `TrustTaskDocument<P>`
 * with `P` chosen by the caller. */
export type TrustTaskLike<P> = { id: string; type: string; payload: P } & Record<string, unknown>;

export interface PackParams {
  senderVid: string;
  receiverVid: string;
  keys: PackKeys;
}

/** Seal `document` into a Direct TSP message from `senderVid` to `receiverVid`
 * (§2, §5.1). The payload is the envelope object `{type: ENVELOPE_TYPE,
 * document}`, HPKE-Auth sealed and signed. */
export async function packTrustTask(
  document: Record<string, unknown>,
  { senderVid, receiverVid, keys }: PackParams,
): Promise<Uint8Array> {
  const body = encoder.encode(JSON.stringify({ type: ENVELOPE_TYPE, document }));
  const packed = await pack(body, senderVid, receiverVid, keys);
  return packed.bytes;
}

/** The sender VID a TSP message names, read from the cleartext envelope without
 * verifying anything. Unauthenticated until {@link unpackTrustTask} opens it —
 * use it only to decide whether to open it, or to resolve the sender's keys. */
export function advertisedSender(wire: Uint8Array): string | undefined {
  try {
    return decodeEnvelope(wire).envelope.sender;
  } catch {
    return undefined;
  }
}

export interface UnpackParams {
  recipientVid: string;
  keys: UnpackKeys;
}

/** Open a Direct TSP message addressed to `recipientVid` and take out the Trust
 * Task document. `keys` carries the recipient's decryption key and the resolved
 * sender's public keys, against which the signature and HPKE authentication are
 * checked. Throws {@link TspEnvelopeError}. */
export async function unpackTrustTask<P = unknown>(
  wire: Uint8Array,
  { recipientVid, keys }: UnpackParams,
): Promise<Unpacked<P>> {
  let message;
  try {
    message = await unpack(wire, keys);
  } catch (cause) {
    throw new TspEnvelopeError("notForThisReceiver", String(cause), undefined, { cause });
  }

  if (message.messageType !== "direct") {
    // A routed/nested layer this VID only relays, or an inner not addressed here.
    throw new TspEnvelopeError(
      "notFinalRecipient",
      `message is ${message.messageType}, not a Direct Trust Task envelope`,
      message.sender,
    );
  }

  let envelope: Envelope;
  try {
    envelope = JSON.parse(decoder.decode(message.payload)) as Envelope;
  } catch (cause) {
    throw new TspEnvelopeError("invalidBody", "payload is not a JSON envelope object", message.sender, { cause });
  }
  if (envelope.type !== ENVELOPE_TYPE) {
    throw new TspEnvelopeError("wrongEnvelopeType", `envelope type ${envelope.type}`, message.sender);
  }
  const document = envelope.document;
  if (typeof document !== "object" || document === null || typeof document.id !== "string" || typeof document.type !== "string") {
    throw new TspEnvelopeError("invalidBody", "envelope carries no Trust Task document", message.sender);
  }

  return {
    document: document as TrustTaskLike<P>,
    transport: new TspTransport(message.receiver || recipientVid, message.sender),
  };
}

/** A fresh `urn:uuid:` identifier (UUID v4), for response and error documents. */
export function newUrnUuid(): string {
  return `urn:uuid:${crypto.randomUUID()}`;
}
