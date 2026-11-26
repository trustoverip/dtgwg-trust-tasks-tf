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
import 'package:trust_tasks/specs/trust_task_error/v0_6/payload.dart' as error;
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
// per document issued. It never reads the system time.
const String start = '2026-11-25T10:00:00Z';

/// An instant as the documents write it: whole seconds,
/// `YYYY-MM-DDTHH:MM:SSZ`. The repository's rule is "never write the system
/// time into a document": every instant the demo writes comes from its
/// injected [Clock], which holds whole seconds only, and goes through this one
/// function.
String iso(DateTime t) {
  final u = t.toUtc();
  if (u.millisecond != 0 || u.microsecond != 0) {
    throw ArgumentError.value(t, 't', "the demo's instants are whole seconds");
  }
  String two(int n) => n.toString().padLeft(2, '0');
  return '${u.year.toString().padLeft(4, '0')}-${two(u.month)}-${two(u.day)}'
      'T${two(u.hour)}:${two(u.minute)}:${two(u.second)}Z';
}

// [Clock.iso] shadows the top-level [iso] inside the class.
String _isoOf(DateTime t) => iso(t);

class Clock {
  DateTime _t = DateTime.parse(start);
  DateTime now() => _t;
  String iso() => _isoOf(_t);

  /// [seconds] after now, as an instant.
  String plus(int seconds) => _isoOf(_t.add(Duration(seconds: seconds)));

  String tick() {
    _t = _t.add(const Duration(seconds: 1));
    return iso();
  }

  /// Let time pass without issuing a document (a client waiting out
  /// `retryAfter`, or a test).
  void advance(int seconds) => _t = _t.add(Duration(seconds: seconds));
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

/// The person, by the pseudonym the provider uses with this client (HCP 3.4).
const String _subject = 'hcp-sub-demo-alex';

/// How long after the response a resubmission still gets the same grant
/// (hcp/grants/request "After approval").
const int afterApprovalWindow = 3600;

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

/// A grant in the provider's table.
final class Grant {
  Grant({
    required this.grantId,
    required this.token,
    required this.tokenExpiresAt,
    required this.client,
    required this.access,
    required this.categories,
    required this.purposes,
    required this.issuedAt,
    required this.expiresAt,
  });

  final String grantId;
  final String token;
  final String tokenExpiresAt;
  final String client; // the client's VID
  final List<grant_request.AccessDetail> access;
  final List<String> categories;
  final List<String> purposes;
  final String issuedAt;
  final String expiresAt;
  String status = 'active'; // 'active' | 'revoked'
  String? revokedAt;
}

/// A grant request on one thread: held, approved by the person, then granted
/// (hcp/grants/request "Holding").
final class Hold {
  Hold(this.issuer, this.payload, this.canonical);

  final String issuer;
  final grant_request.Payload payload;

  /// JCS of the payload as received, to compare resubmissions.
  final String canonical;
  String state = 'pending'; // 'pending' → 'approved' → 'granted'
  String? grantId;

  /// When the response was first sent.
  DateTime? respondedAt;
}

/// What the Context Client keeps for a grant it holds: the token, and the refs
/// of the copies disclosed under it.
final class HeldGrant {
  HeldGrant(this.token);
  final String token;
  final List<String> copies = [];
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
    registeredClients[contextClient.vid] = (
      name: 'Wayfare booking agent',
      operator: const grant_request.Named(
        id: 'booking.example',
        name: 'Wayfare Travel Ltd',
      ),
    );
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

  // ── The provider's state ──
  /// Its one registered client, by VID, with the registration the person is
  /// shown (HCP 6.1.5, 6.1.6).
  final Map<String, ({String name, grant_request.Named operator})>
  registeredClients = {};
  final List<Grant> grants = [];

  /// Grant requests by threadId (the SPEC §8.6 hold).
  final Map<String, Hold> held = {};
  int _tokens = 0;

  // ── The Context Client's state ──
  /// The grants it holds, by grantId.
  final Map<String, HeldGrant> wallet = {};

