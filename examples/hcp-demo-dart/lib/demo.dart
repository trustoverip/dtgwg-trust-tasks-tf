// The Human Context Protocol (HCP) carried as Trust Tasks, end to end, in one
// process. A DEMONSTRATION, not a reference provider: see README.md and
// ../hcp-demo-ts/SCENARIO.md (the language-neutral script every port of this
// demo follows).
//
// Three parties, each with its own did:key and TSP keys:
//   - the HCP Provider (a toy, in-memory: a few entries, a grant table);
//   - a Context Client (a booking agent that wants the person's context);
//   - a Management Client (the person's own app, which can revoke grants).
//
// Every document is built with the generated types, signed with
// package:trust_tasks_proof, sealed with package:trust_tasks_tsp (a Direct TSP
// message: HPKE-sealed, signed), opened, and run through the SPEC §7.2 pipeline
// (proof verified over the document as received, payload validated against its
// schema) before anything acts on it. Nothing touches the network.

import 'dart:math' as math;
import 'dart:typed_data';

import 'package:affinidi_tsp/affinidi_tsp.dart';
import 'package:ssi/ssi.dart' as ssi;
// Framework specifications ship with the runtime package, not the generator.
import 'package:trust_tasks/specs/trust_task_discovery/v0_4/payload.dart'
    as discovery;
import 'package:trust_tasks/specs/trust_task_next_step/v0_1/payload.dart'
    as next_step;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';
import 'package:trust_tasks_tsp/trust_tasks_tsp.dart';

import 'trust_tasks/hcp/entries/query/v0_1/payload.dart' as query;
import 'trust_tasks/hcp/grants/request/v0_1/payload.dart' as grant_request;
import 'trust_tasks/hcp/grants/revoke/v0_1/payload.dart' as revoke;
import 'trust_tasks/hcp/grants/revoked/v0_1/payload.dart' as revoked;
import 'validator.dart';

typedef Json = Map<String, dynamic>;

// ── Deterministic inputs ─────────────────────────────────────────────────────
// A fixed clock the demo controls: it starts at [start] and moves one second
// per document issued. Instants are whole seconds (`…Z`), never the system time.
const String start = '2026-11-25T10:00:00Z';

/// `YYYY-MM-DDTHH:MM:SSZ`, the whole-second form every instant here takes.
String instant(DateTime t) =>
    '${t.toUtc().toIso8601String().substring(0, 19)}Z';

class Clock {
  DateTime _t = DateTime.parse(start);
  DateTime now() => _t;
  String iso() => instant(_t);
  String tick() {
    _t = _t.add(const Duration(seconds: 1));
    return iso();
  }
}

/// Sequential urn:uuid identifiers: …-000000000001, …-000000000002, …
class Ids {
  int _n = 0;
  String next() =>
      'urn:uuid:00000000-0000-4000-8000-${(++_n).toRadixString(16).padLeft(12, '0')}';
}

// ── Parties ──────────────────────────────────────────────────────────────────
// DEMO ONLY keys: each is 32 bytes of one repeated value. Never use such keys
// for anything real. The VID is the did:key of the Ed25519 signing key, and
// the same VID names the party in TSP.
final class Party {
  Party._(this.name, this.signer, this.tsp, this.public);

  final String name;
  final ssi.DidSigner signer;

  /// This party's TSP identity: its Ed25519 signing and X25519 decryption keys.
  final PrivateVid tsp;

  /// What every other party knows of it up front (no DID resolution).
  final PublicVid public;

  String get vid => signer.did;
}

Future<Party> _party(String name, int signingSeed, int encryptionSeed) async {
  final sigSeed = Uint8List.fromList(List.filled(32, signingSeed)); // DEMO ONLY
  final encSecret = Uint8List.fromList(
    List.filled(32, encryptionSeed),
  ); // DEMO ONLY
  final keyPair = ssi.Ed25519KeyPair.fromSeed(sigSeed);
  final didDoc = ssi.DidKey.generateDocument(keyPair.publicKey);
  final signer = ssi.DidSigner(
    did: didDoc.id,
    didKeyId: didDoc.verificationMethod[0].id,
    keyPair: keyPair,
    signatureScheme: ssi.SignatureScheme.ed25519,
  );
  final signing = Ed25519SigningKey.fromSeed(sigSeed);
  final decryption = X25519DecryptionKey.fromSecret(encSecret);
  return Party._(
    name,
    signer,
    PrivateVid(id: signer.did, signingKey: signing, decryptionKey: decryption),
    PublicVid(
      id: signer.did,
      verificationKey: await signing.verificationKey(),
      encryptionKey: X25519EncryptionKey(decryption.publicKey),
    ),
  );
}

