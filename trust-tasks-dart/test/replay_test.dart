// SPEC.md §7.2 item 11, §4.2 freshness, and canonicalization conformance tests.
//
// Mirror trust-tasks-ts/test/replay.test.ts, trust-tasks-go/trusttasks and the
// corresponding cases in trust-tasks-rs/src/replay.rs.

import 'dart:convert';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

import 'consume_test.dart' show Payload, Response, me, peer, payloadToJson;

const consequentialSpec = SpecPolicy(
  typeUri: 'https://trusttasks.org/spec/acl/grant/0.1',
);

final testNow = DateTime.utc(2026, 1, 1);
String fixedClock() => '2026-01-01T00:00:00Z';

/// Carries the `issuedAt` that [consequentialChecks] requires.
TrustTaskDocument<Payload> consequentialDoc({
  Payload payload = const Payload('admin'),
  String? expiresAt,
  Proof? proof,
}) =>
    TrustTaskDocument<Payload>(
      id: 'req-1',
      type: consequentialSpec.typeUri,
      issuer: peer,
      issuedAt: '2026-01-01T00:00:00Z',
      expiresAt: expiresAt,
      proof: proof,
      payload: payload,
    );

/// A counter the handler bumps, so a test can assert the effect happened once.
class Executions {
  int count = 0;
}

Future<ConsumeOutcome<Response>> consequential(
  ReplayGuard guard,
  TrustTaskDocument<Payload> doc,
  Executions executions, {
  ConsumeChecks? checks,
}) =>
    consumeInbound<Payload, Response>(
      transport: const UnauthenticatedTransport(),
      spec: consequentialSpec,
      proofPolicy: const ProofPolicy.acceptUnverified(),
      payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      checks: checks ?? consequentialChecks(guard),
      doc: doc,
      myVid: me,
      now: testNow,
      newErrorId: () => 'err-1',
      payloadToJson: payloadToJson,
      clock: fixedClock,
      handler: (accepted, parties) async {
        executions.count++;
        return respondWith<Payload, Response>(
          accepted,
          'resp-1',
          const Response(true),
          clock: fixedClock,
        );
      },
    );

/// A guard that cannot consult its record.
class _FailingGuard implements ReplayGuard {
  const _FailingGuard();
  @override
  Future<ReplayVerdict> claim(
    String id,
    String digest,
    DateTime? retainUntil,
    DateTime now,
  ) async =>
      throw StateError('redis://cache.internal:6379 connection refused');
  @override
  Future<void> recordResponse(String id, Object? response) async {}
  @override
  Future<void> release(String id, String digest) async {}
}

