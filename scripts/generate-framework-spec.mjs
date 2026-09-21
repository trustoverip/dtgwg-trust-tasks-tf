#!/usr/bin/env node
/*
 * Generate SPEC.md from the canonical framework specification.
 *
 * The normative text of the Trust Tasks framework lives in
 * trustoverip/dtgwg-trust-tasks-spec, as Spec-Up-T sources. This repository
 * serves a copy of it — at trusttasks.org/specification, and as the
 * `/SPEC.md#…` target of every cross-reference in the registry — and until
 * this script existed the copy was ported by hand, which is how it came to
 * lack whole sections of the canonical text. SPEC.md is now an output: nothing
 * in it is edited here.
 *
 *   node scripts/generate-framework-spec.mjs                 # latest canonical main
 *   node scripts/generate-framework-spec.mjs --ref <sha>     # a specific commit
 *   node scripts/generate-framework-spec.mjs --from <dir>    # a local checkout
 *   node scripts/generate-framework-spec.mjs --check         # exit 1 if SPEC.md differs
 *
 * What the transformation does, and nothing more:
 *
 *  - concatenates the `markdown_paths` of the canonical `specs.json`, and renders
 *    the Spec-Up-T term definitions (`[[def: …]]`) into the Terminology section;
 *  - numbers the headings in canonical order (§N, §N.M, …; appendices A, B, …),
 *    because the registry cites the framework by section number;
 *  - rewrites the canonical named anchors (`#the-threadid-member`) to the
 *    numbered ones this document carries (`#49-the-threadid-member`), and
 *    `[[ref: term]]` to an italicised term, as the rendered canonical page shows;
 *  - emits every anchor listed in framework-spec-legacy-anchors.json as an alias
 *    on the heading it now names, so a link written against an earlier numbering
 *    keeps landing on the right section.
 *
 * Section numbers are therefore derived, never maintained: when the canonical
 * text adds or moves a section, the numbers move with it, and the registry build
 * fails on any `/SPEC.md#…` link that no longer resolves.
 */
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const OUT = path.join(ROOT, 'SPEC.md');
const ALIASES = path.join(ROOT, 'scripts', 'framework-spec-legacy-anchors.json');
const CANONICAL_REPO = 'https://github.com/trustoverip/dtgwg-trust-tasks-spec';

/* The website's own slug rule (website/assets/pages.jsx `slugifyHeading`), which
 * is also GitHub's for these headings: the anchors emitted here must be the ones
 * the renderer assigns, or no link resolves. */
export function slugifyHeading(text) {
  return text
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9 \-]+/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function parseArgs(argv) {
  const args = { ref: 'main', from: null, check: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--ref') args.ref = argv[++i];
    else if (a === '--from') args.from = argv[++i];
    else if (a === '--check') args.check = true;
    else throw new Error(`unknown argument '${a}'`);
  }
  return args;
}

function checkout({ ref, from }) {
  if (from) {
    const dir = path.resolve(from);
    let sha = 'local';
    try {
      sha = execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
    } catch {
      /* a plain directory is acceptable for local work */
    }
    return { dir, sha };
  }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'tt-framework-spec-'));
  execFileSync('git', ['init', '-q', dir]);
  execFileSync('git', ['-C', dir, 'fetch', '-q', '--depth', '1', CANONICAL_REPO, ref]);
  execFileSync('git', ['-C', dir, 'checkout', '-q', 'FETCH_HEAD']);
  const sha = execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
  return { dir, sha };
}

/* `[[def: first, alias, …]]` followed by a `~ definition` paragraph. The display
 * form is the first alias that starts with a capital, else the first name
 * capitalised — what the rendered canonical glossary shows. */
function renderTerms(termsDir) {
  const terms = [];
  for (const file of fs.readdirSync(termsDir).filter((f) => f.endsWith('.md')).sort()) {
    const src = fs.readFileSync(path.join(termsDir, file), 'utf8');
    const def = /\[\[def:\s*([^\]]+)\]\]/.exec(src);
    if (!def) throw new Error(`${file}: no [[def: …]] line`);
    const names = def[1].split(',').map((s) => s.trim()).filter(Boolean);
    const display = names.find((n) => /^[A-Z]/.test(n)) ?? names[0][0].toUpperCase() + names[0].slice(1);
    const body = src
      .slice(def.index + def[0].length)
      .split('\n')
      .map((l) => l.replace(/^~\s?/, ''))
      .join('\n')
      .trim()
      .replace(/\s*\n\s*/g, ' ');
    terms.push({ display, body });
  }
  terms.sort((a, b) => a.display.localeCompare(b.display, 'en', { sensitivity: 'base' }));
  return terms.map((t) => `* *${t.display}* — ${t.body}`).join('\n');
}

