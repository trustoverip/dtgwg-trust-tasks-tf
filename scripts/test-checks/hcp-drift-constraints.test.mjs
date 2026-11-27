// Tests for scripts/lib/hcp-drift-constraints.mjs, the `constraints` mode of
// scripts/check-hcp-drift.mjs, against small fixture schemas. Nothing here
// reads the real hcp-spec checkout; the end-to-end tests build a throwaway git
// repository.
//
//   node --test scripts/test-checks/hcp-drift-constraints.test.mjs   (npm run test:checks)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { applyConstraintsAllowlist, compareConstraints, schemaSide } from '../lib/hcp-drift-constraints.mjs';

const CLI = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', 'check-hcp-drift.mjs');

const memory = (files) =>
  schemaSide((f) => {
    if (!(f in files)) throw new Error(`no fixture ${f}`);
    return structuredClone(files[f]);
  });
const run = (oursFiles, hcpFiles, objects) => compareConstraints({ mapping: { objects }, ours: memory(oursFiles), hcp: memory(hcpFiles) });
const one = (ours, hcp, renames) => run({ 'o.json': ours }, { 'h.json': hcp }, [{ id: 'x:request', ours: 'o.json', hcp: 'h.json', renames }]);
const brief = (findings) => findings.map((f) => `${f.path} ${f.kind}`).sort();

test('bounds on one side only, and bounds that differ', () => {
  const { findings } = one(
    { type: 'object', properties: { a: { type: 'string', minLength: 1, maxLength: 64 }, n: { type: 'integer', minimum: 0, maximum: 10 }, l: { type: 'array', maxItems: 5, uniqueItems: true } } },
    { type: 'object', properties: { a: { type: 'string', maxLength: 64 }, n: { type: 'integer', minimum: 1 }, l: { type: 'array', minItems: 1, uniqueItems: true } } },
  );
  assert.deepEqual(brief(findings), ['a minLength', 'l maxItems', 'l minItems', 'n maximum', 'n minimum']);
  assert.equal(findings.find((f) => f.path === 'a').detail, 'ours 1, HCP none');
  assert.equal(findings.find((f) => f.path === 'n' && f.kind === 'minimum').detail, 'ours 0, HCP 1');
  assert.equal(findings.find((f) => f.kind === 'minItems').detail, 'ours none, HCP 1');
});

test('patterns and formats, through $refs and allOf on both sides', () => {
  const ours = {
    'o.json': { type: 'object', properties: { u: { $ref: 'common.json#/$defs/Uri' }, d: { allOf: [{ type: 'string' }, { format: 'date-time' }] }, p: { type: 'string', format: 'uri' } } },
    'common.json': { $defs: { Uri: { type: 'string', pattern: '^[a-z]+:\\S+$', format: 'uri' } } },
  };
  const hcp = {
    'h.json': { type: 'object', properties: { u: { $ref: 'common.json#/$defs/uri' }, d: { type: 'string', format: 'date-time' }, p: { type: 'string' } } },
    'common.json': { $defs: { uri: { type: 'string', pattern: '^[a-z]+:[^\\s]+$', format: 'uri' } } },
  };
  const { findings } = run(ours, hcp, [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }]);
  assert.deepEqual(brief(findings), ['p format', 'u pattern']);
  const pat = findings.find((f) => f.kind === 'pattern');
  assert.deepEqual([pat.ours, pat.hcp], [['^[a-z]+:\\S+$'], ['^[a-z]+:[^\\s]+$']]);
  assert.equal(findings.find((f) => f.kind === 'format').detail, 'ours "uri", HCP none');
});

test('a closed object on one side only, by additionalProperties or unevaluatedProperties', () => {
  const { findings } = one(
    { type: 'object', additionalProperties: false, properties: { inner: { type: 'object', unevaluatedProperties: false }, open: { type: 'object' }, map: { type: 'object', additionalProperties: { type: 'string', maxLength: 9 } } } },
    { type: 'object', properties: { inner: { type: 'object' }, open: { type: 'object', additionalProperties: false }, map: { type: 'object', additionalProperties: { type: 'string' } } } },
  );
  assert.deepEqual(brief(findings), ['(root) closed', 'inner closed', 'map{} maxLength', 'open closed']);
  assert.equal(findings.find((f) => f.path === 'open').detail, 'closed in HCP, open in ours');
});

