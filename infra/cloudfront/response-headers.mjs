// The response headers response-headers-policy.json makes CloudFront add, as
// a plain { name: value } map — so scripts/serve-website.mjs can send the same
// headers locally (and the headless smoke test runs the page under the real
// Content-Security-Policy), and the policy's tests can read it.
//
// Strict-Transport-Security is left out with `{ https: false }`: a browser
// ignores it over plain HTTP, which is all the local server speaks.

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const POLICY_FILE = join(dirname(fileURLToPath(import.meta.url)), 'response-headers-policy.json');

export function loadPolicy() {
  return JSON.parse(readFileSync(POLICY_FILE, 'utf8'));
}

export function responseHeaders(policy = loadPolicy(), { https = true } = {}) {
  const out = {};
  const sec = policy.SecurityHeadersConfig || {};
  if (https && sec.StrictTransportSecurity) {
    const h = sec.StrictTransportSecurity;
    out['Strict-Transport-Security'] = `max-age=${h.AccessControlMaxAgeSec}`
      + (h.IncludeSubdomains ? '; includeSubDomains' : '')
      + (h.Preload ? '; preload' : '');
  }
  if (sec.ContentTypeOptions) out['X-Content-Type-Options'] = 'nosniff';
  if (sec.FrameOptions) out['X-Frame-Options'] = sec.FrameOptions.FrameOption;
  if (sec.ReferrerPolicy) out['Referrer-Policy'] = sec.ReferrerPolicy.ReferrerPolicy;
  if (sec.ContentSecurityPolicy) out['Content-Security-Policy'] = sec.ContentSecurityPolicy.ContentSecurityPolicy;
  const cors = policy.CorsConfig;
  if (cors) out['Access-Control-Allow-Origin'] = cors.AccessControlAllowOrigins.Items.join(', ');
  for (const item of (policy.CustomHeadersConfig && policy.CustomHeadersConfig.Items) || []) {
    out[item.Header] = item.Value;
  }
  return out;
}

/** The policy's Content-Security-Policy as { directive: [source, …] }. */
export function cspDirectives(policy = loadPolicy()) {
  const csp = policy.SecurityHeadersConfig.ContentSecurityPolicy.ContentSecurityPolicy;
  const out = {};
  for (const part of csp.split(';')) {
    const [name, ...sources] = part.trim().split(/\s+/);
    if (name) out[name] = sources;
  }
  return out;
}
