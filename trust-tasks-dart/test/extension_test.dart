// Forward-minor reading along a declared `extends` chain (SPEC §5.2,
// "Declared extension").
//
// These tests pass a constructed index to `readForVersion`, which takes one
// for exactly this reason, and read a real generated library — the current
// auth/authenticate, 0.3 — with documents of a 0.4 and a 0.5 that the index
// declares as extending it, each additive over the one before (0.4 adds an
// optional `deviceHint`, 0.5 a `futureMember`). Neither newer version is
// published: the reading is driven by the index and the base library alone,
// so nothing here depends on a spec edit, and anchoring on the current version
// keeps the test off any version due for withdrawal.

import 'package:test/test.dart';
import 'package:trust_tasks/specs/auth/authenticate/v0_3/payload.dart'
    as auth03;
import 'package:trust_tasks/specs/extends_index.dart';
import 'package:trust_tasks/trust_tasks.dart';

const me = 'did:web:auth.example';
const peer = 'did:key:z6MkPeer';
const base = 'https://trusttasks.org/spec/auth/authenticate/';

/// 0.5 extends 0.4 extends 0.3.
const chained = <String, String>{
  '${base}0.4': '0.3',
  '${base}0.5': '0.4',
};

/// 0.5 extends 0.4, which declares nothing: 0.5 does not reach 0.3.
const broken = <String, String>{'${base}0.5': '0.4'};

final testNow = DateTime.utc(2026, 1, 1);
String fixedClock() => '2026-01-01T00:00:00Z';

/// A stand-in Data Integrity verifier whose "signature" is the digest of the
/// document without its proof — so it verifies exactly when the bytes it is
/// handed are the bytes that were signed, which is the property under test.
class _DigestVerifier implements ProofVerifier {
  const _DigestVerifier();
  @override
  Future<bool> verify(Map<String, dynamic> doc) async {
    final unsigned = Map<String, dynamic>.of(doc)..remove('proof');
    final proof = doc['proof'] as Map<String, dynamic>;
    return proof['proofValue'] == sha256HexOfString(canonicalJson(unsigned));
  }
}

Map<String, dynamic> sign(Map<String, dynamic> doc) => <String, dynamic>{
      ...doc,
      'proof': <String, dynamic>{
        'type': 'DataIntegrityProof',
        'cryptosuite': 'eddsa-jcs-2022',
        'verificationMethod': '$peer#key-1',
        'created': '2026-01-01T00:00:00Z',
        'proofPurpose': 'authentication',
        'proofValue': sha256HexOfString(canonicalJson(doc)),
      },
    };

/// A stand-in item-2 validator: refuses a top-level payload member the schema's
/// root `properties` do not name, as `additionalProperties: false` would.
class _ClosedRoot implements PayloadValidator {
  const _ClosedRoot();
  @override
  String? validate(String schemaJson, Object? payload) {
    final known =
        RegExp(r'"properties":\{"challenge".*?"sessionId"').hasMatch(schemaJson)
            ? const {
                'challenge',
                'delegationEvidence',
                'ext',
                'principal',
                'scope',
                'sessionId',
                'sessionKey',
              }
            : null;
    if (known == null) return 'unexpected schema';
    final extra = (payload as Map<String, dynamic>)
        .keys
        .where((k) => !known.contains(k))
        .toList();
    return extra.isEmpty ? null : 'unknown member(s) ${extra.join(', ')}';
  }
}

Map<String, dynamic> authDoc(
  String version, {
  String id = 'req-1',
  String deviceHint = 'hint-1',
}) =>
    <String, dynamic>{
      'id': id,
      'type': '$base$version',
      'issuer': peer,
      'recipient': me,
      'issuedAt': '2025-12-31T23:59:00Z',
      'payload': <String, dynamic>{
        'challenge': 'c-1',
        'sessionId': 's-1',
        if (version != '0.3') 'deviceHint': deviceHint,
        if (version == '0.5') 'futureMember': <String, dynamic>{'x': 1},
      },
    };

auth03.Payload fromJson(Object? p) =>
    auth03.Payload.fromJson(p as Map<String, dynamic>);

