// Tests for the website's page code — website/index.template.html and
// assets/*.jsx, compiled by scripts/build-website.mjs into the bundle the page
// loads (#1147). A syntax error in pages.jsx used to be found by visitors, and
// a data shape the JSX did not expect blanked the whole site (#1153, #1157).
//
//   node --test scripts/test/website-jsx.test.mjs   (npm run test:website)
//
// 1. Compile: the page code compiles through the build's own esbuild step, and
//    the template loads React by pinned integrity hashes that match the files.
// 2. Render: after `npm run build` (skipped when tasks.generated.js or the
//    bundle is absent), the BUILT bundle that website/index.html names is
//    loaded with the real generated registry and the hand-kept data files, and
//    App is rendered to a string with React's server renderer for a set of
//    routes. A render exception, a missing global or a wrong load order fails
//    here rather than in a browser. tasks.generated.js is only the registry
//    index (#1145): the spec and schema pages fetch their detail from
//    website/specs/, which a server render cannot, so the test hands the page
//    a fetch cache (`ttFetched`) that reads those files from disk — and, with
//    an empty one, checks the loading state too.
//
// The render runs React's DEVELOPMENT build, not the production build the page
// loads, so React's own warnings (bad keys, invalid props) fail the test; its
// version is held to the page's. The CDN files are fetched like marked in
// website-markdown.test.mjs. To run offline, point REACT_JS and
// REACT_DOM_SERVER_JS at local copies of the development builds, and
// REACT_PROD_JS and REACT_DOM_PROD_JS at the production ones; the hashes are
// checked either way.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';
import { compileApp, JSX_FILES, BUNDLE_RE, HASHED_DIR, LOCAL_SCRIPT_RE, contentHash, PAGES_DIR, renderStaticPage } from '../build-website.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const website = path.join(here, '..', '..', 'website');
const assets = path.join(website, 'assets');
const template = fs.readFileSync(path.join(website, 'index.template.html'), 'utf8');

// Neither is loaded by the page, so neither has a pin there. They are pinned
// here, at the React version the page loads (checked below).
const REACT_DEV = {
  url: 'https://unpkg.com/react@18.3.1/umd/react.development.js',
  integrity: 'sha384-hD6/rw4ppMLGNu3tX5cjIb+uRZ7UkRJ6BPkLpg4hAu/6onKUg4lLsHAs9EBPT82L',
};
const REACT_DOM_SERVER = {
  url: 'https://unpkg.com/react-dom@18.3.1/umd/react-dom-server-legacy.browser.development.js',
  integrity: 'sha384-nXyubuGul9KsZKd274KDJE9YAWJPKFZ8O6kLSluMaAQlgKFE7AAUZcqGS/+Y2qyL',
};

function pinFromTemplate(re, what) {
  const m = re.exec(template);
  assert.ok(m, `index.template.html pins ${what} with an integrity hash`);
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

const reactPin = pinFromTemplate(/<script src="(https:\/\/[^"]*\/react@[^"]+\/umd\/react\.production\.min\.js)" integrity="(sha384-[^"]+)"/, 'production react');
const reactDomPin = pinFromTemplate(/<script src="(https:\/\/[^"]*\/react-dom@[^"]+\/umd\/react-dom\.production\.min\.js)" integrity="(sha384-[^"]+)"/, 'production react-dom');
const pageReactVersion = /\/react@([^/]+)\//.exec(reactPin.url)[1];

test('the page code compiles', () => {
  assert.doesNotThrow(() => compileApp());
});

test('every JSX file is in the bundle', () => {
  const onDisk = fs.readdirSync(assets).filter((f) => f.endsWith('.jsx')).sort();
  assert.deepEqual([...JSX_FILES].sort(), onDisk);
});

test('the page loads production React, pinned, and no in-browser compiler', async () => {
  assert.doesNotMatch(template, /text\/babel|@babel\/standalone|\.development\.js/);
  assert.match(reactDomPin.url, new RegExp(`/react-dom@${pageReactVersion}/`), 'react and react-dom are the same version');
  for (const pin of [REACT_DEV, REACT_DOM_SERVER]) {
    assert.match(pin.url, new RegExp(`@${pageReactVersion}/`), `${pin.url} is the React version the page loads`);
  }
  // The integrity attributes are what the browser checks: a wrong one blanks the site.
  await loadPinned(reactPin, 'REACT_PROD_JS');
  await loadPinned(reactDomPin, 'REACT_DOM_PROD_JS');
});

