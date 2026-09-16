/**
 * The ToIP Trust Spanning Protocol transport binding (`bindings/tsp/0.1`) for
 * Trust Tasks, on `@openvtc/vti-tsp-js`.
 *
 * {@link packTrustTask} and {@link unpackTrustTask} move a document in and out
 * of an HPKE-sealed Direct TSP message; {@link TspConsumer} is the guarded
 * inbound path running the SPEC §7.2 pipeline with the item-11 replay record on
 * by default. The sealed `{type, document}` envelope is the same shape the Rust
 * crate, the Dart package and the Go module seal.
 */
export {
  BINDING_URI,
  ENVELOPE_TYPE,
  TspTransport,
  TspEnvelopeError,
  packTrustTask,
  unpackTrustTask,
  advertisedSender,
  newUrnUuid,
  type TspFailureKind,
  type Unpacked,
  type TrustTaskLike,
  type PackParams,
  type UnpackParams,
  type PackKeys,
  type UnpackKeys,
} from "./envelope.js";
export { TspConsumer, type TspConsumerConfig, type SenderPublicKeys, type Received } from "./consumer.js";
