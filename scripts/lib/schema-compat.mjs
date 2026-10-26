/**
 * Is one payload schema a compatible successor of another?
 *
 * Two questions in this repository reduce to the same comparison:
 *
 *   - May a schema that has already been RELEASED be edited in place?
 *     (scripts/check-published-schemas.mjs) A released schema is compiled
 *     into the Rust, TypeScript, Go and Dart libraries; an in-place change
 *     that alters the generated types is a breaking release of every crate in
 *     the release group. #858, #884, #758 and #750 each cost a MINOR that way.
 *   - Does a new MINOR that declares `extends: <older>` really extend it?
 *     A consumer of the older version accepts the newer one's documents by
 *     ignoring the members it does not know (SPEC §5.2), and the libraries
 *     route on that promise — so it has to be true, not asserted.
 *
 * In both, the successor may only ADD: a new optional member, a value
 * appended to an enum, a branch appended to a oneOf/anyOf, a new `$defs`
 * entry, a loosened bound (a smaller minimum, a larger maximum — not a dropped
 * one, which can change the generated type), and any change to `description`, `$comment` or `examples`. Everything
 * else — removing or renaming a member, changing a type, adding a required
 * member or making one optional, narrowing or changing a pattern or format,
 * inserting an enum value before an existing one, changing a `title` (it names
 * a generated type) — is a change a consumer or the generated code can see,
 * and takes a new version.
 *
 * `$ref`s are followed through both trees, so a re-pin to a newer shared
 * version is judged by what it points at: a re-pin that changes nothing is
 * compatible, one that changes the shape is not.
 */
import path from 'node:path';