test('the template loads the data scripts, then the bundle', () => {
  // data.js must run before tasks.generated.js, whose TT_STATS counts its
  // categories, and every data script before the bundle that reads them.
  const order = ['routes.js', 'data.js', 'tasks.generated.js', 'ceremonies.js', '__APP_BUNDLE__'].map((f) => template.indexOf(`/assets/${f}`));
  assert.ok(order.every((i) => i > 0), 'each script is loaded');
  assert.deepEqual([...order].sort((a, b) => a - b), order, 'in that order');
  assert.match(template, /<script defer src="\/assets\/__APP_BUNDLE__"><\/script>/);
});

// ---------------------------------------------------------------------------
// Server-side render smoke test, against the built bundle
// ---------------------------------------------------------------------------

const builtIndex = path.join(website, 'index.html');
const builtHtml = fs.existsSync(builtIndex) ? fs.readFileSync(builtIndex, 'utf8') : '';
// The page's own scripts as the built index.html names them, in order.
const builtScripts = [...builtHtml.matchAll(/<script defer src="\/assets\/hashed\/([^"]+)"><\/script>/g)].map((m) => m[1]);
const bundleName = builtScripts.find((f) => BUNDLE_RE.test(f));
const haveBuild = fs.existsSync(path.join(assets, 'tasks.generated.js')) && !!bundleName && fs.existsSync(path.join(HASHED_DIR, bundleName));