function assemble(dir) {
  const config = JSON.parse(fs.readFileSync(path.join(dir, 'specs.json'), 'utf8')).specs[0];
  const specDir = path.join(dir, config.spec_directory);
  const termsDir = path.join(specDir, config.spec_terms_directory);
  const parts = config.markdown_paths.map((p) => {
    let src = fs.readFileSync(path.join(specDir, p), 'utf8');
    // Spec-Up-T comment lines (`[//]: # '…'`) are instructions to its authors.
    src = src.replace(/^\[\/\/\]: # .*$\n?/gm, '');
    if (p === 'terms-and-definitions-intro.md') src = src.trimEnd() + '\n\n' + renderTerms(termsDir) + '\n';
    return { file: p, src };
  });
  return parts;
}

/* Walk every heading outside a code fence, in document order. */
function* headings(lines) {
  let fence = null;
  for (let i = 0; i < lines.length; i++) {
    const m = /^(\s*)(`{3,}|~{3,})/.exec(lines[i]);
    if (m) {
      if (!fence) fence = m[2][0];
      else if (m[2][0] === fence) fence = null;
      continue;
    }
    if (fence) continue;
    const h = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(lines[i]);
    if (h) yield { i, level: h[1].length, text: h[2] };
  }
}

function number(parts) {
  const lines = [];
  const origin = [];
  for (const { file, src } of parts) {
    for (const l of src.replace(/\s+$/, '').split('\n')) {
      lines.push(l);
      origin.push(file);
    }
    lines.push('');
    origin.push(file);
  }

  const named = new Map(); // canonical named anchor -> numbered anchor
  const numberedSeen = new Set();
  const counters = [0, 0, 0, 0, 0];
  let appendix = null;
  let appendixLetter = 0;
  const drop = new Set();

  for (const h of [...headings(lines)]) {
    const canonicalSlug = slugifyHeading(h.text);
    if (h.level === 1) continue; // the document title
    const inAppendix = origin[h.i] === 'appendix.md';
    let label;
    let level = h.level;
    if (inAppendix) {
      // `## Appendices` is a wrapper; each `### Appendix X: …` is promoted to a
      // top-level appendix, and its subsections are numbered X.1, X.1.1, ….
      if (h.level === 2) {
        drop.add(h.i);
        named.set(canonicalSlug, null);
        continue;
      }
      level = h.level - 1;
      if (level === 2) {
        appendixLetter++;
        appendix = String.fromCharCode(64 + appendixLetter);
        counters.fill(0);
        const title = h.text.replace(/^Appendix\s+[A-Z]\s*:\s*/, '');
        label = `Appendix ${appendix} — ${title}`;
      } else {
        const depth = level - 3;
        counters[depth]++;
        counters.fill(0, depth + 1);
        label = `${[appendix, ...counters.slice(0, depth + 1)].join('.')} ${h.text}`;
      }
    } else if (h.level === 2) {
      counters[0]++;
      counters.fill(0, 1);
      label = `${counters[0]}. ${h.text}`;
    } else {
      const depth = h.level - 2;
      counters[depth]++;
      counters.fill(0, depth + 1);
      label = `${counters.slice(0, depth + 1).join('.')} ${h.text}`;
    }
    let anchor = slugifyHeading(label);
    for (let n = 2; numberedSeen.has(anchor); n++) anchor = `${slugifyHeading(label)}-${n}`;
    numberedSeen.add(anchor);
    // Spec-Up-T disambiguates a repeated heading by suffix; the first one wins
    // the bare slug, which is the one canonical cross-references use.
    if (!named.has(canonicalSlug)) named.set(canonicalSlug, anchor);
    lines[h.i] = `${'#'.repeat(level)} ${label}`;
    h.anchor = anchor;
  }
  return { lines: lines.filter((_, i) => !drop.has(i)), named };
}

function rewriteLinks(text, named, missing) {
  let fence = null;
  return text
    .split('\n')
    .map((line) => {
      const m = /^(\s*)(`{3,}|~{3,})/.exec(line);
      if (m) {
        if (!fence) fence = m[2][0];
        else if (m[2][0] === fence) fence = null;
        return line;
      }
      if (fence) return line;
      return line
        .replace(/\[\[ref:\s*([^\]]+?)\s*\]\]/g, (_m, term) => `*${term}*`)
        .replace(/\]\(#([^)\s]+)\)/g, (m, frag) => {
          const target = named.get(frag);
          if (target == null) {
            missing.add(frag);
            return m;
          }
          return `](#${target})`;
        });
    })
    .join('\n');
}

