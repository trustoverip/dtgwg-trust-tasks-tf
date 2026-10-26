/**
 * @openvtc/trust-tasks-gen — generate TypeScript modules for exactly the Trust
 * Tasks an application pins in `trust-tasks.lock`, from the registry bundles
 * vendored beside it. The CLI is `trust-tasks-gen-ts`; this is the same thing
 * as a library, for build scripts.
 */

export { generate, generateFiles, isFrameworkSlug, MARKER_FILE, type GenerateOptions, type GenerateResult } from "./generate.js";
export { parseLock, taskLabel, LOCK_VERSION, SOURCE_NAME_RE, type Lockfile, type LockedTask } from "./lock.js";
export {
  BUNDLE_VERSION,
  bundlePath,
  OFFICIAL_AUTHORITY,
  parseBundle,
  readLockedBundle,
  sha256Digest,
  withAuthoredOrder,
  type Bundle,
} from "./bundle.js";
export { generateModule, namespaceFor, sourceNamespace, RUNTIME_SPECIFIER, type GeneratedModule } from "./emit.js";