test('a scalar union HCP offers, merged into one shape in ours, is a union finding', () => {
  const { findings } = one(
    { type: 'object', properties: { cat: { type: 'string', pattern: '^([a-z]+|[a-z]+:\\S+)$' }, id: { type: ['string', 'null'], maxLength: 5 } } },
    {
      type: 'object',
      properties: {
        cat: { type: 'string', anyOf: [{ enum: ['a', 'b'] }, { pattern: '^[a-z]+:', format: 'uri' }] },
        // A null branch is nullability, not a union: the other branch is compared as the member.
        id: { anyOf: [{ type: 'string', maxLength: 5 }, { type: 'null' }] },
      },
    },
  );
  assert.deepEqual(brief(findings), ['cat union']);
  assert.match(findings[0].detail, /^ours is a single shape \[pattern .*\]; HCP is a choice of \[one of "a", "b" \| pattern "\^\[a-z\]\+:", format "uri"\]$/);
});

test('same-sized choices are compared branch by branch; a choice of required lists is not a union', () => {
  const { findings } = one(
    { type: 'object', properties: { v: { anyOf: [{ type: 'string', maxLength: 3 }, { type: 'integer' }] } }, anyOf: [{ required: ['v'] }, { required: ['w'] }] },
    { type: 'object', properties: { v: { anyOf: [{ type: 'string' }, { type: 'integer', minimum: 0 }] } } },
  );
  assert.deepEqual(brief(findings), ['v|0 maxLength', 'v|1 minimum']);
});

test('renames, array items, and members a then-branch declares', () => {
  const { findings } = one(
    { type: 'object', properties: { respondedAt: { type: 'string', maxLength: 40 }, list: { type: 'array', items: { type: 'string', minLength: 1 } }, language: { type: 'string', maxLength: 64 } } },
    { type: 'object', properties: { timestamp: { type: 'string' }, list: { type: 'array', items: { type: 'string' } } }, allOf: [{ if: { properties: { p: { const: 'x' } } }, then: { properties: { language: { type: 'string' } } } }] },
    { respondedAt: 'timestamp' },
  );
  assert.deepEqual(brief(findings), ['language maxLength', 'list[] minLength', 'respondedAt maxLength']);
});

test('a node pair reached from two objects is reported once; a mapped pair under its own object', () => {
  const ours = {
    'shared.json': { $defs: { Party: { type: 'object', properties: { id: { type: 'string', minLength: 1 } } } } },
    'a.json': { type: 'object', properties: { who: { $ref: 'shared.json#/$defs/Party' }, also: { $ref: 'shared.json#/$defs/Party', description: 'annotations only' } } },
    'b.json': { type: 'object', properties: { who: { $ref: 'shared.json#/$defs/Party' } } },
  };
  const hcp = {
    'named.json': { type: 'object', properties: { id: { type: 'string' } } },
    'a.json': { type: 'object', properties: { who: { $ref: 'named.json' }, also: { $ref: 'named.json' } } },
    'b.json': { type: 'object', properties: { who: { $ref: 'named.json' } } },
  };
  const unmapped = run(ours, hcp, [
    { id: 'a:request', ours: 'a.json', hcp: 'a.json' },
    { id: 'b:request', ours: 'b.json', hcp: 'b.json' },
  ]);
  assert.deepEqual(unmapped.findings.map((f) => `${f.object} ${f.path} ${f.kind}`), ['a:request also.id minLength']);
  const mapped = run(ours, hcp, [
    { id: 'a:request', ours: 'a.json', hcp: 'a.json' },
    { id: 'hcp/_shared:Party', ours: 'shared.json#/$defs/Party', hcp: 'named.json' },
  ]);
  assert.deepEqual(mapped.findings.map((f) => `${f.object} ${f.path} ${f.kind}`), ['hcp/_shared:Party id minLength']);
});

test('a mapping that names a missing pointer is an error', () => {
  assert.throws(() => run({ 'o.json': {} }, { 'h.json': {} }, [{ id: 'x:request', ours: 'o.json#/$defs/Nope', hcp: 'h.json' }]), /no such pointer/);
});

