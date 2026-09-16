/// The Trust Tasks DIDComm v2.1 transport binding (`bindings/didcomm/0.2`).
///
/// [packTrustTask] and [unpackTrustTask] move a document in and out of an
/// authcrypt envelope; [DidcommConsumer] is the guarded inbound path.
library;

export 'src/consumer.dart';
export 'src/envelope.dart';