// ── The toy provider's data ──────────────────────────────────────────────────
const String purpose = 'https://purposes.example/travel-booking';

/// DEMO ONLY: issued to the person's app at setup.
const String managementToken = 'hcp_mt_demo_management';

/// One fixed text for every refusal (HCP 5.5).
const String _neutralDenial = 'The request is not authorized.';

/// The person's context, as the toy provider stores it (HCP 4.2, much
/// simplified).
const List<({String ref, String category, String value})> _entries = [
  (
    ref: 'hcp-ref-0001',
    category: 'travel_preferences',
    value: 'Prefers aisle seats and early-morning flights.',
  ),
  (
    ref: 'hcp-ref-0002',
    category: 'travel_preferences',
    value: 'Avoids overnight layovers.',
  ),
  (
    ref: 'hcp-ref-0003',
    category: 'health_context',
    value: 'Severe peanut allergy.',
  ),
];

final class Grant {
  Grant({
    required this.grantId,
    required this.token,
    required this.client,
    required this.categories,
    required this.purposes,
  });

  final String grantId;
  final String token;
  final String client;
  final List<String> categories;
  final List<String> purposes;
  String status = 'active'; // 'active' | 'revoked'
}

/// The SpecPolicy each receiving party applies, by bare Type URI.
const Map<String, SpecPolicy> _specs = {
  discovery.typeUri: discovery.spec,
  grant_request.typeUri: grant_request.spec,
  query.typeUri: query.spec,
  revoke.typeUri: revoke.spec,
  revoked.typeUri: revoked.spec,
};

// ── The world: three parties, one transport, one clock ───────────────────────
final class World {
  World._(this.provider, this.contextClient, this.management) {
    final resolver = StaticVidResolver([
      for (final p in [provider, contextClient, management]) p.public,
    ]);
    // One TSP consumer per receiving party: its replay guard is that party's
    // duplicate-execution record.
    for (final me in [provider, contextClient]) {
      _consumers[me.vid] = TspConsumer(
        recipient: me.tsp,
        resolver: resolver,
        proofVerifier: _verifier,
        payloadValidator: const SchemaValidator(),
        clock: clock.now,
      );
    }
  }

  static Future<World> create() async => World._(
    await _party('Provider', 0x11, 0x12),
    await _party('Context Client', 0x21, 0x22),
    await _party('Management Client', 0x31, 0x32),
  );

  final Clock clock = Clock();
  final Ids ids = Ids();
  final Party provider;
  final Party contextClient;
  final Party management;
  final StringBuffer transcript = StringBuffer();
  final List<Grant> grants = [];

  /// Grant requests the person has not decided yet, by threadId (SPEC §8.6
  /// hold).
  final Map<String, ({grant_request.Payload payload, bool decided})> held = {};
  int _tokens = 0;

  late final DataIntegrityProofVerifier _verifier =
      DataIntegrityProofVerifier.forDidKey(clock: clock.now);
  final Map<String, TspConsumer> _consumers = {};

  Party byVid(String vid) => [provider, contextClient, management].firstWhere(
    (p) => p.vid == vid,
    orElse: () => throw StateError('unknown VID $vid'),
  );

  /// Sign [doc] as [by], `created` equal to the clock (= its `issuedAt`).
  Future<Json> sign(Party by, Json doc) =>
      signTrustTask(doc, by.signer, created: clock.now());

