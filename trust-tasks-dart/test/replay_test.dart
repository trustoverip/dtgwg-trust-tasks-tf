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
  DateTime? now,
}) =>
    consumeInbound<Payload, Response>(
      transport: const UnauthenticatedTransport(),
      spec: consequentialSpec,
      proofPolicy: const ProofPolicy.acceptUnverified(),
      payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      checks: checks ?? consequentialChecks(guard),
      doc: doc,
      myVid: me,
      now: now ?? testNow,
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

/// An [InMemoryReplayGuard] that, like a store, keeps only what encodes as
/// JSON, and keeps it as the decoded JSON.
class _SerialisingGuard extends InMemoryReplayGuard {
  @override
  Future<void> recordResponse(String id, Object? response) =>
      super.recordResponse(id, jsonDecode(jsonEncode(response)));
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

    test('releases the claim when the handler throws, so a resend executes',
        () async {
      final guard = InMemoryReplayGuard();
      await expectLater(
        consumeInbound<Payload, Response>(
          transport: const UnauthenticatedTransport(),
          spec: consequentialSpec,
          proofPolicy: const ProofPolicy.acceptUnverified(),
          payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
          checks: consequentialChecks(guard),
          doc: consequentialDoc(),
          myVid: me,
          now: testNow,
          newErrorId: () => 'err-1',
          payloadToJson: payloadToJson,
          handler: (accepted, parties) async =>
              throw StateError('database connection reset'),
        ),
        throwsStateError,
      );

      // The handler never finished, so a resend is a fresh attempt, not an
      // in-flight duplicate held until the record expires.
      final executions = Executions();
      final outcome =
          await consequential(guard, consequentialDoc(), executions);
      expect(outcome, isA<Handled<Response>>());
      expect(executions.count, 1);
    });

    test('settles the claim when the response encoder throws', () async {
      // The handler returned, so the effect happened: the claim is settled
      // (with no response to hand back) rather than left in flight until the
      // record expires, and the encoder's error reaches the caller.
      final guard = InMemoryReplayGuard();
      final executions = Executions();
      Future<ConsumeOutcome<Response>> run() =>
          consumeInbound<Payload, Response>(
            transport: const UnauthenticatedTransport(),
            spec: consequentialSpec,
            proofPolicy: const ProofPolicy.acceptUnverified(),
            payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
            checks: consequentialChecks(guard),
            doc: consequentialDoc(),
            myVid: me,
            now: testNow,
            newErrorId: () => 'err-1',
            payloadToJson: payloadToJson,
            clock: fixedClock,
            responseToJson: (r) => throw StateError('cannot encode'),
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

      await expectLater(run(), throwsStateError);

      final resend = await run();
      expect(resend, isA<DuplicateOutcome<Response>>());
      final duplicate = resend as DuplicateOutcome<Response>;
      expect(duplicate.inFlight, isFalse,
          reason: 'the claim was left in flight');
      expect(duplicate.priorResponse, isNull);
      expect(executions.count, 1,
          reason: 'the consequential effect happened twice');
    });

    test('caches a success response as JSON a persistent guard can store',
        () async {
      // Stands in for a guard backed by a store: it keeps only what
      // jsonEncode can write, as Redis or a database would.
      final guard = _SerialisingGuard();
      Future<ConsumeOutcome<Response>> run() =>
          consumeInbound<Payload, Response>(
            transport: const UnauthenticatedTransport(),
            spec: consequentialSpec,
            proofPolicy: const ProofPolicy.acceptUnverified(),
            payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
            checks: consequentialChecks(guard),
            doc: consequentialDoc(),
            myVid: me,
            now: testNow,
            newErrorId: () => 'err-1',
            payloadToJson: payloadToJson,
            clock: fixedClock,
            responseToJson: (r) => r.toJson(),
            handler: (accepted, parties) async =>
                respondWith<Payload, Response>(
              accepted,
              'resp-1',
              const Response(true),
              clock: fixedClock,
            ),
          );

      expect(await run(), isA<Handled<Response>>());
      final second = await run();
      expect(second, isA<DuplicateOutcome<Response>>());
      final prior = (second as DuplicateOutcome<Response>).priorResponse;
      expect(prior, isA<Map<String, dynamic>>());
      expect((prior! as Map<String, dynamic>)['id'], 'resp-1');
      expect((prior as Map<String, dynamic>)['payload'], {'ok': true});
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

    test('refuses a new document when full of live records, dropping none',
        () async {
      final guard = InMemoryReplayGuard(2, 2);
      final later = testNow.add(const Duration(days: 30));
      await guard.claim('a', 'd1', later, testNow);
      await guard.claim('b', 'd2', null, testNow);
      await expectLater(
        guard.claim('c', 'd3', later, testNow),
        throwsA(isA<ReplayGuardFullException>()),
      );

      // Both records survive, and duplicates are still recognised while full.
      expect(guard.size, 2);
      expect(await guard.claim('a', 'd1', later, testNow), isA<Duplicate>());
      expect(await guard.claim('b', 'd2', null, testNow), isA<Duplicate>());
    });

    test('reclaims expired records first when full', () async {
      final guard = InMemoryReplayGuard(2, 2);
      final hour = testNow.add(const Duration(hours: 1));
      await guard.claim(
          'a', 'd1', testNow.add(const Duration(minutes: 5)), testNow);
      await guard.claim('b', 'd2', hour, testNow);
      final afterA = testNow.add(const Duration(minutes: 10));
      expect(await guard.claim('c', 'd3', hour, afterA), isA<Fresh>());
      expect(guard.size, 2);
      expect(await guard.claim('b', 'd2', hour, afterA), isA<Duplicate>());
    });

    test('no one issuer holds more than its share; only it is refused',
        () async {
      final guard = InMemoryReplayGuard(10); // a tenth of 10 is 1
      final month = testNow.add(const Duration(days: 30));
      expect(await guard.claimFor('did:example:a', 'a1', 'd', month, testNow),
          isA<Fresh>());
      await expectLater(
        guard.claimFor('did:example:a', 'a2', 'd', month, testNow),
        throwsA(isA<ReplayGuardFullException>()
            .having((e) => e.issuerShare, 'issuerShare', isTrue)),
      );
      expect(await guard.claimFor('did:example:a', 'a1', 'd', month, testNow),
          isA<Duplicate>());
      expect(await guard.claimFor('did:example:b', 'b1', 'd', month, testNow),
          isA<Fresh>());
      expect(
          await guard.claimFor(null, 'n1', 'd', month, testNow), isA<Fresh>());
      expect(
          await guard.claimFor('did:example:a', 'a2', 'd', null,
              month.add(const Duration(seconds: 1))),
          isA<Fresh>());
      final wide = InMemoryReplayGuard(10, 3);
      for (final id in ['x1', 'x2', 'x3']) {
        await wide.claimFor('did:example:a', id, 'd', null, testNow);
      }
      await expectLater(
        wide.claimFor('did:example:a', 'x4', 'd', null, testNow),
        throwsA(isA<ReplayGuardFullException>()),
      );
    });

    test('a full guard answers unavailable through the pipeline', () async {
      final guard = InMemoryReplayGuard(1);
      final executions = Executions();
      expect(await consequential(guard, consequentialDoc(), executions),
          isA<Handled<Response>>());
      final refused = await consequential(
          guard, consequentialDoc().copyWith(id: 'req-2'), executions);
      expect(refused, isA<Rejected<Response>>());
      final error = (refused as Rejected<Response>).error;
      expect(error.payload.code, StandardCode.unavailable);
      expect(error.payload.retryable, isTrue);
      // The first document's record was kept: a resend is still absorbed.
      expect(await consequential(guard, consequentialDoc(), executions),
          isA<DuplicateOutcome<Response>>());
      expect(executions.count, 1);
    });

    test('a conflict does not displace the record it conflicts with', () async {
      final guard = InMemoryReplayGuard(2, 2);
      await guard.claim('a', 'd1', deadline, testNow);
      await guard.claim('b', 'd2', deadline, testNow);
      for (var i = 0; i < 5; i++) {
        expect(await guard.claim('a', 'other', deadline, testNow),
            isA<Conflict>());
      }
      // Full: refused, nothing dropped.
      await expectLater(guard.claim('c', 'd3', deadline, testNow),
          throwsA(isA<ReplayGuardFullException>()));
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

    test('refuses a timestamp with no zone designator, as Rust and Go do', () {
      // DateTime.tryParse reads "00:00:00" with no zone as local time, so the
      // instant would depend on where the consumer runs.
      const policy = FreshnessPolicy.defaults;
      for (final zoneless in [
        '2026-01-01T00:00:00',
        '2026-01-01',
        '2026-01-01 00:00:00Z',
        '20260101T000000Z',
      ]) {
        expect(
          validateFreshness(at(issuedAt: zoneless), testNow, policy)?.code,
          StandardCode.malformedRequest,
          reason: 'issuedAt $zoneless',
        );
        expect(
          validateBasic(at(expiresAt: zoneless), testNow, me)?.code,
          StandardCode.malformedRequest,
          reason: 'expiresAt $zoneless',
        );
      }
      for (final zoned in [
        '2026-01-01T00:00:00Z',
        '2026-01-01t00:00:00.250z',
        '2026-01-01T01:00:00+01:00',
      ]) {
        expect(validateFreshness(at(issuedAt: zoned), testNow, policy), isNull,
            reason: zoned);
      }
    });

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

    test(
        'closes the acceptance window at the instant the record may be dropped',
        () async {
      // The window is half-open and ends where the record does. At exactly
      // `recordExpiry` the guard treats the record as absent, so the document
      // must already be refused; one millisecond earlier it is still a
      // duplicate.
      final guard = InMemoryReplayGuard();
      final executions = Executions();
      final end = recordExpiry(
        consequentialDoc(),
        consequentialChecks(guard).freshness,
        testNow,
      )!;
      expect(end, testNow.add(const Duration(minutes: 6)));

      await consequential(guard, consequentialDoc(), executions);

      final justBefore = await consequential(
        guard,
        consequentialDoc(),
        executions,
        now: end.subtract(const Duration(milliseconds: 1)),
      );
      expect(justBefore, isA<DuplicateOutcome<Response>>());

      final atEnd = await consequential(
        guard,
        consequentialDoc(),
        executions,
        now: end,
      );
      expect(atEnd, isA<Rejected<Response>>());
      expect((atEnd as Rejected<Response>).error.payload.code,
          StandardCode.expired);
      expect(executions.count, 1);
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
        testNow.add(defaultMaxAge + defaultSkew),
      );
      // No window at all: the caller must not execute a consequential task.
      expect(recordExpiry(at(), FreshnessPolicy.defaults, testNow), isNull);
    });

    test('a stated expiresAt replaces maxAge, as in Rust', () {
      // Ten minutes old with an hour left.
      expect(
        validateFreshness(
          at(
              issuedAt: '2025-12-31T23:50:00Z',
              expiresAt: '2026-01-01T01:00:00Z'),
          testNow,
          FreshnessPolicy.consequential,
        ),
        isNull,
      );
    });

    test('a distant expiresAt is clamped to the expiry horizon', () {
      const policy = FreshnessPolicy.consequential;
      final d = at(
          issuedAt: '2026-01-01T00:00:00Z', expiresAt: '9999-12-31T23:59:59Z');
      final end = DateTime.utc(2026, 1, 1).add(defaultExpiryHorizon);
      // The record is not pinned for the producer's millennia.
      expect(recordExpiry(d, policy, testNow), end.add(policy.skew));
      // Accepted inside the horizon (plus skew), refused after.
      expect(validateFreshness(d, end.add(policy.skew), policy), isNull);
      expect(
        validateFreshness(
                d, end.add(policy.skew + const Duration(seconds: 1)), policy)
            ?.code,
        StandardCode.expired,
      );
      // A zero horizon clamps at issuedAt; only null means none.
      const immediate =
          FreshnessPolicy(maxAge: defaultMaxAge, expiryHorizon: Duration.zero);
      expect(recordExpiry(d, immediate, testNow),
          DateTime.utc(2026, 1, 1).add(defaultSkew));
      // With no horizon the stated expiresAt stands.
      const unbounded = FreshnessPolicy(maxAge: defaultMaxAge);
      expect(recordExpiry(d, unbounded, testNow)?.year, 9999);
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
