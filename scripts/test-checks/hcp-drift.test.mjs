// Tests for scripts/lib/hcp-drift.mjs and scripts/check-hcp-drift.mjs: the HCP
// schema drift check, against small fixture schemas. Nothing here reads the
// real hcp-spec checkout; the end-to-end tests build a throwaway git repository.
//
//   node --test scripts/test-checks/hcp-drift.test.mjs   (npm run test:checks)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { applyAllowlist, compareMapping, coverage, makeResolver } from '../lib/hcp-drift.mjs';

const CLI = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', 'check-hcp-drift.mjs');

const memory = (files) => makeResolver((f) => {
  if (!(f in files)) throw new Error(`no fixture ${f}`);
  return structuredClone(files[f]);
});
const run = (oursFiles, hcpFiles, objects) =>
  compareMapping({ mapping: { objects }, ours: memory(oursFiles), hcp: memory(hcpFiles) });
const brief = (findings) => findings.map((f) => `${f.path} ${f.kind}`).sort();

test('members on one side only, and required-ness', () => {
  const { findings } = run(
    { 'o.json': { type: 'object', required: ['a', 'b', 'token'], properties: { a: { type: 'string' }, b: { type: 'string' }, token: { type: 'string' } } } },
    { 'h.json': { type: 'object', required: ['a', 'note'], properties: { a: { type: 'string' }, b: { type: 'string' }, extra: { type: 'integer' } } } },
    [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }],
  );
  // `note` is required by HCP but not in its `properties`: still a member.
  assert.deepEqual(brief(findings), ['b required', 'extra hcp-only', 'note hcp-only', 'token ours-only']);
  assert.match(findings.find((f) => f.path === 'b').detail, /required in ours, optional in HCP/);
});

test('types, nullability and enums, through $refs on both sides', () => {
  const ours = {
    'specs/s/payload.json': {
      type: 'object',
      properties: {
        when: { type: ['string', 'null'] },
        count: { type: 'integer' },
        level: { type: 'string', enum: ['high', 'low'] },
        flag: { type: 'boolean', const: false },
        same: { $ref: '../shared/common.json#/$defs/Level' },
      },
    },
    'specs/shared/common.json': { $defs: { Level: { type: ['string', 'null'], enum: ['high', 'medium', 'low', null] } } },
  };
  const hcp = {
    'operations/op.json': {
      type: 'object',
      properties: {
        when: { $ref: '../common.json#/$defs/dateTime' },
        count: { type: 'number' },
        level: { enum: ['high', 'medium', 'low'] },
        flag: { type: 'boolean' },
        same: { $ref: '../common.json#/$defs/level' },
      },
    },
    'common.json': { $defs: { dateTime: { type: 'string', format: 'date-time' }, level: { enum: ['high', 'medium', 'low', null] } } },
  };
  const { findings } = run(ours, hcp, [{ id: 'x:request', ours: 'specs/s/payload.json', hcp: 'operations/op.json' }]);
  assert.deepEqual(brief(findings), ['count type', 'flag enum', 'level enum', 'when nullable']);
  assert.match(findings.find((f) => f.path === 'level').detail, /in HCP only: "medium"/);
  assert.match(findings.find((f) => f.path === 'flag').detail, /closed in ours only \(false\)/);
});

test('anyOf with null is nullability; a scalar anyOf unions its branches', () => {
  const { findings } = run(
    { 'o.json': { type: 'object', properties: { id: { type: ['string', 'null'] }, act: { type: 'string' } } } },
    { 'h.json': { type: 'object', properties: { id: { anyOf: [{ type: 'string' }, { type: 'null' }] }, act: { anyOf: [{ enum: ['share'] }, { type: 'string', format: 'uri' }] } } } },
    [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }],
  );
  // `act` is open on both sides (a named value plus any URI): no difference.
  assert.deepEqual(findings, []);
});

test('required alternatives (anyOf of { required }) are compared', () => {
  const { findings } = run(
    { 'o.json': { type: 'object', properties: { a: {}, b: {} } } },
    { 'h.json': { type: 'object', properties: { a: {}, b: {} }, anyOf: [{ required: ['a'] }, { required: ['b'] }] } },
    [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }],
  );
  assert.deepEqual(brief(findings), ['(root) alternatives']);
});

test('renames pair a member with its HCP name; nested paths use items and branches', () => {
  const { findings } = run(
    {
      'o.json': {
        type: 'object',
        required: ['respondedAt', 'entries'],
        properties: {
          respondedAt: { type: 'string' },
          entries: { type: 'array', items: { anyOf: [{ type: 'object', properties: { ref: { type: 'string' } } }, { type: 'object', properties: { id: { type: 'string' } } }] } },
        },
      },
    },
    {
      'h.json': {
        type: 'object',
        required: ['timestamp', 'entries'],
        properties: {
          timestamp: { type: 'string' },
          entries: { type: 'array', items: { anyOf: [{ type: 'object', properties: { ref: { type: 'integer' } } }, { type: 'object', properties: { id: { type: 'string' } } }] } },
        },
      },
    },
    [{ id: 'x:response', ours: 'o.json', hcp: 'h.json', renames: { respondedAt: 'timestamp' } }],
  );
  assert.deepEqual(brief(findings), ['entries[]|0.ref type']);
});

