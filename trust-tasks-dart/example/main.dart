// Consuming a Trust Task document, end to end.
//
// Run it:
//   dart run example/main.dart
//
// The example is deliberately a *consumer* rather than a producer. Producing a
// document is a struct literal and a JSON encode; consuming one is where the
// framework earns its keep, because SPEC.md §7.2 is a sequence of checks that
// are easy to half-implement and hard to notice having half-implemented.

import 'dart:convert';

import 'package:trust_tasks/specs/acl/grant/v0_1/payload.dart' as acl_grant;
import 'package:trust_tasks/trust_tasks.dart';

const me = 'did:web:maintainer.example';
const peer = 'did:web:org.example';

/// A document arriving over some transport, as JSON.
///
/// `issuedAt` is fixed rather than `DateTime.now()` so that the second delivery
/// below is byte-for-byte identical to the first — which is what SPEC §8.4
/// defines a *retry* to be, and what §7.2 item 11 obliges this consumer to
/// absorb rather than execute twice.
String inboundJson({String role = 'admin'}) => jsonEncode({
      'id': 'urn:uuid:9b2c1e34-0000-4000-8000-000000000001',
      'type': acl_grant.typeUri,
      'issuer': peer,
      'recipient': me,
      'issuedAt': '2026-01-01T00:00:00Z',
      // acl/grant declares proofRequired: REQUIRED, so a proofless document is
      // refused at §7.2 item 7 before the handler is reached. The generated
      // `spec` carries that requirement — you do not have to know it.
      'proof': {
        'type': 'DataIntegrityProof',
        'cryptosuite': 'eddsa-jcs-2022',
        'verificationMethod': '$peer#key-1',
        'created': '2026-01-01T00:00:00Z',
        'proofPurpose': 'assertionMethod',
        'proofValue': 'z3kgExampleOnlyNotARealSignature',
      },
      'payload': {
        'entry': {'subject': 'did:web:alice.example', 'role': role},
      },
    });

/// A stand-in for a real Data Integrity verifier.
///
/// The package ships none on purpose: the cryptosuite is the consumer's choice,
/// so [ProofVerifier] is a seam you fill. A real one would resolve the issuer's
/// DID and check the proof over the canonical form.
class AcceptEverything implements ProofVerifier {
  const AcceptEverything();

  @override
  Future<bool> verify(Map<String, dynamic> doc) async => true;
}

Future<void> main() async {
  // One guard per consumer, held for the process's lifetime — it *is* the
  // duplicate-execution record. Back it with a shared store if you run more
  // than one replica, or each replica will execute the same document once.
  final guard = InMemoryReplayGuard();

  var executions = 0;

  Future<ConsumeOutcome<acl_grant.Response>> deliver(String json) {
    final doc = TrustTaskDocument<acl_grant.Payload>.fromJson(
      jsonDecode(json) as Map<String, dynamic>,
      (payload) => acl_grant.Payload.fromJson(payload! as Map<String, dynamic>),
    );

    return consumeInbound<acl_grant.Payload, acl_grant.Response>(
      // What the transport authenticated about the sender. §4.8.1 cross-checks
      // this against the in-band `issuer` and rejects a disagreement.
      transport: const StaticTransport(TransportContext(issuer: peer)),
      // The generated policy: which members this specification requires, and
      // the schema to validate the payload against.
      spec: acl_grant.spec,
      proofPolicy: const ProofPolicy.verify(AcceptEverything()),
      // A real consumer passes a JSON Schema engine here. Skipping it is a
      // deliberate choice, not a default — see PayloadPolicy.
      payloadPolicy: const PayloadPolicy.acceptUnvalidated(),
      // acl/grant grants access, so it is consequential: a replayed envelope
      // must not grant twice.
      checks: consequentialChecks(guard),
      doc: doc,
      myVid: me,
      now: DateTime.parse('2026-01-01T00:00:30Z'),
      newErrorId: () => 'urn:uuid:9b2c1e34-0000-4000-8000-0000000000ff',
      payloadToJson: (p) => p.toJson(),
      handler: (accepted, parties) async {
        // Only reached once every framework check has passed.
        executions++;
        return respondWith<acl_grant.Payload, acl_grant.Response>(
          accepted,
          'urn:uuid:9b2c1e34-0000-4000-8000-000000000002',
          acl_grant.Response(entry: accepted.payload.entry),
        );
      },
    );
  }

  void report(String label, ConsumeOutcome<acl_grant.Response> outcome) {
    // A sealed hierarchy, so this switch is exhaustive and the compiler will
    // tell you when a case is missing.
    final summary = switch (outcome) {
      Handled(:final response) => 'handled -> ${response.type}',
      Rejected(:final error) => 'rejected -> ${error.payload.code}',
      Suppressed(:final reason) => 'suppressed -> ${reason.code.value}',
      Accepted() => 'accepted (fire-and-forget)',
      DuplicateOutcome(:final inFlight) =>
        'duplicate (already executed, inFlight=$inFlight)',
    };
    print('$label: $summary');
  }

  report('first delivery    ', await deliver(inboundJson()));

  // §8.4: a retry is a bit-for-bit identical resend. Item 11 obliges this
  // consumer to absorb it.
  report('identical resend  ', await deliver(inboundJson()));

  // A *different* document under the same id is the other half of item 11, and
  // must be refused rather than treated as a retry.
  report('escalated resend  ', await deliver(inboundJson(role: 'owner')));

  print('handler ran $executions time(s) — the effect happened exactly once.');
}
