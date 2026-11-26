#!/usr/bin/env node
/**
 * Has HCP drifted from the hcp/* schemas, or they from HCP?
 *
 * The hcp/* family carries operations of the Human Context Protocol (HCP
 * 2.0-draft-3): each spec with a `carries` block names the HCP commit it was
 * written against in `carries.reference`. This compares the family's
 * `hcp/_shared` definitions and each carried spec's request and response with
 * the JSON schemas HCP publishes for the same objects and operations, member by
 * member: members on one side only, required-ness, enum values, and types
 * including nullability (scripts/lib/hcp-drift.mjs says exactly what is and is
 * not compared). That is the `members` mode. The `constraints` mode compares
 * what the first cannot see — bounds, `pattern`, `format`, closed objects and
 * unions merged into one shape — with its own walker and comparison
 * (scripts/lib/hcp-drift-constraints.mjs) and its own allowlist,
 * scripts/lib/hcp-drift-constraints-allowlist.json. Both modes run by default.
 *
 * Which of our schemas corresponds to which of HCP's is data, not inference:
 * scripts/lib/hcp-drift-map.json, which also lists the sides HCP publishes no
 * schema for. Differences the family makes on purpose — a header moved into the
 * payload, a renamed instant, a reading pending an HCP editors' answer — are in
 * scripts/lib/hcp-drift-allowlist.json, each with its reason and a `ref` to the
 * spec's field-mapping row or the editors' question. An entry that matches
 * nothing any more is reported as stale.
 *
 * Not part of `npm run check` or CI: it needs an HCP checkout.
 *
 *   npm run check-hcp-drift -- --hcp <hcp-spec checkout> [--commit <sha>] [--mode members|constraints|both] [--verbose]
 *   npm run check-hcp-drift -- --pinned-commit   (prints the commit the carried specs pin, and exits)
 *
 * HCP's files are read with `git show <commit>:<path>`, so the checkout's HEAD
 * and working tree do not matter. `--commit` defaults to the commit the carried
 * specs pin. `--verbose` lists the allowlisted differences too.
 *
 * Exit status: 0 when every difference is allowlisted and no entry is stale in
 * each mode run, 1 otherwise, 2 on a usage or mapping error.
 */
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import { applyAllowlist, compareMapping, coverage, makeResolver } from './lib/hcp-drift.mjs';
import { applyConstraintsAllowlist, compareConstraints, schemaSide } from './lib/hcp-drift-constraints.mjs';

const SCRIPT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function usage(msg) {
  if (msg) console.error(`check-hcp-drift: ${msg}`);
  console.error('usage: npm run check-hcp-drift -- --hcp <hcp-spec checkout> [--commit <sha>] [--mode members|constraints|both] [--verbose]');
  console.error('       npm run check-hcp-drift -- --pinned-commit');
  process.exit(2);
}

const MODES = ['members', 'constraints', 'both'];
const opts = { verbose: false, mode: 'both' };
const argv = process.argv.slice(2);
for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  const val = () => argv[++i] ?? usage(`${a} needs a value`);
  if (a === '--hcp') opts.hcp = val();
  else if (a === '--commit') opts.commit = val();
  else if (a === '--mode') opts.mode = val();
  else if (a === '--pinned-commit') opts.pinnedOnly = true;
  else if (a === '--verbose' || a === '-v') opts.verbose = true;
  // For the tests: another tree, mapping or allowlist than this checkout's.
  else if (a === '--repo') opts.repo = val();
  else if (a === '--map') opts.map = val();
  else if (a === '--allowlist') opts.allowlist = val();
  else if (a === '--constraints-allowlist') opts.constraintsAllowlist = val();
  else if (a === '--help' || a === '-h') usage();
  else usage(`unknown argument ${a}`);
}
if (!MODES.includes(opts.mode)) usage(`--mode must be one of ${MODES.join(', ')}`);
if (!opts.hcp && !opts.pinnedOnly) usage('--hcp is required');
const runMembers = opts.mode !== 'constraints';
const runConstraints = opts.mode !== 'members';

