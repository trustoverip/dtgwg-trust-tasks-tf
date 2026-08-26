#!/usr/bin/env node
/**
 * Rewrite depth-relative links to the framework spec into the root-relative form.
 *
 *   ](../../../../SPEC.md#451-the-ext-extension-member)  ->  ](/SPEC.md#451-…)
 *
 * Usage:
 *   npm run codemod:spec-links -- --dry-run     # report, change nothing
 *   npm run codemod:spec-links                  # rewrite in place
 *   npm run codemod:spec-links -- --root specs bindings
 *
 * ── Why ─────────────────────────────────────────────────────────────────────
 *
 * A spec's prose links `SPEC.md` 731 times, and the number of `../` in each one
 * is a function of how deep the slug is: four from `auth/passkey/enroll/start/0.1`,
 * three from `acl/grant/0.1`. That makes a link's correctness a property of where
 * the file sits rather than of what it says, so the link breaks when a spec moves,
 * when a slug gains a segment, and — most often — when a paragraph is copied from
 * a neighbouring spec at a different depth, which is exactly how specs get written
 * around here. 135 such links across 65 files had already drifted before
 * build-registry.mjs grew a check for them, and they were broken on the live site,
 * not merely wrong in the repo. The check catches the next one; it does not remove
 * the reason there is a next one.
 *
 * `/SPEC.md#anchor` is the same string from every depth. The website's renderer
 * resolves it onto the SPA's `/specification` route (which is a better
 * destination than the relative form ever reached — that one downloaded a raw
 * markdown file), and the build validates it from the repository root.
 *
 * ── What it will and will not touch ─────────────────────────────────────────
 *
 * Only links whose `../` chain actually resolves to the repository's own
 * SPEC.md are rewritten. A chain that resolves somewhere else, or nowhere, is
 * reported and left alone: it is either a link to a different file or an
 * already-broken one, and silently normalising a broken link into a
 * differently-broken link would destroy the evidence of the bug.
 *
 * Code fences are skipped. A `../../../../SPEC.md` inside an example is
 * illustrating the old form, or is part of a payload, and either way is content
 * rather than a link.
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPEC_MD = path.join(ROOT, 'SPEC.md');

const argv = process.argv.slice(2);
const dryRun = argv.includes('--dry-run');
const rootsIdx = argv.indexOf('--root');
const roots = rootsIdx >= 0
  ? argv.slice(rootsIdx + 1).filter((a) => !a.startsWith('-'))
  : ['specs'];

/** Every markdown file under the given roots. */
function markdownFiles(rel) {
  const base = path.join(ROOT, rel);
  const out = [];
  if (!fs.existsSync(base)) return out;
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === 'node_modules' || entry.name.startsWith('.')) continue;
        walk(full);
      } else if (entry.name.endsWith('.md')) {
        out.push(full);
      }
    }
  })(base);
  return out.sort();
}

/**
 * Split a markdown source into alternating prose / fenced-code spans, so the
 * rewrite can be applied to prose only. Handles ``` and ~~~ fences of any length.
 */
function proseSpans(src) {
  const spans = [];
  const lines = src.split('\n');
  let fence = null;
  let buf = [];
  let inCode = false;
  const flush = () => {
    if (buf.length) spans.push({ code: inCode, text: buf.join('\n') });
    buf = [];
  };
  for (const line of lines) {
    const m = /^\s*(`{3,}|~{3,})/.exec(line);
    if (m) {
      if (!inCode) {
        flush();
        inCode = true;
        fence = m[1][0].repeat(m[1].length);
        buf.push(line);
        continue;
      }
      if (line.trimStart().startsWith(fence)) {
        buf.push(line);
        flush();
        inCode = false;
        fence = null;
        continue;
      }
    }
    buf.push(line);
  }
  flush();
  return spans;
}

const LINK_RE = /\]\(((?:\.\.\/)+)SPEC\.md(#[^)\s]*)?\)/g;

function rewriteFile(file) {
  const src = fs.readFileSync(file, 'utf8');
  if (!/\.\.\/SPEC\.md/.test(src)) return null;

  const dir = path.dirname(file);
  let rewritten = 0;
  const skipped = [];

  const out = proseSpans(src)
    .map((span) => {
      if (span.code) return span.text;
      return span.text.replace(LINK_RE, (whole, ups, frag) => {
        const resolved = path.resolve(dir, `${ups}SPEC.md`);
        if (resolved !== SPEC_MD) {
          // Either a link to some other SPEC.md, or an already-broken chain.
          // Leave it: the build's link check owns that diagnosis, and rewriting
          // it here would convert a visible bug into an invisible one.
          skipped.push(`${ups}SPEC.md${frag || ''}`);
          return whole;
        }
        rewritten++;
        return `](/SPEC.md${frag || ''})`;
      });
    })
    .join('\n');

  return { out, rewritten, skipped, changed: out !== src };
}

function main() {
  let files = 0;
  let links = 0;
  const skippedAll = [];

  for (const rel of roots) {
    for (const file of markdownFiles(rel)) {
      const result = rewriteFile(file);
      if (!result || !result.changed) {
        if (result) skippedAll.push(...result.skipped.map((l) => `${path.relative(ROOT, file)}: ${l}`));
        continue;
      }
      files++;
      links += result.rewritten;
      skippedAll.push(...result.skipped.map((l) => `${path.relative(ROOT, file)}: ${l}`));
      if (!dryRun) fs.writeFileSync(file, result.out);
    }
  }

  console.log(
    `${dryRun ? 'Would rewrite' : 'Rewrote'} ${links} link${links === 1 ? '' : 's'} ` +
    `across ${files} file${files === 1 ? '' : 's'} under ${roots.join(', ')}/`
  );
  if (skippedAll.length) {
    console.warn(`\n${skippedAll.length} link(s) left alone because the '../' chain does not resolve to ${path.relative(ROOT, SPEC_MD)}:`);
    for (const l of skippedAll) console.warn(`  - ${l}`);
    console.warn('\nThese are either links to a different file or already broken. Fix them by hand.');
  }
  if (!dryRun && links) {
    console.log('\nRun `npm run validate` to confirm every rewritten link still resolves.');
  }
}

main();
