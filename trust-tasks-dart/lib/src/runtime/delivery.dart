/// Which delivery of a document a handler is answering, and whether its
/// response may be kept in the SPEC §7.2 item 11 record.
///
/// Hand-written. Mirrors `delivery.rs` in trust-tasks-rs, `Delivery` in
/// @openvtc/trust-tasks (`parties.delivery`) and `trusttasks/delivery.go` in
/// trust-tasks-go.
library;

import 'dart:async';

import 'consume.dart' show Accepted, Handled, Refusal, Rejected, consumeInbound;
import 'replay.dart' show WithholdingReplayGuard;

final Object _zoneKey = Object();

/// The delivery [consumeInbound] is running its handler for, read through
/// [Delivery.current].
///
/// By default a duplicate document is answered from the response its first
/// execution produced and never reaches the handler. A specification can rule
/// that out: `hcp/grants/request` 0.1 issues an access token the provider keeps
/// in no record, and answers a re-delivered copy of an answered request with a
/// newly issued token. Such a handler calls [withholdResponse] while answering
/// the first delivery; the `id` stays recorded as executed, but the response is
/// not stored ([WithholdingReplayGuard]). A duplicate of that document is then
/// handed to the handler again with [isRedelivery] true — the document has
/// already been executed, so the handler answers it (a new token for the same
/// grant) and must not execute it again — and its answer is the [Handled],
/// [Rejected] or [Accepted] outcome.
///
/// The delivery is a zone value: [consumeInbound] runs the handler in a zone
/// that carries it, so [current] returns it in the handler and in everything
/// the handler awaits, and `null` outside. A handler that never calls
/// [withholdResponse] sees no change. The same mechanism exists in every
/// runtime (Rust `Delivery::current()`, TypeScript `parties.delivery`, Go
/// `trusttasks.DeliveryFrom(ctx)`); see `docs/runtime-parity.md`.
final class Delivery {
  Delivery._(this.isRedelivery);

  /// The delivery the calling handler is answering, while [consumeInbound]
  /// runs it; `null` outside.
  static Delivery? get current => Zone.current[_zoneKey] as Delivery?;

  /// True when this document was executed before and its response kept out of
  /// the record: answer it, and do not execute it again (§7.2 item 11). False
  /// on the first execution.
  final bool isRedelivery;

  bool _withheld = false;

  /// Whether [withholdResponse] was called.
  bool get isResponseWithheld => _withheld;

  /// Keep the response the handler returns out of the duplicate-execution
  /// record. The `id` is still recorded as executed, so a duplicate is never
  /// executed again; it is handed back to the handler as a re-delivery instead
  /// of being answered from the record.
  ///
  /// Applies to whatever the first execution answers: a success response, a
  /// non-retryable [Refusal], or nothing. A retryable refusal releases the
  /// claim whatever this says. No effect on a re-delivery (nothing is recorded
  /// for one) or without a replay guard.
  void withholdResponse() {
    _withheld = true;
  }
}

/// A new [Delivery]. Not exported: only the pipeline makes one.
Delivery newDelivery({required bool redelivery}) => Delivery._(redelivery);

/// Run [body] with [delivery] as [Delivery.current]. Not exported.
R runWithDelivery<R>(Delivery delivery, R Function() body) =>
    runZoned(body, zoneValues: {_zoneKey: delivery});