Future<ConsumeOutcome<auth03.Response>> consume(
  VersionAccepted<auth03.Payload> read, {
  bool passReceived = true,
  ConsumeChecks? checks,
  Map<String, dynamic>? received,
}) =>
    consumeInbound<auth03.Payload, auth03.Response>(
      transport: const UnauthenticatedTransport(),
      spec: auth03.spec,
      proofPolicy: const ProofPolicy.verify(_DigestVerifier()),
      payloadPolicy: const PayloadPolicy.validate(_ClosedRoot()),
      checks: checks ?? notConsequentialChecks(),
      doc: read.doc,
      received: passReceived ? (received ?? read.received) : null,
      myVid: me,
      now: testNow,
      newErrorId: () => 'err-1',
      payloadToJson: (p) => p.toJson(),
      clock: fixedClock,
      handler: (accepted, parties) async => respondWith(
        accepted,
        'resp-1',
        auth03.Response.fromJson(<String, dynamic>{
          'session': <String, dynamic>{
            'id': 's-1',
            'subject': peer,
            'issuedAt': '2026-01-01T00:00:00Z',
            'expiresAt': '2026-01-01T01:00:00Z',
          },
          'tokens': <String, dynamic>{
            'accessToken': 'a',
            'tokenType': 'Bearer',
            'expiresIn': 3600,
          },
        }),
        clock: fixedClock,
      ),
    );

VersionAccepted<auth03.Payload> accepted(VersionRead<auth03.Payload> read) {
  expect(read, isA<VersionAccepted<auth03.Payload>>());
  return read as VersionAccepted<auth03.Payload>;
}

RejectReason refused(VersionRead<auth03.Payload> read) {
  expect(read, isA<VersionRefused<auth03.Payload>>());
  return (read as VersionRefused<auth03.Payload>).reason;
}

