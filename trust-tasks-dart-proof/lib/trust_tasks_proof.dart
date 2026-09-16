/// W3C Data Integrity proofs for Trust Tasks.
///
/// [DataIntegrityProofVerifier] fills the `ProofVerifier` seam that
/// `package:trust_tasks` declares, so `ProofPolicy.verify` has a real
/// cryptosuite behind it; [signTrustTask] produces what it verifies.
library;

export 'src/data_integrity_verifier.dart';
export 'src/sign.dart';
