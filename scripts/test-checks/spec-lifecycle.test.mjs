// Tests for scripts/lib/spec-lifecycle.mjs and scripts/lib/member-descriptions.mjs
// (#1132): the lag rule, the promotion rule and the description ratchet.
//
//   node --test scripts/test-checks/spec-lifecycle.test.mjs   (npm run test:checks)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { lifecycleFindings, compareVersions } from '../lib/spec-lifecycle.mjs';
import { undescribedMembers, checkMemberDescriptions } from '../lib/member-descriptions.mjs';

const v = (status, targetFrameworkVersion = '0.6.0') => ({ status, targetFrameworkVersion });
const keys = (findings) => findings.map((f) => f.key);

test('framework versions compare numerically, two-part as .0', () => {
  assert.equal(compareVersions('0.5', '0.5.0'), 0);
  assert.ok(compareVersions('0.10.0', '0.6.0') > 0);
  assert.ok(compareVersions('0.5.0', '0.6.0') < 0);
});

test('a new version must target the current framework', () => {
  const base = new Map([['a/0.1', v('draft', '0.5.0')]]);
  const head = new Map([...base, ['a/0.2', v('draft', '0.5.0')], ['b/0.1', v('draft', '0.6.0')], ['c/0.1', v('draft', '0.7.0')]]);
  assert.deepEqual(keys(lifecycleFindings({ base, head, current: '0.6.0' })), ['a/0.2']);
});

test('an existing version keeps its target until its next revision', () => {
  const base = new Map([['a/0.1', v('draft', '0.4.0')]]);
  const head = new Map([['a/0.1', { ...v('draft', '0.4.0') }]]);
  assert.deepEqual(lifecycleFindings({ base, head, current: '0.6.0' }), []);
});

test('a new retired or withdrawn version is not held to the current framework', () => {
  const head = new Map([['a/0.1', v('retired', '0.3.0')]]);
  assert.deepEqual(lifecycleFindings({ base: new Map(), head, current: '0.6.0' }), []);
});

test('promoting the latest live version is allowed', () => {
  const base = new Map([['a/0.1', v('draft')], ['a/0.2', v('draft')], ['a/0.3', v('retired')]]);
  const head = new Map([['a/0.1', v('draft')], ['a/0.2', v('candidate')], ['a/0.3', v('retired')]]);
  assert.deepEqual(lifecycleFindings({ base, head }), []);
});

test('promoting an older version above a newer draft is refused', () => {
  const base = new Map([['a/0.1', v('draft')], ['a/0.2', v('draft')]]);
  const head = new Map([['a/0.1', v('candidate')], ['a/0.2', v('draft')]]);
  const found = lifecycleFindings({ base, head });
  assert.deepEqual(keys(found), ['a/0.1']);
  assert.match(found[0].message, /a\/0\.2 is a later live version/);
});

test('candidate to standard on an older version is refused too', () => {
  const base = new Map([['a/0.1', v('candidate')], ['a/0.2', v('candidate')]]);
  const head = new Map([['a/0.1', v('standard')], ['a/0.2', v('candidate')]]);
  assert.deepEqual(keys(lifecycleFindings({ base, head })), ['a/0.1']);
});

test('a newer draft after a promotion is fine: nothing deprecated becomes current', () => {
  const base = new Map([['a/0.1', v('candidate')]]);
  const head = new Map([['a/0.1', v('candidate')], ['a/0.2', v('draft')]]);
  assert.deepEqual(lifecycleFindings({ base, head }), []);
});

test('retiring or withdrawing is not a promotion', () => {
  const base = new Map([['a/0.1', v('candidate')], ['a/0.2', v('draft')]]);
  const head = new Map([['a/0.1', v('retired')], ['a/0.2', v('draft')]]);
  assert.deepEqual(lifecycleFindings({ base, head }), []);
});

test('undescribed members are found at any depth; a $ref counts as described', () => {
  const schema = {
    type: 'object',
    properties: {
      a: { type: 'string', description: 'x' },
      b: { type: 'string' },
      c: { $ref: '#/$defs/C' },
      d: { type: 'object', properties: { e: { type: 'integer' } } },
    },
    examples: [{ properties: { notASchema: {} } }],
    $defs: { Response: { properties: { f: { type: 'boolean' } } } },
  };
  assert.deepEqual(undescribedMembers(schema), [
    '/properties/b',
    '/properties/d',
    '/properties/d/properties/e',
    '/$defs/Response/properties/f',
  ]);
});

function corpus(files, baseline) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tt-desc-'));
  const specs = path.join(root, 'specs');
  for (const [rel, text] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(specs, rel)), { recursive: true });
    fs.writeFileSync(path.join(specs, rel), typeof text === 'string' ? text : JSON.stringify(text));
  }
  const baselinePath = path.join(root, 'baseline.json');
  fs.writeFileSync(baselinePath, JSON.stringify({ baseline }));
  const out = { fail: [], warn: [], log: [] };
  checkMemberDescriptions({
    specsDir: specs,
    baselinePath,
    fail: (loc, msg) => out.fail.push(`${loc}: ${msg}`),
    warn: (msg) => out.warn.push(msg),
    log: (msg) => out.log.push(msg),
  });
  fs.rmSync(root, { recursive: true, force: true });
  return out;
}

const draft = '---\nstatus: draft\n---\n';
const two = { properties: { x: { type: 'string' }, y: { type: 'string' } } };
const one = { properties: { x: { type: 'string' }, y: { type: 'string', description: 'y' } } };

test('the ratchet fails a slug over its entry, and a new slug over zero', () => {
  const out = corpus({ 'a/0.1/spec.md': draft, 'a/0.1/payload.schema.json': two, 'b/0.1/spec.md': draft, 'b/0.1/payload.schema.json': one }, { a: 1 });
  assert.equal(out.fail.length, 2);
  assert.match(out.fail[0], /^specs\/a: 2 payload member/);
  assert.match(out.fail[1], /^specs\/b: 1 payload member.*allows 0/);
});

test('the ratchet passes at its entry and asks to be lowered below it', () => {
  assert.deepEqual(corpus({ 'a/0.1/spec.md': draft, 'a/0.1/payload.schema.json': two }, { a: 2 }).fail, []);
  const out = corpus({ 'a/0.1/spec.md': draft, 'a/0.1/payload.schema.json': one }, { a: 2 });
  assert.deepEqual(out.fail, []);
  assert.match(out.warn[0], /--lower/);
});

test('only the latest live version of a slug is counted', () => {
  const retired = '---\nstatus: retired\n---\n';
  const out = corpus({
    'a/0.1/spec.md': draft, 'a/0.1/payload.schema.json': two,
    'a/0.2/spec.md': draft, 'a/0.2/payload.schema.json': one,
    'r/0.1/spec.md': retired, 'r/0.1/payload.schema.json': two,
  }, { a: 1 });
  assert.deepEqual(out.fail, []);
});