test('the built index.html names content-hashed copies of every page script (#1148)', { skip: !haveBuild && 'run `npm run build` first' }, () => {
  // Every script the template names, in its order, and the bundle last.
  const templateScripts = [...template.matchAll(LOCAL_SCRIPT_RE)].map((m) => m[1]);
  assert.equal(builtScripts.length, templateScripts.length + 1, 'one hashed name per template script, and the bundle');
  assert.equal(builtScripts.at(-1), bundleName, 'the bundle runs last');
  assert.doesNotMatch(builtHtml, /<script defer src="\/assets\/(?!hashed\/)/, 'no page script is loaded by its unhashed name');
  assert.doesNotMatch(builtHtml, /__APP_BUNDLE__/);
  templateScripts.forEach((f, i) => {
    const bytes = fs.readFileSync(path.join(HASHED_DIR, builtScripts[i]));
    assert.equal(builtScripts[i], `${f.replace(/\.js$/, '')}.${contentHash(bytes)}.js`, `${f}'s copy is named by its content hash, so it can be cached as immutable`);
    assert.ok(bytes.equals(fs.readFileSync(path.join(assets, f))), `${builtScripts[i]} is current: rerun \`npm run build\``);
  });
  const bundle = fs.readFileSync(path.join(HASHED_DIR, bundleName));
  assert.equal(bundleName, `app.${contentHash(bundle)}.js`, 'the bundle is named by its content hash');
  assert.equal(bundle.toString('utf8'), compileApp(), 'the bundle is current: rerun `npm run build`');
  // deploy.yml uploads the whole of assets/hashed/ as immutable.
  assert.deepEqual(fs.readdirSync(HASHED_DIR).sort(), [...builtScripts].sort(), 'assets/hashed/ holds exactly what index.html names');
  assert.deepEqual(fs.readdirSync(assets).filter((f) => BUNDLE_RE.test(f)), [], 'no bundle is left in assets/ itself');
});

test('the built index.html carries the link-preview and canonical tags (#1151)', { skip: !haveBuild && 'run `npm run build` first' }, () => {
  assert.match(builtHtml, /<link rel="canonical" href="https:\/\/trusttasks\.org\/">/);
  for (const tag of ['property="og:title"', 'property="og:description"', 'property="og:url"', 'property="og:type"', 'property="og:site_name"', 'name="twitter:card"', 'name="twitter:title"', 'name="twitter:description"']) {
    assert.match(builtHtml, new RegExp(`<meta ${tag} content="[^"]+">`), tag);
  }
});

test('every spec version, binding, shared schema and ceremony has a static page with its own head (#1151)', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  const routeMeta = vm.runInContext('routeMeta', ctx);
  const unescape = (s) => s.replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
  const routes = [
    ...ctx.TT_TASKS.map((t) => `/spec/${t.slug}/${t.version}`),
    ...ctx.TT_BINDINGS.map((b) => `/binding/${b.slug}/${b.version}`),
    ...ctx.TT_SHARED.map((s) => `/schema/${s.slug}`),
    ...ctx.TT_CEREMONIES.map((c) => `/ceremony/${c.slug}/${c.version}`),
  ];
  const onDisk = [];
  (function walk(dir) {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      if (e.isDirectory()) walk(path.join(dir, e.name));
      else onDisk.push('/' + path.relative(PAGES_DIR, dir).split(path.sep).join('/'));
    }
  })(PAGES_DIR);
  assert.deepEqual(onDisk.sort(), [...routes].sort(), 'website/pages/ holds exactly one index.html per page route');
  // The rest of each page is index.html: the same scripts, so the app renders over it.
  const shell = (html) => html.replace(/<title>[^<]*<\/title>|<meta (name|property)="(description|og:url|og:title|og:description|twitter:title|twitter:description)" content="[^"]*">|<link rel="canonical" href="[^"]*">|<div class="tt-static">[\s\S]*?<\/div>|<!--tt-static-->/g, '');
  for (const route of routes) {
    const html = fs.readFileSync(path.join(PAGES_DIR, ...route.split('/').filter(Boolean), 'index.html'), 'utf8');
    ctx.location.pathname = route;
    const meta = routeMeta(ctx.__parsePath());
    const head = (re) => unescape((re.exec(html) || [])[1] || '');
    assert.equal(head(/<title>([^<]*)<\/title>/), meta.title, `${route}: the title is the app's`);
    assert.equal(head(/<meta property="og:title" content="([^"]*)">/), meta.title, `${route}: og:title`);
    if (meta.description) assert.equal(head(/<meta name="description" content="([^"]*)">/), meta.description, `${route}: the description is the app's`);
    assert.equal(head(/<link rel="canonical" href="([^"]*)">/), `https://trusttasks.org${route}`, `${route}: canonical`);
    assert.equal(head(/<meta property="og:url" content="([^"]*)">/), `https://trusttasks.org${route}`, `${route}: og:url`);
    assert.match(html, /<div class="tt-static">[\s\S]*?<h1>[^<]+<\/h1>/, `${route}: a heading in the static body`);
    assert.equal(shell(html), shell(builtHtml), `${route}: otherwise identical to index.html`);
  }
  // Text is escaped, never markup.
  const sample = renderStaticPage(builtHtml, { route: '/spec/x/0.1', title: '<script>"a"&</script>', description: '<b>', body: '' }, 'site');
  assert.match(sample, /<title>&lt;script&gt;&quot;a&quot;&amp;&lt;\/script&gt;<\/title>/);
  assert.doesNotMatch(sample, /<script>"a"/);
});

/** The page's fetch cache, answered from the built website/ tree: what the
 *  spec and schema pages would fetch, as though it had already arrived. */
function diskFetchCache() {
  const file = (url) => path.join(website, ...url.split('/').filter(Boolean));
  const ok = (url) => typeof url === 'string' && url.startsWith('/specs/') && !url.includes('..') && fs.existsSync(file(url));
  return {
    has: ok,
    get: (url) => (ok(url) ? JSON.parse(fs.readFileSync(file(url), 'utf8')) : undefined),
    set() {},
  };
}

/** A browser-ish global for the page scripts, with React and the data loaded.
 *  `fetched: false` starts the page with nothing fetched, as a browser does. */