// Keywords whose value may change freely: annotations that generate nothing but documentation.
const ANNOTATIONS = new Set(['description', '$comment', 'examples', '$id', '$schema']);
// Bounds a successor may only loosen: lower bounds may fall, upper bounds may rise, either may go.
const LOWER = new Set(['minLength', 'minimum', 'exclusiveMinimum', 'minItems', 'minProperties']);
const UPPER = new Set(['maxLength', 'maximum', 'exclusiveMaximum', 'maxItems', 'maxProperties']);
// Keywords handled structurally below.
const STRUCTURAL = new Set([
  'type', 'properties', 'required', 'enum', 'const', 'items', 'prefixItems', 'additionalProperties',
  '$defs', 'definitions', 'oneOf', 'anyOf', 'allOf', '$ref', 'title',
]);

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const esc = (k) => String(k).replace(/~/g, '~0').replace(/\//g, '~1');
const typesOf = (node) => [].concat(node?.type ?? []).sort();

/**
 * Compare `before` and `after`. `loadBefore(file)` / `loadAfter(file)` return
 * the parsed JSON of a file in each tree (or throw), for following `$ref`s;
 * `beforeFile` / `afterFile` are the files the two documents came from.
 * `acrossVersions` compares two versions of a slug (an `extends` declaration)
 * rather than one file over time: titles, which name generated types within a
 * module, may then differ.
 * Returns `[{ pointer, reason }]` — empty when `after` is a compatible successor.
 */
export function incompatibilities(before, after, { beforeFile, afterFile, loadBefore, loadAfter, acrossVersions = false }) {
  const found = [];
  const visiting = new Set();

  const resolve = (ref, file, load) => {
    const [target, fragment = ''] = ref.split('#');
    const targetFile = target ? path.join(path.dirname(file), target) : file;
    let node = load(targetFile);
    for (const raw of fragment.replace(/^\//, '').split('/').filter(Boolean)) {
      node = node?.[raw.replace(/~1/g, '/').replace(/~0/g, '~')];
    }
    return { node, file: targetFile };
  };

  const walk = (b, a, bFile, aFile, pointer) => {
    if (same(b, a) && bFile === aFile) return;
    if (b === true && a === true) return;
    if (typeof b !== 'object' || b === null || typeof a !== 'object' || a === null) {
      if (!same(b, a)) found.push({ pointer, reason: `changed from ${JSON.stringify(b)} to ${JSON.stringify(a)}` });
      return;
    }
    // A `$ref` on either side: compare what each side points at.
    if (b.$ref || a.$ref) {
      if (!b.$ref || !a.$ref) {
        found.push({ pointer, reason: 'replaced a `$ref` with an inline schema, or the reverse' });
        return;
      }
      if (b.$ref.startsWith('../../') && b.$ref.includes('/_framework/') && a.$ref.includes('/_framework/')) {
        // A framework `$ref` re-pin (Ext and friends): the framework owns its own compatibility.
      } else {
        let rb;
        let ra;
        try {
          rb = resolve(b.$ref, bFile, loadBefore);
          ra = resolve(a.$ref, aFile, loadAfter);
        } catch (e) {
          found.push({ pointer, reason: `a \`$ref\` could not be followed (${e.message})` });
          return;
        }
        const key = `${rb.file}#${b.$ref.split('#')[1] ?? ''}|${ra.file}#${a.$ref.split('#')[1] ?? ''}`;
        if (!visiting.has(key)) {
          visiting.add(key);
          // A renamed `$defs` target renames the generated type.
          const bName = b.$ref.split('/').at(-1);
          const aName = a.$ref.split('/').at(-1);
          if (bName !== aName) found.push({ pointer, reason: `\`$ref\` now names \`${aName}\` instead of \`${bName}\`, which renames a generated type` });
          walk(rb.node, ra.node, rb.file, ra.file, `${pointer}→${a.$ref}`);
        }
      }
      for (const k of Object.keys({ ...b, ...a })) {
        if (k !== '$ref' && !ANNOTATIONS.has(k) && !same(b[k], a[k])) {
          found.push({ pointer: `${pointer}/${esc(k)}`, reason: 'a keyword beside a `$ref` changed' });
        }
      }
      return;
    }
    if (!same(typesOf(b), typesOf(a))) {
      found.push({ pointer: `${pointer}/type`, reason: `type changed from ${JSON.stringify(b.type)} to ${JSON.stringify(a.type)}` });
    }
    // Across versions each is its own generated module, so a title naming the version may differ.
    if (!acrossVersions && !same(b.title, a.title) && b.title !== undefined) {
      found.push({ pointer: `${pointer}/title`, reason: 'the `title` names a generated type; changing it renames that type' });
    }
    if (!same(b.const, a.const)) found.push({ pointer: `${pointer}/const`, reason: 'the `const` changed' });
    if (Array.isArray(b.enum) || Array.isArray(a.enum)) {
      const be = b.enum ?? [];
      const ae = a.enum ?? [];
      if (!Array.isArray(b.enum) || !Array.isArray(a.enum) || !be.every((v, i) => same(v, ae[i]))) {
        found.push({ pointer: `${pointer}/enum`, reason: `values may only be appended (base ${JSON.stringify(be)}, now ${JSON.stringify(ae)})` });
      }
    }
    // Members: none removed; no new required member; none made optional.
    const bp = b.properties ?? {};
    const ap = a.properties ?? {};
    for (const k of Object.keys(bp)) {
      if (!(k in ap)) found.push({ pointer: `${pointer}/properties/${esc(k)}`, reason: `member \`${k}\` was removed or renamed` });
      else walk(bp[k], ap[k], bFile, aFile, `${pointer}/properties/${esc(k)}`);
    }
    const br = new Set(b.required ?? []);
    const ar = new Set(a.required ?? []);
    for (const k of ar) if (!br.has(k)) found.push({ pointer: `${pointer}/required`, reason: `\`${k}\` became required` });
    for (const k of br) if (!ar.has(k) && k in ap) found.push({ pointer: `${pointer}/required`, reason: `\`${k}\` is no longer required — its generated type changes from T to Option<T>` });
    if (!same(b.additionalProperties, a.additionalProperties)) {
      if (typeof b.additionalProperties === 'object' && typeof a.additionalProperties === 'object') {
        walk(b.additionalProperties, a.additionalProperties, bFile, aFile, `${pointer}/additionalProperties`);
      } else {
        found.push({ pointer: `${pointer}/additionalProperties`, reason: `changed from ${JSON.stringify(b.additionalProperties)} to ${JSON.stringify(a.additionalProperties)}` });
      }
    }
    for (const kw of ['items', 'prefixItems']) {
      if (b[kw] === undefined && a[kw] === undefined) continue;
      if (b[kw] === undefined || a[kw] === undefined) found.push({ pointer: `${pointer}/${kw}`, reason: `\`${kw}\` added or removed` });
      else walk(b[kw], a[kw], bFile, aFile, `${pointer}/${kw}`);
    }
    for (const kw of ['$defs', 'definitions']) {
      for (const k of Object.keys(b[kw] ?? {})) {
        if (!(k in (a[kw] ?? {}))) found.push({ pointer: `${pointer}/${kw}/${esc(k)}`, reason: `definition \`${k}\` was removed or renamed` });
        else walk(b[kw][k], a[kw][k], bFile, aFile, `${pointer}/${kw}/${esc(k)}`);
      }
    }
    // oneOf / anyOf may gain branches at the end; allOf may not change shape.
    for (const kw of ['oneOf', 'anyOf', 'allOf']) {
      const bb = b[kw];
      const ab = a[kw];
      if (bb === undefined && ab === undefined) continue;
      if (!Array.isArray(bb) || !Array.isArray(ab) || ab.length < bb.length || (kw === 'allOf' && ab.length !== bb.length)) {
        found.push({ pointer: `${pointer}/${kw}`, reason: kw === 'allOf' ? '`allOf` changed shape' : `branches may only be appended to \`${kw}\`` });
        continue;
      }
      bb.forEach((x, i) => walk(x, ab[i], bFile, aFile, `${pointer}/${kw}/${i}`));
    }
    for (const k of new Set([...Object.keys(b), ...Object.keys(a)])) {
      if (ANNOTATIONS.has(k) || STRUCTURAL.has(k) || same(b[k], a[k])) continue;
      // Loosened, not dropped: a string or number that loses its last bound can generate a
      // different type (a constrained newtype becomes a plain String), which is a break.
      if (LOWER.has(k) && b[k] !== undefined && a[k] !== undefined && a[k] <= b[k]) continue;
      if (UPPER.has(k) && b[k] !== undefined && a[k] !== undefined && a[k] >= b[k]) continue;
      found.push({
        pointer: `${pointer}/${esc(k)}`,
        reason: LOWER.has(k) || UPPER.has(k) ? `\`${k}\` narrowed (${JSON.stringify(b[k])} → ${JSON.stringify(a[k])})` : `\`${k}\` changed (${JSON.stringify(b[k])} → ${JSON.stringify(a[k])})`,
      });
    }
  };

  walk(before, after, beforeFile, afterFile, '');
  return found;
}

/**
 * Front-matter changes that alter a released spec's behaviour in the
 * libraries: the proof policy (`IS_PROOF_REQUIRED` and friends) and the
 * declared error codes (`ERROR_CODES` and its constants), which may only grow.
 */
export function frontMatterIncompatibilities(before, after) {
  const found = [];
  if (!same(before.proofRequirement?.requirement, after.proofRequirement?.requirement)) {
    found.push({
      pointer: 'proofRequirement.requirement',
      reason: `changed from ${before.proofRequirement?.requirement} to ${after.proofRequirement?.requirement} — a consumer now accepts or refuses documents it did not`,
    });
  }
  const codes = (fm) => (fm.errorCodes ?? []).map((c) => c.code);
  const now = new Set(codes(after));
  for (const c of codes(before)) if (!now.has(c)) found.push({ pointer: 'errorCodes', reason: `declared error code \`${c}\` was removed` });
  return found;
}
