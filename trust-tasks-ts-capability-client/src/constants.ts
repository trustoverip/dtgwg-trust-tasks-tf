/**
 * Type URIs and extended error codes for the capability Trust Task families.
 * These are the wire contract a capability producer (a community service) and a
 * management UI share, so both agree on what to send and how to read a reply.
 */

/** The `trust-tasks-didcomm` binding envelope type a registry's DIDComm handler
 * listens for. */
export const TRUST_TASK_ENVELOPE_TYPE = "https://trusttasks.org/binding/didcomm/0.1/envelope";

/** `governance/capability/*` type URIs. */
export const CAPABILITY_LIST_TYPE = "https://trusttasks.org/spec/governance/capability/list/0.1";
export const CAPABILITY_ENABLE_TYPE = "https://trusttasks.org/spec/governance/capability/enable/0.1";
export const CAPABILITY_DISABLE_TYPE = "https://trusttasks.org/spec/governance/capability/disable/0.1";

/** `git-trust/*` type URIs. */
export const GIT_TRUST_GRANT_TYPE = "https://trusttasks.org/spec/git-trust/grant/0.1";
export const GIT_TRUST_REVOKE_TYPE = "https://trusttasks.org/spec/git-trust/revoke/0.1";

/**
 * The extended error code `git-trust/grant` declares for "an active grant
 * already exists for this subject and resource" (SPEC §8.5). This is the control
 * surface for idempotent success on a grant — never decide it from the free-text
 * `message`, which SPEC §8.2 makes non-normative.
 */
export const GIT_TRUST_ALREADY_GRANTED_CODE = "git-trust/grant:already_granted";

/** The extended error code `git-trust/revoke` declares for "no active grant
 * exists for this subject and resource". */
export const GIT_TRUST_NOT_GRANTED_CODE = "git-trust/revoke:not_granted";

/**
 * The lowerCamelCase spellings of the two codes above. The registry entries
 * declare the snake_case forms a conforming emitter sends today; SPEC §4.10
 * rule 4 SHOULDs lowerCamelCase, so the registry may normalise. Accepting both
 * means that normalisation is not a flag day. Both are namespaced extended
 * codes either way — neither is free text.
 */
export const GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL = "git-trust/grant:alreadyGranted";
export const GIT_TRUST_NOT_GRANTED_CODE_CAMEL = "git-trust/revoke:notGranted";
