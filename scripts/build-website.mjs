#!/usr/bin/env node
/**
 * Compiles the website's page code into one minified, content-hashed bundle,
 * gives every other script the page loads a content-hashed copy too, and
 * writes `website/index.html` naming them (#1147, #1148).
 *
 * The page used to load `@babel/standalone` (3 MB) and React's development
 * builds from unpkg and compile ~240 KB of JSX on every visit. Now:
 *
 *   website/assets/{components,pages,app}.jsx
 *     → website/assets/hashed/app.<hash>.js    (esbuild: JSX → React.createElement, minified)
 *   website/assets/<name>.js, for every `<script defer src="/assets/<name>.js">`
 *   in the template — markdown.js, routes.js, the hand-kept data files and
 *   tasks.generated.js (the registry index; each spec's full record is
 *   fetched on demand from /specs/<slug>/<M.N>/detail.json, a fixed URL
 *   that is not hashed — #1145)
 *     → website/assets/hashed/<name>.<hash>.js (a byte-for-byte copy)
 *   website/index.template.html
 *     → website/index.html                     (names the hashed files)
 *
 * <hash> is the first 16 hex digits of the file's SHA-256, so a name is only
 * ever served with one content, and deploy.yml uploads `assets/hashed/` as
 * `immutable`. The unhashed sources stay where they are: the template names
 * them and the tests read them. Every output is .gitignore'd, and
 * `assets/hashed/` holds exactly what the current index.html names — the
 * build empties it first, since a stale file there would be uploaded too. React and ReactDOM stay UMD globals, loaded
 * from unpkg as production builds pinned by `integrity` in the template, as
 * `marked` is, so this script adds no runtime dependency and no network fetch.
 *
 * The three JSX files are classic scripts that share one global scope —
 * pages.jsx calls components.jsx's top-level functions, app.jsx calls both —
 * which is how the browser ran them when Babel compiled each into its own
 * <script>. So each is compiled on its own (an error names its file) without a
 * module format, which keeps their top-level names as they are, and the
 * results are concatenated in this order. Do not switch to `bundle: true`:
 * esbuild would wrap each file as a module and hide those names.
 *
 *   node scripts/build-website.mjs            (also run by `npm run build`)
 *
 * `compileApp()` is exported for scripts/test/website-jsx.test.mjs.
 */
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';
import { transformSync } from 'esbuild';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const WEBSITE_DIR = path.join(ROOT, 'website');
const ASSETS_DIR = path.join(WEBSITE_DIR, 'assets');
export const TEMPLATE = path.join(WEBSITE_DIR, 'index.template.html');

/** The page code, in the order it runs. Every .jsx in assets/ must be here. */
export const JSX_FILES = ['components.jsx', 'pages.jsx', 'app.jsx'];

const PLACEHOLDER = '/assets/__APP_BUNDLE__';
/** Where the content-hashed files go, and the URL prefix they are served at. */
export const HASHED_DIR = path.join(ASSETS_DIR, 'hashed');
export const HASHED_PREFIX = '/assets/hashed/';
export const BUNDLE_RE = /^app\.[0-9a-f]{16}\.js$/;
/** The page's own scripts in the template (the bundle placeholder aside). */
export const LOCAL_SCRIPT_RE = /<script defer src="\/assets\/([A-Za-z0-9_.-]+\.js)"><\/script>/g;

export const contentHash = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex').slice(0, 16);

/** `data.js` → `data.<hash>.js`. */
export const hashedName = (file, bytes) => `${file.replace(/\.js$/, '')}.${contentHash(bytes)}.js`;
const TEMPLATE_NOTE = /[ \t]*<!-- The template for index\.html[\s\S]*?-->\n/;

/** Compile the page code to one script. Throws on a syntax error, naming the
 * file and line. */