const ROOT = path.resolve(opts.repo ?? SCRIPT_ROOT);
const MAP = opts.map ?? path.join(SCRIPT_ROOT, 'scripts/lib/hcp-drift-map.json');
const ALLOW = opts.allowlist ?? path.join(SCRIPT_ROOT, 'scripts/lib/hcp-drift-allowlist.json');
const CALLOW = opts.constraintsAllowlist ?? path.join(SCRIPT_ROOT, 'scripts/lib/hcp-drift-constraints-allowlist.json');
const readJson = (file) => JSON.parse(fs.readFileSync(file, 'utf8'));

function fail(msg) {
  console.error(`check-hcp-drift: ${msg}`);
  process.exit(2);
}

let mapping, allowlist, constraintsAllowlist;
try {
  mapping = readJson(MAP);
  if (runMembers && !opts.pinnedOnly) allowlist = readJson(ALLOW);
  if (runConstraints && !opts.pinnedOnly) constraintsAllowlist = readJson(CALLOW);
} catch (e) {
  fail(e.message);
}

// The carried specs: every version directory under the family with a `carries` block.
const familyDir = path.join(ROOT, 'specs', mapping.family);
const carried = [];
(function walk(dir) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    if (!ent.isDirectory()) continue;
    const sub = path.join(dir, ent.name);
    const md = path.join(sub, 'spec.md');
    if (fs.existsSync(md)) {
      const fm = /^---\r?\n([\s\S]*?)\r?\n---/.exec(fs.readFileSync(md, 'utf8'));
      const meta = fm ? YAML.parse(fm[1]) : null;
      if (meta?.carries) {
        const schema = readJson(path.join(sub, 'payload.schema.json'));
        carried.push({ slug: meta.slug, version: meta.version, carries: meta.carries, hasResponse: Boolean(schema.$defs?.Response) });
      }
    }
    walk(sub);
  }
})(familyDir);
if (!carried.length) fail(`no spec under specs/${mapping.family} has a \`carries\` block`);

