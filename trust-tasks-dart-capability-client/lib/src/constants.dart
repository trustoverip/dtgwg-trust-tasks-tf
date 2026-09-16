/// Type URIs and extended error codes for the capability Trust Task families —
/// the wire contract a capability producer and a management UI share.
library;

/// The `trust_tasks_didcomm` binding envelope type a registry's DIDComm handler
/// listens for.
const trustTaskEnvelopeType =
    'https://trusttasks.org/binding/didcomm/0.1/envelope';

/// `governance/capability/*` type URIs.
const capabilityListType =
    'https://trusttasks.org/spec/governance/capability/list/0.1';
const capabilityEnableType =
    'https://trusttasks.org/spec/governance/capability/enable/0.1';
const capabilityDisableType =
    'https://trusttasks.org/spec/governance/capability/disable/0.1';

/// `git-trust/*` type URIs.
const gitTrustGrantType = 'https://trusttasks.org/spec/git-trust/grant/0.1';
const gitTrustRevokeType = 'https://trusttasks.org/spec/git-trust/revoke/0.1';

/// The extended error code `git-trust/grant` declares for "an active grant
/// already exists" (SPEC §8.5) — the control surface for idempotent success,
/// never decided from the non-normative free-text `message`.
const gitTrustAlreadyGrantedCode = 'git-trust/grant:already_granted';

/// The `git-trust/revoke` counterpart.
const gitTrustNotGrantedCode = 'git-trust/revoke:not_granted';

/// The lowerCamelCase spellings the registry may normalise to (SPEC §4.10
/// rule 4). Accepting both means that normalisation is not a flag day.
const gitTrustAlreadyGrantedCodeCamel = 'git-trust/grant:alreadyGranted';
const gitTrustNotGrantedCodeCamel = 'git-trust/revoke:notGranted';
