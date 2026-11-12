// Tests for the website's page code — website/index.html and assets/*.jsx —
// which nothing else checks: the site has no build step, so a syntax error in
// pages.jsx was first found by visitors, and a data shape the JSX did not
// expect blanked the whole site (#1153, #1157).
//
//   node --test scripts/test/website-jsx.test.mjs   (npm run test:website)
//
// 1. Compile: every JSX file, and the inline <script type="text/babel"> in
//    index.html, through the @babel/standalone build index.html pins — the
//    compiler visitors actually run — refused unless its SHA-384 matches the
//    pinned integrity hash.
// 2. Render: after `npm run build` (skipped when tasks.generated.js is absent),
//    the app is rendered to a string with React's server renderer for a set of
//    routes, against the real generated registry and the hand-kept data files.
//    A render exception, a missing global or a wrong load order fails here
//    rather than in a browser.
//
// The CDN files are fetched like marked in website-markdown.test.mjs. To run
// offline, point BABEL_JS, REACT_JS and REACT_DOM_SERVER_JS at local copies;
// the hashes are checked either way.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const website = path.join(here, '..', '..', 'website');
const assets = path.join(website, 'assets');
const indexHtml = fs.readFileSync(path.join(website, 'index.html'), 'utf8');

// react-dom/server is not loaded by the page, so it has no pin there. It is
// pinned here, at the React version index.html loads (checked below).
const REACT_DOM_SERVER = {
  url: 'https://unpkg.com/react-dom@18.3.1/umd/react-dom-server-legacy.browser.development.js',
  integrity: 'sha384-nXyubuGul9KsZKd274KDJE9YAWJPKFZ8O6kLSluMaAQlgKFE7AAUZcqGS/+Y2qyL',
};

function pinFromIndex(re, what) {
  const m = re.exec(indexHtml);
  assert.ok(m, `index.html pins ${what} with an integrity hash`);
  return { url: m[1], integrity: m[2] };
}

async function loadPinned({ url, integrity }, envVar) {
  let bytes;
  if (process.env[envVar]) {
    bytes = fs.readFileSync(process.env[envVar]);
  } else {
    const res = await fetch(url);
    assert.ok(res.ok, `fetching ${url}: HTTP ${res.status}`);
    bytes = Buffer.from(await res.arrayBuffer());
  }
  const actual = 'sha384-' + crypto.createHash('sha384').update(bytes).digest('base64');
  assert.equal(actual, integrity, `${url} does not match its pinned integrity hash`);
  return bytes.toString('utf8');
}

const babelPin = pinFromIndex(/<script src="(https:\/\/[^"]*\/@babel\/standalone@[^"]+)" integrity="(sha384-[^"]+)"/, '@babel/standalone');
const reactPin = pinFromIndex(/<script src="(https:\/\/[^"]*\/react@[^"]+\/umd\/react\.development\.js)" integrity="(sha384-[^"]+)"/, 'react');

const babelCtx = { console };
vm.createContext(babelCtx);
vm.runInContext(await loadPinned(babelPin, 'BABEL_JS'), babelCtx);
const Babel = babelCtx.Babel;

/** Compile JSX as the page does; throws with a location on a syntax error. */
function compile(source, filename) {
  return Babel.transform(source, { presets: ['react'], filename, sourceType: 'script' }).code;
}

const INLINE_RE = /<script type="text\/babel" data-presets="react">([\s\S]*?)<\/script>/;
const jsxFiles = fs.readdirSync(assets).filter((f) => f.endsWith('.jsx')).sort();

test('every JSX file compiles', () => {
  assert.ok(jsxFiles.length > 0, 'found JSX files');
  for (const f of jsxFiles) {
    assert.doesNotThrow(() => compile(fs.readFileSync(path.join(assets, f), 'utf8'), f), `${f} compiles`);
  }
});

test("index.html's inline app script compiles", () => {
  const m = INLINE_RE.exec(indexHtml);
  assert.ok(m, 'index.html has an inline text/babel script');
  assert.doesNotThrow(() => compile(m[1], 'index.html'));
});

