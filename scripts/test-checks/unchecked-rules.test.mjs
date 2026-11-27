// The `uncheckedRules` front-matter check (scripts/lib/unchecked-rules.mjs),
// over a throwaway schema tree: pointers that resolve directly, through a
// local `$ref`, through a `$ref` into a shared file and through an `$anchor`;
// dangling pointers, pointers that name no schema and broken `$ref`s fail the
// build; a description that says a rule is not checked must be declared;
// declared rules are logged, not failed; and build-registry.mjs wires it.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  MAX_REF_HOPS,
  checkUncheckedRules,
  findMarkedSchemas,
  makeUncheckedRulesReporter,
  pointerTokens,
  resolveSchemaPointer,
} from '../lib/unchecked-rules.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');

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

/** Rewrite the tree's payload schema through `edit(schema)`. */
function editPayload(payload, edit) {
  const schema = JSON.parse(fs.readFileSync(payload, 'utf8'));
  edit(schema);
  fs.writeFileSync(payload, JSON.stringify(schema));
}

test('a member must name a schema, not a map of schemas, an array of schemas or a keyword value', () => {
  const { payload } = tree();
  editPayload(payload, (schema) => {
    schema.required = ['grantId'];
    schema.anyOf = [{ required: ['grantId'] }, { required: ['local'] }];
    schema.properties.required = { type: 'string' };
    schema.properties.withConst = { const: { a: 1 } };
  });
  const cases = {
    '/properties': /ends at 'properties', a map of schemas/,
    '/$defs': /ends at '\$defs', a map of schemas/,
    '/anyOf': /ends at 'anyOf', an array of schemas/,
    '/required': /keyword value/,
    '/anyOf/0/required': /keyword value/,
    '/properties/local/properties': /a map of schemas/,
    '/properties/withConst/const': /keyword value/,
  };
  for (const [pointer, reason] of Object.entries(cases)) {
    const resolved = resolveSchemaPointer(payload, pointer);
    assert.equal(resolved.ok, false, pointer);
    assert.match(resolved.reason, reason, pointer);
  }
  // A member named like a keyword is a schema, and so is an item of `anyOf`.
  for (const pointer of ['/properties/required', '/anyOf/1']) {
    assert.equal(resolveSchemaPointer(payload, pointer).ok, true, pointer);
  }
});

test('a $ref cycle stops at MAX_REF_HOPS and fails, rather than looping', () => {
  const { payload } = tree();
  editPayload(payload, (schema) => {
    schema.$defs.A = { $ref: '#/$defs/B' };
    schema.$defs.B = { $ref: '#/$defs/A' };
    schema.properties.loop = { $ref: '#/$defs/A' };
  });
  const through = resolveSchemaPointer(payload, '/properties/loop/properties/x');
  assert.equal(through.ok, false);
  assert.match(through.reason, new RegExp(`longer than ${MAX_REF_HOPS} hops`));
  // A pointer that ends on the cycle fails too: its own $ref chain never reaches a schema.
  const at = resolveSchemaPointer(payload, '/properties/loop');
  assert.equal(at.ok, false);
  assert.match(at.reason, /a cycle\?/);
  // The cross-check walks the same cycle without looping.
  assert.deepEqual(findMarkedSchemas(payload).marked, []);
});

test('a $ref naming a missing $anchor, or a file that is not JSON, fails with a reason', () => {
  const { root, payload } = tree();
  fs.writeFileSync(path.join(root, 'specs', 'fam', '_shared', '0.1', 'bad.schema.json'), '{ "$defs": ');
  editPayload(payload, (schema) => {
    schema.properties.noAnchor = { $ref: '../../_shared/0.1/common.schema.json#nosuch' };
    schema.properties.bad = { $ref: '../../_shared/0.1/bad.schema.json#/$defs/X' };
  });
  const noAnchor = resolveSchemaPointer(payload, '/properties/noAnchor/properties/x');
  assert.equal(noAnchor.ok, false);
  assert.match(noAnchor.reason, /names no \$anchor 'nosuch'/);
  const bad = resolveSchemaPointer(payload, '/properties/bad/properties/x');
  assert.equal(bad.ok, false);
  assert.match(bad.reason, /bad\.schema\.json is not valid JSON/);
  // Through checkUncheckedRules, both are failures, not exceptions.
  const { failures } = run(
    {
      uncheckedRules: [
        { member: '/properties/noAnchor/properties/x', rule: 'r', source: 's' },
        { member: '/properties/bad/properties/x', rule: 'r', source: 's' },
      ],
    },
    payload
  );
  assert.equal(failures.length, 2);
});

