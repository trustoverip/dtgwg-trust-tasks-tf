import 'dart:convert';
import 'dart:typed_data';

import 'package:ssi/ssi.dart' as ssi;
import 'package:trust_tasks/trust_tasks.dart';

import 'ed25519_point.dart';

/// Why a proof failed to verify.
///
/// The same taxonomy `trust-tasks-proof` uses in Rust, so a failure logged by a
/// Dart consumer reads the same as one logged by a Rust consumer. All of them
/// reach the wire as `proofInvalid`: [ConsumeOutcome] never carries a
/// verifier's own words (SPEC §12.4). The kind and [ProofVerification.detail]
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
  /// resolver was asked about, and whether it answered (SPEC §12.4).
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
  /// Only the JCS suites: a Trust Task document is processed as JSON whether
  /// or not it carries the optional `@context` (SPEC §4.7.1), so it is never
  /// expanded and the RDF canonicalization the `-rdfc-` suites depend on does
  /// not apply. ML-DSA is absent because its verifiers exist only in `ssi` 4,
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

    final purpose = proof['proofPurpose'] as String;
    final purposeError = proofPurposeError(purpose);
    if (purposeError != null) return _malformed(purposeError);

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
    // Every field must be in range, as for `expires`: DateTime.parse rolls
    // month 13 or February 30 over into a later date instead of refusing it.
    final created = proof['created'];
    if (created != null) {
      final at = created is String ? parseDateTimeStamp(created) : null;
      if (at == null) {
        return _malformed(
          'proof.created must be a dateTimeStamp with Z or an offset: $created',
        );
      }
      final horizon =
          _clock().add(clockSkew.isNegative ? Duration.zero : clockSkew);
      if (at.isAfter(horizon)) {
        return _malformed(
          'proof.created is in the future beyond the '
          '${clockSkew.inSeconds}s clock-skew allowance',
        );
      }
    }

    // `expires`, if present, is a dateTimeStamp that has not yet passed (VC
    // Data Integrity §2.1). No allowance applies: the signer chose the instant.
    // Checked here rather than left to ssi, which reads an offset-less value as
    // local time.
    final expires = proof['expires'];
    if (expires != null) {
      final at = expires is String ? parseDateTimeStamp(expires) : null;
      if (at == null) {
        return _malformed(
          'proof.expires must be a dateTimeStamp with Z or an offset: $expires',
        );
      }
      if (_clock().isAfter(at)) {
        return _malformed('proof.expires $expires has passed');
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

    // ssi decodes base58btc at a cost that grows with the square of the
    // value's length, so an over-long proofValue or did:key is refused here,
    // before it reaches ssi. The bounds are the Go, Rust and TypeScript
    // packages': no value this package reads is longer.
    final proofValueError = _base58Bound(proof['proofValue'] as String);
    if (proofValueError != null) {
      return ProofVerification.failed(
        ProofFailureKind.signatureInvalid,
        'proofValue: $proofValueError',
      );
    }
    if (vmDid.startsWith(_didKeyPrefix)) {
      final keyError = _base58Bound(vmDid.substring(_didKeyPrefix.length));
      if (keyError != null) {
        return ProofVerification.failed(
          ProofFailureKind.signatureInvalid,
          'verificationMethod: $keyError',
        );
      }
    }

    final contextError = _proofContextError(doc, proof);
    if (contextError != null) return _malformed(contextError);

    // One resolution per verification: the relationship check below and ssi
    // read the same document.
    final resolver = _OnceResolver(_resolver);
    final suite = switch (cryptosuite) {
      'eddsa-jcs-2022' => _EddsaJcsAsReceived(
          verifierDid: issuer,
          didResolver: resolver,
          getNow: _clock,
        ),
      'ecdsa-jcs-2019' => _EcdsaJcsAsReceived(
          verifierDid: issuer,
          didResolver: resolver,
          getNow: _clock,
        ),
      _ => throw StateError('unreachable: $cryptosuite'),
    };

    try {
      // The key must be one the issuer authorised for the purpose the proof
      // declares (SPEC §4.7.3). ssi finds the key by id alone — or by fragment
      // — whatever relationship lists it.
      final notAuthorised = _notAuthorised(
        await resolver.resolveDid(issuer),
        issuer,
        proof['verificationMethod'] as String,
        purpose,
      );
      if (notAuthorised != null) {
        return ProofVerification.failed(
          ProofFailureKind.signatureInvalid,
          notAuthorised,
        );
      }

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

  static const _didKeyPrefix = 'did:key:';

  /// The most bytes a base58btc value this package reads may carry. The
  /// largest is a P-384 signature (96 bytes); a `did:key` multikey is at most
  /// 51 (a 2-byte multicodec header and a 49-byte compressed P-384 point).
  static const _maxBase58Bytes = 128;

  /// The longest base58btc string that can encode [_maxBase58Bytes] bytes:
  /// ceil(128 * log(256) / log(58)) = 175 characters.
  static const _maxBase58Chars = 175;

  /// Why the multibase-base58btc [value] is out of bounds, or `null` when it is
  /// within them (or is not base58btc at all, which ssi refuses itself). The
  /// length is checked before anything is decoded.
  static String? _base58Bound(String value) {
    if (!value.startsWith('z')) return null;
    final chars = value.length - 1;
    if (chars > _maxBase58Chars) {
      return 'base58 value is $chars characters, longer than the '
          '$_maxBase58Chars this verifier accepts';
    }
    final Uint8List bytes;
    try {
      bytes = ssi.multiBaseToUint8List(value);
    } on Object {
      return null;
    }
    if (bytes.length > _maxBase58Bytes) {
      return 'base58 value decodes to more than $_maxBase58Bytes bytes';
    }
    return null;
  }

  /// Why [vm] is not a key [did]'s document authorises for [purpose], or
  /// `null` when it is. The rule `trust-tasks-proof`'s `CachedDidResolver`
  /// applies: the document is [did]'s; the relationship lists [vm] by absolute
  /// reference, by a fragment relative to [did], or embedded; a reference
  /// names a method in `verificationMethod`; and that method's controller is
  /// [did]. The method ssi will then select must be the one checked.
  static String? _notAuthorised(
    ssi.DidDocument doc,
    String did,
    String vm,
    String purpose,
  ) {
    if (doc.id != did) {
      return "the resolved DID document's id is not the verificationMethod's DID";
    }
    bool namesVm(String id) => id.startsWith('#') ? vm == '$did$id' : id == vm;

    final relationship = switch (purpose) {
      'authentication' => doc.authentication,
      'assertionMethod' => doc.assertionMethod,
      'capabilityInvocation' => doc.capabilityInvocation,
      'capabilityDelegation' => doc.capabilityDelegation,
      _ => const <ssi.VerificationMethod>[],
    };
    ssi.VerificationMethod? listed;
    for (final entry in relationship) {
      final names = switch (entry) {
        ssi.VerificationMethodRef(:final reference) => namesVm(reference),
        _ => namesVm(entry.id),
      };
      if (names) {
        listed = entry;
        break;
      }
    }
    if (listed == null) {
      return 'verificationMethod is not listed under $purpose '
          'in its DID document';
    }

    // ssi takes the first `verificationMethod` entry whose id is [vm] or ends
    // with its fragment.
    final fragment = vm.substring(vm.indexOf('#') + 1);
    ssi.EmbeddedVerificationMethod? selected;
    for (final method in doc.verificationMethod) {
      if (method.id == vm || method.id.endsWith('#$fragment')) {
        selected = method;
        break;
      }
    }
    if (selected == null || !namesVm(selected.id)) {
      return 'verificationMethod is not defined under verificationMethod '
          'in its DID document';
    }
    // An embedded entry is a method in its own right; its controller must be
    // the issuer too, as trust-tasks-proof requires of the method it uses.
    if (listed is! ssi.VerificationMethodRef && listed.controller != did) {
      return "verificationMethod's controller is not the DID that names it";
    }
    if (listed is! ssi.VerificationMethodRef &&
        !_sameKey(listed.asMultiKey(), selected.asMultiKey())) {
      return 'verificationMethod is embedded under $purpose with a key other '
          'than the one its DID document defines';
    }
    if (selected.controller != did) {
      return "verificationMethod's controller is not the DID that names it";
    }
    // An Ed25519 key must be a canonical point of large order; see
    // ed25519PublicKeyError. ssi verifies with selected.asJwk(), which every
    // format (publicKeyMultibase, publicKeyBase58, publicKeyJwk) reaches
    // through the 0xed 0x01 multikey, with no length check. So the key is read
    // from that JWK — the bytes ssi will use — and an Ed25519 multikey of any
    // length is checked, not only a 34-byte one.
    final multikey = selected.asMultiKey();
    final ed25519Multikey =
        multikey.length >= 2 && multikey[0] == 0xed && multikey[1] == 0x01;
    final jwk = selected.asJwk().toJson();
    if (ed25519Multikey || jwk['crv'] == 'Ed25519') {
      final x = jwk['x'];
      if (x is! String) return 'Ed25519 verificationMethod carries no key';
      final List<int> raw;
      try {
        raw = base64Url.decode(base64Url.normalize(x));
      } on FormatException {
        return 'Ed25519 verificationMethod key is not base64url';
      }
      final keyError = ed25519PublicKeyError(raw);
      if (keyError != null) return keyError;
    }
    return null;
  }

  static bool _sameKey(List<int> a, List<int> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
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

/// Why [proof]'s `@context` disagrees with [doc]'s, or `null` when it does
/// not. eddsa-jcs-2022 / ecdsa-jcs-2019 §3.3.2 step 4: "If
/// proofOptions.@context exists: Check that the securedDocument.@context
/// starts with all values contained in the proofOptions.@context in the same
/// order. Otherwise, set verified to false". A proof without `@context` is
/// hashed as received (step 4 does not apply).
///
/// This verifier requires the two to be **equal**, not only the proof's to be
/// a prefix. The cryptosuite verifies a longer document `@context` by hashing
/// the proof's in its place (step 4.2) and returning that rewritten document
/// as the verified one; a [ProofVerifier] returns no document, so the caller
/// would go on to act on `@context` entries no signature covers. The Rust, Go
/// and TypeScript verifiers apply the same rule.
String? _proofContextError(
  Map<String, dynamic> doc,
  Map<String, dynamic> proof,
) {
  if (!proof.containsKey('@context')) return null;
  if (!doc.containsKey('@context')) {
    return 'proof carries @context but the document has none';
  }
  try {
    if (ssi.JcsUtil.canonicalize(proof['@context']) !=
        ssi.JcsUtil.canonicalize(doc['@context'])) {
      return "proof @context does not equal the document's @context";
    }
  } on Object catch (e) {
    return '@context cannot be canonicalized: $e';
  }
  return null;
}

/// ssi's eddsa-jcs-2022 verifier, hashing the proof configuration **as
/// received**.
///
/// ssi's JCS verifiers copy the document's `@context` into the proof
/// configuration before hashing it, whether or not the proof carries one. A
/// proof without `@context` over a document with one — what the Rust, Go and
/// TypeScript signers emitted before #1113, and what eddsa-jcs-2022 §3.3.2
/// verifies, since its step 4 applies only when the proof carries `@context` —
/// then failed here. [_proofContextError] has already checked a proof
/// `@context` against the document's, so ssi's own prefix check, which
/// compares inline contexts by identity, is not repeated.
final class _EddsaJcsAsReceived extends ssi.DataIntegrityEddsaJcsVerifier {
  _EddsaJcsAsReceived({
    required super.verifierDid,
    super.getNow,
    super.didResolver,
  });

  @override
  Map<String, dynamic> prepareProofForVerification(
    Map<String, dynamic> proof,
    Map<String, dynamic> document,
  ) =>
      Map<String, dynamic>.of(proof);

  @override
  Future<ssi.VerificationResult> validateCryptosuite(
    Map<String, dynamic> document,
    Map<String, dynamic> proof,
  ) async =>
      ssi.VerificationResult.ok();
}

/// ssi's ecdsa-jcs-2019 verifier, hashing the proof configuration as
/// received; see [_EddsaJcsAsReceived].
final class _EcdsaJcsAsReceived extends ssi.DataIntegrityEcdsaJcsVerifier {
  _EcdsaJcsAsReceived({
    required super.verifierDid,
    super.getNow,
    super.didResolver,
  });

  @override
  Map<String, dynamic> prepareProofForVerification(
    Map<String, dynamic> proof,
    Map<String, dynamic> document,
  ) =>
      Map<String, dynamic>.of(proof);

  @override
  Future<ssi.VerificationResult> validateCryptosuite(
    Map<String, dynamic> document,
    Map<String, dynamic> proof,
  ) async =>
      ssi.VerificationResult.ok();
}

/// The `proofPurpose` values a proof may carry: the verification relationships
/// a signature can be made under. `keyAgreement` is deliberately absent — a
/// key-agreement key never authorises a signature (SPEC §4.7.3) — as it is
/// from the Rust, Go and TypeScript verifiers.
const signingPurposes = {
  'authentication',
  'assertionMethod',
  'capabilityInvocation',
  'capabilityDelegation',
};

/// Why [purpose] cannot be a proof's `proofPurpose`, or `null` when it can.
String? proofPurposeError(String purpose) {
  if (signingPurposes.contains(purpose)) return null;
  return purpose == 'keyAgreement'
      ? 'proofPurpose keyAgreement never authorises a signature'
      : 'proofPurpose names no signing verification relationship';
}

final _dateTimeStampParts = RegExp(
  r'^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(\.\d+)?'
  r'(?:Z|([+-])(\d{2}):(\d{2}))$',
);

/// [value] as a UTC instant when it is a dateTimeStamp with every field in
/// range — month 1-12, a day the month has (leap years included), hour 0-23,
/// minute and second 0-59 (no leap second), an offset of at most 14:00 with
/// minutes 0-59 — else `null`. `DateTime.tryParse` rolls an out-of-range
/// field over instead of refusing it; the Go, Rust and TypeScript verifiers
/// apply these ranges.
DateTime? parseDateTimeStamp(String value) {
  final m = _dateTimeStampParts.firstMatch(value);
  if (m == null) return null;
  final f = [for (var i = 1; i <= 6; i++) int.parse(m.group(i)!)];
  final [year, month, day, hour, minute, second] = f;
  if (month < 1 || month > 12) return null;
  final daysInMonth = DateTime.utc(year, month + 1, 0).day;
  if (day < 1 || day > daysInMonth || hour > 23 || minute > 59 || second > 59) {
    return null;
  }
  var micros = 0;
  final fraction = m.group(7);
  if (fraction != null) {
    final digits = fraction.substring(1).padRight(6, '0').substring(0, 6);
    micros = int.parse(digits);
  }
  var at = DateTime.utc(year, month, day, hour, minute, second, 0, micros);
  if (m.group(8) != null) {
    final oh = int.parse(m.group(9)!);
    final om = int.parse(m.group(10)!);
    if (oh > 14 || om > 59 || (oh == 14 && om != 0)) return null;
    final offset = Duration(hours: oh, minutes: om);
    at = m.group(8) == '+' ? at.subtract(offset) : at.add(offset);
  }
  return at;
}

/// Resolves each DID once through [_inner] and answers again from memory, so
/// one verification's checks all read the same document.
final class _OnceResolver implements ssi.DidResolver {
  _OnceResolver(this._inner);

  final ssi.DidResolver _inner;
  final _resolved = <String, Future<ssi.DidDocument>>{};

  @override
  Future<ssi.DidDocument> resolveDid(String did) =>
      _resolved.putIfAbsent(did, () => _inner.resolveDid(did));
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
