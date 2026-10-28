#!/usr/bin/env node
/**
 * Withdraw every specification version that has been superseded or retired
 * for at least 90 days (SPEC "Superseded and Withdrawn Versions").
 *
 * A withdrawn version keeps its schema and prose — already-issued documents
 * stay verifiable — but is no longer generated into the libraries. Removing a
 * generated module is a breaking release of each library, so withdrawals are
 * batched: `.github/workflows/withdraw.yml` runs this on the first day of each
 * quarter and opens one pull request for the batch. Run it by hand to see what
 * the next batch holds:
 *
 *   node scripts/withdraw-due.mjs --dry                    # as of today
 *   node scripts/withdraw-due.mjs --dry --as-of 2027-01-01
 *   node scripts/withdraw-due.mjs --as-of 2027-01-01       # flip them
 *
 * A version is SUPERSEDED when a later version of its slug exists that is
 * neither retired nor withdrawn and whose maturity is at least its own (draft <
 * candidate < standard); it was superseded on the date the earliest such
 * version was added to the repository. A RETIRED version was retired on the
 * date its `status: retired` was committed. Both dates come from git history,
 * so this needs a full clone (`fetch-depth: 0` in CI). A version is due when
 * that date is at least 90 days before `--as-of`.
 *
 * Flipping one sets `status: withdrawn` and, when it declares none, adds a
 * `supersededBy` naming its successor (SPEC §7.3 item 11), so the registry and
 * the libraries' deprecation note keep pointing somewhere.
 */
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SPECS = path.join(ROOT, 'specs');
const GRACE_DAYS = 90;

/**
 * Every version due for withdrawal as of `asOf` (a Date): superseded or retired
 * at least GRACE_DAYS earlier, dated from git history. Exported for
 * scripts/check-withdrawal-readiness.mjs, which asks what the NEXT batch holds.
 */
export function dueVersions(asOf) {
  const RANK = { draft: 0, candidate: 1, standard: 2 };
  const compare = (a, b) => {
    const [x, y] = a.split('.').map(Number);
    const [u, v] = b.split('.').map(Number);
    return x - u || y - v;
  };
  const git = (...a) => execFileSync('git', a, { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 28 }).trim();

  // slug → [{ version, status, supersededBy, file }]
  const slugs = new Map();
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory() || entry.name.startsWith('_')) continue;
      const full = path.join(dir, entry.name);
      const spec = path.join(full, 'spec.md');
      if (/^\d+\.\d+$/.test(entry.name) && fs.existsSync(spec)) {
        const slug = path.relative(SPECS, dir).split(path.sep).join('/');
        const text = fs.readFileSync(spec, 'utf8');
        const fm = text.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
        const field = (k) => fm.match(new RegExp(`^${k}:\\s*["']?([^"'\\n]+?)["']?\\s*$`, 'm'))?.[1];
        if (!slugs.has(slug)) slugs.set(slug, []);
        slugs.get(slug).push({ version: entry.name, status: field('status'), supersededBy: field('supersededBy'), file: spec });
      } else {
        walk(full);
      }
    }
  })(SPECS);
  for (const list of slugs.values()) list.sort((a, b) => compare(a.version, b.version));

  const live = (v) => v.status !== 'retired' && v.status !== 'withdrawn';
  const latestLive = (slug) => (slugs.get(slug) ?? []).filter(live).at(-1);

  // Dates each spec.md was first added, in one pass over history.
  const added = new Map();
  {
    let date = null;
    for (const line of git('log', '--diff-filter=A', '--format=@%cs', '--name-only', '--', 'specs').split('\n')) {
      if (line.startsWith('@')) date = line.slice(1);
      else if (line.endsWith('/spec.md')) added.set(path.join(ROOT, line), date); // log is newest first; the last write is the oldest
    }
  }
  const retiredOn = (file) => {
    // The oldest commit that made the file say `status: retired`.
    const dates = git('log', '--format=%cs', '-G', '^status: *["\']?retired', '--', path.relative(ROOT, file)).split('\n').filter(Boolean);
    return dates.at(-1) ?? added.get(file);
  };

  const due = [];
  for (const [slug, list] of slugs) {
    list.forEach((v, i) => {
      if (v.status === 'withdrawn') return;
      let since;
      let successor;
      let reason;
      if (v.status === 'retired') {
        since = retiredOn(v.file);
        reason = 'retired';
        if (v.supersededBy) {
          successor = /\/\d+\.\d+$/.test(v.supersededBy) ? v.supersededBy : latestLive(v.supersededBy) && `${v.supersededBy}/${latestLive(v.supersededBy).version}`;
        } else {
          successor = latestLive(slug) && `${slug}/${latestLive(slug).version}`;
        }
      } else {
        const supersedes = list.slice(i + 1).filter((w) => live(w) && (RANK[w.status] ?? 0) >= (RANK[v.status] ?? 0));
        if (!supersedes.length) return;
        since = supersedes.map((w) => added.get(w.file)).filter(Boolean).sort()[0];
        successor = `${slug}/${latestLive(slug).version}`;
        reason = 'superseded';
      }
      if (!since) return;
      const days = Math.floor((asOf - new Date(`${since}T00:00:00Z`)) / 86_400_000);
      if (days >= GRACE_DAYS) due.push({ slug, ...v, since, days, successor, reason });
    });
  }

  return due;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const dry = args.includes('--dry');
  const asOfArg = args[args.indexOf('--as-of') + 1];
  const asOf = args.includes('--as-of') ? new Date(`${asOfArg}T00:00:00Z`) : new Date();
  if (Number.isNaN(asOf.getTime())) {
    console.error(`--as-of must be a date (YYYY-MM-DD), got ${asOfArg}`);
    process.exit(2);
  }


    const due = dueVersions(asOf);
  const asOfText = asOf.toISOString().slice(0, 10);
  if (!due.length) {
    console.log(`Nothing to withdraw as of ${asOfText}: no version has been superseded or retired for ${GRACE_DAYS} days.`);
    process.exit(0);
  }
  for (const d of due) {
    console.log(`${d.slug}/${d.version}  ${d.reason} ${d.since} (${d.days} days)${d.successor ? ` → ${d.successor}` : ''}`);
    if (dry) continue;
    let text = fs.readFileSync(d.file, 'utf8');
    text = text.replace(/^status:\s*["']?\w+["']?\s*$/m, 'status: withdrawn');
    if (!d.supersededBy && d.successor) text = text.replace(/^status: withdrawn$/m, `status: withdrawn\nsupersededBy: ${d.successor}`);
    fs.writeFileSync(d.file, text);
  }
  console.log(`\n${dry ? 'Would withdraw' : 'Withdrew'} ${due.length} version(s) as of ${asOfText}.`);

}