  /// Build and sign a request; party members and issuedAt are set BEFORE
  /// signing.
  Future<Json> request(
    Party from,
    Party to,
    String type,
    Object? payload, {
    String? threadId,
  }) {
    final id = ids.next();
    final doc = TrustTaskDocument<Object?>(
      id: id,
      type: type,
      issuer: from.vid,
      recipient: to.vid,
      issuedAt: clock.tick(),
      threadId: threadId ?? id,
      payload: payload,
    );
    return sign(from, doc.toJson((p) => p));
  }

  /// Seal [doc] over TSP from → to, let `to` consume it, open the sealed reply.
  Future<Json> exchange(Party from, Party to, Json doc) async {
    _print(from, to, doc);
    final wire = await packTrustTask(
      doc,
      sender: from.tsp,
      receiver: to.public,
    );

    // ── receiving side ──
    final consumer = _consumers[to.vid]!;
    final sender = byVid(advertisedSender(wire)!);
    final unpacked = await unpackTrustTask(
      wire,
      recipient: to.tsp,
      sender: sender.public,
    );
    final type = bareTypeUri(unpacked.document['type'] as String);
    final spec = _specs[type];
    if (spec == null) throw StateError('no handler for $type');
    final received = await consumer.consume<Json, Object?>(
      unpacked,
      spec: spec,
      // The payload stays JSON through the pipeline; the handler reads it with
      // the generated type once the schema has accepted it.
      decode: (p) => p,
      encode: (p) => p,
      encodeResponse: (r) => r,
      handler: (accepted, _) => _handle(to, accepted),
    );
    if (received.reply == null) {
      throw StateError('no reply: ${received.outcome.runtimeType}');
    }

    // ── back at the sender: open the sealed reply and check it before use ──
    final replyWire = await consumer.packReply(received);
    final opened = await unpackTrustTask(
      replyWire,
      recipient: from.tsp,
      sender: to.public,
    );
    final reply = opened.document;
    if (reply['proof'] != null && !await _verifier.verify(reply)) {
      throw StateError('reply proof does not verify');
    }
    validateReply(reply);
    _print(to, from, reply);
    return reply;
  }

