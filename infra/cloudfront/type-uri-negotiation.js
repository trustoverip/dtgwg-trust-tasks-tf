// CloudFront Function (viewer-request) — Type URI content negotiation, and the
// single-page-app fallback.
//
// Two jobs, in one function because both are URI rewrites on the way in and the
// second depends on the first not having fired.
//
// ## 1. Content negotiation (SPEC §6.2)
//
// §6.2 requires a server hosting a Type URI to support content negotiation, and
// §7.2 items 1 and 2 — the two mandatory consumer validation steps — both obtain
// their schema by content-negotiating one. The schemas are deployed; they are
// just at a different URL from the one the specification hands implementers.
//
//   /spec/trust-task/<M.m>  + application/schema+json -> /specs/_framework/<M.m>/trust-task.schema.json
//   /spec/<slug…>/<M.m>     + application/schema+json -> /specs/<slug…>/<M.m>/payload.schema.json
//
// Fragments (#request, #response) never reach the server, so they need no
// handling: §4.4.1 publishes both variants in one schema document at the bare
// Type URI anyway.
//
// JSON-LD (application/ld+json, §7.3 item 10) is deliberately unhandled — no
// specification in the registry publishes a context yet. Add a branch when one
// does.
//
// ## 2. SPA fallback
//
// The distribution previously mapped 404 -> /index.html with a **200**, the
// conventional SPA catch-all. Fine for browsers, wrong for a schema consumer:
// an unknown or mistyped Type URI returned the site shell under a success
// status, so "fetch the schema and validate against it" quietly validated
// against nothing.
//
// That mapping could not be narrowed — `CustomErrorResponses` is
// distribution-wide with no per-cache-behaviour override — nor corrected on the
// way out, because viewer-response functions do not run for responses
// CloudFront generates itself, and a custom error page is exactly that. So the
// mapping is removed and the fallback lives here, where the rule is ours to
// scope.
//
// The usual "no file extension means an SPA route" heuristic fails in this
// registry: `/spec/acl/grant/0.1` ends in what looks like a `.1` extension. The
// rule is inverted instead — everything is an SPA route *except* the real asset
// trees and root files, which are precisely what should 404 when absent.
// Nothing can be missed, because the SPA is the default.
//
// Runtime note: CloudFront Functions run a constrained JS engine. Keep to ES5.1
// idioms — no optional chaining, no String.prototype.includes, no async.

// Real files on the origin. A request under one of these is left alone, so a
// missing object produces a genuine 404 rather than the SPA shell.
var ASSET_PREFIXES = ['/assets/', '/specs/', '/bindings/'];

// Files served from the root of website/. Must track that directory: a new root
// file added without updating this list is served the SPA shell instead of
// itself.
var ROOT_FILES = ['/index.html', '/registry.json', '/SPEC.md', '/_redirects', '/vercel.json'];

/**
 * Rewrite a Type URI to its schema when the client asked for one.
 * Returns true if the request was rewritten.
 */
function negotiateSchema(request) {
  // Only /spec/… paths name a Type URI. /specs/… is the raw asset tree; matching
  // it here would loop.
  if (request.uri.indexOf('/spec/') !== 0) {
    return false;
  }

  var headers = request.headers;
  var accept = headers && headers.accept ? headers.accept.value : '';
  if (accept.indexOf('application/schema+json') === -1) {
    return false;
  }

  // Tolerate a trailing slash: /spec/acl/grant/0.1/ behaves as /spec/acl/grant/0.1.
  var path = request.uri;
  if (path.length > 1 && path.charAt(path.length - 1) === '/') {
    path = path.substring(0, path.length - 1);
  }

  var segments = path.substring('/spec/'.length).split('/');
  if (segments.length < 2) {
    return false;
  }

  // Last segment is the MAJOR.MINOR version; everything before it is the slug.
  var version = segments[segments.length - 1];
  if (!/^[0-9]+\.[0-9]+$/.test(version)) {
    return false;
  }
  var slug = segments.slice(0, segments.length - 1).join('/');

  // `trust-task` is reserved for the framework (§6.1); its envelope schema lives
  // outside the task tree.
  if (slug === 'trust-task') {
    request.uri = '/specs/_framework/' + version + '/trust-task.schema.json';
    return true;
  }

  // Refuse anything that is not a plain slug — a stray `.` or `..` would
  // otherwise build a path escaping the specs tree.
  for (var i = 0; i < segments.length - 1; i++) {
    if (!/^[a-z][a-z0-9]*(-[a-z0-9]+)*$/.test(segments[i])) {
      return false;
    }
  }

  request.uri = '/specs/' + slug + '/' + version + '/payload.schema.json';
  return true;
}

/** Whether the URI addresses a real file on the origin. */
function isAsset(uri) {
  for (var i = 0; i < ASSET_PREFIXES.length; i++) {
    if (uri.indexOf(ASSET_PREFIXES[i]) === 0) {
      return true;
    }
  }
  for (var j = 0; j < ROOT_FILES.length; j++) {
    if (uri === ROOT_FILES[j]) {
      return true;
    }
  }
  return false;
}

function handler(event) {
  var request = event.request;

  // A negotiated schema path is an asset path; return before the fallback can
  // rewrite it back to the shell.
  if (negotiateSchema(request)) {
    return request;
  }

  // Real file: leave it alone so a missing object 404s honestly.
  if (isAsset(request.uri)) {
    return request;
  }

  // Everything else is a client-side route.
  request.uri = '/index.html';
  return request;
}