export function compileApp() {
  const onDisk = fs.readdirSync(ASSETS_DIR).filter((f) => f.endsWith('.jsx')).sort();
  const unlisted = onDisk.filter((f) => !JSX_FILES.includes(f));
  if (unlisted.length) {
    throw new Error(`website/assets/${unlisted.join(', ')}: not in JSX_FILES in scripts/build-website.mjs, so the page would never load it`);
  }
  const parts = JSX_FILES.map((f) => {
    const source = fs.readFileSync(path.join(ASSETS_DIR, f), 'utf8');
    const { code } = transformSync(source, {
      loader: 'jsx',
      jsx: 'transform', // React.createElement / React.Fragment, as Babel's preset-react did
      sourcefile: `website/assets/${f}`,
      minify: true,
      target: 'es2020',
      legalComments: 'none',
    });
    return code;
  });
  return parts.join('\n');
}

/** Build the bundle, the hashed copies and index.html. Returns the bundle's
 * file name. */
export function buildWebsite({ log = console.log } = {}) {
  const code = compileApp();
  const name = hashedName('app.js', code);

  const template = fs.readFileSync(TEMPLATE, 'utf8');
  if (template.split(PLACEHOLDER).length !== 2) {
    throw new Error(`website/index.template.html must name ${PLACEHOLDER} exactly once`);
  }
  const copies = [...template.matchAll(LOCAL_SCRIPT_RE)].map(([, f]) => {
    const src = path.join(ASSETS_DIR, f);
    if (!fs.existsSync(src)) {
      throw new Error(`website/assets/${f}: named by index.template.html but missing` +
        (f === 'tasks.generated.js' ? ' (`npm run build` generates it)' : ''));
    }
    const bytes = fs.readFileSync(src);
    return { f, bytes, hashed: hashedName(f, bytes) };
  });

  fs.rmSync(HASHED_DIR, { recursive: true, force: true });
  fs.mkdirSync(HASHED_DIR, { recursive: true });
  fs.writeFileSync(path.join(HASHED_DIR, name), code);
  for (const { hashed, bytes } of copies) fs.writeFileSync(path.join(HASHED_DIR, hashed), bytes);
  // The bundle's previous home (#1257), before it moved under hashed/.
  for (const f of fs.readdirSync(ASSETS_DIR)) {
    if (BUNDLE_RE.test(f)) fs.rmSync(path.join(ASSETS_DIR, f));
  }

  const names = Object.fromEntries(copies.map(({ f, hashed }) => [f, hashed]));
  const html = template
    .replace(TEMPLATE_NOTE, '  <!-- Generated from index.template.html by scripts/build-website.mjs. Edit the template. -->\n')
    .replace(PLACEHOLDER, `${HASHED_PREFIX}${name}`)
    .replace(LOCAL_SCRIPT_RE, (_, f) => `<script defer src="${HASHED_PREFIX}${names[f]}"></script>`);
  fs.writeFileSync(path.join(WEBSITE_DIR, 'index.html'), html);

  log(`Wrote website/assets/hashed/${name} (${(code.length / 1024).toFixed(0)} KB), ` +
    `content-hashed copies of ${copies.length} scripts, and website/index.html.`);
  writeStaticPages(html, { log });
  return name;
}

/* ============================================================
 * Static per-page copies of index.html (#1151)
 * ============================================================
 *
 * The page is a single-page app: a crawler or link unfurler that runs no
 * script sees index.html's site-wide title and a loading message for every
 * specification. So for each page with an identity of its own — every
 * specification version, binding, shared schema and ceremony the registry
 * lists — this writes a copy of the built index.html with that route's title,
 * description, canonical link and og:/twitter: tags filled in, and the
 * page's heading, summary and key facts as plain HTML in place of
 * `<!--tt-static-->` inside the boot message. The scripts are the same, so in
 * a browser the app renders over it as it does over index.html.
 *
 *   /spec/<slug>/<M.N>      → website/pages/spec/<slug>/<M.N>/index.html
 *   /binding/<slug>/<M.N>   → website/pages/binding/<slug>/<M.N>/index.html
 *   /schema/<slug…>         → website/pages/schema/<slug…>/index.html
 *   /ceremony/<slug>/<M.N>  → website/pages/ceremony/<slug>/<M.N>/index.html
 *
 * infra/cloudfront/type-uri-negotiation.js serves them to crawlers and link
 * unfurlers (STATIC_PREFIXES, CRAWLER_UA there). Written here, from the same
 * html as index.html and in the same run, so a page never names a bundle
 * that index.html does not; deploy.yml uploads pages/ in index.html's sync.
 *
 * The titles and descriptions are app.jsx's routeMeta() for the same route —
 * keep the two in step (website-jsx.test.mjs compares them). The data is read
 * from the scripts the page itself loads, so this needs no registry records
 * beyond what tasks.generated.js already holds. */