test('the allowlist: detail globs, first match, pattern equivalences, stale and malformed entries', () => {
  const findings = [
    { object: 'hcp/a:request', path: 'note', kind: 'maxLength', detail: 'ours 4096, HCP none' },
    { object: 'hcp/a:request', path: 'size', kind: 'maxLength', detail: 'ours 10, HCP 20' },
    { object: 'hcp/b:request', path: 'id', kind: 'minLength', detail: 'ours 1, HCP none' },
    { object: 'hcp/b:request', path: 'u', kind: 'pattern', detail: '', ours: ['^a\\S$'], hcp: ['^a[^\\s]$'] },
    { object: 'hcp/b:request', path: 'v', kind: 'pattern', detail: '', ours: ['^x$'], hcp: ['^y$'] },
  ];
  const e = (object, p, kind, extra = {}) => ({ object, path: p, kind, reason: 'r', ref: 'Q1', ...extra });
  const r = applyConstraintsAllowlist(findings, {
    entries: [e('hcp/b:request', 'id', 'minLength', { ref: 'specific' }), e('*', '*', 'maxLength', { detail: 'ours *, HCP none' }), e('*', '*', 'minLength'), e('hcp/c:request', 'x', 'closed')],
    patternEquivalences: [{ ours: '^a\\S$', hcp: '^a[^\\s]$', reason: 'same', ref: 'Q2' }, { ours: '^q$', hcp: '^r$', reason: 'same', ref: 'Q3' }],
  });
  // A cap HCP also sets, at another value, is not an ours-only cap.
  assert.deepEqual(r.unexpected.map((f) => f.path), ['size', 'v']);
  assert.deepEqual(r.allowed.map((f) => `${f.path} ${f.entry.ref}`), ['note Q1', 'id specific', 'u Q2']);
  assert.deepEqual(r.stale.map((s) => s.label), ['* * minLength', 'hcp/c:request x closed', 'pattern "^q$" ~ "^r$"']);
  assert.deepEqual(r.invalid, []);
  const bad = applyConstraintsAllowlist([], { entries: [{ object: 'x', path: 'y', kind: 'type', reason: 'r' }], patternEquivalences: [{ ours: 'a', hcp: 'b', ref: 'Q1' }] });
  assert.deepEqual(bad.invalid, ['entries[0]: `ref` is missing', 'entries[0]: unknown kind "type"', 'patternEquivalences[0]: `reason` is missing']);
});

// End to end: a throwaway hcp-spec repository with two commits, and a tiny tree of ours.
const gitEnv = {
  ...process.env,
  GIT_CONFIG_GLOBAL: os.devNull,
  GIT_CONFIG_NOSYSTEM: '1',
  GIT_AUTHOR_NAME: 'fixture',
  GIT_AUTHOR_EMAIL: 'fixture@example.invalid',
  GIT_COMMITTER_NAME: 'fixture',
  GIT_COMMITTER_EMAIL: 'fixture@example.invalid',
};

function fixture() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hcp-drift-constraints-'));
  const hcpDir = path.join(dir, 'hcp');
  const write = (base, rel, value) => {
    fs.mkdirSync(path.dirname(path.join(base, rel)), { recursive: true });
    fs.writeFileSync(path.join(base, rel), typeof value === 'string' ? value : JSON.stringify(value, null, 2));
  };
  const git = (...args) => execFileSync('git', ['-C', hcpDir, ...args], { env: gitEnv, encoding: 'utf8' }).trim();
  fs.mkdirSync(hcpDir);
  git('init', '-q');
  write(hcpDir, 'schemas/d/rule.json', { type: 'object', required: ['ruleId'], properties: { ruleId: { type: 'string', maxLength: 64 } } });
  git('add', '-A');
  git('commit', '-q', '-m', 'one');
  const first = git('rev-parse', 'HEAD');
  // A later HCP commit drops its bound; the pinned commit must still be what is read.
  write(hcpDir, 'schemas/d/rule.json', { type: 'object', required: ['ruleId'], properties: { ruleId: { type: 'string' } } });
  git('commit', '-q', '-am', 'two');
  const second = git('rev-parse', 'HEAD');

  const repo = path.join(dir, 'repo');
  write(repo, 'specs/hcp/_shared/0.1/rule.schema.json', { $defs: { Rule: { type: 'object', required: ['ruleId'], properties: { ruleId: { type: 'string', maxLength: 64 } } } } });
  write(repo, 'specs/hcp/rules/set/0.1/payload.schema.json', {
    type: 'object',
    additionalProperties: false,
    properties: { accessToken: { type: 'string' } },
    $defs: { Response: { type: 'object', properties: { rule: { $ref: '../../../_shared/0.1/rule.schema.json#/$defs/Rule' } } } },
  });
  write(repo, 'specs/hcp/rules/set/0.1/spec.md', `---\nslug: hcp/rules/set\nversion: "0.1"\ncarries:\n  protocol: HCP\n  version: "x"\n  reference: https://example.invalid/hcp/blob/${first.slice(0, 7)}/spec.md\n  operation: Set rule\n---\n\n# Set rule\n`);
  const map = path.join(dir, 'map.json');
  write(dir, 'map.json', {
    family: 'hcp',
    hcpSchemaDir: 'schemas/d',
    objects: [
      { id: 'hcp/_shared:Rule', ours: 'specs/hcp/_shared/0.1/rule.schema.json#/$defs/Rule', hcp: 'rule.json' },
      { id: 'hcp/rules/set:response', ours: 'specs/hcp/rules/set/0.1/payload.schema.json#/$defs/Response/properties/rule', hcp: 'rule.json' },
    ],
    notCompared: { 'hcp/rules/set:request': 'prose only' },
  });
  const allow = (name, doc) => {
    write(dir, name, doc);
    return path.join(dir, name);
  };
  const members = allow('members.json', { entries: [] });
  const cli = (constraints, ...args) =>
    spawnSync(process.execPath, [CLI, '--hcp', hcpDir, '--repo', repo, '--map', map, '--allowlist', members, '--constraints-allowlist', constraints, ...args], { encoding: 'utf8' });
  return { dir, repo, allow, cli, first, second };
}

