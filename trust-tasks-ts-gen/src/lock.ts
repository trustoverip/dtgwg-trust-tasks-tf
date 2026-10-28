/**
 * `trust-tasks.lock` — read, never written.
 *
 * The lockfile is written only by the `trust-tasks` CLI, in one fixed shape:
 *
 * ```toml
 * # Written by `trust-tasks`. Do not edit by hand.
 * version = 1
 * registry = "https://trusttasks.org"
 *
 * [[task]]
 * slug = "vault/upsert"
 * version = "0.4"
 * type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
 * digest = "sha256:<hex>"
 * ```
 *
 * A lock may also name third-party sources (SPEC §6.5 private specifications),
 * each a `[sources.<name>]` table with exactly one of `url` / `path`, and a
 * task from one carries `source = "<name>"`. Generation reads only vendored
 * files, so `url` / `path` are checked for shape and otherwise ignored.
 *
 * This is a strict parser of exactly that shape rather than a TOML library:
 * comments and blank lines, two top-level keys, `[sources.<name>]` tables,
 * `[[task]]` tables of four or five string keys. Anything else is refused, so
 * a hand edit that the CLI would not have written fails loudly here instead of
 * being half-understood.
 */

/** The lockfile shape this generator reads. */
export const LOCK_VERSION = 1;

/** One pinned specification version. */
export interface LockedTask {
  /** The `[sources.<name>]` it comes from, or `null` for the official registry. */
  readonly source: string | null;
  /** e.g. `vault/upsert`. */
  readonly slug: string;
  /** `MAJOR.MINOR`, e.g. `0.4`. */
  readonly version: string;
  /** The request Type URI. */
  readonly typeUri: string;
  /** `sha256:<lowercase hex>` of the bundle file's bytes. */
  readonly digest: string;
}

/** A parsed `trust-tasks.lock`. */
export interface Lockfile {
  readonly version: number;
  /** The registry the versions were resolved against, when recorded. */
  readonly registry: string | null;
  /** `[sources.<name>]` → where it was fetched from (unused by generation). */
  readonly sources: ReadonlyMap<string, { readonly url?: string; readonly path?: string }>;
  /** In file order (official first, then by source, slug and version). */
  readonly tasks: readonly LockedTask[];
}

const TASK_KEYS: Record<string, keyof LockedTask> = {
  slug: "slug",
  version: "version",
  type_uri: "typeUri",
  digest: "digest",
};
const REQUIRED_TASK_KEYS = Object.keys(TASK_KEYS);

/** A source name: what `[sources.<name>]` and a task's `source` may say. */
export const SOURCE_NAME_RE = /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/;

// The CLI's `check_slug`: lowercase path segments of a-z, 0-9 and `-`, none
// starting with `-`. Also what keeps a slug from naming a path outside the
// vendor and output directories.
const SLUG_RE = /^[a-z0-9][a-z0-9-]*(?:\/[a-z0-9][a-z0-9-]*)*$/;
const VERSION_RE = /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/;
const DIGEST_RE = /^sha256:[0-9a-f]{64}$/;

/** Parse a TOML basic string starting at `s[0] === '"'`; returns the value and the rest. */
function basicString(s: string, where: string): { value: string; rest: string } {
  let out = "";
  let i = 1;
  for (; i < s.length; i++) {
    const ch = s[i]!;
    if (ch === '"') return { value: out, rest: s.slice(i + 1) };
    if (ch === "\\") {
      const esc = s[++i];
      switch (esc) {
        case '"': out += '"'; break;
        case "\\": out += "\\"; break;
        case "b": out += "\b"; break;
        case "t": out += "\t"; break;
        case "n": out += "\n"; break;
        case "f": out += "\f"; break;
        case "r": out += "\r"; break;
        case "u":
        case "U": {
          const len = esc === "u" ? 4 : 8;
          const hex = s.slice(i + 1, i + 1 + len);
          if (!/^[0-9A-Fa-f]+$/.test(hex) || hex.length !== len) throw new Error(`${where}: bad \\${esc} escape`);
          out += String.fromCodePoint(parseInt(hex, 16));
          i += len;
          break;
        }
        default:
          throw new Error(`${where}: unsupported escape \\${esc ?? ""}`);
      }
    } else if (ch < " " && ch !== "\t") {
      throw new Error(`${where}: control character in string`);
    } else {
      out += ch;
    }
  }
  throw new Error(`${where}: unterminated string`);
}

/** The value of `key = <value>`, with an optional trailing comment. */
function parseValue(text: string, where: string): string | number {
  let value: string | number;
  let rest: string;
  if (text.startsWith('"')) {
    ({ value, rest } = basicString(text, where));
  } else {
    const m = /^(0|[1-9][0-9]*)(.*)$/.exec(text);
    if (!m) throw new Error(`${where}: expected a string or an integer, got ${JSON.stringify(text)}`);
    value = Number(m[1]);
    rest = m[2]!;
  }
  rest = rest.trim();
  if (rest !== "" && !rest.startsWith("#")) throw new Error(`${where}: unexpected ${JSON.stringify(rest)} after the value`);
  return value;
}

type SourceTable = { url?: string; path?: string };
type Section =
  | { kind: "top" }
  | { kind: "source"; table: SourceTable }
  | { kind: "task"; table: Record<string, string> };

