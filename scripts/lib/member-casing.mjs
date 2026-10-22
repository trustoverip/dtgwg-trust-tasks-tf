/**
 * The payload member-name and enumerated-value casing lint.
 *
 * SPEC §4.10 rule 3 says a specification SHOULD name its payload members in
 * lowerCamelCase, and rule 4 says the same of string values drawn from a closed
 * set the specification itself defines (statuses, kinds, event types). Rule 5
 * is the exception: a value or member name fixed by an external vocabulary —
 * WebAuthn, JOSE, OpenID, JSON-LD — MUST be carried verbatim. Nothing checked
 * any of this, so the registry drifted: kebab-case permission names, snake_case
 * event types and PascalCase kinds sit beside lowerCamelCase ones in the same
 * families.
 *
 * ── What is checked ─────────────────────────────────────────────────────────
 *
 * Every `*.schema.json` under specs/ — payload schemas and the shared `$defs`
 * files alike — for:
 *
 *   - the names under every `properties` object, and
 *   - every string in an `enum`, and every string `const`.
 *
 * lowerCamelCase is /^[a-z][a-zA-Z0-9]*$/, as in error-code-casing.mjs. A value
 * of dotted lowerCamelCase segments (`git.repo.create`, `vault.upserted`) is a
 * namespaced identifier and conforms. A value containing `:` or `/` is a URI,
 * a DID method or a media type — each with its own grammar, out of scope as
 * slugs are under rule 6 — and is not checked.
 *
 * ── The allowlist (member-casing-allowlist.json) ────────────────────────────
 *
 *   - `external.members` / `external.values`: names and values owned by an
 *     external vocabulary (rule 5), each mapped to that vocabulary. Accepted
 *     anywhere. Add to it only with the vocabulary named.
 *   - `frozen`: deviations published before this lint, keyed by spec family
 *     (the schema's directory under specs/ with its version directory removed,
 *     e.g. `acl/grant`, `auth/_shared`). Re-casing one is a breaking change
 *     (§4.10, final paragraph), so a family keeps its own across versions, but
 *     another family cannot adopt one. This list is debt: it may only shrink,
 *     and an entry no longer found is reported as stale.
 *
 * `retired` specs are skipped and counted, as in the other casing lint: a
 * retired schema is frozen (§5.3) and can take no new version.
 */
import fs from 'node:fs';
import path from 'node:path';

import { LOWER_CAMEL } from './error-code-casing.mjs';

const DOTTED_LOWER_CAMEL = /^[a-z][a-zA-Z0-9]*(\.[a-z][a-zA-Z0-9]*)+$/;

/** Whether `value` is outside this lint's scope or already conforms. */
function conformingValue(value) {
  return LOWER_CAMEL.test(value) || DOTTED_LOWER_CAMEL.test(value) || /[:/]/.test(value);
}

/** The spec family a schema file belongs to: its directory, version removed. */
export function familyOf(relPath) {
  return path
    .dirname(relPath)
    .split(path.sep)
    .filter((segment) => !/^\d+\.\d+$/.test(segment))
    .join('/');
}

/** Every `{ kind, name }` in `schema` that is not lowerCamelCase. */
export function offendersIn(schema) {
  const found = [];
  const visit = (node, seen) => {
    if (!node || typeof node !== 'object' || seen.has(node)) return;
    seen.add(node);
    if (Array.isArray(node)) {
      for (const item of node) visit(item, seen);
      return;
    }
    if (node.properties && typeof node.properties === 'object' && !Array.isArray(node.properties)) {
      for (const name of Object.keys(node.properties)) {
        if (!LOWER_CAMEL.test(name)) found.push({ kind: 'member', name });
      }
    }
    for (const value of Array.isArray(node.enum) ? node.enum : []) {
      if (typeof value === 'string' && !conformingValue(value)) found.push({ kind: 'value', name: value });
    }
    if (typeof node.const === 'string' && !conformingValue(node.const)) {
      found.push({ kind: 'value', name: node.const });
    }
    for (const [key, child] of Object.entries(node)) {
      // `examples` and `default` restate values, they do not define them.
      if (key !== 'enum' && key !== 'examples' && key !== 'default') visit(child, seen);
    }
  };
  visit(schema, new Set());
  return found;
}

function schemaFiles(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) schemaFiles(p, out);
    else if (entry.name.endsWith('.schema.json') && entry.name !== 'spec.meta.schema.json') out.push(p);
  }
  return out;
}

function statusOf(schemaPath) {
  const prose = path.join(path.dirname(schemaPath), 'spec.md');
  if (!fs.existsSync(prose)) return null;
  return fs.readFileSync(prose, 'utf8').match(/^status:\s*["']?([a-z]+)/m)?.[1] ?? null;
}

/**
 * Run the lint over every schema under `specsDir`. Failures go to `fail`, the
 * summary line and stale-entry notices to `log` and `warn`.
 */
export function checkMemberCasing({ specsDir, allowlistPath, fail, warn, log }) {
  let allowlist;
  try {
    allowlist = JSON.parse(fs.readFileSync(allowlistPath, 'utf8'));
  } catch (e) {
    // Never fail open: a lint that cannot read its allowlist checks nothing.
    fail(path.relative(path.dirname(specsDir), allowlistPath), `could not be read: ${e.message}`);
    return;
  }
  const externalMembers = new Set(Object.keys(allowlist.external?.members ?? {}));
  const externalValues = new Set(Object.keys(allowlist.external?.values ?? {}));
  const frozen = new Map(Object.entries(allowlist.frozen ?? {}).map(([f, names]) => [f, new Set(names)]));
  const frozenSeen = new Map();
  const counts = { schemas: 0, external: 0, frozen: 0, retired: 0, offending: 0 };

  for (const file of schemaFiles(specsDir)) {
    const rel = path.relative(specsDir, file);
    let schema;
    try {
      schema = JSON.parse(fs.readFileSync(file, 'utf8'));
    } catch {
      continue; // an unparseable schema is reported by the schema checks
    }
    counts.schemas++;
    const family = familyOf(rel);
    const retired = statusOf(file) === 'retired';
    for (const { kind, name } of offendersIn(schema)) {
      if ((kind === 'member' ? externalMembers : externalValues).has(name)) {
        counts.external++;
      } else if (frozen.get(family)?.has(name)) {
        counts.frozen++;
        if (!frozenSeen.has(family)) frozenSeen.set(family, new Set());
        frozenSeen.get(family).add(name);
      } else if (retired) {
        counts.retired++;
      } else {
        counts.offending++;
        fail(
          `specs/${rel}`,
          `${kind === 'member' ? 'member name' : 'enumerated value'} ${JSON.stringify(name)} is not ` +
            `lowerCamelCase (SPEC §4.10 rules 3–4). If an external vocabulary fixes it (rule 5), ` +
            `add it to scripts/lib/member-casing-allowlist.json under external with that vocabulary named.`
        );
      }
    }
  }

  for (const [family, names] of frozen) {
    for (const name of names) {
      if (!frozenSeen.get(family)?.has(name)) {
        warn(`member-casing-allowlist.json: frozen ${JSON.stringify(name)} for ${family} is no longer used — delete the entry`);
      }
    }
  }

  log(
    `  Member and value casing: ${counts.schemas} schema(s) swept; ${counts.external} external-vocabulary ` +
      `occurrence(s) (§4.10 rule 5), ${counts.frozen} frozen pre-lint deviation(s), ` +
      `${counts.retired} in retired spec(s), ${counts.offending} non-conforming`
  );
}
