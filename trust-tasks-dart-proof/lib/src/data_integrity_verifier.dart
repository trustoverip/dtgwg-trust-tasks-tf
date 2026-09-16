import 'package:ssi/ssi.dart' as ssi;
import 'package:trust_tasks/trust_tasks.dart';

/// Why a proof failed to verify.
///
/// The same taxonomy `trust-tasks-proof` uses in Rust, so a failure logged by a
/// Dart consumer reads the same as one logged by a Rust consumer. All of them
/// reach the wire as `proofInvalid`: [ConsumeOutcome] never carries a
/// verifier's own words (SPEC §10.4). The kind and [ProofVerification.detail]
/// are for your logs.
enum ProofFailureKind {
  /// The document has no `proof`, or the proof is missing a member or carries
  /// one of the wrong shape.
  malformedProof,

  /// The proof's `verificationMethod` is not controlled by the document's
  /// `issuer`, or the document has no `issuer` to bind the proof to.
  issuerMismatch,

  /// The proof names a cryptosuite this verifier does not accept.
  unsupportedCryptosuite,

  /// The signature does not verify, or the key could not be resolved.
  signatureInvalid,
}

/// The result of [DataIntegrityProofVerifier.verifyDetailed].
final class ProofVerification {
  const ProofVerification._(this.failure, this.detail);

  /// A proof that verified.
  static const ok = ProofVerification._(null, null);

  /// A proof that did not.
  const ProofVerification.failed(ProofFailureKind this.failure, this.detail);

  /// Why verification failed; `null` when it succeeded.
  final ProofFailureKind? failure;

  /// Diagnostic text for logs. Never send it to the peer: it can name DIDs a
  /// resolver was asked about, and whether it answered (SPEC §10.4).
  final String? detail;

  /// Whether the proof verified.
  bool get isValid => failure == null;

  @override
  String toString() =>
      isValid ? 'ProofVerification.ok' : 'ProofVerification($failure: $detail)';
}

/// A [ProofVerifier] for W3C Data Integrity proofs, backed by Affinidi's
/// `package:ssi`.
///
/// Verifies `eddsa-jcs-2022` and `ecdsa-jcs-2019`, over the document minus its
/// `proof` member, and binds the proof to the document's `issuer`: a valid
/// signature proves only that *some* key signed, so the key's DID must also be
/// the `issuer` (SPEC §4.7, §7.2 item 7).
///
/// ```dart
/// final outcome = await consumeInbound(
///   proofPolicy: ProofPolicy.verify(DataIntegrityProofVerifier.forDidKey()),
///   // ...
/// );
/// ```
final class DataIntegrityProofVerifier implements ProofVerifier {
  /// A verifier that resolves `did:key` issuers locally, with no I/O, and
  /// rejects every other DID method. Suitable for self-certifying peers, tests,
  /// and anywhere a network fetch on an unauthenticated input is unwanted.
  DataIntegrityProofVerifier.forDidKey({
    Set<String> cryptosuites = supportedCryptosuites,
    Duration clockSkew = defaultClockSkew,
    DateTime Function() clock = DateTime.now,
  }) : this.withResolver(
          const _DidKeyResolver(),
          cryptosuites: cryptosuites,
          clockSkew: clockSkew,
          clock: clock,
        );

  /// A verifier that resolves issuers with [resolver] — for example
  /// `ssi`'s `UniversalDIDResolver`, which also fetches `did:web` documents.
  ///
  /// Resolution happens on input from a peer that has not yet been
  /// authenticated. Prefer a resolver that caches and bounds its fetches.
  DataIntegrityProofVerifier.withResolver(
    ssi.DidResolver resolver, {
    Set<String> cryptosuites = supportedCryptosuites,
    this.clockSkew = defaultClockSkew,
    DateTime Function() clock = DateTime.now,
  })  : _resolver = resolver,
        _clock = clock,
        cryptosuites = Set.unmodifiable(cryptosuites) {
    final unknown = cryptosuites.difference(supportedCryptosuites);
    if (unknown.isNotEmpty) {
      throw ArgumentError.value(
        cryptosuites,
        'cryptosuites',
        'not supported by this verifier: ${unknown.join(', ')}',
      );
    }
  }

  /// The cryptosuites this package can verify.
  ///
  /// Only the JCS suites: a Trust Task document carries no JSON-LD `@context`,
  /// so the RDF canonicalization the `-rdfc-` suites depend on has nothing to
  /// expand. ML-DSA is absent because its verifiers exist only in `ssi` 4,
  /// and this package also supports `ssi` 3 — see the README.
  static const supportedCryptosuites = {'eddsa-jcs-2022', 'ecdsa-jcs-2019'};

  /// How far into this verifier's future a proof's `created` may sit before it
  /// is rejected. The same default as `affinidi-data-integrity`.
  static const defaultClockSkew = Duration(seconds: 60);

  /// The cryptosuites this instance accepts; a subset of
  /// [supportedCryptosuites].
  final Set<String> cryptosuites;

  /// The allowance applied to a proof's `created`.
  final Duration clockSkew;

  final ssi.DidResolver _resolver;
  final DateTime Function() _clock;

