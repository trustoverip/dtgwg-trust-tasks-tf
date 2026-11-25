// The `uncheckedRules` front-matter check (scripts/lib/unchecked-rules.mjs),
// over a throwaway schema tree: pointers that resolve directly, through a
// local `$ref`, through a `$ref` into a shared file and through an `$anchor`;
// dangling pointers fail the build; declared rules are logged, not failed.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { checkUncheckedRules, pointerTokens, resolveSchemaPointer } from '../lib/unchecked-rules.mjs';

function tree() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'unchecked-rules-'));
  const shared = path.join(root, 'specs', 'fam', '_shared', '0.1');
  const spec = path.join(root, 'specs', 'fam', 'get', '0.1');
  fs.mkdirSync(shared, { recursive: true });
  fs.mkdirSync(spec, { recursive: true });
  fs.writeFileSync(
    path.join(shared, 'common.schema.json'),
    JSON.stringify({
      $defs: {
        Entry: {
          type: 'object',
          properties: { type: { type: 'array' }, content: { $ref: '#/$defs/Content' } },
        },
        Content: { type: 'object', properties: { value: { type: 'string' } } },
        Tagged: { $anchor: 'tagged', type: 'object', properties: { tag: { type: 'string' } } },
      },
    })
  );
  const payload = path.join(spec, 'payload.schema.json');
  fs.writeFileSync(
    payload,
    JSON.stringify({
      type: 'object',
      properties: {
        grantId: { type: 'string' },
        local: { $ref: '#/$defs/Local' },
        tagged: { $ref: '../../_shared/0.1/common.schema.json#tagged' },
      },
      $defs: {
        Local: { type: 'object', properties: { note: { type: 'string' } } },
        Response: {
          $anchor: 'response',
          type: 'object',
          properties: {
            entries: { type: 'array', items: { anyOf: [{ $ref: '../../_shared/0.1/common.schema.json#/$defs/Entry' }] } },
          },
        },
      },
    })
  );
  return { root, payload };
}

function run(meta, payload) {
  const failures = [];
  const logs = [];
  const count = checkUncheckedRules(meta, payload, 'fam/get/0.1', {
    fail: (loc, msg) => failures.push(`${loc}: ${msg}`),
    log: (line) => logs.push(line),
  });
  return { count, failures, logs };
}

test('pointer tokens decode RFC 6901 escapes and reject a relative pointer', () => {
  assert.deepEqual(pointerTokens('/a~1b/c~0d'), ['a/b', 'c~d']);
  assert.deepEqual(pointerTokens(''), []);
  assert.equal(pointerTokens('properties/x'), null);
});

test('pointers resolve directly, through a local $ref, into a shared file and through an $anchor', () => {
  const { payload } = tree();
  for (const pointer of [
    '/properties/grantId',
    '/properties/local/properties/note',
    '/$defs/Response/properties/entries/items/anyOf/0/properties/type',
    '/$defs/Response/properties/entries/items/anyOf/0/properties/content/properties/value',
    '/properties/tagged/properties/tag',
  ]) {
    const resolved = resolveSchemaPointer(payload, pointer);
    assert.equal(resolved.ok, true, `${pointer}: ${resolved.reason}`);
  }
});

test('declared rules are logged, one line each, and are not failures', () => {
  const { payload } = tree();
  const { count, failures, logs } = run(
    {
      uncheckedRules: [
        { member: '/properties/grantId', rule: 'Names a live grant.', source: 'HCP 6.2' },
        {
          member: '/$defs/Response/properties/entries/items/anyOf/0/properties/type',
          rule: 'Includes `ContextEntry`.',
          source: 'HCP 4.2',
        },
      ],
    },
    payload
  );
  assert.equal(count, 2);
  assert.deepEqual(failures, []);
  assert.equal(logs.length, 3);
  assert.match(logs[0], /fam\/get\/0\.1 declares 2 rule\(s\)/);
  assert.match(logs[2], /Includes `ContextEntry`\. \(HCP 4\.2\)/);
});

test('a dangling member fails the build, naming the entry and where resolution stopped', () => {
  const { payload } = tree();
  const { failures } = run(
    {
      uncheckedRules: [
        { member: '/properties/grantId', rule: 'ok', source: 'x' },
        { member: '/$defs/Response/properties/entries/items/anyOf/0/properties/missing', rule: 'r', source: 's' },
      ],
    },
    payload
  );
  assert.equal(failures.length, 1);
  assert.match(failures[0], /^fam\/get\/0\.1\/spec\.md: uncheckedRules\[1\]\.member/);
  assert.match(failures[0], /no 'missing'/);
});

test('a $ref into a missing file, a relative pointer and a keyword value all fail', () => {
  const { payload } = tree();
  const schema = JSON.parse(fs.readFileSync(payload, 'utf8'));
  schema.properties.broken = { $ref: '../../_shared/0.1/absent.schema.json#/$defs/X' };
  fs.writeFileSync(payload, JSON.stringify(schema));
  const { failures } = run(
    {
      uncheckedRules: [
        { member: '/properties/broken/properties/x', rule: 'r', source: 's' },
        { member: 'properties/grantId', rule: 'r', source: 's' },
        { member: '/properties/grantId/type', rule: 'r', source: 's' },
      ],
    },
    payload
  );
  assert.equal(failures.length, 3);
  assert.match(failures[0], /does not exist/);
  assert.match(failures[1], /not a JSON Pointer/);
  assert.match(failures[2], /keyword value/);
});

test('a spec that declares nothing logs nothing', () => {
  const { payload } = tree();
  assert.deepEqual(run({}, payload), { count: 0, failures: [], logs: [] });
});
