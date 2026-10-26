#!/usr/bin/env node
/**
 * Rewrite every stale TEXT reference the stale-references check reports — a
 * prose link or mention, or a schema description, naming a superseded
 * version of another specification — to name that specification's latest
 * version. See scripts/lib/stale-references.mjs for what counts.
 *
 * It never touches:
 *   - a WIRE pin (a schema `const`/`enum`/`pattern`/`$ref`): changing it
 *     changes what validates, so it takes a new version of the referencing
 *     spec (SPEC §5);
 *   - an EXAMPLE (a Type URI inside a sample document): the sample's payload
 *     was written for the old version and has to be checked against the new
 *     one by hand;
 *   - a NORMATIVE value (a full Type URI in prose, or a code-span version in a
 *     sentence with an RFC 2119 keyword): it says what a conforming
 *     implementation emits or accepts, so moving it changes conformance;
 *   - a link whose `#anchor` the latest version no longer has — the section
 *     moved, and only a reader can say where it went.
 * Each is listed at the end, for a person to handle.
 *
 * When a link's target moves, a version number standing alone in its label
 * (`[`approve-request` 0.3](…/0.3/spec.md)`) moves with it.
 *
 *   npm run fix-stale-references            # rewrite in place
 *   npm run fix-stale-references -- --dry   # list what would change
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { staleReferences } from './lib/stale-references.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS_DIR = path.join(ROOT, 'specs');
const dry = process.argv.includes('--dry');
const allowlist = JSON.parse(fs.readFileSync(path.join(ROOT, 'scripts', 'lib', 'stale-references-allowlist.json'), 'utf8')).exceptions ?? {};

/** GitHub-style heading anchors of a spec.md. */
const anchorCache = new Map();
function anchorsOf(file) {
  if (!anchorCache.has(file)) {
    const set = new Set();
    let fence = false;
    for (const line of fs.readFileSync(file, 'utf8').split('\n')) {
      if (line.startsWith('```')) fence = !fence;
      const m = !fence && line.match(/^#{1,6}\s+(.*?)\s*#*\s*$/);
      if (!m) continue;
      const slug = m[1]
        .toLowerCase()
        .replace(/<[^>]+>/g, '')
        .replace(/[^\p{L}\p{N}\s_-]/gu, '')
        .replace(/\s/g, '-');
      set.add(slug);
    }
    for (const m of fs.readFileSync(file, 'utf8').matchAll(/<a\s+(?:name|id)="([^"]+)"/g)) set.add(m[1]);
    anchorCache.set(file, set);
  }
  return anchorCache.get(file);
}

const findings = staleReferences(SPECS_DIR, allowlist).filter((f) => !f.allowed);
const manual = findings.filter((f) => f.kind !== 'text');
const byFile = new Map();
for (const f of findings.filter((f) => f.kind === 'text')) {
  const file = path.join(f.dir, f.file);
  if (!byFile.has(file)) byFile.set(file, []);
  byFile.get(file).push(f);
}

let rewritten = 0;
for (const [file, list] of byFile) {
  let text = fs.readFileSync(file, 'utf8');
  // Highest offset first, so an edit never shifts one still to be made.
  for (const f of list.sort((a, b) => b.index - a.index)) {
    const lineStart = text.lastIndexOf('\n', f.index) + 1;
    const lineEnd = text.indexOf('\n', f.index) === -1 ? text.length : text.indexOf('\n', f.index);
    const line = text.slice(lineStart, lineEnd);
    const at = f.index - lineStart;
    // Is the version inside a markdown link target? Then check its anchor, and move its label too.
    let link = null;
    for (const m of line.matchAll(/\[((?:[^\]\\]|\\.)*)\]\(([^)\s]*)\)/g)) {
      const hrefAt = m.index + m[0].length - 1 - m[2].length;
      if (at >= hrefAt && at < hrefAt + m[2].length) link = { m, hrefAt };
    }
    if (link) {
      const anchor = link.m[2].split('#')[1];
      const target = path.join(SPECS_DIR, f.target, f.latest, 'spec.md');
      if (anchor && !anchorsOf(target).has(anchor)) {
        manual.push({ ...f, kind: 'anchor', anchor });
        continue;
      }
    }
    let newLine = line.slice(0, at) + f.latest + line.slice(at + f.version.length);
    if (link) {
      // The label precedes the target; a lone version number in it names the same spec.
      const labelAt = link.m.index + 1;
      const label = link.m[1];
      const lone = new RegExp(`(?<![\\d./])${f.version.replace('.', '\\.')}(?![\\d.])`, 'g');
      const moved = label.replace(lone, f.latest);
      if (moved !== label) newLine = newLine.slice(0, labelAt) + moved + newLine.slice(labelAt + label.length);
    }
    text = text.slice(0, lineStart) + newLine + text.slice(lineEnd);
    rewritten++;
    if (dry) console.log(`${path.relative(ROOT, file)}:${f.line}: ${f.target}/${f.version} → ${f.latest}`);
  }
  if (!dry) fs.writeFileSync(file, text);
}

console.log(`${dry ? 'Would rewrite' : 'Rewrote'} ${rewritten} reference(s) in ${byFile.size} file(s).`);
if (manual.length) {
  console.log(`\n${manual.length} reference(s) need a person:`);
  for (const f of manual) {
    const why =
      f.kind === 'wire'
        ? 'on the wire — a new version of the referencing spec'
        : f.kind === 'example'
          ? 'inside a sample document — check its payload against the new version'
          : f.kind === 'normative'
            ? 'a value a conforming implementation emits or checks — moving it changes conformance'
            : `#${f.anchor} does not exist in ${f.target}/${f.latest} — find where the section went`;
    console.log(`  specs/${f.slug}/${f.specVersion}/${f.file}:${f.line}: ${f.target}/${f.version} → ${f.latest} (${why})`);
  }
}
