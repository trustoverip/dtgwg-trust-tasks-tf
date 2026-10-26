#!/usr/bin/env node
/**
 * List version pairs that could declare `extends` but do not.
 *
 * `extends: "<M.K>"` (SPEC §5.2 "Declared extension") lets the libraries route a
 * newer version's documents to a handler for the older one, and keeps every
 * reference to the older version current — but only an author can declare it,
 * because the promise covers what members MEAN as well as their shape, and a
 * schema comparison cannot see meaning. This lists each consecutive pair of
 * versions (same slug, same MAJOR) whose schemas and front matter already pass
 * the check the build applies to a declaration (scripts/lib/schema-compat.mjs),
 * for an author to read and declare where the prose agrees.
 *
 *   npm run extends-candidates             # every candidate pair
 *   npm run extends-candidates -- vault    # only slugs containing "vault"
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import { incompatibilities, frontMatterIncompatibilities } from './lib/schema-compat.mjs';

const SPECS = path.join(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'), 'specs');
const filter = process.argv[2] ?? '';
const compare = (a, b) => {
  const [x, y] = a.split('.').map(Number);
  const [u, v] = b.split('.').map(Number);
  return x - u || y - v;
};
const frontMatter = (dir) => YAML.parse(fs.readFileSync(path.join(dir, 'spec.md'), 'utf8').match(/^---\n([\s\S]*?)\n---/)[1]);
const load = (f) => JSON.parse(fs.readFileSync(f, 'utf8'));

const slugs = new Map();
(function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (!entry.isDirectory() || entry.name.startsWith('_')) continue;
    const full = path.join(dir, entry.name);
    if (/^\d+\.\d+$/.test(entry.name) && fs.existsSync(path.join(full, 'payload.schema.json'))) {
      const slug = path.relative(SPECS, dir).split(path.sep).join('/');
      if (!slugs.has(slug)) slugs.set(slug, []);
      slugs.get(slug).push(entry.name);
    } else {
      walk(full);
    }
  }
})(SPECS);

let pairs = 0;
const candidates = [];
for (const [slug, versions] of [...slugs].sort()) {
  if (!slug.includes(filter)) continue;
  versions.sort(compare);
  for (let i = 1; i < versions.length; i++) {
    const [older, newer] = [versions[i - 1], versions[i]];
    if (older.split('.')[0] !== newer.split('.')[0]) continue;
    pairs++;
    const newerDir = path.join(SPECS, slug, newer);
    const olderDir = path.join(SPECS, slug, older);
    const meta = frontMatter(newerDir);
    if (meta.extends || meta.status === 'withdrawn') continue;
    const beforeFile = path.join(olderDir, 'payload.schema.json');
    const afterFile = path.join(newerDir, 'payload.schema.json');
    const found = [
      ...incompatibilities(load(beforeFile), load(afterFile), { beforeFile, afterFile, loadBefore: load, loadAfter: load, acrossVersions: true }),
      ...frontMatterIncompatibilities(frontMatter(olderDir), meta),
    ];
    if (!found.length) candidates.push(`${slug}/${newer}  could declare  extends: "${older}"`);
  }
}
console.log(candidates.join('\n'));
console.log(`\n${candidates.length} of ${pairs} same-MAJOR version pair(s) are additive by their schemas and declare no extends.`);
console.log('Declare it only after reading both versions: the declaration also promises no member changed meaning.');