test('a member HCP declares only in a then-branch counts as an optional member', () => {
  const { findings } = run(
    { 'o.json': { type: 'object', required: ['profile'], properties: { profile: { type: 'string' }, value: { type: 'string' } } } },
    { 'h.json': { type: 'object', required: ['profile'], properties: { profile: { type: 'string' } }, allOf: [{ if: { properties: { profile: { const: 'x' } } }, then: { properties: { value: { type: 'string' } }, required: ['value'] } }] } },
    [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }],
  );
  assert.deepEqual(findings, []);
});

test('an object one side details and the other leaves bare reports the detailed members', () => {
  const { findings } = run(
    { 'o.json': { type: 'object', properties: { proposal: { type: 'object', additionalProperties: true } } } },
    { 'h.json': { type: 'object', properties: { proposal: { type: 'object', required: ['act'], properties: { act: { enum: ['accept'] } } } } } },
    [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }],
  );
  assert.deepEqual(brief(findings), ['proposal.act hcp-only']);
  // Both bare: nothing to compare.
  const both = run({ 'o.json': { type: 'object' } }, { 'h.json': { type: 'object' } }, [{ id: 'x:request', ours: 'o.json', hcp: 'h.json' }]);
  assert.deepEqual(both.findings, []);
});

test('a $ref pair that is a mapped object is compared once, as that object', () => {
  const ours = {
    'shared.json': { $defs: { Rule: { type: 'object', properties: { ruleId: { type: 'string' }, extra: { type: 'string' } } } } },
    'list.json': { type: 'object', properties: { rules: { type: 'array', items: { $ref: 'shared.json#/$defs/Rule' } } } },
  };
  const hcp = {
    'rule.json': { type: 'object', properties: { ruleId: { type: 'string' } } },
    'list-result.json': { type: 'object', properties: { rules: { type: 'array', items: { $ref: 'rule.json' } } } },
  };
  const result = run(ours, hcp, [
    { id: 'hcp/_shared:Rule', ours: 'shared.json#/$defs/Rule', hcp: 'rule.json' },
    { id: 'x:response', ours: 'list.json', hcp: 'list-result.json' },
    { id: 'y:response', ours: 'list.json#/properties/rules/items', hcp: 'rule.json' },
  ]);
  assert.deepEqual(result.findings.map((f) => `${f.object} ${f.path}`), ['hcp/_shared:Rule extra']);
  assert.deepEqual(result.covered, [{ object: 'y:response', by: 'hcp/_shared:Rule' }]);
});

test('a mapping that names a missing pointer is an error', () => {
  assert.throws(() => run({ 'o.json': {} }, { 'h.json': {} }, [{ id: 'x:request', ours: 'o.json#/$defs/Nope', hcp: 'h.json' }]), /no such pointer/);
});

test('the allowlist: wildcards, stale entries and malformed entries', () => {
  const findings = [
    { object: 'hcp/a:request', path: 'accessToken', kind: 'ours-only', detail: '' },
    { object: 'hcp/b:request', path: 'accessToken', kind: 'ours-only', detail: '' },
    { object: 'hcp/b:request', path: 'other', kind: 'required', detail: '' },
  ];
  const entry = (object, p, kind) => ({ object, path: p, kind, reason: 'r', ref: 'Q1' });
  const r = applyAllowlist(findings, [entry('hcp/*:request', 'accessToken', 'ours-only'), entry('hcp/b:request', 'other', 'type'), entry('hcp/c:request', 'x', 'ours-only')]);
  assert.equal(r.allowed.length, 2);
  assert.deepEqual(r.unexpected.map((f) => f.path), ['other']);
  assert.deepEqual(r.stale.map((e) => `${e.object} ${e.kind}`), ['hcp/b:request type', 'hcp/c:request ours-only']);
  assert.deepEqual(r.invalid, []);
  const bad = applyAllowlist([], [{ object: 'x', path: 'y', kind: 'missing-kind', reason: 'r' }]);
  assert.deepEqual(bad.invalid, ['entry 0: missing `ref`', 'entry 0: unknown kind "missing-kind"']);
});