  /// Revocation notices it has processed: noticeId → grantId
  /// (hcp/grants/revoked "Deduplication").
  final Map<String, String> processedNotices = {};

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
    checkReply(reply, to, (doc['threadId'] ?? doc['id']) as String);
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
        return _grantRequest(me, doc, reply, refuse, signed);

      // Steps 3 and 5 — hcp/entries/query: the grant is checked on every
      // request (HCP 5.2.1, 6.2.2).
      case query.typeUri:
        final p = query.Payload.fromJson(doc.payload);
        final grant = grants.where((g) => g.token == p.accessToken).firstOrNull;
        if (grant == null) {
          return refuse('hcp:invalidToken', 'The access token is not valid.');
        }
        // A token issued to another client is no use from this VID (query
        // "The access token"); checked before the status, so it tells that
        // VID nothing about the grant.
        if (grant.client != doc.issuer) {
          return refuse('permissionDenied', _neutralDenial);
        }
        if (grant.status != 'active') {
          return refuse('hcp:invalidToken', 'The access token is not valid.');
        }
        final asked = p.categories ?? grant.categories;
        if (!grant.purposes.contains(p.purpose) ||
            asked.any((c) => !grant.categories.contains(c))) {
          return refuse('permissionDenied', _neutralDenial);
        }
        final useUntil = clock.plus(3600);
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
            // A Context Client always gets the list; the person set none
            // (HCP 6.4).
            directives: const [],
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
        if (grant.status == 'active') {
          // Immediate (HCP 6.2.2); a second revoke is idempotent.
          grant.status = 'revoked';
          grant.revokedAt = clock.iso();
        }
        return reply(<String, dynamic>{});

