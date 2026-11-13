// Tests for response-headers-policy.json, the CloudFront response headers
// policy apply-response-headers.sh attaches to the distribution.
//
// The policy cannot be exercised against CloudFront in CI, but two things can
// be checked here: that it sets every header it is meant to, and that its
// Content-Security-Policy still allows what the page actually loads. A page
// change that adds a script origin, an inline handler or a cross-origin fetch
// would otherwise only fail in visitors' browsers after the next deploy.
// (`npm run smoke:website` runs the built page under the same policy, through
// scripts/serve-website.mjs.)

import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it } from 'node:test';
import { cspDirectives, loadPolicy, responseHeaders } from './response-headers.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, '..', '..');
const template = readFileSync(join(repo, 'website', 'index.template.html'), 'utf8');
const policy = loadPolicy();
const csp = cspDirectives(policy);

/** Whether a CSP source list allows a URL (host-source with optional path
 *  prefix, `'self'` for a root-relative URL, or a bare scheme). */
function allows(sources, url) {
  if (url.startsWith('/') && !url.startsWith('//')) return sources.includes("'self'");
  return sources.some((s) => {
    if (/^[a-z][a-z0-9+.-]*:$/.test(s)) return url.startsWith(s);
    if (!/^https:\/\//.test(s)) return false;
    return s.endsWith('/') ? url.startsWith(s) : url === s || url.startsWith(`${s}/`);
  });
}

describe('response-headers-policy.json', () => {
  it('sets every security header, overriding the origin', () => {
    const sec = policy.SecurityHeadersConfig;
    for (const name of ['StrictTransportSecurity', 'ContentTypeOptions', 'FrameOptions', 'ReferrerPolicy', 'ContentSecurityPolicy']) {
      assert.ok(sec[name], `${name} is set`);
      assert.equal(sec[name].Override, true, `${name} overrides any origin value`);
    }
    assert.ok(sec.StrictTransportSecurity.AccessControlMaxAgeSec >= 31536000, 'HSTS lasts at least a year');
    assert.equal(sec.FrameOptions.FrameOption, 'DENY');
    const headers = responseHeaders(policy);
    assert.match(headers['Permissions-Policy'], /camera=\(\)/);
    assert.match(headers.Vary, /\bAccept\b/);
    assert.equal(headers['Access-Control-Allow-Origin'], '*');
  });

  it('keeps the Comment within CloudFront\'s 128 characters', () => {
    assert.ok(policy.Comment.length <= 128);
  });

  it('declares each header list\'s Quantity as its length', () => {
    for (const list of [policy.CustomHeadersConfig, ...Object.values(policy.CorsConfig).filter((v) => v && v.Items)]) {
      assert.equal(list.Quantity, list.Items.length);
    }
  });

  it('allows no eval, no inline script and no framing', () => {
    assert.ok(!policy.SecurityHeadersConfig.ContentSecurityPolicy.ContentSecurityPolicy.includes("'unsafe-eval'"));
    assert.ok(!csp['script-src'].includes("'unsafe-inline'"));
    assert.deepEqual(csp['frame-ancestors'], ["'none'"]);
    assert.deepEqual(csp['object-src'], ["'none'"]);
    assert.deepEqual(csp['base-uri'], ["'self'"]);
    assert.deepEqual(csp['connect-src'], ["'self'"]);
  });
});

describe('the page loads only what the Content-Security-Policy allows', () => {
  it('every <script src> in index.template.html', () => {
    const srcs = [...template.matchAll(/<script\b[^>]*\bsrc="([^"]+)"/g)].map((m) => m[1]);
    assert.ok(srcs.length > 0);
    for (const src of srcs) assert.ok(allows(csp['script-src'], src), `script-src allows ${src}`);
  });

  it('no inline <script> and no inline event handler', () => {
    const inline = [...template.matchAll(/<script\b([^>]*)>/g)].filter((m) => !/\bsrc=/.test(m[1]));
    assert.deepEqual(inline.map((m) => m[0]), []);
    assert.doesNotMatch(template, /<[^>]+\son[a-z]+\s*=/i, 'an on…= attribute would not run');
    assert.doesNotMatch(template, /href="javascript:/i);
  });

  it('every stylesheet and preconnect', () => {
    for (const m of template.matchAll(/<link\b[^>]*\brel="(stylesheet|preconnect)"[^>]*>/g)) {
      const href = /\bhref="([^"]+)"/.exec(m[0])[1];
      if (m[1] === 'stylesheet') {
        assert.ok(allows(csp['style-src'], href), `style-src allows ${href}`);
      } else {
        // A preconnect names the font CSS host or the font file host.
        assert.ok(allows(csp['style-src'], `${href}/`) || allows(csp['font-src'], `${href}/`), `${href} is an allowed origin`);
      }
    }
  });

  it('every fetch() in the page code is same-origin', () => {
    const dir = join(repo, 'website', 'assets');
    for (const f of readdirSync(dir).filter((n) => /\.jsx?$/.test(n) && !n.startsWith('tasks.generated'))) {
      const src = readFileSync(join(dir, f), 'utf8');
      for (const m of src.matchAll(/\bfetch\(\s*(["'`])([^"'`]*)/g)) {
        assert.ok(!/^(https?:)?\/\//.test(m[2]), `${f}: fetch(${m[2]}) is cross-origin; connect-src is 'self'`);
      }
    }
  });
});
