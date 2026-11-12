// The spec-examples lint (scripts/lib/spec-examples.mjs, #1131), over a
// throwaway specs/ tree: it finds examples by `type`, validates payloads
// against the spec's own schema with cross-file $refs resolved on disk, and
// its baseline only shrinks.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { checkSpecExamples, exampleFindings, jsonBlocks } from '../lib/spec-examples.mjs';

function tree(specs) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'spec-examples-'));
  const specsDir = path.join(root, 'specs');
  fs.mkdirSync(path.join(specsDir, 'fam', '_shared', '0.1'), { recursive: true });
  fs.writeFileSync(
    path.join(specsDir, 'fam', '_shared', '0.1', 'ids.schema.json'),
    JSON.stringify({ $id: 'https://trusttasks.org/spec/fam/_shared/0.1/ids', $defs: { Id: { type: 'string', pattern: '^id-' } } })
  );
  for (const [slugVersion, { md, schema }] of Object.entries(specs)) {
    const dir = path.join(specsDir, slugVersion);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, 'spec.md'), md);
    fs.writeFileSync(path.join(dir, 'payload.schema.json'), JSON.stringify(schema));
  }
  return { root, specsDir };
}

const schema = {
  $schema: 'https://json-schema.org/draft/2020-12/schema',
  type: 'object',
  required: ['id'],
  properties: { id: { $ref: '../../_shared/0.1/ids.schema.json#/$defs/Id' } },
  additionalProperties: false,
  $defs: { Response: { $anchor: 'response', type: 'object', required: ['ok'], properties: { ok: { type: 'boolean' } } } },
};
const doc = (type, payload) => '```json\n' + JSON.stringify({ id: 'x', type, payload }, null, 2) + '\n```\n';
const URI = 'https://trusttasks.org/spec/fam/do/0.2';

test('jsonBlocks reads indented and quoted fences and skips fragments', () => {
  const md = '  ```json\n  {"a": 1}\n  ```\n> ```json\n> {"b": 2}\n> ```\n```json\n{ … }\n```\n';
  assert.deepEqual(jsonBlocks(md), [{ a: 1 }, { b: 2 }]);
});

test('examples are found by type anywhere and validated against the own schema', () => {
  const { specsDir } = tree({
    'fam/do/0.1': { md: '---\nstatus: draft\n---\n', schema },
    'fam/do/0.2': {
      md: '---\nstatus: draft\n---\n## Usage\n' + doc(URI, { id: 'id-1' }) + doc(`${URI}#response`, { ok: 'yes' }),
      schema,
    },
    'fam/gone/0.1': { md: '---\nstatus: retired\n---\n', schema },
  });
  const { findings, checked } = exampleFindings(specsDir);
  assert.equal(checked, 1, 'only the latest live version is checked');
  assert.deepEqual(
    findings.map((f) => `${f.key} ${f.rule}`),
    ['fam/do/0.2 invalid-response']
  );
});

test('a $ref into a shared schema is resolved on disk', () => {
  const { specsDir } = tree({
    'fam/do/0.2': { md: '---\nstatus: draft\n---\n' + doc(URI, { id: 'nope' }) + doc(`${URI}#response`, { ok: true }), schema },
  });
  assert.deepEqual(
    exampleFindings(specsDir).findings.map((f) => f.rule),
    ['invalid-request']
  );
});

test('the baseline excuses listed findings and fails on new and stale ones', () => {
  const { root, specsDir } = tree({
    'fam/do/0.2': { md: '---\nstatus: draft\n---\n' + doc(URI, { id: 'id-1' }), schema },
    'fam/other/0.1': { md: '---\nstatus: draft\n---\n', schema },
  });
  const allowlistPath = path.join(root, 'allow.json');
  fs.writeFileSync(
    allowlistPath,
    JSON.stringify({
      exceptions: {
        'fam/do/0.2': { 'missing-response': 'baseline' },
        'fam/other/0.1': { 'missing-request': 'baseline', 'invalid-request': 'stale' },
      },
    })
  );
  const failures = [];
  checkSpecExamples({ specsDir, allowlistPath, fail: (at, msg) => failures.push(`${at}: ${msg}`), warn() {}, log() {} });
  assert.equal(failures.length, 2, failures.join('\n'));
  assert.match(failures[0], /fam\/other\/0\.1\/spec\.md: missing-response/);
  assert.match(failures[1], /"invalid-request" for fam\/other\/0\.1 no longer applies/);
});