      // Step 6 — hcp/grants/revoked, at the Context Client (HCP 6.2.2, 9.9;
      // hcp/grants/revoked "Delivery rules").
      case revoked.typeUri:
        final p = revoked.Payload.fromJson(doc.payload);
        // Deduplicate on noticeId: a later attempt of a processed notice is
        // acknowledged, not acted on again. (A real client keeps the record at
        // least 25 hours; the demo keeps it for the run.)
        if (!processedNotices.containsKey(p.noticeId)) {
          processedNotices[p.noticeId] = p.grantId;
          forgetGrant(p.grantId);
        }
        return reply(<String, dynamic>{});
    }
    throw StateError('unhandled ${doc.type}');
  }

  /// The provider's side of hcp/grants/request: the hold, the grant, and
  /// "After approval".
  Future<TrustTaskDocument<Object?>> _grantRequest(
    Party me,
    TrustTaskDocument<Json> doc,
    Future<TrustTaskDocument<Object?>> Function(Object?) reply,
    Future<Never> Function(String, String) refuse,
    Future<TrustTaskDocument<Object?>> Function(Json) signed,
  ) async {
    final issuer = doc.issuer!;
    final threadId = doc.threadId!;
    // Only a registered client may ask (hcp/grants/request "Authorization").
    if (!registeredClients.containsKey(issuer)) {
      return refuse('permissionDenied', _neutralDenial);
    }

    // JCS of the payload as received (the canonicaliser trust_tasks_proof
    // signs with), never Dart's own serialisation: members in another order
    // are the same payload.
    final canonical = ssi.JcsUtil.canonicalize(doc.payload);
    final hold = held[threadId];
    if (hold == null) {
      held[threadId] = Hold(
        issuer,
        grant_request.Payload.fromJson(doc.payload),
        canonical,
      );
      return signed(_nextStep(me, doc));
    }
    // A resubmission is matched by (threadId, issuer); a refusal leaves the
    // held request as it is.
    if (hold.issuer != issuer) {
      return refuse('permissionDenied', _neutralDenial);
    }
    if (hold.canonical != canonical) {
      return refuse(
        'malformedRequest',
        'The resubmission differs from the held request.',
      );
    }

    switch (hold.state) {
      case 'pending':
        return signed(_nextStep(me, doc));
      case 'approved':
        // The person approved (the toy approves everything asked, read only):
        // issue the one grant.
        final asked = hold.payload.access;
        final grant = Grant(
          grantId: ids.next(),
          token:
              'hcp_at_demo_${(++_tokens).toString().padLeft(4, '0')}', // DEMO ONLY
          tokenExpiresAt: clock.plus(3600), // HCP 13.4 default
          client: issuer,
          access: [
            // approved as asked
            for (final a in asked)
              grant_request.AccessDetail(
                type: a.type,
                actions: [
                  for (final x in a.actions)
                    grant_request.AccessDetailActionsItem(x.value),
                ],
                categories: a.categories,
                purposes: [
                  for (final p
                      in a.purposes ?? <grant_request.RequestedPurpose>[])
                    grant_request.Purpose(
                      code: p.code,
                      description: p.description,
                    ),
                ],
              ),
          ],
          categories: [for (final a in asked) ...a.categories],
          purposes: [for (final a in asked) ...?a.purposes?.map((p) => p.code)],
          issuedAt: clock.iso(),
          // HCP 6.1.3 default
          expiresAt: clock.plus(hold.payload.expiresIn ?? 4 * 3600),
        );
        grants.add(grant);
        hold
          ..state = 'granted'
          ..grantId = grant.grantId
          ..respondedAt = clock.now();
        return reply(_grantResponse(grant).toJson());
      case 'granted':
        // After approval: the same grant within the hour, never a second one;
        // after it, permissionDenied.
        final grant = grants.firstWhere((g) => g.grantId == hold.grantId);
        final late =
            clock.now().difference(hold.respondedAt!).inSeconds >
            afterApprovalWindow;
        if (late || grant.status != 'active') {
          return refuse('permissionDenied', _neutralDenial);
        }
        return reply(_grantResponse(grant).toJson());
    }
    throw StateError('unknown hold state ${hold.state}');
  }

  /// The hold: a next step naming the request Type URI, continuation resubmit
  /// (SPEC §8.6).
  Json _nextStep(Party me, TrustTaskDocument<Json> doc) =>
      TrustTaskDocument<next_step.Payload>(
        id: ids.next(),
        type: next_step.typeUri,
        issuer: me.vid,
        recipient: doc.issuer,
        issuedAt: clock.tick(),
        threadId: doc.threadId,
        payload: next_step.Payload(
          expects: [
            next_step.PayloadExpectsItem(
              typeUri: '${grant_request.typeUri}#request',
              reason:
                  'The request is with the person, who has not decided yet.',
              hint: {'retryAfter': 1},
            ),
          ],
          continuation: 'resubmit',
          inResponseTo: next_step.PayloadInResponseTo(
            typeUri: doc.type,
            id: doc.id,
          ),
        ),
      ).toJson((p) => p.toJson());

  /// The grant response: the token and HCP's grant object, as
  /// hcp/grants/read-own would return it.
  grant_request.Response _grantResponse(Grant grant) {
    final registration = registeredClients[grant.client]!;
    return grant_request.Response(
      accessToken: grant.token,
      tokenExpiresAt: grant.tokenExpiresAt,
      grant: grant_request.Grant(
        grantId: grant.grantId,
        subject: const grant_request.GrantParty(id: _subject),
        authorizingPrincipal: const grant_request.AuthorizingPrincipal(
          id: _subject,
        ),
        client: grant_request.GrantClient(
          id: grant.client,
          name: registration.name,
          operator: registration.operator,
        ),
        access: grant.access,
        grantType: grant_request.GrantGrantType.temporary,
        issuedAt: grant.issuedAt,
        // A temporary grant's expiresAt is never null, so the generated
        // encoder's omission of a null member does not arise here.
        expiresAt: grant.expiresAt,
        status: grant_request.GrantStatus(grant.status),
      ),
    );
  }

  /// The person decides, in the provider's own modality (here: approves at
  /// once).
  void personApproves(String threadId) {
    held[threadId]!.state = 'approved';
    transcript.write(
      "[person] approves the grant request on thread $threadId in the provider's app\n",
    );
  }

  /// The Context Client stops using a grant: drops its token and every copy
  /// disclosed under it (HCP 6.2.2, 7.6).
  void forgetGrant(String grantId) => wallet.remove(grantId);

  // ── Transcript ─────────────────────────────────────────────────────────────
  void step(String title) => transcript.write('\n=== $title ===\n');

  void _print(Party from, Party to, Json doc) => transcript.write(
    '--> ${from.name} -> ${to.name} (TSP) ${doc['type']}\n'
    '${pretty(redact(doc))}\n',
  );
}