  // ── Handlers: what each receiving party does with an accepted document ──
  Future<TrustTaskDocument<Object?>> _handle(
    Party me,
    TrustTaskDocument<Json> doc,
  ) async {
    // A reply the handler signs itself: the library passes it through as built.
    Future<TrustTaskDocument<Object?>> signed(Json reply) async =>
        TrustTaskDocument<Object?>.fromJson(await sign(me, reply), (p) => p);
    Future<TrustTaskDocument<Object?>> reply(Object? payload) => signed(
      respondWith<Json, Object?>(
        doc,
        ids.next(),
        payload,
        clock: clock.tick,
      ).toJson((p) => p),
    );
    Future<Never> refuse(String code, String message) async {
      final error = rejectWith(
        doc,
        ids.next(),
        ErrorPayload(code: code, message: message, retryable: false),
        clock: clock.tick,
      );
      final json = await sign(me, error.toJson((p) => p.toJson()));
      throw Refusal(
        ErrorResponse.fromJson(json, (p) => ErrorPayload.fromJson(p! as Json)),
      );
    }

    switch (bareTypeUri(doc.type)) {
      // Step 1 — trust-task-discovery: what the provider serves (SPEC §10).
      case discovery.typeUri:
        return reply(
          const discovery.Response(
            supportedTypes: [
              grant_request.typeUri,
              query.typeUri,
              revoke.typeUri,
              revoked.typeUri,
            ],
          ).toJson(),
        );

      // Step 2 — hcp/grants/request: hold with a next step until the person
      // decides (HCP 6.1.6, SPEC §8.6).
      case grant_request.typeUri:
        final threadId = doc.threadId!;
        final hold = held[threadId];
        if (hold == null) {
          held[threadId] = (
            payload: grant_request.Payload.fromJson(doc.payload),
            decided: false,
          );
        }
        if (hold == null || !hold.decided) {
          final step = TrustTaskDocument<next_step.Payload>(
            id: ids.next(),
            type: next_step.typeUri,
            issuer: me.vid,
            recipient: doc.issuer,
            issuedAt: clock.tick(),
            threadId: threadId,
            payload: next_step.Payload(
              expects: [
                next_step.PayloadExpectsItem(
                  typeUri: '${grant_request.typeUri}#request',
                  reason:
                      'The request is with the person, who has not decided yet.',
                  hint: {'retryAfter': 5},
                ),
              ],
              continuation: 'resubmit',
              inResponseTo: next_step.PayloadInResponseTo(
                typeUri: doc.type,
                id: doc.id,
              ),
            ),
          );
          return signed(step.toJson((p) => p.toJson()));
        }
        // The person approved (the toy approves everything asked, read only).
        final asked = hold.payload.access;
        final grant = Grant(
          grantId: ids.next(),
          token: 'hcp_at_demo_${(++_tokens).toString().padLeft(4, '0')}',
          client: doc.issuer!,
          categories: [for (final a in asked) ...a.categories],
          purposes: [for (final a in asked) ...?a.purposes?.map((p) => p.code)],
        );
        grants.add(grant);
        String plus(int seconds) =>
            instant(clock.now().add(Duration(seconds: seconds)));
        return reply(
          grant_request.Response(
            accessToken: grant.token,
            tokenExpiresAt: plus(3600), // HCP 13.4 default
            grantId: grant.grantId,
            access: [
              // approved as asked
              for (final a in asked)
                grant_request.GrantedAccess(
                  type: a.type,
                  actions: a.actions,
                  categories: a.categories,
                  purposes: a.purposes!,
                ),
            ],
            grantType: grant_request.ResponseGrantType.temporary,
            grantedAt: clock.iso(),
            // HCP 6.1.3 default
            grantExpiresAt: plus(hold.payload.expiresIn ?? 4 * 3600),
          ).toJson(),
        );

      // Step 3 — hcp/entries/query: the grant is checked on every request
      // (HCP 5.2.1, 6.2.2).
      case query.typeUri:
        final p = query.Payload.fromJson(doc.payload);
        final grant = grants
            .where((g) => g.token == p.accessToken && g.client == doc.issuer)
            .firstOrNull;
        if (grant == null || grant.status != 'active') {
          return refuse('hcp:invalidToken', 'The access token is not valid.');
        }
        final asked = p.categories ?? grant.categories;
        if (!grant.purposes.contains(p.purpose) ||
            asked.any((c) => !grant.categories.contains(c))) {
          return refuse('permissionDenied', _neutralDenial);
        }
        final useUntil = instant(clock.now().add(const Duration(hours: 1)));
        final entries = [
          for (final e in _entries)
            if (asked.contains(e.category))
              <String, Object?>{
                'ref': e.ref,
                'version': 1,
                'content': {
                  'profile':
                      'https://hcp.example/spec/profiles/general-context/2.0',
                  'value': e.value,
                  'language': 'en',
                },
                'assertion': {'declarationType': 'person-declared'},
                'classification': {
                  'categories': [e.category],
                },
                'standing': {'assurance': 'authenticated', 'confirmed': true},
                'useUntil': useUntil,
              },
        ].take(math.min(p.maxEntries ?? 5, 20)).toList();
        return reply(
          query.Response(
            entries: entries,
            retrievalId: ids.next(),
            minimization: const query.MinimizationReport(method: 'lexical'),
            categoriesSearched: asked,
            respondedAt: clock.iso(),
          ).toJson(),
        );

      // Step 4 — hcp/grants/revoke: only the person's Management Client may
      // (HCP 6.1.9, 6.2.1).
      case revoke.typeUri:
        final p = revoke.Payload.fromJson(doc.payload);
        if (p.accessToken != managementToken) {
          return refuse('permissionDenied', _neutralDenial);
        }
        final grant = grants.where((g) => g.grantId == p.grantId).firstOrNull;
        if (grant == null) return refuse('permissionDenied', _neutralDenial);
        // Immediate (HCP 6.2.2); a second revoke is idempotent.
        grant.status = 'revoked';
        return reply(<String, dynamic>{});

      // Step 5 — hcp/grants/revoked, at the Context Client: stop using what it
      // holds (HCP 6.2.2, 9.9).
      case revoked.typeUri:
        return reply(<String, dynamic>{});
    }
    throw StateError('unhandled ${doc.type}');
  }

