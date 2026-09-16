/// The Trust Tasks ToIP Trust Spanning Protocol transport binding
/// (`bindings/tsp/0.1`).
///
/// [packTrustTask] and [unpackTrustTask] move a document in and out of an
/// HPKE-sealed TSP message; [TspConsumer] is the guarded inbound path.
library;

export 'src/consumer.dart';
export 'src/envelope.dart';
