#!/usr/bin/env node
/**
 * Compiles the website's page code into one minified, content-hashed bundle
 * and writes `website/index.html` naming it (#1147).
 *
 * The page used to load `@babel/standalone` (3 MB) and React's development
 * builds from unpkg and compile ~240 KB of JSX on every visit. Now:
 *
 *   website/assets/{components,pages,app}.jsx
 *     → website/assets/app.<hash>.js      (esbuild: JSX → React.createElement, minified)
 *   website/index.template.html
 *     → website/index.html                (names that bundle)
 *
 * Both outputs are .gitignore'd. React and ReactDOM stay UMD globals, loaded
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
import { fileURLToPath } from 'node:url';
import { transformSync } from 'esbuild';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const WEBSITE_DIR = path.join(ROOT, 'website');
const ASSETS_DIR = path.join(WEBSITE_DIR, 'assets');
export const TEMPLATE = path.join(WEBSITE_DIR, 'index.template.html');

/** The page code, in the order it runs. Every .jsx in assets/ must be here. */
export const JSX_FILES = ['components.jsx', 'pages.jsx', 'app.jsx'];

const PLACEHOLDER = '/assets/__APP_BUNDLE__';
export const BUNDLE_RE = /^app\.[0-9a-f]{16}\.js$/;
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

/** Build the bundle and index.html. Returns the bundle's file name. */
export function buildWebsite({ log = console.log } = {}) {
  const code = compileApp();
  const hash = crypto.createHash('sha256').update(code).digest('hex').slice(0, 16);
  const name = `app.${hash}.js`;

  for (const f of fs.readdirSync(ASSETS_DIR)) {
    if (BUNDLE_RE.test(f) && f !== name) fs.rmSync(path.join(ASSETS_DIR, f));
  }
  fs.writeFileSync(path.join(ASSETS_DIR, name), code);

  const template = fs.readFileSync(TEMPLATE, 'utf8');
  if (template.split(PLACEHOLDER).length !== 2) {
    throw new Error(`website/index.template.html must name ${PLACEHOLDER} exactly once`);
  }
  const html = template
    .replace(TEMPLATE_NOTE, '  <!-- Generated from index.template.html by scripts/build-website.mjs. Edit the template. -->\n')
    .replace(PLACEHOLDER, `/assets/${name}`);
  fs.writeFileSync(path.join(WEBSITE_DIR, 'index.html'), html);

  log(`Wrote website/assets/${name} (${(code.length / 1024).toFixed(0)} KB) and website/index.html.`);
  return name;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    buildWebsite();
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}