void main() {
  group('SPEC §7.2 item 11 — duplicate execution', () {
    test('absorbs a bit-for-bit resend and never executes twice (§8.4)',
        () async {
      final guard = InMemoryReplayGuard();
      final executions = Executions();

      final first = await consequential(guard, consequentialDoc(), executions);
      expect(first, isA<Handled<Response>>());

      final second = await consequential(guard, consequentialDoc(), executions);
      expect(second, isA<DuplicateOutcome<Response>>());
      expect(executions.count, 1,
          reason: 'the consequential effect happened twice');

      // §7.2 (*Disposition of a duplicate*): return the prior response.
      final prior = (second as DuplicateOutcome<Response>).priorResponse;
      expect(prior, isA<TrustTaskDocument<Response>>());
      expect((prior! as TrustTaskDocument<Response>).id, 'resp-1');
    });

    test('rejects differing content under a reused id with idConflict',
        () async {
      final guard = InMemoryReplayGuard();
      final executions = Executions();

      await consequential(guard, consequentialDoc(), executions);
      final outcome = await consequential(
        guard,
        consequentialDoc(payload: const Payload('owner')),
        executions,
      );

      expect(outcome, isA<Rejected<Response>>());
      expect((outcome as Rejected<Response>).error.payload.code, 'idConflict');
      expect(executions.count, 1, reason: 'the escalated document executed');
    });

    test('treats a re-signed proof over identical content as a conflict', () {
      // §7.2: "a re-signed proof over identical content makes a different
      // document — that is the idConflict case, and the distinction is the whole
      // point."
      String digestWith(String proofValue) => documentDigest(
            consequentialDoc(
              proof: Proof(
                type: 'DataIntegrityProof',
                cryptosuite: 'eddsa-jcs-2022',
                verificationMethod: '$peer#key-1',
                created: '2026-01-01T00:00:00Z',
                proofPurpose: 'assertionMethod',
                proofValue: proofValue,
              ),
            ),
            payloadToJson,
          );
      expect(digestWith('zAAA'), isNot(digestWith('zBBB')));
    });

    test('does not burn the id of a document refused before the claim',
        () async {
      final guard = InMemoryReplayGuard();
      final executions = Executions();

      // Refused by §7.2 item 4 — expiry — which runs well before the claim.
      final refused = await consequential(
        guard,
        consequentialDoc(expiresAt: '2025-01-01T00:00:00Z'),
        executions,
      );
      expect(refused, isA<Rejected<Response>>());
      expect(guard.size, 0, reason: 'a document never accepted left a record');

      // The corrected resend under the same id must not come back as idConflict.
      final corrected =
          await consequential(guard, consequentialDoc(), executions);
      expect(corrected, isA<Handled<Response>>());
    });

    test('refuses a document it cannot place in any window', () async {
      // §7.2 (*Bounding the record*): no expiresAt and no usable age means no
      // window, and a consequential task MUST NOT execute.
      final guard = InMemoryReplayGuard();
      final executions = Executions();
      final unbounded = ConsumeChecks(
        freshness: FreshnessPolicy.defaults,
        replay: ReplayPolicy.guarded(guard),
      );

      final doc = TrustTaskDocument<Payload>(
        id: 'req-1',
        type: consequentialSpec.typeUri,
        issuer: peer,
        payload: const Payload('admin'),
      );
      final outcome =
          await consequential(guard, doc, executions, checks: unbounded);

      expect(outcome, isA<Rejected<Response>>());
      expect((outcome as Rejected<Response>).error.payload.code, 'expired');
      expect(executions.count, 0);
    });

    test('fails closed when the record cannot be consulted', () async {
      final executions = Executions();
      final outcome = await consequential(
        const _FailingGuard(),
        consequentialDoc(),
        executions,
      );

      expect(outcome, isA<Rejected<Response>>());
      final payload = (outcome as Rejected<Response>).error.payload;
      expect(payload.code, 'unavailable');
      expect(payload.retryable, isTrue);
      expect(executions.count, 0);
      // §10.4: the store's hostname must not reach the wire.
      expect(payload.message, replayRecordUnavailable);
      expect(payload.message, isNot(contains('redis')));
    });

    test('keeps no record when the task is declared not consequential',
        () async {
      final guard = InMemoryReplayGuard();
      final executions = Executions();
      for (var i = 0; i < 2; i++) {
        final outcome = await consequential(
          guard,
          consequentialDoc(),
          executions,
          checks: notConsequentialChecks(),
        );
        expect(outcome, isA<Handled<Response>>());
      }
      expect(executions.count, 2);
    });
  });

  group('InMemoryReplayGuard', () {
    final deadline = testNow.add(const Duration(hours: 1));

    test('evicts the least recently used record at capacity', () async {
      final guard = InMemoryReplayGuard(2);
      await guard.claim('a', 'd1', deadline, testNow);
      await guard.claim('b', 'd2', deadline, testNow);
      await guard.claim('a', 'd1', deadline, testNow); // touch a
      await guard.claim('c', 'd3', deadline, testNow); // evicts b

      expect(guard.size, 2);
      // Assert `a` before re-claiming `b`, which would itself evict something.
      expect(await guard.claim('a', 'd1', deadline, testNow), isA<Duplicate>());
      expect(await guard.claim('b', 'd2', deadline, testNow), isA<Fresh>());
    });

    test('a conflict does not displace the record it conflicts with', () async {
      final guard = InMemoryReplayGuard(2);
      await guard.claim('a', 'd1', deadline, testNow);
      await guard.claim('b', 'd2', deadline, testNow);
      for (var i = 0; i < 5; i++) {
        expect(await guard.claim('a', 'other', deadline, testNow),
            isA<Conflict>());
      }
      await guard.claim(
          'c', 'd3', deadline, testNow); // evicts the LRU, which is a
      expect(await guard.claim('b', 'd2', deadline, testNow), isA<Duplicate>());
    });

    test('treats a record past its deadline as absent', () async {
      final guard = InMemoryReplayGuard();
      final soon = testNow.add(const Duration(minutes: 1));
      await guard.claim('a', 'd1', soon, testNow);
      final later = testNow.add(const Duration(minutes: 2));
      expect(await guard.claim('a', 'different', soon, later), isA<Fresh>());
    });

    test('releases an unfinished claim but not a completed one', () async {
      final guard = InMemoryReplayGuard();

      await guard.claim('a', 'd1', deadline, testNow);
      await guard.release('a', 'd1');
      expect(guard.size, 0);

      await guard.claim('b', 'd2', deadline, testNow);
      await guard.recordResponse('b', <String, dynamic>{'ok': true});
      await guard.release('b', 'd2');
      expect(guard.size, 1, reason: 'a completed claim was released');

      // Another document's cleanup must not take the key away.
      await guard.claim('c', 'd3', deadline, testNow);
      await guard.release('c', 'some-other-digest');
      expect(guard.size, 2);
    });

    test('never retains nothing', () async {
      // A guard that retains nothing answers Fresh to everything, which is a
      // silent total defeat of item 11 rather than a visible misconfiguration.
      for (final capacity in <int>[0, -1]) {
        final guard = InMemoryReplayGuard(capacity);
        await guard.claim('a', 'd1', deadline, testNow);
        expect(
          await guard.claim('a', 'd1', deadline, testNow),
          isA<Duplicate>(),
          reason: 'capacity $capacity retained no record',
        );
      }
    });

    test('purges expired records', () async {
      final guard = InMemoryReplayGuard();
      await guard.claim(
          'a', 'd1', testNow.add(const Duration(minutes: 1)), testNow);
      await guard.claim('b', 'd2', deadline, testNow);
      guard.purgeExpired(testNow.add(const Duration(minutes: 2)));
      expect(guard.size, 1);
    });
  });

  group('SPEC §4.2 / §7.2 — freshness bounds', () {
    TrustTaskDocument<Payload> at({String? issuedAt, String? expiresAt}) =>
        TrustTaskDocument<Payload>(
          id: 'req-1',
          type: consequentialSpec.typeUri,
          issuedAt: issuedAt,
          expiresAt: expiresAt,
          payload: const Payload('admin'),
        );

    test('rejects an issuedAt beyond the skew tolerance', () {
      const policy = FreshnessPolicy.defaults;
      expect(
        validateFreshness(
            at(issuedAt: '2026-01-01T00:00:30Z'), testNow, policy),
        isNull,
        reason: '30s ahead is inside the 60s skew',
      );

      final reason = validateFreshness(
          at(issuedAt: '2026-01-01T00:05:00Z'), testNow, policy);
      expect(reason, isNotNull);
      expect(reason!.code, StandardCode.malformedRequest);
      expect(reason.message, futureIssuedAt);
      // §10.4: the message must not render the consumer's clock or tolerance.
      expect(reason.message, isNot(contains('2026')));
    });

    test('rejects an expiresAt at or before issuedAt', () {
      final reason = validateFreshness(
        at(issuedAt: '2026-01-01T00:00:00Z', expiresAt: '2026-01-01T00:00:00Z'),
        testNow,
        FreshnessPolicy.defaults,
      );
      expect(reason?.message, expiryNotAfterIssuance);
    });

    test('bounds the acceptance window with maxAge', () {
      const policy = FreshnessPolicy.consequential;
      expect(
        validateFreshness(
            at(issuedAt: '2025-12-31T23:58:00Z'), testNow, policy),
        isNull,
        reason: '2 minutes old is inside the 5 minute window',
      );
      final reason = validateFreshness(
          at(issuedAt: '2025-12-31T23:00:00Z'), testNow, policy);
      expect(reason?.code, StandardCode.expired);
      expect(reason?.message, staleWireMessage);
    });

    test('refuses a document with no timestamp once a window is configured',
        () {
      const windowed = FreshnessPolicy(maxAge: defaultMaxAge);
      expect(validateFreshness(at(), testNow, windowed)?.code,
          StandardCode.expired);
      // An expiresAt is enough to place it, even with no issuedAt.
      expect(
        validateFreshness(
            at(expiresAt: '2026-01-01T01:00:00Z'), testNow, windowed),
        isNull,
      );
    });

    test('requireIssuedAt rejects a document without one', () {
      final reason = validateFreshness(
        at(expiresAt: '2026-01-01T01:00:00Z'),
        testNow,
        FreshnessPolicy.consequential,
      );
      expect(reason?.message, issuedAtRequired);
    });

    test('recordExpiry prefers expiresAt, then issuedAt plus the window', () {
      const policy = FreshnessPolicy.consequential;
      expect(
        recordExpiry(
          at(
              issuedAt: '2026-01-01T00:00:00Z',
              expiresAt: '2026-01-01T02:00:00Z'),
          policy,
          testNow,
        ),
        DateTime.utc(2026, 1, 1, 2),
      );
      expect(
        recordExpiry(at(issuedAt: '2026-01-01T00:00:00Z'), policy, testNow),
        testNow.add(defaultMaxAge),
      );
      // No window at all: the caller must not execute a consequential task.
      expect(recordExpiry(at(), FreshnessPolicy.defaults, testNow), isNull);
    });
  });

  group('canonicalization and digest', () {
    test('SHA-256 matches the published vectors', () {
      expect(
        sha256HexOfString(''),
        'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      );
      expect(
        sha256HexOfString('abc'),
        'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
      );
      expect(
        sha256HexOfString(
          'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq',
        ),
        '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1',
      );
    });

    test('orders members and ignores the whitespace of the received bytes', () {
      final a =
          canonicalJson(jsonDecode('{ "b" : 2, "a" : [ 1, {"d":4,"c":3} ] }'));
      expect(a, '{"a":[1,{"c":3,"d":4}],"b":2}');
      final b = canonicalJson(jsonDecode('{"a":[1,{"c":3,"d":4}],"b":2}'));
      expect(a, b);
    });

    test('applies only the JCS escape set', () {
      // Escapes are assembled rather than written literally, so this source file
      // carries none of the sequences the test is about.
      const bs = r'\';
      final lineSeparator = String.fromCharCode(0x2028);

      final got = canonicalJson(jsonDecode('{"k":"a<b>c&d${bs}u2028e"}'));
      expect(got, contains('a<b>c&d${lineSeparator}e'));
      expect(got, isNot(contains('${bs}u003c')));
      expect(got, isNot(contains('${bs}u2028')));

      // Control characters are escaped, and only as JCS spells them.
      final want = '{"k":"a${bs}u0001b${bs}tc"}';
      expect(canonicalJson(jsonDecode(want)), want);
    });

    test('the document digest is stable and content-sensitive', () {
      final first = documentDigest(consequentialDoc(), payloadToJson);
      expect(documentDigest(consequentialDoc(), payloadToJson), first);
      expect(
        documentDigest(
            consequentialDoc(payload: const Payload('owner')), payloadToJson),
        isNot(first),
      );
    });
  });
}