test('index.html loads every JSX file and the scripts the JSX needs', () => {
  for (const f of jsxFiles) {
    assert.match(indexHtml, new RegExp(`src="/assets/${f.replace('.', '\\.')}"`), `index.html loads ${f}`);
  }
  // routes.js must run before tasks.generated.js, whose TT_STATS footer uses it.
  const order = ['routes.js', 'data.js', 'tasks.generated.js'].map((f) => indexHtml.indexOf(`/assets/${f}`));
  assert.ok(order.every((i) => i > 0) && order[0] < order[1] && order[1] < order[2], 'routes.js, data.js, tasks.generated.js load in that order');
});

// ---------------------------------------------------------------------------
// Server-side render smoke test
// ---------------------------------------------------------------------------

const generated = path.join(assets, 'tasks.generated.js');
const haveBuild = fs.existsSync(generated);

/** A browser-ish global for the page scripts, with React and the data loaded. */
async function pageContext() {
  const ctx = {
    console: { log() {}, info() {}, warn() {}, debug() {}, error: (...a) => ctx.__errors.push(a.map(String).join(' ')) },
    __errors: [],
    URL, URLSearchParams, TextEncoder, TextDecoder, setTimeout, clearTimeout, queueMicrotask,
    requestAnimationFrame: () => 0,
    navigator: {},
    history: { pushState() {}, replaceState() {} },
    location: { pathname: '/', search: '', hash: '', origin: 'https://trusttasks.org' },
    fetch: () => new Promise(() => {}),
    IntersectionObserver: class { observe() {} disconnect() {} },
    addEventListener() {}, removeEventListener() {}, scrollTo() {},
  };
  const meta = { getAttribute: () => 'site description', setAttribute() {} };
  ctx.document = {
    title: '',
    querySelector: () => meta,
    getElementById: () => null,
    body: { dataset: {} },
    documentElement: { style: { setProperty() {} } },
  };
  ctx.window = ctx;
  ctx.self = ctx;
  vm.createContext(ctx);
  vm.runInContext(await loadPinned(reactPin, 'REACT_JS'), ctx, { filename: 'react.development.js' });
  assert.match(REACT_DOM_SERVER.url, new RegExp(`@${ctx.React.version}/`), 'react-dom/server pin matches the React version index.html loads');
  vm.runInContext(await loadPinned(REACT_DOM_SERVER, 'REACT_DOM_SERVER_JS'), ctx, { filename: 'react-dom-server.js' });

  // The deferred data scripts, in index.html's order (markdown.js needs marked
  // and is only called from effects, which a server render does not run).
  const scripts = [...indexHtml.matchAll(/<script defer src="\/assets\/([^"]+)"><\/script>/g)].map((m) => m[1]);
  assert.ok(scripts.includes('tasks.generated.js'), 'index.html loads tasks.generated.js');
  for (const f of scripts) {
    if (f === 'markdown.js') continue;
    vm.runInContext(fs.readFileSync(path.join(assets, f), 'utf8'), ctx, { filename: f });
  }
  for (const f of ['components.jsx', 'pages.jsx']) {
    vm.runInContext(compile(fs.readFileSync(path.join(assets, f), 'utf8'), f), ctx, { filename: f });
  }
  // The inline app, minus its mount: expose App instead.
  const inline = INLINE_RE.exec(indexHtml)[1].replace(
    /ReactDOM\.createRoot\([^;]*;/,
    'window.__App = App; window.__parsePath = parsePath;'
  );
  vm.runInContext(compile(inline, 'index.html'), ctx, { filename: 'index.html (inline)' });
  assert.equal(typeof ctx.__App, 'function', 'the inline script defines App');
  return ctx;
}

function render(ctx, url) {
  const u = new URL(url, 'https://trusttasks.org');
  ctx.location.pathname = u.pathname;
  ctx.location.search = u.search;
  ctx.location.hash = u.hash;
  ctx.__errors.length = 0;
  // `<!-- -->` is how the server renderer separates adjacent text nodes.
  const html = ctx.ReactDOMServer.renderToString(ctx.React.createElement(ctx.__App)).replace(/<!-- -->/g, '');
  // React reports render-time problems (bad keys, invalid props) through
  // console.error in development builds; treat them as failures.
  const errors = ctx.__errors.filter((e) => !/useLayoutEffect does nothing on the server/.test(e));
  return { html, errors };
}

test('the app renders every kind of route', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  const tasks = ctx.TT_TASKS;
  const firstShared = (ctx.TT_SHARED || [])[0];
  const routes = [
    '/', '/registry', '/registry/identity?q=grant&status=draft', '/registry?kw=acl',
    '/spec/acl/grant/0.1', '/spec/acl/grant', '/bindings', '/binding/tsp/0.1', '/binding/https',
    '/schema', firstShared ? `/schema/${firstShared.slug}` : '/schema', '/implementations',
    '/implementations/rust', '/implementations/go', '/implementations/dart', '/ceremonies',
    '/ceremony/vtc/member-onboarding/0.1', '/categories', '/ecosystem', '/about', '/contributing',
    '/glossary', '/specification', '/spec/does-not-exist/0.1', '/registy', '/nonexistent/route',
  ];
  for (const url of routes) {
    let out;
    assert.doesNotThrow(() => { out = render(ctx, url); }, `${url} renders`);
    assert.deepEqual(out.errors, [], `${url} renders without React errors`);
    assert.ok(out.html.length > 500, `${url} renders a page`);
  }
  assert.ok(tasks.length > 0, 'the generated registry has tasks');
});

