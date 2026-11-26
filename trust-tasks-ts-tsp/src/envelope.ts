import type { TransportContext, TransportHandler } from "@openvtc/trust-tasks/runtime";
import {
  pack,
  unpack,
  decodeEnvelope,
  isRevisionError,
  peekRevision,
  SUPPORTED_MAJOR,
  type PackKeys,
  type UnpackKeys,
} from "@openvtc/vti-tsp-js";

/** The TSP version this binding speaks: TSP 1.0 (`YTSP-BAA`), the released
 * specification, as `@openvtc/vti-tsp-js` 0.4 and `affinidi-tsp` 0.5 do. A
 * message from a peer still on a draft (Rev 2 `YTSP-AAB`, Rev 3 `YTSP-AAC`) is
 * refused as `unsupportedTspVersion`: TSP has no version negotiation, so that
 * peer has to upgrade. */
export const TSP_VERSION = "1.0";

/** The binding's stable identifier (`bindings/tsp/0.3` §1), which requires TSP
 * 1.0. Never on the wire. The envelope `type` is still the `tsp/0.1` one. */
export const BINDING_URI = "https://trusttasks.org/binding/tsp/0.3";

/** The `type` member of the envelope object a TSP message carries (§1, §2). A
 * TSP VID is a framework VID, so — unlike DIDComm — no identifier is transformed. */
export const ENVELOPE_TYPE = "https://trusttasks.org/binding/tsp/0.1/envelope";

/** The keys needed to seal and open a message (from `@openvtc/vti-tsp-js`). TSP
 * 1.0 seals with HPKE-Base, so neither carries the sender's X25519 key: the
 * sender is authenticated by the outer Ed25519 signature and the ESSR sender
 * field. */
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
  | "unsupportedTspVersion"
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

/** Seal `document` into a TSP 1.0 Direct message from `senderVid` to
 * `receiverVid` (§2, §5.1). The payload is the envelope object `{type:
 * ENVELOPE_TYPE, document}`, HPKE-Base sealed and signed. */
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
 * use it only to decide whether to open it, or to resolve the sender's keys.
 * Undefined for bytes that are not a TSP 1.0 message, including a draft one. */
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

/** The refusal for a message that declares a TSP version this binding does not
 * speak, read from the version marker without keys; else undefined (the
 * message is TSP 1.0, or not readable as TSP at all). */
export function unsupportedVersion(wire: Uint8Array): TspEnvelopeError | undefined {
  try {
    peekRevision(wire);
    return undefined;
  } catch (cause) {
    return versionRefusal(cause);
  }
}

function versionRefusal(cause: unknown): TspEnvelopeError | undefined {
  // A revision error with no MAJOR is bytes that are not a TSP frame at all,
  // which is not a version question. One at MAJOR 1 is vti-tsp-js reporting a
  // 1.x frame with a MINOR it does not know that also failed to open: MINOR
  // never gates a message (TSP 1.0 §3.1), so that is an unreadable message, not a
  // peer on another version.
  if (!isRevisionError(cause) || cause.major === undefined || cause.major === SUPPORTED_MAJOR) return undefined;
  return new TspEnvelopeError(
    "unsupportedTspVersion",
    `${cause.message.replace(/^tsp: /, "")} (the sender must move to TSP ${TSP_VERSION}; there is no version negotiation)`,
    undefined,
    { cause },
  );
}

/** Open a TSP 1.0 Direct message addressed to `recipientVid` and take out the
 * Trust Task document. `keys` carries the recipient's decryption key and the
 * resolved sender's Ed25519 public key, against which the signature is
 * checked. Throws {@link TspEnvelopeError}: `unsupportedTspVersion` for a
 * message from a peer on a TSP draft. */
export async function unpackTrustTask<P = unknown>(
  wire: Uint8Array,
  { recipientVid, keys }: UnpackParams,
): Promise<Unpacked<P>> {
  let message;
  try {
    message = await unpack(wire, keys);
  } catch (cause) {
    throw versionRefusal(cause) ?? new TspEnvelopeError("notForThisReceiver", String(cause), undefined, { cause });
  }

  // The receiver VID on the wire is the one the sender wrote: the seal proves
  // only that the message was encrypted to this recipient's key. A message
  // naming another VID is not for this consumer, whose identity it would
  // otherwise become (and so the issuer of every answer). An empty receiver is
  // the NULL-VID form, opened as `recipientVid`.
  if (message.receiver && message.receiver !== recipientVid) {
    throw new TspEnvelopeError(
      "notForThisReceiver",
      "the message is not addressed to this receiver",
      message.sender,
    );
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
    transport: new TspTransport(recipientVid, message.sender),
  };
}

/** A fresh `urn:uuid:` identifier (UUID v4), for response and error documents. */
export function newUrnUuid(): string {
  return `urn:uuid:${crypto.randomUUID()}`;
}