let commit = opts.commit;
if (!commit) {
  const pinned = new Set(carried.map((c) => /\/blob\/([0-9a-f]{7,40})\//.exec(c.carries.reference ?? '')?.[1] ?? '(none)'));
  if (pinned.size !== 1 || pinned.has('(none)')) fail(`the carried specs do not pin one commit in carries.reference (${[...pinned].join(', ')}); pass --commit`);
  commit = [...pinned][0];
}
if (opts.pinnedOnly) {
  if (opts.commit) fail('--pinned-commit and --commit do not go together');
  console.log(commit);
  process.exit(0);
}

const git = (...args) => execFileSync('git', ['-C', opts.hcp, ...args], { encoding: 'utf8', maxBuffer: 1 << 26, stdio: ['ignore', 'pipe', 'pipe'] });
let resolvedCommit;
try {
  resolvedCommit = git('rev-parse', '--verify', `${commit}^{commit}`).trim();
} catch (e) {
  fail(`${opts.hcp}: cannot resolve commit ${commit}: ${String(e.stderr || e.message).trim()}`);
}

const loadOurs = (file) => readJson(path.join(ROOT, file));
const loadHcp = (file) => {
  const p = `${mapping.hcpSchemaDir}/${file}`;
  try {
    return JSON.parse(git('show', `${resolvedCommit}:${p}`));
  } catch (e) {
    throw new Error(`HCP ${p} at ${commit}: ${String(e.stderr || e.message).trim()}`);
  }
};

const versions = new Set(carried.map((c) => c.carries.version));
console.log(`check-hcp-drift: specs/${mapping.family} (${carried.length} carried specs, HCP ${[...versions].join(', ')}) against ${opts.hcp} at ${commit} (${resolvedCommit.slice(0, 12)})`);

const order = new Map(mapping.objects.map((o, i) => [o.id, i]));
function listFindings(unexpected, allowed) {
  const byObject = new Map();
  for (const f of [...unexpected.map((f) => ({ ...f, entry: null })), ...(opts.verbose ? allowed : [])]) {
    if (!byObject.has(f.object)) byObject.set(f.object, []);
    byObject.get(f.object).push(f);
  }
  for (const [object, list] of [...byObject].sort((a, b) => order.get(a[0]) - order.get(b[0]))) {
    const o = mapping.objects.find((x) => x.id === object);
    console.log(`${object}\n  ours ${o.ours}\n  HCP  ${mapping.hcpSchemaDir}/${o.hcp}`);
    for (const f of list.sort((a, b) => a.path.localeCompare(b.path) || a.kind.localeCompare(b.kind))) {
      const tag = f.entry ? `  [allowlisted: ${f.entry.ref}]` : '';
      console.log(`  ${f.entry ? ' ' : '!'} ${f.kind.padEnd(12)} ${f.path}: ${f.detail}${tag}`);
    }
    console.log('');
  }
}
const plural = (n) => `entr${n === 1 ? 'y' : 'ies'}`;
let problems = 0;

const { missing, unknown } = coverage({ carried, mapping });
if (missing.length) {
  console.log(`\nCarried sides the mapping does not account for (add a mapping object or a notCompared reason):`);
  for (const id of missing) console.log(`  ! ${id}`);
}
if (unknown.length) {
  console.log(`\nMapping ids that name no carried spec side:`);
  for (const id of unknown) console.log(`  ! ${id}`);
}
problems += missing.length + unknown.length;

if (runMembers) {
  // Mode 1, `members`: members, required-ness, types, nullability, enums.
  const ours = makeResolver(loadOurs);
  const hcp = makeResolver(loadHcp);
  let result;
  try {
    result = compareMapping({ mapping, ours, hcp });
  } catch (e) {
    fail(`mapping error: ${e.message}`);
  }
  const { allowed, unexpected, stale, invalid } = applyAllowlist(result.findings, allowlist.entries ?? []);
  if (invalid.length) fail(`${path.relative(SCRIPT_ROOT, ALLOW)}: ${invalid.join('; ')}`);
  console.log(`\n== members: ${result.compared.length} objects compared, ${result.covered.length} covered by another mapped object, ${Object.keys(mapping.notCompared ?? {}).length} sides with no HCP schema\n`);
  listFindings(unexpected, allowed);
  if (stale.length) {
    console.log(`Stale allowlist entries (they match no difference any more; remove them):`);
    for (const e of stale) console.log(`  ! ${e.object} ${e.path} ${e.kind} (${e.ref})`);
    console.log('');
  }
  problems += unexpected.length + stale.length;
  console.log(
    `check-hcp-drift: ${result.findings.length} difference(s), ${allowed.length} allowlisted, ${unexpected.length} not allowlisted; ` +
      `${stale.length} stale allowlist ${plural(stale.length)}; ${missing.length + unknown.length} mapping coverage problem(s)` +
      (opts.verbose || !allowed.length ? '' : ' (--verbose lists the allowlisted ones)'),
  );
}

if (runConstraints) {
  // Mode 2, `constraints`: bounds, patterns, formats, closed objects, merged unions.
  let result;
  try {
    result = compareConstraints({ mapping, ours: schemaSide(loadOurs), hcp: schemaSide(loadHcp) });
  } catch (e) {
    fail(`mapping error (constraints): ${e.message}`);
  }
  const { allowed, unexpected, stale, invalid } = applyConstraintsAllowlist(result.findings, constraintsAllowlist);
  if (invalid.length) fail(`${path.relative(SCRIPT_ROOT, CALLOW)}: ${invalid.join('; ')}`);
  console.log(`\n== constraints: ${result.objects} mapped objects walked\n`);
  listFindings(unexpected, allowed);
  if (stale.length) {
    console.log(`Stale constraints allowlist entries (they match no difference any more; remove them):`);
    for (const e of stale) console.log(`  ! ${e.label} (${e.ref})`);
    console.log('');
  }
  problems += unexpected.length + stale.length;
  console.log(
    `check-hcp-drift constraints: ${result.findings.length} difference(s), ${allowed.length} allowlisted, ${unexpected.length} not allowlisted; ` +
      `${stale.length} stale allowlist ${plural(stale.length)}` +
      (opts.verbose || !allowed.length ? '' : ' (--verbose lists the allowlisted ones)'),
  );
}
process.exit(problems ? 1 : 0);
