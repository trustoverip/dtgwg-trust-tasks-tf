// Tests for website/assets/routes.js — how the page maps a link written in
// spec or binding prose onto a site route (#1146), and the shared version
// helpers.
//
//   node --test scripts/test/website-routes.test.mjs   (npm run test:website)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const ctx = { URL };
ctx.window = ctx;
vm.createContext(ctx);
vm.runInContext(fs.readFileSync(path.join(here, '..', '..', 'website', 'assets', 'routes.js'), 'utf8'), ctx);
const R = ctx.ttRoutes;

const GRANT = '/specs/acl/grant/0.1/spec.md';
const shared = new Map([['/specs/acl/_shared/0.2/acl-entry.schema.json', 'acl/_shared/0.2/acl-entry']]);
const map = (href, from = GRANT) => R.resolveProseHref(href, from, shared).href;

test('a sibling spec link lands on its spec page, with the family kept', () => {
  // The issue's own example: the browser resolved this against the page URL
  // and produced /spec/change-role/0.1/spec.md.
  assert.equal(map('../../change-role/0.1/spec.md'), '/spec/acl/change-role/0.1');
  assert.equal(map('../../change-role/0.1/spec.md#response'), '/spec/acl/change-role/0.1#response');
  assert.equal(map('../0.2/spec.md'), '/spec/acl/grant/0.2');
  assert.equal(map('../0.2/'), '/spec/acl/grant/0.2');
});

test('framework links land on /specification, from any depth', () => {
  assert.equal(map('../../../../SPEC.md#62-content-negotiation'), '/specification#62-content-negotiation');
  assert.equal(map('/SPEC.md#4-trust-task-documents'), '/specification#4-trust-task-documents');
});

test('binding links land on the binding page', () => {
  assert.equal(map('../../../../bindings/https/0.2/spec.md'), '/binding/https/0.2');
  assert.equal(map('../../https/0.2/spec.md#3-x', '/bindings/didcomm/0.2/spec.md'), '/binding/https/0.2#3-x');
});

test('shared schemas land on /schema, other files stay files', () => {
  assert.equal(map('../../_shared/0.2/acl-entry.schema.json'), '/schema/acl/_shared/0.2/acl-entry');
  assert.equal(map('payload.schema.json'), '/specs/acl/grant/0.1/payload.schema.json');
  assert.equal(map('../../_shared/0.2/notes.md'), '/specs/acl/_shared/0.2/notes.md');
});

test('a link out of the published trees goes to the file on GitHub', () => {
  assert.equal(map('../../../../trust-tasks-rs/README.md'), `${R.REPO_BLOB}/trust-tasks-rs/README.md`);
});

test('absolute URLs, fragments and other schemes are left alone', () => {
  for (const href of ['https://example.com/x', '#response', 'mailto:a@b.c', '//cdn.example/x', '']) {
    assert.equal(map(href), href);
  }
});

test('rewriteProseLinks rewrites hrefs and image sources in rendered HTML', () => {
  const html = '<p><a href="../../change-role/0.1/spec.md">x</a> <a href="#a">y</a> <img src="diagram.svg" alt=""></p>';
  assert.equal(
    R.rewriteProseLinks(html, GRANT, shared),
    '<p><a href="/spec/acl/change-role/0.1">x</a> <a href="#a">y</a> <img src="/specs/acl/grant/0.1/diagram.svg" alt=""></p>'
  );
});

test('isSpaPath tells routes from files', () => {
  for (const p of ['/spec/acl/grant/0.1', '/registry', '/specification', '/binding/https/0.2', '/']) assert.equal(R.isSpaPath(p), true, p);
  for (const p of ['/specs/acl/grant/0.1/spec.md', '/assets/x.js', '/SPEC.md', '/robots.txt', '/bindings/https/0.2/spec.md']) assert.equal(R.isSpaPath(p), false, p);
});

test('versions compare numerically, and latestPerSlug prefers a live version', () => {
  assert.ok(R.cmpVersion('0.10', '0.9') > 0);
  assert.ok(R.cmpVersion('1.0', '0.99') > 0);
  assert.equal(R.cmpVersion('0.5.0', '0.5'), 0);
  const tasks = [
    { slug: 'a', version: '0.1', status: 'draft' },
    { slug: 'a', version: '0.10', status: 'draft' },
    { slug: 'a', version: '0.11', status: 'retired' },
    { slug: 'b', version: '0.2', status: 'retired' },
  ];
  const latest = Object.fromEntries(R.latestPerSlug(tasks).map((t) => [t.slug, t.version]));
  assert.deepEqual(latest, { a: '0.10', b: '0.2' });
});
