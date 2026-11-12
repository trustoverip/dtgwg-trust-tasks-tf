#!/usr/bin/env node
/**
 * Local static server for ./website that routes requests exactly as production
 * does. Production is S3 behind CloudFront, with a viewer-request function —
 * infra/cloudfront/type-uri-negotiation.js — deciding what each request is: a
 * Type URI or ceremony definition negotiated to its JSON, a real file, a JSON
 * request that names nothing (a generated 404), or a client-side route (the
 * SPA shell). This server loads that same file and runs its `handler()` on
 * every request, then serves the file the handler chose — or a 404 if it does
 * not exist, as the origin would — so 404s and content negotiation behave in
 * the preview the way they do on trusttasks.org.
 *
 * Differences that remain: no compression, no Cache-Control, and the deploy's
 * gzip-encoded upload of tasks.generated.js is not reproduced.
 *
 *   node scripts/serve-website.mjs            # default port 8765
 *   node scripts/serve-website.mjs --port 3000
 *   node scripts/serve-website.mjs --host 0.0.0.0   # every IPv4 interface
 *   node scripts/serve-website.mjs --host ::        # every interface, IPv6 and
 *                                                   # (on most systems) IPv4
 *
 * Listens on 127.0.0.1 unless `--host` (or the `SERVE_WEBSITE_HOST` environment
 * variable) names another address.
 */
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ROOT = path.join(REPO, 'website');

// The CloudFront function declares a bare `function handler(event)` with no
// module system, so it is evaluated rather than imported — as its tests do.
const functionSource = fs.readFileSync(path.join(REPO, 'infra', 'cloudfront', 'type-uri-negotiation.js'), 'utf8');
const handler = new Function(`${functionSource}; return handler;`)();

const portArg = process.argv.indexOf('--port');
const PORT = portArg >= 0 ? Number(process.argv[portArg + 1]) : 8765;
const hostArg = process.argv.indexOf('--host');
const HOST = (hostArg >= 0 ? process.argv[hostArg + 1] : process.env.SERVE_WEBSITE_HOST) || '127.0.0.1';

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css':  'text/css; charset=utf-8',
  '.js':   'text/javascript; charset=utf-8',
  '.mjs':  'text/javascript; charset=utf-8',
  '.jsx':  'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.md':   'text/markdown; charset=utf-8',
  '.txt':  'text/plain; charset=utf-8',
  '.xml':  'application/xml; charset=utf-8',
  '.svg':  'image/svg+xml',
  '.png':  'image/png',
  '.jpg':  'image/jpeg',
  '.ico':  'image/x-icon',
  '.webmanifest': 'application/manifest+json'
};

function contentType(file) {
  // deploy.yml uploads schemas with the media type a Type URI is negotiated for.
  if (file.endsWith('.schema.json')) return 'application/schema+json';
  return TYPES[path.extname(file).toLowerCase()] || 'application/octet-stream';
}

function notFound(res) {
  res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
  res.end('not found');
}

const server = http.createServer((req, res) => {
  let url;
  try {
    url = new URL(req.url, 'http://localhost');
  } catch {
    res.writeHead(400); res.end('bad request'); return;
  }

  const request = {
    uri: url.pathname,
    headers: req.headers.accept ? { accept: { value: req.headers.accept } } : {},
  };
  const out = handler({ request });

  // A response the function generated itself (the JSON 404).
  if (out && out.statusCode) {
    const headers = {};
    for (const [k, v] of Object.entries(out.headers || {})) headers[k] = v.value;
    res.writeHead(out.statusCode, headers);
    const body = out.body && typeof out.body === 'object' ? out.body.data : out.body;
    res.end(body || '');
    return;
  }

  let rel;
  try {
    rel = decodeURIComponent(out.uri);
  } catch {
    res.writeHead(400); res.end('bad request'); return;
  }
  // Prevent path traversal.
  const filePath = path.join(ROOT, path.normalize(rel).replace(/^(\.\.[/\\])+/, ''));
  if (!filePath.startsWith(ROOT + path.sep) && filePath !== ROOT) return notFound(res);
  if (!fs.existsSync(filePath) || !fs.statSync(filePath).isFile()) return notFound(res);
  res.writeHead(200, { 'content-type': contentType(filePath) });
  fs.createReadStream(filePath).pipe(res);
});

server.listen(PORT, HOST, () => {
  // The address actually bound, so the log says what is reachable.
  const { address, family, port } = server.address();
  const base = `http://${family === 'IPv6' ? `[${address}]` : address}:${port}`;
  console.log(`Trust Tasks dev server: ${base}/`);
  console.log(`  Try:`);
  console.log(`    ${base}/spec/did-management/did/register/0.1`);
  console.log(`    ${base}/schema/did-management/_shared/0.1/did-record`);
  console.log(`    ${base}/schema`);
  console.log(`    curl -H 'Accept: application/schema+json' ${base}/spec/acl/grant/0.1`);
});