export const PAGES_DIR = path.join(WEBSITE_DIR, 'pages');
const SITE_ORIGIN = 'https://trusttasks.org';
const STATIC_MARKER = '<!--tt-static-->';

const esc = (s) => String(s == null ? '' : s)
  .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;').replace(/'/g, '&#39;');

/** The page's data scripts, evaluated as the browser would. */
export function loadPageData() {
  const ctx = { URL, console };
  ctx.window = ctx;
  vm.createContext(ctx);
  for (const f of ['routes.js', 'data.js', 'tasks.generated.js', 'bindings.js', 'ceremonies.js']) {
    vm.runInContext(fs.readFileSync(path.join(ASSETS_DIR, f), 'utf8'), ctx, { filename: f });
  }
  return ctx;
}

/** Every static page: its route, head values and body summary. */
export function staticPages(data = loadPageData()) {
  const t = (s) => `${s} · Trust Tasks`;
  const catName = (id) => ((data.TT_CATEGORIES || []).find((c) => c.id === id) || {}).name || id;
  const facts = (rows) => '<dl>' + rows.filter(([, v]) => v).map(([k, v]) => `<dt>${esc(k)}</dt><dd>${v}</dd>`).join('') + '</dl>';
  const links = (items) => '<ul>' + items.map(([href, label]) => `<li><a href="${esc(href)}">${esc(label)}</a></li>`).join('') + '</ul>';
  const crumbs = (items) => '<p>' + items.map(([href, label]) => `<a href="${esc(href)}">${esc(label)}</a>`).join(' / ') + '</p>';
  const pages = [];

  for (const s of data.TT_TASKS || []) {
    const route = `/spec/${s.slug}/${s.version}`;
    const succ = s.lifecycle && s.lifecycle.successor;
    const state = s.lifecycle && s.lifecycle.state && s.lifecycle.state !== 'current'
      ? `${esc(s.lifecycle.state)}${succ ? ` by <a href="/spec/${esc(succ)}">${esc(succ)}</a>` : ''}` : '';
    pages.push({
      route,
      title: t(`${s.title} (${s.slug} v${s.version})`),
      description: s.summary,
      body: crumbs([['/registry', 'Registry'], [`/registry/${s.category}`, catName(s.category)]]) +
        `<h1>${esc(s.title)}</h1><p>${esc(s.summary)}</p>` +
        facts([
          ['Type URI', `<code>${esc(SITE_ORIGIN + route)}</code>`],
          ['Version', esc(s.version)],
          ['Status', esc(s.status)],
          ['Lifecycle', state],
          ['Category', esc(catName(s.category))],
          ['Keywords', esc((s.keywords || []).join(', '))],
          ['Published', esc(s.created)],
          ['Updated', esc(s.updated)],
        ]) +
        links([
          [`/specs/${s.slug}/${s.version}/spec.md`, 'Specification (Markdown)'],
          [`/specs/${s.slug}/${s.version}/payload.schema.json`, 'Payload schema (JSON Schema)'],
        ]),
    });
  }
  for (const b of data.TT_BINDINGS || []) {
    pages.push({
      route: `/binding/${b.slug}/${b.version}`,
      title: t(`${b.title} binding v${b.version}`),
      description: b.summary,
      body: crumbs([['/bindings', 'Transport bindings']]) +
        `<h1>${esc(b.title)}</h1><p>${esc(b.summary)}</p>` +
        facts([
          ['Binding URI', `<code>${esc(b.bindingURI)}</code>`],
          ['Version', esc(b.version)],
          ['Status', esc(b.status)],
          ['Envelope', esc(b.envelopeType)],
        ]) +
        links([[b.prosePath, 'Binding specification (Markdown)']]),
    });
  }
  for (const r of data.TT_SHARED || []) {
    pages.push({
      route: `/schema/${r.slug}`,
      title: t(`${r.slug} — shared schema`),
      description: r.description,
      body: crumbs([['/schema', 'Schemas']]) +
        `<h1>${esc(r.title)}</h1><p>${esc(r.description)}</p>` +
        facts([
          ['Schema ID', `<code>${esc(r.schemaId)}</code>`],
          ['Kind', esc(r.kind)],
          ['Used by', esc(`${r.usedByCount} specification version${r.usedByCount === 1 ? '' : 's'}`)],
        ]) +
        links([[r.sourcePath, 'Schema (JSON)']]),
    });
  }
  for (const c of data.TT_CEREMONIES || []) {
    pages.push({
      route: `/ceremony/${c.slug}/${c.version}`,
      title: t(`${c.title} ceremony v${c.version}`),
      description: c.summary,
      body: crumbs([['/ceremonies', 'Ceremonies']]) +
        `<h1>${esc(c.title)}</h1><p>${esc(c.summary)}</p>` +
        facts([
          ['Definition URI', `<code>${esc(c.definitionURI)}</code>`],
          ['Version', esc(c.version)],
          ['Status', esc(c.status)],
        ]) +
        links([[c.definitionPath, 'Ceremony definition (JSON)']]),
    });
  }
  return pages;
}