  /// The person decides, in the provider's own modality (here: approves at
  /// once).
  void personApproves(String threadId) {
    held[threadId] = (payload: held[threadId]!.payload, decided: true);
    transcript.write(
      "[person] approves the grant request on thread $threadId in the provider's app\n",
    );
  }

  // ── Transcript ─────────────────────────────────────────────────────────────
  void step(String title) => transcript.write('\n=== $title ===\n');

  void _print(Party from, Party to, Json doc) => transcript.write(
    '--> ${from.name} -> ${to.name} (TSP) ${doc['type']}\n'
    '${pretty(redact(doc))}\n',
  );
}

/// Secrets are shown as present, never in full: the first 6 characters and an
/// ellipsis.
Object? redact(Object? value) => switch (value) {
  List<Object?>() => [for (final v in value) redact(v)],
  Map<String, Object?>() => {
    for (final MapEntry(:key, :value) in value.entries)
      key: key == 'accessToken' && value is String
          ? '${value.substring(0, math.min(6, value.length))}…'
          : redact(value),
  },
  _ => value,
};

/// JSON with object keys sorted by code point (as JCS orders them) and
/// two-space indents, strings escaped as JavaScript's JSON.stringify escapes
/// them: the same in every language. Dart's jsonEncode keeps insertion order,
/// so the keys are sorted here.
String pretty(Object? value, [String indent = '']) {
  final next = '$indent  ';
  switch (value) {
    case List<Object?>():
      if (value.isEmpty) return '[]';
      final items = [for (final v in value) '$next${pretty(v, next)}'];
      return '[\n${items.join(',\n')}\n$indent]';
    case Map<String, Object?>():
      if (value.isEmpty) return '{}';
      final keys = value.keys.toList()..sort(_byCodePoint);
      final members = [
        for (final k in keys) '$next${_quote(k)}: ${pretty(value[k], next)}',
      ];
      return '{\n${members.join(',\n')}\n$indent}';
    case String():
      return _quote(value);
    // JSON.stringify writes an integral double without a fraction.
    case double() when value.isFinite && value == value.truncateToDouble():
      return value.toInt().toString();
    case null || bool() || num():
      return '$value';
  }
  throw ArgumentError.value(value, 'value', 'not JSON');
}

int _byCodePoint(String a, String b) {
  final x = a.runes.toList(), y = b.runes.toList();
  for (var i = 0; i < math.min(x.length, y.length); i++) {
    if (x[i] != y[i]) return x[i] - y[i];
  }
  return x.length - y.length;
}

/// A string as JSON.stringify writes it: `"` `\` and control characters
/// escaped, lone surrogates as `\uXXXX`, everything else (non-ASCII too) as is.
String _quote(String s) {
  bool high(int c) => c >= 0xd800 && c <= 0xdbff;
  bool low(int c) => c >= 0xdc00 && c <= 0xdfff;
  final out = StringBuffer('"');
  for (var i = 0; i < s.length; i++) {
    final c = s.codeUnitAt(i);
    final lone =
        (high(c) && !(i + 1 < s.length && low(s.codeUnitAt(i + 1)))) ||
        (low(c) && !(i > 0 && high(s.codeUnitAt(i - 1))));
    out.write(switch (c) {
      0x22 => r'\"',
      0x5c => r'\\',
      0x08 => r'\b',
      0x0c => r'\f',
      0x0a => r'\n',
      0x0d => r'\r',
      0x09 => r'\t',
      _ when c < 0x20 || lone => '\\u${c.toRadixString(16).padLeft(4, '0')}',
      _ => String.fromCharCode(c),
    });
  }
  out.write('"');
  return out.toString();
}

// ── The five steps ───────────────────────────────────────────────────────────
final class DemoResult {
  const DemoResult(this.transcript, this.postRevokeError);
  final String transcript;
  final Json postRevokeError;
}