test('the CLI runs both modes by default, at the pinned commit, and either one alone', (t) => {
  const f = fixture();
  t.after(() => fs.rmSync(f.dir, { recursive: true, force: true }));
  const empty = f.allow('c.json', { entries: [] });
  const both = f.cli(empty);
  assert.equal(both.status, 0, both.stdout + both.stderr);
  assert.match(both.stdout, new RegExp(`at ${f.first.slice(0, 7)} `));
  assert.match(both.stdout, /== members:[\s\S]*check-hcp-drift: 0 difference\(s\)/);
  assert.match(both.stdout, /== constraints:[\s\S]*check-hcp-drift constraints: 0 difference\(s\)/);
  const only = f.cli(empty, '--mode', 'constraints');
  assert.equal(only.status, 0, only.stdout + only.stderr);
  assert.doesNotMatch(only.stdout, /== members/);
  const bad = f.cli(empty, '--mode', 'all');
  assert.equal(bad.status, 2);
  assert.match(bad.stderr, /--mode must be one of members, constraints, both/);
});

test('the CLI fails on a constraint difference, passes when it is allowlisted, and reports a stale entry', (t) => {
  const f = fixture();
  t.after(() => fs.rmSync(f.dir, { recursive: true, force: true }));
  const failing = f.cli(f.allow('c.json', { entries: [] }), '--commit', f.second);
  assert.equal(failing.status, 1, failing.stdout + failing.stderr);
  assert.match(failing.stdout, /hcp\/_shared:Rule\n[^\n]*\n[^\n]*\n {2}! maxLength +ruleId: ours 64, HCP none/);
  // Reported once, under the mapped shared object, not again under rules/set's response.
  assert.doesNotMatch(failing.stdout, /hcp\/rules\/set:response\n/);

  const entry = { object: 'hcp/_shared:Rule', path: 'ruleId', kind: 'maxLength', detail: 'ours *, HCP none', reason: 'framework bound', ref: 'Q98' };
  const passing = f.cli(f.allow('c.json', { entries: [entry] }), '--commit', f.second, '--verbose');
  assert.equal(passing.status, 0, passing.stdout + passing.stderr);
  assert.match(passing.stdout, /ruleId: ours 64, HCP none {2}\[allowlisted: Q98\]/);

  const stale = f.cli(f.allow('c.json', { entries: [entry] }));
  assert.equal(stale.status, 1, stale.stdout + stale.stderr);
  assert.match(stale.stdout, /Stale constraints allowlist entries[^\n]*\n {2}! hcp\/_shared:Rule ruleId maxLength \(ours \*, HCP none\) \(Q98\)/);

  const invalid = f.cli(f.allow('c.json', { entries: [{ ...entry, ref: '' }] }));
  assert.equal(invalid.status, 2);
  assert.match(invalid.stderr, /`ref` is missing/);
});

test('--pinned-commit prints the commit the carried specs pin', (t) => {
  const f = fixture();
  t.after(() => fs.rmSync(f.dir, { recursive: true, force: true }));
  const r = spawnSync(process.execPath, [CLI, '--pinned-commit', '--repo', f.repo], { encoding: 'utf8' });
  assert.equal(r.status, 0, r.stdout + r.stderr);
  assert.equal(r.stdout.trim(), f.first.slice(0, 7));
});
