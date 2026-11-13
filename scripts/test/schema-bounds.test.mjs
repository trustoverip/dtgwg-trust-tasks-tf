// The schema-bounds lint (scripts/lib/schema-bounds.mjs), over a throwaway
// specs/ tree: request-side bounds, URL scheme constraints on either side,
// shared definitions followed with the side that reaches them, and a baseline
// that only shrinks.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { baselineFor, boundsFindings, checkSchemaBounds } from '../lib/schema-bounds.mjs';

function tree(specs, shared = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'schema-bounds-'));
  const specsDir = path.join(root, 'specs');
  fs.mkdirSync(path.join(specsDir, 'fam', '_shared', '0.1'), { recursive: true });
  fs.writeFileSync(path.join(specsDir, 'fam', '_shared', '0.1', 'common.schema.json'), JSON.stringify({ $defs: shared }));
  for (const [slugVersion, { status = 'draft', schema }] of Object.entries(specs)) {
    const dir = path.join(specsDir, slugVersion);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, 'spec.md'), `---\nstatus: ${status}\n---\n`);
    fs.writeFileSync(path.join(dir, 'payload.schema.json'), JSON.stringify(schema));
  }
  return { root, specsDir };
}

const ids = (findings) => findings.map((f) => `${f.owner} ${f.rule} ${f.pointer}`).sort();

test('request-side arrays, limit and cursor need bounds; the response side does not', () => {
  const unbounded = {
    type: 'object',
    properties: {
      tags: { type: 'array', items: { type: 'string' } },
      pair: { type: 'array', prefixItems: [{ type: 'string' }], items: false },
      ok: { type: 'array', items: { type: 'string' }, maxItems: 10 },
      limit: { type: 'integer', minimum: 1 },
      cursor: { type: 'string' },
    },
  };
  const { specsDir } = tree({
    'fam/list/0.1': { schema: { ...unbounded, $defs: { Response: unbounded } } },
  });
  assert.deepEqual(ids(boundsFindings(specsDir).findings), [
    'fam/list/0.1 array-max-items /properties/tags',
    'fam/list/0.1 cursor-max-length /properties/cursor',
    'fam/list/0.1 limit-maximum /properties/limit',
  ]);
});

test('a bounded limit and cursor pass, including through a $ref', () => {
  const { specsDir } = tree({
    'fam/list/0.1': {
      schema: {
        type: 'object',
        properties: {
          limit: { type: 'integer', minimum: 1, maximum: 100 },
          cursor: { $ref: '#/$defs/Cursor' },
        },
        $defs: { Cursor: { type: 'string', maxLength: 512 } },
      },
    },
  });
  assert.deepEqual(boundsFindings(specsDir).findings, []);
});

test('URL members need a scheme constraint on either side', () => {
  const { specsDir } = tree({
    'fam/site/0.1': {
      schema: {
        type: 'object',
        properties: {
          logoUrl: { type: 'string', format: 'uri' },
          publicUrl: { type: 'string', format: 'uri', pattern: '^https://' },
          serviceEndpoint: { type: 'string', pattern: '^(https|wss)://' },
          peers: { type: 'array', maxItems: 4, items: { type: 'string', pattern: '^did:' } },
          callbackUris: { type: 'array', maxItems: 4, items: { type: 'string', format: 'uri' } },
          homeHref: { type: 'string', format: 'uri', description: 'An `https` URL.' },
          kind: { type: 'string', enum: ['a'] },
          typeUri: { type: 'string' },
          payloadBase64Url: { type: 'string' },
          link: { type: 'object', properties: { rel: { type: 'string' } } },
        },
        $defs: { Response: { type: 'object', properties: { installUrl: { type: 'string' } } } },
      },
    },
  });
  assert.deepEqual(ids(boundsFindings(specsDir).findings), [
    'fam/site/0.1 url-scheme /$defs/Response/properties/installUrl',
    'fam/site/0.1 url-scheme /properties/callbackUris',
    'fam/site/0.1 url-scheme /properties/logoUrl',
  ]);
});

test('a shared definition is checked with the side that reaches it and owned by its file', () => {
  const { specsDir } = tree(
    {
      'fam/a/0.1': {
        schema: {
          type: 'object',
          properties: { items: { $ref: '../../_shared/0.1/common.schema.json#/$defs/Items' } },
          $defs: {
            Response: { type: 'object', properties: { rows: { $ref: '../../_shared/0.1/common.schema.json#/$defs/Rows' } } },
            Unused: { type: 'array' },
          },
        },
      },
      'fam/b/0.1': {
        schema: { type: 'object', properties: { items: { $ref: '../../_shared/0.1/common.schema.json#/$defs/Items' } } },
      },
    },
    { Items: { type: 'array', items: { type: 'string' } }, Rows: { type: 'array', items: { type: 'string' } } }
  );
  assert.deepEqual(ids(boundsFindings(specsDir).findings), [
    'fam/_shared/0.1/common.schema.json array-max-items /$defs/Items',
  ]);
});

test('only the latest live version is checked', () => {
  const bad = { type: 'object', properties: { tags: { type: 'array' } } };
  const { specsDir } = tree({
    'fam/a/0.1': { schema: bad },
    'fam/a/0.2': { schema: { type: 'object', properties: {} } },
    'fam/gone/0.1': { status: 'retired', schema: bad },
  });
  const { findings, checked } = boundsFindings(specsDir);
  assert.equal(checked, 1);
  assert.deepEqual(findings, []);
});

test('the baseline excuses listed findings and fails on new and stale ones', () => {
  const { root, specsDir } = tree({
    'fam/a/0.1': { schema: { type: 'object', properties: { tags: { type: 'array' } } } },
    'fam/b/0.1': { schema: { type: 'object', properties: { cursor: { type: 'string' } } } },
  });
  const allowlistPath = path.join(root, 'allow.json');
  fs.writeFileSync(
    allowlistPath,
    JSON.stringify({
      exceptions: {
        'fam/a/0.1': { 'array-max-items /properties/tags': 'baseline', 'limit-maximum /properties/limit': 'stale' },
      },
    })
  );
  const failures = [];
  checkSchemaBounds({ specsDir, allowlistPath, fail: (at, msg) => failures.push(`${at}: ${msg}`), warn() {}, log() {} });
  assert.equal(failures.length, 2, failures.join('\n'));
  assert.match(failures[0], /fam\/b\/0\.1\/payload\.schema\.json#\/properties\/cursor: cursor-max-length/);
  assert.match(failures[1], /"limit-maximum \/properties\/limit" for fam\/a\/0\.1 no longer applies/);

  // The generated baseline is exactly what the tree needs.
  fs.writeFileSync(allowlistPath, JSON.stringify({ exceptions: baselineFor(specsDir) }));
  const none = [];
  checkSchemaBounds({ specsDir, allowlistPath, fail: (at, msg) => none.push(`${at}: ${msg}`), warn() {}, log() {} });
  assert.deepEqual(none, []);
});

test('an unreadable baseline fails rather than checking nothing', () => {
  const { root, specsDir } = tree({});
  const failures = [];
  checkSchemaBounds({
    specsDir,
    allowlistPath: path.join(root, 'missing.json'),
    fail: (at, msg) => failures.push(`${at}: ${msg}`),
    warn() {},
    log() {},
  });
  assert.equal(failures.length, 1);
  assert.match(failures[0], /could not be read/);
});