/** index.html with one route's head values and summary. Throws when the
 * built page lacks a tag this fills, rather than writing pages that are
 * silently the home page's. */
export function renderStaticPage(html, page, siteDescription) {
  const url = SITE_ORIGIN + page.route;
  const description = page.description || siteDescription;
  const swaps = [
    [/<title>[^<]*<\/title>/, `<title>${esc(page.title)}</title>`],
    [/<meta name="description" content="[^"]*">/, `<meta name="description" content="${esc(description)}">`],
    [/<link rel="canonical" href="[^"]*">/, `<link rel="canonical" href="${esc(url)}">`],
    [/<meta property="og:url" content="[^"]*">/, `<meta property="og:url" content="${esc(url)}">`],
    [/<meta property="og:title" content="[^"]*">/, `<meta property="og:title" content="${esc(page.title)}">`],
    [/<meta property="og:description" content="[^"]*">/, `<meta property="og:description" content="${esc(description)}">`],
    [/<meta name="twitter:title" content="[^"]*">/, `<meta name="twitter:title" content="${esc(page.title)}">`],
    [/<meta name="twitter:description" content="[^"]*">/, `<meta name="twitter:description" content="${esc(description)}">`],
  ];
  let out = html;
  for (const [re, value] of swaps) {
    if (!re.test(out)) throw new Error(`website/index.html has no ${re.source}; the static pages cannot fill it`);
    out = out.replace(re, () => value);
  }
  if (out.split(STATIC_MARKER).length !== 2) throw new Error(`website/index.html must hold ${STATIC_MARKER} exactly once`);
  return out.replace(STATIC_MARKER, () => `<div class="tt-static">${page.body}</div>`);
}

/** Write every static page under website/pages/, emptied first so a page
 * whose route is gone does not linger (and get uploaded). */
export function writeStaticPages(html, { log = console.log } = {}) {
  const started = Date.now();
  const siteDescription = ((/<meta name="description" content="([^"]*)">/.exec(html) || [])[1] || '')
    .replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
  const pages = staticPages();
  fs.rmSync(PAGES_DIR, { recursive: true, force: true });
  let bytes = 0;
  for (const page of pages) {
    const body = renderStaticPage(html, page, siteDescription);
    const file = path.join(PAGES_DIR, ...page.route.split('/').filter(Boolean), 'index.html');
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, body);
    bytes += Buffer.byteLength(body);
  }
  log(`Wrote ${pages.length} static pages under website/pages/ (${(bytes / 1e6).toFixed(1)} MB, ${Date.now() - started} ms).`);
  return pages.length;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    buildWebsite();
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}
