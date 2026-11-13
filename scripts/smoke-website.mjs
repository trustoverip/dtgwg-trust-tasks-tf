#!/usr/bin/env node
/**
 * Headless-browser smoke test of the built website (#1157).
 *
 * Serves ./website with scripts/serve-website.mjs — which routes requests
 * through the production CloudFront function — loads a handful of routes in
 * headless Chrome, and fails on any of:
 *
 *   - an uncaught exception, or a console.error / console.assert;
 *   - a browser log entry at level "error" (a blocked or failed resource,
 *     an integrity mismatch, a CSP violation);
 *   - a request that failed, or answered with HTTP 400 or above;
 *   - a page with no <h1>, a page still showing the static boot message, or
 *     a page that rendered the not-found view;
 *   - a page still loading, or showing a failed fetch, once it has settled —
 *     the spec and schema pages fetch their detail on demand (#1145);
 *   - a canonical link that is not https://trusttasks.org + the route (#1151).
 *
 * website-jsx.test.mjs already renders every route on the server; what this
 * adds is the real page: the built index.html, its hashed script URLs, the
 * pinned CDN scripts and their integrity hashes, the server's routing, and
 * the effects a server render never runs (prose fetches, the head tags).
 *
 * No npm dependency: it drives Chrome over the DevTools protocol with the
 * WebSocket client built into Node (22 and later). The browser is the first
 * of $CHROME_PATH, google-chrome, google-chrome-stable, chromium,
 * chromium-browser on PATH, or the macOS Chrome application. CI's Ubuntu
 * runners have Chrome preinstalled.
 *
 *   npm run build && npm run smoke:website
 *   CHROME_PATH=/path/to/chrome npm run smoke:website
 *
 * The page loads React and marked from unpkg.com and fonts from Google, so
 * this needs network access.
 */
import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const SITE_ORIGIN = 'https://trusttasks.org';

/** The routes to load. Each must render a page of its own, not the not-found view. */
export const ROUTES = [
  '/',
  '/spec/acl/grant/0.3',
  '/binding/https/0.3',
  '/schema',
  '/schema/acl/_shared/0.1/acl-entry',
  '/implementations/rust',
];

const PAGE_TIMEOUT_MS = 30_000;
// After the <h1> appears, how long to keep listening for late failures — the
// prose and schema fetches a page starts from its effects.
const SETTLE_MS = 2_000;

function findChrome() {
  if (process.env.CHROME_PATH) return process.env.CHROME_PATH;
  for (const name of ['google-chrome', 'google-chrome-stable', 'chromium', 'chromium-browser']) {
    const r = spawnSync('which', [name], { encoding: 'utf8' });
    if (r.status === 0 && r.stdout.trim()) return r.stdout.trim();
  }
  const mac = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  if (fs.existsSync(mac)) return mac;
  return null;
}

/** Start a child process and resolve with the first match of `re` on its output. */
function startAndWaitFor(cmd, args, re, what) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let out = '';
    const timer = setTimeout(() => reject(new Error(`${what} did not start within 30s:\n${out}`)), 30_000);
    const onData = (d) => {
      out += d;
      const m = re.exec(out);
      if (m) {
        clearTimeout(timer);
        resolve({ child, match: m });
      }
    };
    child.stdout.on('data', onData);
    child.stderr.on('data', onData);
    child.on('error', (e) => { clearTimeout(timer); reject(e); });
    child.on('exit', (code) => { clearTimeout(timer); reject(new Error(`${what} exited (${code}) before it was ready:\n${out}`)); });
  });
}

/** A minimal DevTools protocol client over one browser-level WebSocket. */
class Cdp {
  constructor(ws) {
    this.ws = ws;
    this.nextId = 0;
    this.pending = new Map();
    this.listeners = new Set();
    ws.addEventListener('message', (e) => {
      const msg = JSON.parse(e.data);
      if (msg.id && this.pending.has(msg.id)) {
        const { resolve, reject, method } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        if (msg.error) reject(new Error(`${method}: ${msg.error.message}`));
        else resolve(msg.result);
      } else if (msg.method) {
        for (const l of this.listeners) l(msg);
      }
    });
  }

