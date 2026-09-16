/// Duplicate-execution protection — SPEC §7.2 item 11, §8.4, §10.1.
///
/// Hand-written. Mirrors `replay.rs` in trust-tasks-rs, `_runtime/replay.ts` in
/// @openvtc/trust-tasks and `trusttasks/replay.go` in trust-tasks-go, check for
/// check.
///
/// ## The rule
///
/// §7.2 item 11 is normative and unconditional for a *consequential Trust Task*:
/// once a consumer has accepted a document with a given `id` for execution,
/// receiving that same document again **MUST NOT** cause the consequential
/// effect a second time, and receiving a *different* document under the same
/// `id` **MUST** be rejected with `idConflict`.
///
/// §8.4 is the same mechanism from the producer's end: a retry is a bit-for-bit
/// identical resend, and it is safe *precisely because* item 11 obliges the
/// consumer to absorb it. Every transport binding in this repo delegates replay
/// defence to the consumer — `bindings/https/0.2` §5 says "Freshness / replay:
/// None", and the DIDComm and TSP bindings say the same — so if the consumer
/// does not do it, nobody does, and an ordinary mediator redelivery grants an
/// ACL entry twice by accident. §10.1: "The rule deliberately does not
/// distinguish a hostile replay from a legitimate transport retry, because at
/// the document layer the two are indistinguishable."
///
/// ## The key
///
/// The `id` **alone** — "Transport request identifiers, transport message
/// identifiers, and execution handles **MUST NOT** substitute" — plus a digest
/// of the canonical serialization, because "an `id` alone cannot distinguish the
/// retry it must absorb from the conflict it must reject".
///
/// [documentDigest] hashes the canonicalization of the whole document rather
/// than the octets as received: a re-indented body or a member order chosen by
/// an intermediary would otherwise make a legitimate §8.4 retry look like a
/// *different* document.
///
/// The digest covers the **entire** document, `proof` included. That differs
/// deliberately from the §4.9.3 *task digest*, computed over
/// `JCS(document ∖ proof)`, and the spec spells the distinction out: "Item 11
/// and §8.4 ask *which serialization arrived*, so a re-signed `proof` over
/// identical content makes a different document — that is the `idConflict` case,
/// and the distinction is the whole point of the rule."
library;

import 'dart:collection';

import 'canonical.dart';
import 'document.dart';

/// The content identity of a document, per SPEC §7.2's keying paragraph:
/// SHA-256 over the canonical serialization of the whole document, as hex.
///
/// Consumer-local. Never on the wire, and **not** the §4.9.3 task digest.
String documentDigest<P>(
  TrustTaskDocument<P> doc,
  Object? Function(P payload) payloadToJson,
) =>
    sha256HexOfString(canonicalJson(doc.toJson(payloadToJson)));

/// What a [ReplayGuard] says about a document offered for execution.
sealed class ReplayVerdict {
  const ReplayVerdict();
}

/// Not seen before. The caller may execute.
final class Fresh extends ReplayVerdict {
  const Fresh();
}

/// Already accepted under the *same* digest — a §8.4 retry, or a replay. The
/// caller **MUST NOT** execute again.
final class Duplicate extends ReplayVerdict {
  const Duplicate({this.priorResponse, this.inFlight = false});

  /// The response the first execution produced, where one was retained.
  final Object? priorResponse;

  /// Whether that first execution is still running.
  final bool inFlight;
}

/// Already accepted under a *different* digest. §7.2 item 11 requires
/// `idConflict`, and requires that this not be treated as a retry.
final class Conflict extends ReplayVerdict {
  const Conflict();
}

/// The consumer-side record that makes SPEC §7.2 item 11 true.
///
/// An interface, so a deployment can back it with Redis, Postgres, or anything
/// that survives a process restart. [InMemoryReplayGuard] is the default and is
/// correct for a single-process consumer; it is **not** correct behind a load
/// balancer, where two replicas would each accept the same document once.
abstract interface class ReplayGuard {
  /// Claim [id] for execution on behalf of a document with identity [digest].
  ///
  /// [retainUntil] is the instant past which the record may be dropped —
  /// [recordExpiry], which SPEC §7.2 makes the same instant as the end of the
  /// consumer's willingness to execute the document. An implementation SHOULD
  /// treat a record whose [retainUntil] has passed as absent, so the key is
  /// released rather than conflicting forever with a document nobody would
  /// execute.
  ///
  /// Implementations **MUST** make claim-and-record atomic with respect to
  /// concurrent calls: two simultaneous deliveries of the same document must not
  /// both receive [Fresh]. That is the whole guarantee.
  ///
  /// Throwing means the record could not be consulted. [consumeInbound] fails
  /// closed on that, mapping it to `unavailable` with `retryable: true` — a
  /// consumer that cannot establish whether a document is a duplicate has not
  /// satisfied item 11, and executing anyway is the double execution the rule
  /// forbids.
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  );

  /// Attach the response a completed execution produced, so a later duplicate
  /// can be answered with it per §7.2 (*Disposition of a duplicate*) rather than
  /// merely absorbed in silence.
  ///
  /// A guard that records nothing still satisfies item 11 — the effect does not
  /// happen twice — and is the right shape for a fire-and-forget specification,
  /// which has no response to return.
  Future<void> recordResponse(String id, Object? response);

  /// Release a claim whose execution is not to stand — for example a refusal the
  /// consumer marked `retryable`, where §8.4 has just invited the producer to
  /// re-send the same bytes. Without this, that invited retry would come back as
  /// an absorbed duplicate carrying the same failure forever.
  Future<void> release(String id, String digest);
}