test('unknown routes render the not-found view, not the home page', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  for (const url of ['/registy', '/nonexistent/route', '/about/extra']) {
    assert.match(render(ctx, url).html, /Page not found/, url);
  }
  assert.doesNotMatch(render(ctx, '/').html, /Page not found/);
});

test('a superseded version shows the lifecycle banner and its successor', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  const sup = ctx.TT_TASKS.find((t) => t.lifecycle && t.lifecycle.state === 'superseded');
  assert.ok(sup, 'the registry has a superseded version');
  const { html } = render(ctx, `/spec/${sup.slug}/${sup.version}`);
  assert.match(html, /tt-lifecycle--superseded/);
  const [, ver] = /\/(\d+\.\d+)$/.exec(sup.lifecycle.successor);
  assert.ok(html.includes(`v${ver}`), 'the banner names the successor version');
  const cur = ctx.TT_TASKS.find((t) => t.lifecycle && t.lifecycle.state === 'current');
  assert.doesNotMatch(render(ctx, `/spec/${cur.slug}/${cur.version}`).html, /tt-lifecycle/);
});

test('the Proof requirement row is filled for the per-variant form', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  const t = ctx.TT_TASKS.find((x) => x.proofRequirement && typeof x.proofRequirement.requirement !== 'string' && x.proofRequirement.request);
  assert.ok(t, 'the registry has a per-variant proofRequirement');
  const { html } = render(ctx, `/spec/${t.slug}/${t.version}`);
  const row = /<dt>Proof requirement<\/dt><dd>([^<]*)<\/dd>/.exec(html);
  assert.ok(row, 'the row renders');
  assert.match(row[1], /request /);
});

test('registry filters are read from the URL', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  assert.deepEqual(
    JSON.parse(JSON.stringify((() => { ctx.location.pathname = '/registry/identity'; ctx.location.search = '?q=grant&kw=acl&status=retired'; return ctx.__parsePath(); })())),
    { name: 'registry', cat: 'identity', q: 'grant', kw: 'acl', status: 'retired' }
  );
  const { html } = render(ctx, '/registry?q=grant');
  assert.match(html, /matching <b>“grant”<\/b>/);
  // Status chips come from the statuses present; the registry holds retired versions.
  if (ctx.TT_TASKS.some((t) => t.status === 'retired')) assert.match(html, /aria-pressed="false"[^>]*>.*?retired/s);
});