test('a non-string member fails rather than throwing', () => {
  const { payload } = tree();
  const { failures } = run(
    {
      uncheckedRules: [
        { member: 42, rule: 'r', source: 's' },
        { member: null, rule: 'r', source: 's' },
        { rule: 'r', source: 's' },
      ],
    },
    payload
  );
  assert.equal(failures.length, 3);
  for (const f of failures) assert.match(f, /is not a JSON Pointer/);
});

/** A tree whose schemas carry marker phrases, in the payload and in a shared file. */
function markedTree() {
  const t = tree();
  const sharedFile = path.join(t.root, 'specs', 'fam', '_shared', '0.1', 'common.schema.json');
  const shared = JSON.parse(fs.readFileSync(sharedFile, 'utf8'));
  shared.$defs.Entry.properties.type.description = 'It includes `X`; a validator does not check it.';
  shared.$defs.Content.description = 'Needs `value` under one profile; not checked by the schema.';
  shared.$defs.Unused = { type: 'object', description: 'Unreached; not checked by the schema.' };
  fs.writeFileSync(sharedFile, JSON.stringify(shared));
  editPayload(t.payload, (schema) => {
    schema.properties.grantId.description = 'Names a live grant. Not checked by the schema.';
  });
  return t;
}

test('the cross-check finds every marked description the payload schema reaches, and only those', () => {
  const { payload } = markedTree();
  const pointers = findMarkedSchemas(payload).marked.map((m) => m.pointer);
  assert.deepEqual(pointers.sort(), [
    '/$defs/Response/properties/entries/items/anyOf/0/properties/content',
    '/$defs/Response/properties/entries/items/anyOf/0/properties/type',
    '/properties/grantId',
  ]);
  // Each reported pointer is itself a valid member.
  for (const pointer of pointers) assert.equal(resolveSchemaPointer(payload, pointer).ok, true, pointer);
});

test('a marked description with no entry at or under it fails; an entry at it or under it covers it', () => {
  const { payload } = markedTree();
  const none = run({}, payload);
  assert.equal(none.failures.length, 3);
  assert.match(none.failures[0], /says a rule is not checked/);
  assert.match(none.failures.join('\n'), /member '\/properties\/grantId'/);

  const covered = run(
    {
      uncheckedRules: [
        { member: '/properties/grantId', rule: 'r', source: 's' },
        // Under the marked `Content` (through the $ref at `content`).
        {
          member: '/$defs/Response/properties/entries/items/anyOf/0/properties/content/properties/value',
          rule: 'r',
          source: 's',
        },
        { member: '/$defs/Response/properties/entries/items/anyOf/0/properties/type', rule: 'r', source: 's' },
      ],
    },
    payload
  );
  assert.deepEqual(covered.failures, []);
});

test('an entry naming a $ref covers the marked schema the $ref leads to', () => {
  const { payload } = markedTree();
  const { failures } = run(
    {
      uncheckedRules: [
        { member: '/properties/grantId', rule: 'r', source: 's' },
        // `content` is `{ $ref: '#/$defs/Content' }`: the entry is at the marked schema.
        { member: '/$defs/Response/properties/entries/items/anyOf/0/properties/content', rule: 'r', source: 's' },
      ],
    },
    payload
  );
  assert.equal(failures.length, 1);
  assert.match(failures[0], /anyOf\/0\/properties\/type/);
});

test('the reporter prints one summary line, and the per-rule detail only under verbose', () => {
  const { payload } = tree();
  const meta = { uncheckedRules: [{ member: '/properties/grantId', rule: 'Names a live grant.', source: 'HCP 6.2' }] };
  for (const verbose of [false, true]) {
    const logs = [];
    const failures = [];
    const reporter = makeUncheckedRulesReporter({ fail: (l, m) => failures.push(m), log: (l) => logs.push(l), verbose });
    reporter.check(meta, payload, 'fam/get/0.1');
    reporter.check({}, payload, 'fam/other/0.1');
    assert.deepEqual(failures, []);
    assert.equal(logs.length, verbose ? 2 : 0);
    assert.match(reporter.summary(), /1 rule\(s\) declared by 1 spec\(s\)/);
    assert.equal(/--verbose lists them/.test(reporter.summary()), !verbose);
  }
});

test('build-registry.mjs runs the check over the registry: a summary line, the rules under --verbose', () => {
  const result = spawnSync(process.execPath, [path.join(ROOT, 'scripts', 'build-registry.mjs'), '--validate-only', '--verbose'], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /^ {2}Unchecked rules: \d+ rule\(s\) declared by \d+ spec\(s\)/m);
  assert.match(result.stdout, /^ {2}Unchecked rules: hcp\/entries\/query\/0\.1 declares \d+ rule\(s\)/m);
  assert.match(result.stdout, /^ {4}- \/\$defs\/Response\/properties\/entries\/items\/anyOf\/1\/properties\/type: /m);
});