/// The codes the receiving end's SPEC §7.2 pipeline raises before any handler
/// runs. The library builds those error responses itself, unsigned; every
/// other reply (each response, next step and handler refusal) is signed.
const Set<String> pipelineCodes = {
  'malformedRequest',
  'unsupportedType',
  'unsupportedVersion',
  'unsupportedExtension',
  'expired',
  'proofRequired',
  'proofInvalid',
  'wrongRecipient',
  'identityMismatch',
  'idConflict',
};

/// The sender's checks on a reply before it trusts it: who it is from, which
/// thread, and that it is signed (SCENARIO.md "Every receiving end", checks
/// 1-3; the proof and the schema follow in [World.exchange]).
void checkReply(Json reply, Party from, String threadId) {
  if (reply['issuer'] != from.vid) {
    throw StateError('reply issuer ${reply['issuer']} is not ${from.vid}');
  }
  if (reply['threadId'] != threadId) {
    throw StateError('reply threadId ${reply['threadId']} is not $threadId');
  }
  final payload = reply['payload'];
  final libraryError =
      reply['type'] == error.typeUri &&
      payload is Map &&
      pipelineCodes.contains(payload['code']);
  if (reply['proof'] == null && !libraryError) {
    throw StateError('reply ${reply['type']} carries no proof');
  }
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

// ── The six steps, one function each ─────────────────────────────────────────

/// What the Context Client asks for in step 2.
final Json ask = const grant_request.Payload(
  access: [
    grant_request.AccessRequest(
      type: 'https://hcp.example/spec/authorization/context-access',
      actions: [grant_request.GrantAction.read],
      categories: ['travel_preferences'],
      purposes: [
        grant_request.RequestedPurpose(
          code: purpose,
          description: 'Book trips that match how you like to travel',
        ),
      ],
    ),
  ],
  expiresIn: 14400,
  subjectHint: 'acct:alex@provider.example',
).toJson();

/// The query the Context Client sends in steps 3 and 5, under the grant's
/// token.
Json queryPayload(String accessToken) => query.Payload(
  query: 'How does Alex like to fly?',
  categories: const ['travel_preferences'],
  purpose: purpose,
  maxEntries: 5,
  accessToken: accessToken,
).toJson();

/// 1. Discover: does the provider speak hcp/*? (trust-task-discovery 0.4,
/// SPEC §10)
Future<void> stepDiscover(World w) async {
  final (p, c) = (w.provider, w.contextClient);
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
}

/// 2. Grant request: held with a next step while the person decides, then
/// resubmitted on the same thread (HCP 6.1.6, 6.1.7; SPEC §8.6). The client
/// keeps the token in its wallet. Returns the thread and the grant.
Future<({String threadId, String grantId})> stepGrantRequest(World w) async {
  final (p, c) = (w.provider, w.contextClient);
  w.step(
    '2. Grant request (hcp/grants/request 0.1), held with trust-task-next-step',
  );
  final first = await w.request(c, p, '${grant_request.typeUri}#request', ask);
  final held = await w.exchange(c, p, first);
  if (held['type'] != next_step.typeUri) {
    throw StateError('expected a next step');
  }
  final threadId = first['threadId'] as String;
  w.personApproves(threadId);
  // The client SHOULD NOT resubmit sooner than the next step's retryAfter
  // (hcp/grants/request "Holding").
  final expects = (held['payload'] as Json)['expects'] as List<dynamic>;
  final hint = (expects.first as Json)['hint'] as Json?;
  final retryAfter = hint?['retryAfter'];
  if (retryAfter is! int) throw StateError('expected a retryAfter hint');
  w.clock.advance(retryAfter);
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
  final response = grant_request.Response.fromJson(granted['payload'] as Json);
  w.wallet[response.grant.grantId] = HeldGrant(response.accessToken);
  return (threadId: threadId, grantId: response.grant.grantId);
}

/// 3. Query under the grant (hcp/entries/query 0.1, HCP 5.2.1). The client
/// keeps the refs of the copies.
Future<void> stepQuery(World w, String grantId) async {
  final (p, c) = (w.provider, w.contextClient);
  w.step('3. Query (hcp/entries/query 0.1)');
  final held = w.wallet[grantId]!;
  final answer = await w.exchange(
    c,
    p,
    await w.request(c, p, '${query.typeUri}#request', queryPayload(held.token)),
  );
  if (answer['type'] != query.responseTypeUri) {
    throw StateError('expected entries');
  }
  final entries = query.Response.fromJson(answer['payload'] as Json).entries;
  held.copies.addAll([for (final e in entries) (e! as Json)['ref'] as String]);
}

/// 4. Revoke from the person's Management Client (hcp/grants/revoke 0.1,
/// HCP 6.2.1).
Future<void> stepRevoke(World w, String grantId) async {
  final (p, m) = (w.provider, w.management);
  w.step('4. Revoke (hcp/grants/revoke 0.1) from the Management Client');
  final done = await w.exchange(
    m,
    p,
    await w.request(
      m,
      p,
      '${revoke.typeUri}#request',
      revoke.Payload(grantId: grantId, accessToken: managementToken).toJson(),
    ),
  );
  if (done['type'] != revoke.responseTypeUri) {
    throw StateError('expected the revocation');
  }
}

/// 5. The same query, sent before any notice arrives, fails with
/// hcp:invalidToken: the grant is checked on every request, so revocation
/// takes effect at once (HCP 6.2.2). The client acts on that at once too: it
/// drops the token and the copies (hcp/grants/revoked "Independence").
Future<Json> stepQueryAfterRevoke(World w, String grantId) async {
  final (p, c) = (w.provider, w.contextClient);
  w.step(
    '5. Query after revoke (hcp/entries/query 0.1) fails with hcp:invalidToken',
  );
  final held = w.wallet[grantId]!;
  final refused = await w.exchange(
    c,
    p,
    await w.request(c, p, '${query.typeUri}#request', queryPayload(held.token)),
  );
  if (refused['type'] == error.typeUri &&
      (refused['payload'] as Json)['code'] == 'hcp:invalidToken') {
    w.forgetGrant(grantId);
  }
  return refused;
}

/// 6. Revocation notice to the grant's client (hcp/grants/revoked 0.1, HCP
/// 6.2.3, 9.9). The client records the noticeId and the grant and stops using
/// the grant; it never sends the old token again. Returns the notice sent.
Future<Json> stepNotice(World w, String grantId) async {
  final p = w.provider;
  w.step('6. Revocation notice (hcp/grants/revoked 0.1)');
  final grant = w.grants.firstWhere((g) => g.grantId == grantId);
  final client = w.byVid(grant.client);
  final notice = await w.request(
    p,
    client,
    '${revoked.typeUri}#request',
    revoked.Payload(
      grantId: grantId,
      // HCP's jti, minted once per notice and kept on every delivery attempt.
      noticeId: w.ids.next(),
      revokedAt: grant.revokedAt!,
    ).toJson(),
  );
  final ack = await w.exchange(p, client, notice);
  if (ack['type'] != revoked.responseTypeUri) {
    throw StateError('expected the acknowledgement');
  }
  return notice;
}

final class DemoResult {
  const DemoResult({
    required this.transcript,
    required this.threadId,
    required this.grantId,
    required this.postRevokeError,
    required this.notice,
  });
  final String transcript;
  final String threadId;
  final String grantId;
  final Json postRevokeError;
  final Json notice;
}

/// The six steps in order.
Future<DemoResult> runDemo([World? world]) async {
  final w = world ?? await World.create();
  await stepDiscover(w);
  final (:threadId, :grantId) = await stepGrantRequest(w);
  await stepQuery(w, grantId);
  await stepRevoke(w, grantId);
  final postRevokeError = await stepQueryAfterRevoke(w, grantId);
  final notice = await stepNotice(w, grantId);
  return DemoResult(
    transcript: w.transcript.toString(),
    threadId: threadId,
    grantId: grantId,
    postRevokeError: postRevokeError,
    notice: notice,
  );
}