/// How a consumer applies SPEC §7.2 item 11 in [consumeInbound].
sealed class ReplayPolicy {
  const ReplayPolicy();

  /// Apply item 11 using [guard]. Correct for any consequential spec.
  const factory ReplayPolicy.guarded(ReplayGuard guard) = GuardedReplay;

  /// Keep no duplicate-execution record. Conformant **only** where the task is
  /// not consequential (§2), or where the specification "explicitly declares
  /// repeated execution safe and intended" — a property of the operation, not of
  /// the consumer's convenience.
  const factory ReplayPolicy.notConsequential() = NotConsequentialReplay;
}

final class GuardedReplay extends ReplayPolicy {
  const GuardedReplay(this.guard);
  final ReplayGuard guard;
}

final class NotConsequentialReplay extends ReplayPolicy {
  const NotConsequentialReplay();
}

class _Entry {
  _Entry(this.digest, this.retainUntil);
  final String digest;
  final DateTime? retainUntil;
  Object? response;
  bool completed = false;
}

/// The record count [InMemoryReplayGuard] retains when given no explicit
/// capacity.
const int defaultReplayCapacity = 10000;

/// A bounded, in-process [ReplayGuard]: an LRU map from `id` to the digest
/// accepted under it, its retention deadline, and the response it produced.
///
/// **Suitable when** one process is the sole consumer for the `recipient` VID it
/// serves and losing the record on restart is acceptable.
///
/// **Not suitable when** the consumer is replicated: two replicas hold separate
/// maps, so a document accepted by replica A is fresh at replica B and the
/// effect happens twice — the exact failure item 11 exists to prevent.
/// Replicated deployments MUST back the guard with a shared store.
///
/// Eviction is by capacity as well as by `retainUntil`: a burst of distinct
/// documents can push an older record out before its deadline, and a replay
/// arriving after that would be accepted. Size the capacity above the number of
/// distinct documents the widest acceptance window can hold.
///
/// [LinkedHashMap] preserves insertion order, which is what makes the LRU a
/// remove-then-reinsert rather than a second index. Dart is single-threaded per
/// isolate and none of these methods await, so claim-and-record is atomic by
/// construction.
class InMemoryReplayGuard implements ReplayGuard {
  /// Builds a guard retaining at most [capacity] records.
  ///
  /// A capacity below 1 takes [defaultReplayCapacity]. It is not treated as
  /// "retain nothing": a guard that retains nothing answers [Fresh] to
  /// everything, which is a silent total defeat of item 11 rather than a visible
  /// misconfiguration.
  InMemoryReplayGuard([int capacity = defaultReplayCapacity])
      : _capacity = capacity < 1 ? defaultReplayCapacity : capacity;

  final int _capacity;
  final LinkedHashMap<String, _Entry> _entries =
      LinkedHashMap<String, _Entry>();

  /// How many records are currently retained. For tests and metrics.
  int get size => _entries.length;

  /// Drop every record whose `retainUntil` has passed.
  void purgeExpired(DateTime now) {
    _entries.removeWhere((_, e) {
      final until = e.retainUntil;
      return until != null && !until.isAfter(now);
    });
  }

  @override
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  ) async {
    final existing = _entries[id];
    if (existing != null) {
      final until = existing.retainUntil;
      // An expired record is treated as absent: the consumer would refuse the
      // document under §7.2 item 4 or the acceptance window anyway, so holding
      // the key would only manufacture a permanent `idConflict` for an `id`
      // nobody can use.
      if (until != null && !until.isAfter(now)) {
        _entries.remove(id);
      } else {
        if (existing.digest != digest) {
          // A conflicting document is not a *use* of the record, so it does not
          // refresh recency — otherwise a flood of conflicts could pin an entry
          // and evict live ones.
          return const Conflict();
        }
        _entries.remove(id);
        _entries[id] = existing;
        return Duplicate(
          priorResponse: existing.response,
          inFlight: !existing.completed,
        );
      }
    }

    _entries[id] = _Entry(digest, retainUntil);
    while (_entries.length > _capacity) {
      _entries.remove(_entries.keys.first);
    }
    return const Fresh();
  }

  @override
  Future<void> recordResponse(String id, Object? response) async {
    final entry = _entries[id];
    if (entry == null) return;
    entry.response = response;
    entry.completed = true;
  }

  @override
  Future<void> release(String id, String digest) async {
    final entry = _entries[id];
    // Only release the claim this digest made, and only while it is unfinished:
    // a concurrent arrival that legitimately holds the key must not have it
    // taken away by another document's cleanup.
    if (entry != null && entry.digest == digest && !entry.completed) {
      _entries.remove(id);
    }
  }
}