Future<DemoResult> runDemo([World? world]) async {
  final w = world ?? await World.create();
  final (p, c, m) = (w.provider, w.contextClient, w.management);

  // 1. Discover: does the provider speak hcp/*? (trust-task-discovery 0.4,
  //    SPEC §10)
  w.step('1. Discover (trust-task-discovery 0.4)');
  final found = await w.exchange(
    c,
    p,
    await w.request(
      c,
      p,
      discovery.typeUri,
      const discovery.Payload(patterns: ['hcp/*']).toJson(),
    ),
  );
  final served = discovery.Response.fromJson(found['payload'] as Json);
  if (!served.supportedTypes.contains(grant_request.typeUri)) {
    throw StateError('provider does not serve grant requests');
  }

  // 2. Grant request: held with a next step while the person decides, then
  //    resubmitted (HCP 6.1.6, 6.1.7; SPEC §8.6)
  w.step(
    '2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step',
  );
  final ask = const grant_request.Payload(
    access: [
      grant_request.AccessRequest(
        type: 'https://hcp.example/spec/authorization/context-access',
        actions: [grant_request.GrantAction.read],
        categories: ['travel_preferences'],
        purposes: [
          grant_request.Purpose(
            code: purpose,
            description: 'Book trips that match how you like to travel',
          ),
        ],
      ),
    ],
    expiresIn: 14400,
    subjectHint: 'acct:alex@provider.example',
  ).toJson();
  final first = await w.request(c, p, '${grant_request.typeUri}#request', ask);
  final held = await w.exchange(c, p, first);
  if (held['type'] != next_step.typeUri) {
    throw StateError('expected a next step');
  }
  final threadId = first['threadId'] as String;
  w.personApproves(threadId);
  // Resubmission: a new document, fresh id, same threadId and payload.
  final granted = await w.exchange(
    c,
    p,
    await w.request(
      c,
      p,
      '${grant_request.typeUri}#request',
      ask,
      threadId: threadId,
    ),
  );
  if (granted['type'] != grant_request.responseTypeUri) {
    throw StateError('expected the grant');
  }
  final grant = grant_request.Response.fromJson(granted['payload'] as Json);

  // 3. Query under the grant (hcp/entries/query 0.1, HCP 5.2.1)
  w.step('3. Query (hcp/entries/query 0.1)');
  final queryPayload = query.Payload(
    query: 'How does Alex like to fly?',
    categories: const ['travel_preferences'],
    purpose: purpose,
    maxEntries: 5,
    accessToken: grant.accessToken,
  ).toJson();
  final answer = await w.exchange(
    c,
    p,
    await w.request(c, p, '${query.typeUri}#request', queryPayload),
  );
  if (answer['type'] != query.responseTypeUri) {
    throw StateError('expected entries');
  }

  // 4. Revoke from the person's Management Client (hcp/grants/revoke 0.1,
  //    HCP 6.2.1)
  w.step('4. Revoke (hcp/grants/revoke 0.1) from the Management Client');
  final done = await w.exchange(
    m,
    p,
    await w.request(
      m,
      p,
      '${revoke.typeUri}#request',
      revoke.Payload(
        grantId: grant.grantId,
        accessToken: managementToken,
      ).toJson(),
    ),
  );
  if (done['type'] != revoke.responseTypeUri) {
    throw StateError('expected the revocation');
  }

  // 5. Revocation notice to the Context Client (hcp/grants/revoked 0.1,
  //    HCP 6.2.3, 9.9)
  w.step('5. Revocation notice (hcp/grants/revoked 0.1)');
  final ack = await w.exchange(
    p,
    c,
    await w.request(
      p,
      c,
      '${revoked.typeUri}#request',
      revoked.Payload(
        grantId: grant.grantId,
        revokedAt: w.clock.iso(),
      ).toJson(),
    ),
  );
  if (ack['type'] != revoked.responseTypeUri) {
    throw StateError('expected the acknowledgement');
  }

  // After: the same query again fails — the grant is checked on every request
  // (HCP 6.2.2)
  w.step('After revoke: the same query fails with hcp:invalidToken');
  final refused = await w.exchange(
    c,
    p,
    await w.request(c, p, '${query.typeUri}#request', queryPayload),
  );

  return DemoResult(w.transcript.toString(), refused);
}
