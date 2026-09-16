// A conforming Data Integrity JCS signer, for tests only.
//
// ssi's own generators stamp `created` with a local DateTime, which serializes
// without a zone designator — non-conforming under VC Data Integrity §2.1, and
// rejected by this package exactly as trust-tasks-proof rejects it. Until an ssi
// release carries the fix (affinidi-ssi-dart#305), tests build the proof
// configuration themselves, with a UTC `created`, and hash and sign it the way
// the eddsa-jcs-2022 and ecdsa-jcs-2019 specifications describe.

import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:ssi/ssi.dart';

/// A did:key identity that can sign.
final class TestIdentity {
  TestIdentity._(this.signer, this.cryptosuite);

  /// An Ed25519 did:key, for `eddsa-jcs-2022`.
  factory TestIdentity.ed25519(int seedByte) {
    final keyPair =
        Ed25519KeyPair.fromSeed(Uint8List.fromList(List.filled(32, seedByte)));
    return TestIdentity._(
      _signer(keyPair, SignatureScheme.ed25519),
      'eddsa-jcs-2022',
    );
  }

  /// A P-256 did:key, for `ecdsa-jcs-2019`.
  factory TestIdentity.p256(int seedByte) {
    final keyPair =
        P256KeyPair.fromSeed(Uint8List.fromList(List.filled(32, seedByte)));
    return TestIdentity._(
      _signer(keyPair, SignatureScheme.ecdsa_p256_sha256),
      'ecdsa-jcs-2019',
    );
  }

  /// A P-384 did:key, for `ecdsa-jcs-2019`.
  factory TestIdentity.p384(int seedByte) {
    final keyPair =
        P384KeyPair.fromSeed(Uint8List.fromList(List.filled(48, seedByte)));
    return TestIdentity._(
      _signer(keyPair, SignatureScheme.ecdsa_p384_sha384),
      'ecdsa-jcs-2019',
    );
  }

  static DidSigner _signer(KeyPair keyPair, SignatureScheme scheme) {
    final doc = DidKey.generateDocument(keyPair.publicKey);
    return DidSigner(
      did: doc.id,
      didKeyId: doc.verificationMethod[0].id,
      keyPair: keyPair,
      signatureScheme: scheme,
    );
  }

  final DidSigner signer;
  final String cryptosuite;

  String get did => signer.did;

  /// Returns [doc] with a `proof` over everything else in it.
  Future<Map<String, dynamic>> sign(
    Map<String, dynamic> doc, {
    DateTime? created,
    String? createdText,
  }) async {
    final unsigned = Map<String, dynamic>.of(doc)..remove('proof');
    final config = <String, dynamic>{
      'type': 'DataIntegrityProof',
      'cryptosuite': cryptosuite,
      'created':
          createdText ?? (created ?? DateTime.now()).toUtc().toIso8601String(),
      'verificationMethod': signer.didKeyId,
      'proofPurpose': 'assertionMethod',
    };
    // ecdsa-jcs-2019 hashes with the curve's own digest: SHA-384 for P-384.
    final digest = signer.signatureScheme == SignatureScheme.ecdsa_p384_sha384
        ? sha384
        : sha256;
    final hash = Uint8List.fromList([
      ...digest.convert(utf8.encode(JcsUtil.canonicalize(config))).bytes,
      ...digest.convert(utf8.encode(JcsUtil.canonicalize(unsigned))).bytes,
    ]);
    final signature = await signer.sign(hash);
    return {
      ...unsigned,
      'proof': {...config, 'proofValue': toMultiBase(signature)},
    };
  }
}
