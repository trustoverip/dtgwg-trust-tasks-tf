/// Client-side wire helpers for the capability Trust Task families —
/// `governance/capability/*` (enable / disable / list a community capability)
/// and `git-trust/*` (grant / revoke commit-signing trust).
///
/// This package owns the *documents*, not a transport: it builds request
/// documents, parses inbound envelope replies, and classifies them. Signing is
/// deliberately not here — attach a Data Integrity proof with
/// `package:trust_tasks_proof` (over the document minus its `proof` member,
/// `eddsa-jcs-2022`) — so this package stays free of any crypto dependency, and
/// a capability producer and a management UI cannot drift on the contract.
library;

export 'src/constants.dart';
export 'src/document.dart';
export 'src/replies.dart';