/** Parse the text of a `trust-tasks.lock`. Throws on anything the CLI would not have written. */
export function parseLock(text: string): Lockfile {
  let version: number | undefined;
  let registry: string | null = null;
  const sources = new Map<string, SourceTable>();
  const tasks: Record<string, string>[] = [];
  // Which table the following keys belong to.
  let section: Section = { kind: "top" };

  const lines = text.replace(/^﻿/, "").split(/\r?\n/);
  lines.forEach((rawLine, index) => {
    const where = `trust-tasks.lock line ${index + 1}`;
    const line = rawLine.trim();
    if (line === "" || line.startsWith("#")) return;
    if (/^\[\[\s*task\s*\]\]\s*(#.*)?$/.test(line)) {
      const table: Record<string, string> = {};
      tasks.push(table);
      section = { kind: "task", table };
      return;
    }
    const src = /^\[\s*sources\.([^\]\s]+)\s*\]\s*(#.*)?$/.exec(line);
    if (src) {
      const name = src[1]!.replace(/^"(.*)"$/, "$1");
      if (!SOURCE_NAME_RE.test(name)) {
        throw new Error(`${where}: ${JSON.stringify(name)} is not a source name ([a-z][a-z0-9]*(-[a-z0-9]+)*)`);
      }
      if (sources.has(name)) throw new Error(`${where}: [sources.${name}] given twice`);
      const table: SourceTable = {};
      sources.set(name, table);
      section = { kind: "source", table };
      return;
    }
    if (line.startsWith("[")) throw new Error(`${where}: unexpected table ${line}`);
    const m = /^([A-Za-z0-9_-]+)\s*=\s*(.*)$/.exec(line);
    if (!m) throw new Error(`${where}: expected \`key = value\`, got ${JSON.stringify(line)}`);
    const key = m[1]!;
    const value = parseValue(m[2]!, where);
    const s = section as Section;
    if (s.kind === "top") {
      if (key === "version") {
        if (typeof value !== "number") throw new Error(`${where}: version must be an integer`);
        if (version !== undefined) throw new Error(`${where}: version given twice`);
        version = value;
      } else if (key === "registry") {
        if (typeof value !== "string") throw new Error(`${where}: registry must be a string`);
        if (registry !== null) throw new Error(`${where}: registry given twice`);
        registry = value;
      } else {
        throw new Error(`${where}: unknown top-level key ${key}`);
      }
    } else if (s.kind === "source") {
      if (key !== "url" && key !== "path") throw new Error(`${where}: unknown [sources.*] key ${key}`);
      if (typeof value !== "string") throw new Error(`${where}: ${key} must be a string`);
      if (key in s.table) throw new Error(`${where}: ${key} given twice`);
      s.table[key] = value;
    } else {
      if (!(key in TASK_KEYS) && key !== "source") throw new Error(`${where}: unknown [[task]] key ${key}`);
      if (typeof value !== "string") throw new Error(`${where}: ${key} must be a string`);
      if (key in s.table) throw new Error(`${where}: ${key} given twice in one [[task]]`);
      s.table[key] = value;
    }
  });

  if (version === undefined) throw new Error("trust-tasks.lock: no `version`");
  if (version !== LOCK_VERSION) {
    throw new Error(
      `trust-tasks.lock: lockfile version ${version} is not supported (this generator reads version ${LOCK_VERSION}) — upgrade @openvtc/trust-tasks-gen`,
    );
  }
  for (const [name, table] of sources) {
    if ((table.url === undefined) === (table.path === undefined)) {
      throw new Error(`trust-tasks.lock: [sources.${name}] must have exactly one of url / path`);
    }
  }

  const seen = new Set<string>();
  const parsed: LockedTask[] = tasks.map((t, i) => {
    for (const key of REQUIRED_TASK_KEYS) {
      if (!(key in t)) throw new Error(`trust-tasks.lock: [[task]] #${i + 1} has no ${key}`);
    }
    const source = t.source ?? null;
    if (source !== null && !sources.has(source)) {
      throw new Error(
        `trust-tasks.lock: [[task]] ${t.slug} ${t.version} names source ${JSON.stringify(source)}, which has no [sources.${source}] table`,
      );
    }
    const task: LockedTask = { source, slug: t.slug!, version: t.version!, typeUri: t.type_uri!, digest: t.digest! };
    const label = taskLabel(task);
    if (!SLUG_RE.test(task.slug)) throw new Error(`trust-tasks.lock: ${JSON.stringify(task.slug)} is not a slug`);
    if (!VERSION_RE.test(task.version)) {
      throw new Error(`trust-tasks.lock: ${task.slug}: \`${task.version}\` is not a MAJOR.MINOR version`);
    }
    if (!DIGEST_RE.test(task.digest)) {
      throw new Error(`trust-tasks.lock: ${label}: digest must be sha256:<64 lowercase hex>`);
    }
    if (seen.has(label)) throw new Error(`trust-tasks.lock: ${label} is locked more than once`);
    seen.add(label);
    return task;
  });

  return { version, registry, sources, tasks: parsed };
}

/** `slug/M.N`, or `@source/slug/M.N` for a sourced task — how notes and errors name a task. */
export function taskLabel(task: Pick<LockedTask, "source" | "slug" | "version">): string {
  return `${task.source === null ? "" : `@${task.source}/`}${task.slug}/${task.version}`;
}