  static final _dateTimeStamp = RegExp(
    r'^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$',
  );

  @override
  Future<bool> verify(Map<String, dynamic> doc) async =>
      (await verifyDetailed(doc)).isValid;

  /// Verifies [doc]'s proof and says why it failed, if it did.
  ///
  /// Never throws for a bad document; every failure is a [ProofVerification].
  Future<ProofVerification> verifyDetailed(Map<String, dynamic> doc) async {
    final proof = doc['proof'];
    if (proof is! Map<String, dynamic>) {
      return _malformed(proof == null
          ? 'document carries no proof member'
          : 'proof is not a JSON object');
    }

    if (proof['type'] != 'DataIntegrityProof') {
      return _malformed('proof type must be DataIntegrityProof');
    }
    for (final member in const [
      'cryptosuite',
      'verificationMethod',
      'proofPurpose',
      'proofValue',
    ]) {
      final value = proof[member];
      if (value is! String || value.isEmpty) {
        return _malformed('proof.$member must be a non-empty string');
      }
    }

    final cryptosuite = proof['cryptosuite'] as String;
    if (!cryptosuites.contains(cryptosuite)) {
      return ProofVerification.failed(
        ProofFailureKind.unsupportedCryptosuite,
        cryptosuite,
      );
    }

    // VC Data Integrity §2.1: `created` is a dateTimeStamp — UTC or an explicit
    // offset. An offset-less value is refused rather than guessed at, which is
    // also what the Rust verifier does; DateTime.parse would read it as *local*
    // time, while the specification says to read it as UTC.
    final created = proof['created'];
    if (created != null) {
      if (created is! String || !_dateTimeStamp.hasMatch(created)) {
        return _malformed(
          'proof.created must be a dateTimeStamp with Z or an offset: $created',
        );
      }
      final horizon =
          _clock().add(clockSkew.isNegative ? Duration.zero : clockSkew);
      if (DateTime.parse(created).isAfter(horizon)) {
        return _malformed(
          'proof.created is in the future beyond the '
          '${clockSkew.inSeconds}s clock-skew allowance',
        );
      }
    }

    // Bind the proof to the in-band issuer. Exact string comparison, no
    // normalization (SPEC §4.8). `ssi` performs the same check internally, but
    // against a DID *we* pass it — so the check that matters is that the DID we
    // pass is the document's issuer, made explicit here.
    final issuer = doc['issuer'];
    if (issuer is! String) {
      return const ProofVerification.failed(
        ProofFailureKind.issuerMismatch,
        'document carries a proof but no in-band issuer to bind it to',
      );
    }
    final vmDid = (proof['verificationMethod'] as String).split('#').first;
    if (vmDid != issuer) {
      return ProofVerification.failed(
        ProofFailureKind.issuerMismatch,
        'verificationMethod is controlled by $vmDid, '
        'not the document issuer $issuer',
      );
    }

    final suite = switch (cryptosuite) {
      'eddsa-jcs-2022' => ssi.DataIntegrityEddsaJcsVerifier(
          verifierDid: issuer,
          didResolver: _resolver,
          getNow: _clock,
        ),
      'ecdsa-jcs-2019' => ssi.DataIntegrityEcdsaJcsVerifier(
          verifierDid: issuer,
          didResolver: _resolver,
          getNow: _clock,
        ),
      _ => throw StateError('unreachable: $cryptosuite'),
    };

    try {
      // A deep copy: ssi removes `proofValue` from the proof it is handed.
      final result = await suite.verify(_deepCopy(doc), getNow: _clock);
      if (result.isValid) return ProofVerification.ok;
      return ProofVerification.failed(
        ProofFailureKind.signatureInvalid,
        result.errors.join('; '),
      );
    } on Object catch (e) {
      // Resolution failures, undecodable keys and multibase errors all surface
      // as exceptions from ssi. None of them is a proof that verified.
      return ProofVerification.failed(
        ProofFailureKind.signatureInvalid,
        e.toString(),
      );
    }
  }

  static ProofVerification _malformed(String detail) =>
      ProofVerification.failed(ProofFailureKind.malformedProof, detail);

  static Map<String, dynamic> _deepCopy(Map<String, dynamic> map) =>
      map.map((k, v) => MapEntry(k, _copyValue(v)));

  static Object? _copyValue(Object? value) => switch (value) {
        Map<String, dynamic>() => _deepCopy(value),
        Map() => {
            for (final e in value.entries) e.key as String: _copyValue(e.value),
          },
        List() => [for (final v in value) _copyValue(v)],
        _ => value,
      };
}

/// Resolves `did:key` only, locally. Anything else is refused before any I/O.
final class _DidKeyResolver implements ssi.DidResolver {
  const _DidKeyResolver();

  @override
  Future<ssi.DidDocument> resolveDid(String did) async {
    if (!did.startsWith('did:key:')) {
      throw ssi.SsiException(
        message: 'this verifier resolves did:key only; got $did',
        code: ssi.SsiExceptionType.unableToResolveDid.code,
      );
    }
    return ssi.DidKey.resolve(did);
  }
}