  static connect(url) {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(url);
      ws.addEventListener('open', () => resolve(new Cdp(ws)));
      ws.addEventListener('error', () => reject(new Error(`could not connect to ${url}`)));
    });
  }

  send(method, params = {}, sessionId) {
    const id = ++this.nextId;
    this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    return new Promise((resolve, reject) => this.pending.set(id, { resolve, reject, method }));
  }

  close() { this.ws.close(); }
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Load one route in a fresh tab and return the problems found. */
async function checkRoute(cdp, base, route) {
  const problems = [];
  const { targetId } = await cdp.send('Target.createTarget', { url: 'about:blank' });
  const { sessionId } = await cdp.send('Target.attachToTarget', { targetId, flatten: true });
  const urls = new Map(); // requestId → url, for loadingFailed
  const listener = (msg) => {
    if (msg.sessionId !== sessionId) return;
    const p = msg.params;
    switch (msg.method) {
      case 'Runtime.exceptionThrown': {
        const d = p.exceptionDetails;
        problems.push(`uncaught exception: ${(d.exception && d.exception.description) || d.text}`);
        break;
      }
      case 'Runtime.consoleAPICalled':
        if (p.type === 'error' || p.type === 'assert') {
          problems.push(`console.${p.type}: ${p.args.map((a) => a.value ?? a.description ?? '').join(' ')}`);
        }
        break;
      case 'Log.entryAdded':
        if (p.entry.level === 'error') problems.push(`browser error: ${p.entry.text}${p.entry.url ? ` (${p.entry.url})` : ''}`);
        break;
      case 'Network.requestWillBeSent':
        urls.set(p.requestId, p.request.url);
        break;
      case 'Network.responseReceived':
        if (p.response.status >= 400) problems.push(`HTTP ${p.response.status}: ${p.response.url}`);
        break;
      case 'Network.loadingFailed':
        problems.push(`request failed (${p.errorText}${p.blockedReason ? `, ${p.blockedReason}` : ''}): ${urls.get(p.requestId) || p.requestId}`);
        break;
    }
  };
  cdp.listeners.add(listener);
  try {
    for (const domain of ['Runtime', 'Log', 'Network', 'Page']) await cdp.send(`${domain}.enable`, {}, sessionId);
    const nav = await cdp.send('Page.navigate', { url: base + route }, sessionId);
    if (nav.errorText) problems.push(`navigation failed: ${nav.errorText}`);

    const deadline = Date.now() + PAGE_TIMEOUT_MS;
    let state;
    for (;;) {
      const { result } = await cdp.send('Runtime.evaluate', {
        expression: `(() => {
          const h1 = document.querySelector('#main h1') || document.querySelector('h1');
          const canonical = document.querySelector('link[rel="canonical"]');
          return {
            ready: document.readyState,
            h1: h1 ? h1.textContent.trim() : null,
            boot: !!document.querySelector('.tt-boot'),
            title: document.title,
            canonical: canonical ? canonical.getAttribute('href') : null,
          };
        })()`,
        returnByValue: true,
      }, sessionId);
      state = result.value || {};
      if (state.ready === 'complete' && state.h1) break;
      if (Date.now() > deadline) break;
      await sleep(200);
    }
    if (state.h1) await sleep(SETTLE_MS);
    const { result } = await cdp.send('Runtime.evaluate', {
      expression: `({
        title: document.title,
        canonical: (document.querySelector('link[rel="canonical"]') || {}).href || null,
        loading: !!document.querySelector('#main [data-tt-loading]'),
        fetchError: (document.querySelector('#main [data-tt-fetch-error]') || {}).textContent || null,
      })`,
      returnByValue: true,
    }, sessionId);
    const final = result.value || {};

    if (!state.h1) problems.push(`no <h1> within ${PAGE_TIMEOUT_MS / 1000}s${state.boot ? ' (the static boot message is still showing: the app never rendered)' : ''}`);
    else if (state.boot) problems.push('the static boot message is still showing');
    if (/not found/i.test(final.title)) problems.push(`rendered the not-found view (title "${final.title}")`);
    if (final.fetchError) problems.push(`shows a failed fetch: ${final.fetchError.trim()}`);
    else if (state.h1 && final.loading) problems.push(`still loading ${SETTLE_MS / 1000}s after its <h1> appeared`);
    const wantCanonical = SITE_ORIGIN + route;
    if (final.canonical !== wantCanonical) problems.push(`canonical link is ${final.canonical}, expected ${wantCanonical}`);
    return { problems, h1: state.h1, title: final.title };
  } finally {
    cdp.listeners.delete(listener);
    await cdp.send('Target.closeTarget', { targetId }).catch(() => {});
  }
}

async function main() {
  if (!fs.existsSync(path.join(ROOT, 'website', 'index.html'))) {
    throw new Error('website/index.html is missing: run `npm run build` first');
  }
  if (typeof WebSocket !== 'function') throw new Error(`Node ${process.version} has no built-in WebSocket; use Node 22 or later`);
  const chrome = findChrome();
  if (!chrome) throw new Error('no Chrome or Chromium found: set CHROME_PATH');

  const children = [];
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'tt-smoke-chrome-'));
  try {
    const server = await startAndWaitFor(process.execPath, [path.join(ROOT, 'scripts', 'serve-website.mjs'), '--port', '0'],
      /Trust Tasks dev server: (http:\/\/\S+?)\/\s/, 'the website server');
    children.push(server.child);
    const base = server.match[1];

    const args = [
      '--headless=new', '--remote-debugging-port=0', `--user-data-dir=${profile}`,
      '--no-first-run', '--no-default-browser-check', '--disable-extensions', '--disable-gpu',
      '--disable-background-networking', '--disable-component-update', '--disable-sync',
    ];
    // GitHub's Ubuntu 24.04 runners restrict the unprivileged user namespaces
    // Chrome's sandbox needs. The browser only loads this repository's site.
    if (process.env.CI) args.push('--no-sandbox');
    args.push('about:blank');
    const browser = await startAndWaitFor(chrome, args, /DevTools listening on (ws:\/\/\S+)/, 'Chrome');
    children.push(browser.child);

    const cdp = await Cdp.connect(browser.match[1]);
    let failed = 0;
    console.log(`Smoke-testing ${base} with ${chrome}`);
    for (const route of ROUTES) {
      const { problems, h1, title } = await checkRoute(cdp, base, route);
      if (problems.length) {
        failed++;
        console.log(`✗ ${route}`);
        for (const p of problems) console.log(`    ${p}`);
      } else {
        console.log(`✓ ${route} — h1 "${h1}", title "${title}"`);
      }
    }
    await cdp.send('Browser.close').catch(() => {});
    cdp.close();
    if (failed) {
      console.error(`\n${failed} of ${ROUTES.length} routes failed the smoke test.`);
      process.exitCode = 1;
    }
  } finally {
    for (const c of children) if (c.exitCode === null) c.kill();
    fs.rmSync(profile, { recursive: true, force: true, maxRetries: 3 });
  }
}

main().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
