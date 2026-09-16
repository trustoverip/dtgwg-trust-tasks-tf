/// W3C Data Integrity proof verification for Trust Tasks.
///
/// [DataIntegrityProofVerifier] fills the `ProofVerifier` seam that
/// `package:trust_tasks` declares, so `ProofPolicy.verify` has a real
/// cryptosuite behind it.
library;

export 'src/data_integrity_verifier.dart';
