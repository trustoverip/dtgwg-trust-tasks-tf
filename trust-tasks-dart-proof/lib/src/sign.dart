import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:ssi/ssi.dart' as ssi;

/// Why [signTrustTask] refused to sign.
enum SignFailureKind {
  /// The document has no in-band `issuer`. The verifier binds every proof to
  /// it, so a proof minted without one could never verify.
  missingIssuer,

  /// The document's `issuer` is not the DID of the signer's verification
  /// method. The signature would be valid and the document still rejected by
  /// every conforming verifier, as an issuer-spoofing attempt.
  issuerMismatch,

  /// The signer's key cannot produce the requested cryptosuite, or no JCS
  /// cryptosuite exists for its key type.
  unsupportedKey,
}

/// Thrown by [signTrustTask] for a document that could never verify.
final class SignException implements Exception {
  const SignException(this.kind, this.message);

  final SignFailureKind kind;
  final String message;

  @override
  String toString() => 'SignException($kind): $message';
}

/// Signs a Trust Task document and returns a copy with an embedded `proof`.
///
/// The counterpart of `DataIntegrityProofVerifier`, and of `sign_trust_task`
/// in the Rust `trust-tasks-proof` crate: what this emits, both verify.
///
/// - The proof covers the document minus its `proof` member. **An existing
///   `proof` is replaced**, never nested or signed over.
/// - The document's `issuer` must equal the DID of [signer]'s verification
///   method, exactly. That is the binding the verifier enforces, so a document
///   that could not verify is refused here with a [SignException] rather than
///   at the consumer.
/// - [cryptosuite] defaults from the key: `eddsa-jcs-2022` for Ed25519,
///   `ecdsa-jcs-2019` for P-256 and P-384.
/// - `created` is emitted in UTC with a trailing `Z`, as VC Data Integrity
///   §2.1 requires, and without a fraction when that would be all zeros, as the
///   Rust crate writes it. Pass [created] to fix it; it defaults to now.
///
/// This builds the proof itself rather than calling `ssi`'s generators, whose
/// `created` lacked a zone designator before ssi 4.3.0 — and this package
/// supports ssi 3.9 too.
Future<Map<String, dynamic>> signTrustTask(
  Map<String, dynamic> doc,
  ssi.DidSigner signer, {
  String? cryptosuite,
  String proofPurpose = 'assertionMethod',
  DateTime? created,
}) async {
  final unsigned = Map<String, dynamic>.of(doc)..remove('proof');

  final issuer = unsigned['issuer'];
  if (issuer is! String) {
    throw const SignException(
      SignFailureKind.missingIssuer,
      'document carries no in-band issuer to bind the proof to',
    );
  }
  final verificationMethod = signer.didKeyId;
  final vmDid = verificationMethod.split('#').first;
  if (vmDid != issuer) {
    throw SignException(
      SignFailureKind.issuerMismatch,
      "signer's verificationMethod is controlled by $vmDid, "
      'not the document issuer $issuer',
    );
  }

  final (suite, digest) = _suiteFor(signer.signatureScheme, cryptosuite);

  final config = <String, dynamic>{
    'type': 'DataIntegrityProof',
    'cryptosuite': suite,
    'created': _timestamp(created ?? DateTime.now()),
    'verificationMethod': verificationMethod,
    'proofPurpose': proofPurpose,
  };

  // eddsa-jcs-2022 §3.3.4 and ecdsa-jcs-2019 §3.3.4: hash the canonical proof
  // configuration and the canonical document separately, and sign the two
  // digests concatenated, configuration first.
  final hashData = Uint8List.fromList([
    ...digest.convert(utf8.encode(ssi.JcsUtil.canonicalize(config))).bytes,
    ...digest.convert(utf8.encode(ssi.JcsUtil.canonicalize(unsigned))).bytes,
  ]);
  final signature = await signer.sign(hashData);

  return {
    ...unsigned,
    'proof': {...config, 'proofValue': ssi.toMultiBase(signature)},
  };
}

/// UTC, `Z`-terminated, and without a fraction when it would be all zeros —
/// the form chrono's `AutoSi` produces, so a proof signed here and one signed
/// by the Rust crate at the same instant carry the same bytes.
String _timestamp(DateTime t) =>
    t.toUtc().toIso8601String().replaceFirst(RegExp(r'\.0+Z$'), 'Z');

(String, Hash) _suiteFor(ssi.SignatureScheme scheme, String? requested) {
  final (String suite, Hash digest) = switch (scheme) {
    ssi.SignatureScheme.ed25519 => ('eddsa-jcs-2022', sha256),
    ssi.SignatureScheme.ecdsa_p256_sha256 => ('ecdsa-jcs-2019', sha256),
    ssi.SignatureScheme.ecdsa_p384_sha384 => ('ecdsa-jcs-2019', sha384),
    _ => throw SignException(
        SignFailureKind.unsupportedKey,
        'no supported JCS cryptosuite for signature scheme ${scheme.name}',
      ),
  };
  if (requested != null && requested != suite) {
    throw SignException(
      SignFailureKind.unsupportedKey,
      'a ${scheme.name} key cannot produce $requested; it produces $suite',
    );
  }
  return (suite, digest);
}