function withAliases(text, named, aliases) {
  // legacy anchor -> canonical named anchor, emitted before that heading.
  const byNumbered = new Map();
  const unresolved = [];
  const headingAnchors = new Set([...named.values()].filter(Boolean));
  for (const [legacy, canonicalSlug] of Object.entries(aliases)) {
    const target = named.get(canonicalSlug);
    if (!target) {
      unresolved.push(`${legacy} -> ${canonicalSlug}`);
      continue;
    }
    if (legacy === target) continue;
    // An old anchor that is now some *other* section's anchor cannot be an
    // alias: the heading would win, and the link would land on the wrong text.
    if (headingAnchors.has(legacy)) {
      unresolved.push(`${legacy} -> ${canonicalSlug} (now the anchor of a different section)`);
      continue;
    }
    if (!byNumbered.has(target)) byNumbered.set(target, []);
    byNumbered.get(target).push(legacy);
  }
  const lines = text.split('\n');
  const out = [];
  for (const line of lines) {
    const h = /^(#{2,6})\s+(.*)$/.exec(line);
    if (h) {
      const legacy = byNumbered.get(slugifyHeading(h[2]));
      if (legacy) out.push(legacy.map((id) => `<a id="${id}"></a>`).join(''));
    }
    out.push(line);
  }
  return { text: out.join('\n'), unresolved };
}

function banner(sha, ref) {
  const at = sha === 'local' ? 'a local checkout' : `[\`${sha.slice(0, 12)}\`](${CANONICAL_REPO}/tree/${sha})`;
  return [
    `> **Generated file — do not edit.** This is the Trust Tasks framework specification, generated`,
    `> from ${at}${ref && sha !== 'local' ? ` (\`${ref}\`)` : ''} of the canonical source,`,
    `> <${CANONICAL_REPO}>, by \`scripts/generate-framework-spec.mjs\`. Changes to the text belong`,
    `> there. Section numbers are assigned here, in the canonical order, because the registry cites`,
    `> the framework by number; they are not part of the canonical text.`,
    '',
  ].join('\n');
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const { dir, sha } = checkout(args);
  const { lines, named } = number(assemble(dir));
  const missing = new Set();
  let text = rewriteLinks(lines.join('\n'), named, missing);
  const aliases = fs.existsSync(ALIASES) ? JSON.parse(fs.readFileSync(ALIASES, 'utf8')) : {};
  delete aliases.$comment;
  const aliased = withAliases(text, named, aliases);
  text = aliased.text;

  // Title first, then the banner, then the rest of the canonical header.
  const titleEnd = text.indexOf('\n') + 1;
  text = text.slice(0, titleEnd) + '\n' + banner(sha, args.from ? null : args.ref) + text.slice(titleEnd);
  text = text.replace(/\n{3,}/g, '\n\n').replace(/\s*$/, '\n');

  const problems = [];
  if (missing.size) problems.push(`canonical cross-reference(s) to no heading: ${[...missing].map((f) => '#' + f).join(', ')}`);
  if (aliased.unresolved.length) problems.push(`legacy anchor alias(es) naming no canonical heading: ${aliased.unresolved.join('; ')}`);
  if (problems.length) {
    for (const p of problems) console.error(`  error: ${p}`);
    process.exit(1);
  }

  if (args.check) {
    const current = fs.existsSync(OUT) ? fs.readFileSync(OUT, 'utf8') : '';
    if (current !== text) {
      console.error(`SPEC.md differs from the framework specification generated from ${sha.slice(0, 12)}.`);
      console.error('Run `npm run generate-framework-spec` and commit the result.');
      process.exit(1);
    }
    console.log(`SPEC.md matches the framework specification at ${sha.slice(0, 12)}.`);
    return;
  }
  fs.writeFileSync(OUT, text);
  console.log(`Generated SPEC.md from ${CANONICAL_REPO} @ ${sha.slice(0, 12)}.`);
}

if (import.meta.url === `file://${process.argv[1]}`) main();