async function pageContext({ fetched = true } = {}) {
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
    ttFetched: fetched ? diskFetchCache() : new Map(),
  };
  const meta = { getAttribute: () => 'site description', setAttribute() {} };
  ctx.document = {
    title: '',
    querySelector: () => meta,
    getElementById: () => null, // no #root, so the bundle does not mount
    body: { dataset: {} },
    documentElement: { style: { setProperty() {} } },
  };
  ctx.window = ctx;
  ctx.self = ctx;
  vm.createContext(ctx);
  vm.runInContext(await loadPinned(REACT_DEV, 'REACT_JS'), ctx, { filename: 'react.development.js' });
  assert.equal(ctx.React.version, pageReactVersion);
  vm.runInContext(await loadPinned(REACT_DOM_SERVER, 'REACT_DOM_SERVER_JS'), ctx, { filename: 'react-dom-server.js' });

  // The deferred scripts the built index.html names, in its order: the
  // hashed copies, then the bundle. markdown.js is skipped: it needs marked
  // and is only called from effects, which a server render does not run.
  assert.ok(builtScripts.some((f) => f.startsWith('tasks.generated.')), 'index.html loads tasks.generated.js');
  for (const f of builtScripts) {
    if (f.startsWith('markdown.')) continue;
    vm.runInContext(fs.readFileSync(path.join(HASHED_DIR, f), 'utf8'), ctx, { filename: f });
  }
  // The bundle's top-level declarations are globals, as they are in the page.
  ctx.__App = vm.runInContext('App', ctx);
  ctx.__parsePath = vm.runInContext('parsePath', ctx);
  assert.equal(typeof ctx.__App, 'function', 'the bundle defines App');
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
  const detail = (x) => ctx.ttFetched.get(`/specs/${x.slug}/${x.version}/detail.json`);
  const t = ctx.TT_TASKS.map(detail).find((x) => x.proofRequirement && typeof x.proofRequirement.requirement !== 'string' && x.proofRequirement.request);
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

test('the registry index is slim, and every version and shared schema has its detail file (#1145)', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext();
  for (const t of ctx.TT_TASKS) {
    for (const heavy of ['schema', 'errorCodes', 'proofRequirement', 'uses', 'partiesDetail']) {
      assert.equal(t[heavy], undefined, `${t.slug}/${t.version}: ${heavy} is detail, not index`);
    }
    const d = ctx.ttFetched.get(`/specs/${t.slug}/${t.version}/detail.json`);
    assert.ok(d, `${t.slug}/${t.version} has a detail.json`);
    assert.equal(d.slug, t.slug);
    assert.equal(d.version, t.version);
    assert.ok(d.schema && d.prosePath, `${t.slug}/${t.version}: detail.json is the full record`);
  }
  for (const s of ctx.TT_SHARED) {
    assert.equal(s.schema, undefined, `${s.slug}: schema is detail, not index`);
    const d = ctx.ttFetched.get(s.detailPath);
    assert.ok(d && d.schema, `${s.slug} has a detail file at ${s.detailPath}`);
    assert.equal(d.usedBy.length, s.usedByCount, `${s.slug}: the index counts the users its detail lists`);
  }
  assert.equal(ctx.TT_STATS.categories, ctx.TT_CATEGORIES.length);
});

test('a spec or schema page shows its heading and a loading state until its detail arrives', { skip: !haveBuild && 'run `npm run build` first' }, async () => {
  const ctx = await pageContext({ fetched: false });
  const spec = render(ctx, '/spec/acl/grant/0.1');
  assert.deepEqual(spec.errors, []);
  assert.match(spec.html, /<h1 class="tt-spec__title">[^<]+<\/h1>/);
  assert.match(spec.html, /data-tt-loading/);
  assert.doesNotMatch(spec.html, /id="metadata"/);
  const sharedSlug = ctx.TT_SHARED[0].slug;
  const schema = render(ctx, `/schema/${sharedSlug}`);
  assert.deepEqual(schema.errors, []);
  assert.match(schema.html, /data-tt-loading/);
  // With the detail in hand, the full page renders.
  const full = await pageContext();
  const done = render(full, '/spec/acl/grant/0.1');
  assert.match(done.html, /id="metadata"/);
  assert.doesNotMatch(done.html, /data-tt-loading=""[^>]*>Loading specification/);
  assert.match(render(full, `/schema/${sharedSlug}`).html, /id="used-by"/);
});