test('coverage: every carried side is mapped or listed as not compared', () => {
  const carried = [{ slug: 'hcp/a', hasResponse: true }, { slug: 'hcp/b', hasResponse: false }];
  const mapping = { objects: [{ id: 'hcp/a:request' }, { id: 'hcp/_shared:X' }, { id: 'hcp/gone:request' }], notCompared: { 'hcp/b:request': 'prose only' } };
  assert.deepEqual(coverage({ carried, mapping }), { missing: ['hcp/a:response'], unknown: ['hcp/gone:request'] });
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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hcp-drift-'));
  const hcpDir = path.join(dir, 'hcp');
  const write = (base, rel, value) => {
    fs.mkdirSync(path.dirname(path.join(base, rel)), { recursive: true });
    fs.writeFileSync(path.join(base, rel), typeof value === 'string' ? value : JSON.stringify(value, null, 2));
  };
  const git = (...args) => execFileSync('git', ['-C', hcpDir, ...args], { env: gitEnv, encoding: 'utf8' }).trim();
  fs.mkdirSync(hcpDir);
  git('init', '-q');
  const rule = (extra) => ({ type: 'object', required: ['ruleId'], properties: { ruleId: { $ref: 'common.json#/$defs/urnUuid' }, ...extra } });
  write(hcpDir, 'schemas/d/common.json', { $defs: { urnUuid: { type: 'string' } } });
  write(hcpDir, 'schemas/d/rule.json', rule({}));
  git('add', '-A');
  git('commit', '-q', '-m', 'one');
  const first = git('rev-parse', 'HEAD');
  // A later HCP commit adds a member; the pinned commit must still be what is read.
  write(hcpDir, 'schemas/d/rule.json', rule({ added: { type: 'string' } }));
  git('commit', '-q', '-am', 'two');
  const second = git('rev-parse', 'HEAD');

  const repo = path.join(dir, 'repo');
  write(repo, 'specs/hcp/_shared/0.1/rule.schema.json', { $defs: { Rule: { type: 'object', required: ['ruleId'], properties: { ruleId: { type: 'string' } } } } });
  write(repo, 'specs/hcp/rules/set/0.1/payload.schema.json', {
    type: 'object',
    required: ['accessToken'],
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
  const allow = (entries) => {
    write(dir, 'allow.json', { entries });
    return path.join(dir, 'allow.json');
  };
  const cli = (allowlist, ...args) => spawnSync(process.execPath, [CLI, '--hcp', hcpDir, '--repo', repo, '--map', map, '--allowlist', allowlist, ...args], { encoding: 'utf8' });
  return { dir, repo, map, write, allow, cli, first, second };
}

test('the CLI reads HCP at the pinned commit, whatever the checkout HEAD is', (t) => {
  const f = fixture();
  t.after(() => fs.rmSync(f.dir, { recursive: true, force: true }));
  const r = f.cli(f.allow([]));
  assert.equal(r.status, 0, r.stdout + r.stderr);
  assert.match(r.stdout, new RegExp(`at ${f.first.slice(0, 7)} `));
  assert.match(r.stdout, /0 difference\(s\)/);
  assert.match(r.stdout, /covered by another mapped object/);
});

test('the CLI fails on a difference, passes when it is allowlisted, and reports a stale entry', (t) => {
  const f = fixture();
  t.after(() => fs.rmSync(f.dir, { recursive: true, force: true }));
  const failing = f.cli(f.allow([]), '--commit', f.second);
  assert.equal(failing.status, 1, failing.stdout + failing.stderr);
  assert.match(failing.stdout, /hcp\/_shared:Rule\n/);
  assert.match(failing.stdout, /! hcp-only +added: optional in HCP/);

  const entry = { object: 'hcp/_shared:Rule', path: 'added', kind: 'hcp-only', reason: 'not carried', ref: 'Q99' };
  const passing = f.cli(f.allow([entry]), '--commit', f.second, '--verbose');
  assert.equal(passing.status, 0, passing.stdout + passing.stderr);
  assert.match(passing.stdout, /added: optional in HCP.*\[allowlisted: Q99\]/);

  const stale = f.cli(f.allow([entry]));
  assert.equal(stale.status, 1, stale.stdout + stale.stderr);
  assert.match(stale.stdout, /Stale allowlist entries[^\n]*\n {2}! hcp\/_shared:Rule added hcp-only \(Q99\)/);
});

test('the CLI fails when a carried side is not in the mapping, and on a bad commit', (t) => {
  const f = fixture();
  t.after(() => fs.rmSync(f.dir, { recursive: true, force: true }));
  const m = JSON.parse(fs.readFileSync(f.map, 'utf8'));
  m.notCompared = {};
  f.write(f.dir, 'map.json', m);
  const r = f.cli(f.allow([]));
  assert.equal(r.status, 1, r.stdout + r.stderr);
  assert.match(r.stdout, /does not account for[^\n]*\n {2}! hcp\/rules\/set:request/);
  const bad = f.cli(f.allow([]), '--commit', 'deadbeef');
  assert.equal(bad.status, 2);
  assert.match(bad.stderr, /cannot resolve commit deadbeef/);
  const usage = spawnSync(process.execPath, [CLI], { encoding: 'utf8' });
  assert.equal(usage.status, 2);
  assert.match(usage.stderr, /--hcp is required/);
});