void main() {
  group('generated metadata', () {
    test('every library carries its direct extends declaration', () {
      expect(auth03.extendsVersion, isNull);
    });

    test('the index maps bare request Type URIs to MAJOR.MINOR', () {
      for (final entry in extendsIndex.entries) {
        expect(entry.key, startsWith('https://trusttasks.org/spec/'));
        expect(entry.key, isNot(contains('#')));
        expect(entry.value, matches(RegExp(r'^\d+\.\d+$')));
      }
    });
  });

  group('extensionChain / extendsTo', () {
    test('walks the chain, nearest first', () {
      expect(extensionChain('${base}0.5', index: chained), ['0.4', '0.3']);
      expect(
          extensionChain('${base}0.5#request', index: chained), ['0.4', '0.3']);
      expect(extensionChain('${base}0.3', index: chained), isEmpty);
      expect(extendsTo('${base}0.5', '${base}0.3', index: chained), isTrue);
      expect(extendsTo('${base}0.5', '${base}0.3', index: broken), isFalse);
      expect(extendsTo('${base}0.3', '${base}0.3', index: chained), isFalse);
    });

    test('a cycle ends the walk', () {
      const cyclic = <String, String>{'${base}0.4': '0.3', '${base}0.3': '0.4'};
      expect(extensionChain('${base}0.4', index: cyclic), ['0.3']);
    });

    test('another slug at the same version is never reached', () {
      const index = <String, String>{
        'https://trusttasks.org/spec/auth/other/0.4': '0.3',
      };
      expect(
        extendsTo('https://trusttasks.org/spec/auth/other/0.4', '${base}0.3',
            index: index),
        isFalse,
      );
    });
  });

  group('readForVersion', () {
    test('the implemented version is read unchanged', () async {
      final json = sign(authDoc('0.3'));
      final read =
          accepted(readForVersion(json, auth03.spec, fromJson, index: chained));
      expect(read.forward, isFalse);
      expect(read.doc.type, auth03.typeUri);
      expect(identical(read.received, json), isTrue);

      final outcome = await consume(read);
      expect(outcome, isA<Handled<auth03.Response>>());
      expect((outcome as Handled<auth03.Response>).response.type,
          auth03.responseTypeUri);
    });

    test('an extended version is routed, pruned and answered as received',
        () async {
      final json = sign(authDoc('0.4'));
      final before = canonicalJson(json);
      final read =
          accepted(readForVersion(json, auth03.spec, fromJson, index: chained));

      expect(read.forward, isTrue);
      // The type is the received one, not rewritten to 0.3.
      expect(read.doc.type, '${base}0.4');
      // The member 0.3 does not know is gone from the typed read...
      expect(read.doc.payload.toJson().containsKey('deviceHint'), isFalse);
      // ...and still there, untouched, in what was received.
      expect(identical(read.received, json), isTrue);
      expect(canonicalJson(json), before);

      final outcome = await consume(read);
      expect(outcome, isA<Handled<auth03.Response>>());
      // Answered under the producer's version.
      expect((outcome as Handled<auth03.Response>).response.type,
          '${base}0.4#response');
    });

    test('a chain of two links is followed', () async {
      final json = sign(authDoc('0.5'));
      final read =
          accepted(readForVersion(json, auth03.spec, fromJson, index: chained));
      expect(read.forward, isTrue);
      expect(read.doc.payload.toJson().keys,
          unorderedEquals(['challenge', 'sessionId']));
      final outcome = await consume(read);
      expect((outcome as Handled<auth03.Response>).response.type,
          '${base}0.5#response');
    });

    test('a version whose chain does not reach is unsupportedVersion', () {
      final json = authDoc('0.5');
      expect(
        refused(readForVersion(json, auth03.spec, fromJson, index: broken))
            .code,
        StandardCode.unsupportedVersion,
      );
      // And with no declaration at all, which is today's generated index.
      final read = readForVersion(authDoc('0.4'), auth03.spec, fromJson,
          index: const <String, String>{});
      expect(refused(read).code, StandardCode.unsupportedVersion);
      // The refusal carries a document to route the error response from.
      final document = (read as VersionRefused<auth03.Payload>).document;
      final error = reject(
          const UnauthenticatedTransport(), document, 'err-1', read.reason,
          clock: fixedClock)!;
      expect(error.recipient, peer);
      expect(error.payload.inResponseTo!.typeUri, '${base}0.4');
    });

    test('another specification is unsupportedType', () {
      final json = authDoc('0.4')
        ..['type'] = 'https://trusttasks.org/spec/auth/refresh/0.2';
      expect(
        refused(readForVersion(json, auth03.spec, fromJson, index: chained))
            .code,
        StandardCode.unsupportedType,
      );
    });

    test('a request is not read as a response', () {
      expect(
        refused(readForVersion(authDoc('0.4'), auth03.responseSpec, fromJson,
                index: chained))
            .code,
        StandardCode.unsupportedType,
      );
    });

    test('an explicit #request fragment reads as the request variant', () {
      final json = authDoc('0.4')..['type'] = '${base}0.4#request';
      expect(
        accepted(readForVersion(json, auth03.spec, fromJson, index: chained))
            .forward,
        isTrue,
      );
    });
  });

  group('proofs stay over the document as received', () {
    test('a signed extended document verifies', () async {
      final read = accepted(readForVersion(
          sign(authDoc('0.4')), auth03.spec, fromJson,
          index: chained));
      expect(await consume(read), isA<Handled<auth03.Response>>());
    });

    test('without received, the pruned re-encoding fails to verify', () async {
      // The reason `received` exists: the typed document is not what was signed.
      final read = accepted(readForVersion(
          sign(authDoc('0.4')), auth03.spec, fromJson,
          index: chained));
      final outcome = await consume(read, passReceived: false);
      expect(outcome, isA<Rejected<auth03.Response>>());
      expect((outcome as Rejected<auth03.Response>).error.payload.code,
          'proofInvalid');
    });

    test('a received document that is not the one consumed is proofInvalid',
        () async {
      final read = accepted(readForVersion(
          sign(authDoc('0.4')), auth03.spec, fromJson,
          index: chained));
      final other = sign(authDoc('0.4', id: 'req-2'));
      final outcome = await consume(read, received: other);
      expect((outcome as Rejected<auth03.Response>).error.payload.code,
          'proofInvalid');
    });

    test('a tampered pruned member still breaks the proof', () async {
      final json = sign(authDoc('0.4'));
      (json['payload'] as Map<String, dynamic>)['deviceHint'] = 'tampered';
      final read =
          accepted(readForVersion(json, auth03.spec, fromJson, index: chained));
      final outcome = await consume(read);
      expect((outcome as Rejected<auth03.Response>).error.payload.code,
          'proofInvalid');
    });
  });

  group('item 11 keys a forward document on what arrived', () {
    test('two documents differing only in a pruned member conflict', () async {
      final guard = InMemoryReplayGuard();
      final first = accepted(readForVersion(
          sign(authDoc('0.4')), auth03.spec, fromJson,
          index: chained));
      expect(await consume(first, checks: consequentialChecks(guard)),
          isA<Handled<auth03.Response>>());

      final second = accepted(readForVersion(
          sign(authDoc('0.4', deviceHint: 'hint-2')), auth03.spec, fromJson,
          index: chained));
      final outcome = await consume(second, checks: consequentialChecks(guard));
      expect((outcome as Rejected<auth03.Response>).error.payload.code,
          'idConflict');
    });

    test('a bit-for-bit resend is a duplicate', () async {
      final guard = InMemoryReplayGuard();
      final json = sign(authDoc('0.4'));
      final read =
          accepted(readForVersion(json, auth03.spec, fromJson, index: chained));
      await consume(read, checks: consequentialChecks(guard));
      final again = accepted(readForVersion(
          Map<String, dynamic>.of(json), auth03.spec, fromJson,
          index: chained));
      expect(await consume(again, checks: consequentialChecks(guard)),
          isA<DuplicateOutcome<auth03.Response>>());
    });
  });

  group('prunePayload', () {
    const schema = '''
{
  "type": "object",
  "additionalProperties": false,
  "required": ["kind"],
  "properties": {
    "kind": {"type": "string"},
    "inner": {"\$ref": "#/\$defs/Inner"},
    "list": {"type": "array", "items": {"\$ref": "#/\$defs/Inner"}},
    "open": {"type": "object", "properties": {"a": {"type": "string"}}},
    "map": {"type": "object", "additionalProperties": {"\$ref": "#/\$defs/Inner"}},
    "choice": {"oneOf": [
      {"type": "object", "additionalProperties": false, "required": ["a"],
       "properties": {"a": {"const": "x"}}},
      {"type": "object", "additionalProperties": false, "required": ["b"],
       "properties": {"b": {"type": "string"}}}
    ]},
    "ambiguous": {"anyOf": [
      {"type": "object", "additionalProperties": false, "properties": {"a": {}}},
      {"type": "object", "additionalProperties": false, "properties": {"b": {}}}
    ]}
  },
  "\$defs": {
    "Inner": {"type": "object", "additionalProperties": false,
              "properties": {"keep": {"type": "string"}}}
  }
}''';

    test('removes only what a closed object does not know', () {
      final payload = <String, dynamic>{
        'kind': 'k',
        'extra': 1,
        'inner': {'keep': 'y', 'drop': 2},
        'list': [
          {'keep': 'a', 'drop': 1},
          {'drop': 2},
        ],
        'open': {'a': 'b', 'stays': true},
        'map': {
          'k1': {'keep': 'v', 'drop': 3},
        },
        'choice': {'b': 'only-b-fits', 'drop': 4},
        'ambiguous': {'a': 1, 'b': 2, 'c': 3},
      };
      final before = canonicalJson(payload);
      expect(prunePayload(payload, schema), <String, dynamic>{
        'kind': 'k',
        'inner': {'keep': 'y'},
        'list': [
          {'keep': 'a'},
          <String, dynamic>{},
        ],
        'open': {'a': 'b', 'stays': true},
        'map': {
          'k1': {'keep': 'v'},
        },
        'choice': {'b': 'only-b-fits'},
        // Both branches plausible: left untouched rather than guessed at.
        'ambiguous': {'a': 1, 'b': 2, 'c': 3},
      });
      // The input is not modified.
      expect(canonicalJson(payload), before);
    });

    test('follows a response schema wrapper', () {
      final pruned = prunePayload(<String, dynamic>{
        'session': {
          'id': 's',
          'subject': peer,
          'issuedAt': 'a',
          'expiresAt': 'b',
          'deviceHint': 'hint',
        },
        'tokens': {'accessToken': 'a', 'tokenType': 'Bearer', 'expiresIn': 1},
        'newer': true,
      }, auth03.responsePayloadSchemaJson) as Map<String, dynamic>;
      expect(pruned.containsKey('newer'), isFalse);
      expect((pruned['session'] as Map).containsKey('deviceHint'), isFalse);
    });

    test('a value that is not the schema\'s shape is left alone', () {
      expect(prunePayload('a string', schema), 'a string');
      expect(prunePayload(null, schema), isNull);
    });
  });
}
