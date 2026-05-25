#!/usr/bin/env node
/**
 * Tiny local static server for ./website with SPA fallback to /index.html.
 * Used to preview the registry, schema pages, and binding pages locally where
 * the production hosts (Netlify / Vercel) handle the rewrite for us.
 *
 *   node scripts/serve-website.mjs            # default port 8765
 *   node scripts/serve-website.mjs --port 3000
 */
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', 'website');

const portArg = process.argv.indexOf('--port');
const PORT = portArg >= 0 ? Number(process.argv[portArg + 1]) : 8765;

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css':  'text/css; charset=utf-8',
  '.js':   'text/javascript; charset=utf-8',
  '.mjs':  'text/javascript; charset=utf-8',
  '.jsx':  'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.md':   'text/markdown; charset=utf-8',
  '.svg':  'image/svg+xml',
  '.png':  'image/png',
  '.jpg':  'image/jpeg',
  '.ico':  'image/x-icon',
  '.webmanifest': 'application/manifest+json'
};

function sendFile(res, file) {
  const ext = path.extname(file).toLowerCase();
  res.writeHead(200, { 'content-type': TYPES[ext] || 'application/octet-stream' });
  fs.createReadStream(file).pipe(res);
}

const server = http.createServer((req, res) => {
  // Strip query string; decode percent-escapes.
  let urlPath;
  try {
    urlPath = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  } catch {
    res.writeHead(400); res.end('bad request'); return;
  }
  // Prevent path traversal.
  const safe = path.normalize(urlPath).replace(/^(\.\.[/\\])+/, '');
  let filePath = path.join(ROOT, safe);
  // Directory → index.html in that directory if present
  if (fs.existsSync(filePath) && fs.statSync(filePath).isDirectory()) {
    filePath = path.join(filePath, 'index.html');
  }
  if (fs.existsSync(filePath) && fs.statSync(filePath).isFile()) {
    sendFile(res, filePath);
    return;
  }
  // SPA fallback — any unmatched route serves the app shell so client-side
  // routing can take over (mirrors the Netlify _redirects / Vercel rewrites).
  const fallback = path.join(ROOT, 'index.html');
  if (fs.existsSync(fallback)) {
    sendFile(res, fallback);
    return;
  }
  res.writeHead(404); res.end('not found');
});

server.listen(PORT, () => {
  console.log(`Trust Tasks dev server: http://localhost:${PORT}/`);
  console.log(`  Try:`);
  console.log(`    http://localhost:${PORT}/spec/did-management/did/register/0.1`);
  console.log(`    http://localhost:${PORT}/schema/did-management/_shared/0.1/did-record`);
  console.log(`    http://localhost:${PORT}/schema`);
});
