// CloudFront Function (viewer-request) — content negotiation on Type URIs.
//
// SPEC.md §6.2 requires a server hosting a Type URI to support HTTP content
// negotiation, and §7.2 items 1 and 2 — the two mandatory consumer validation
// steps — both obtain their schema by content-negotiating a Type URI. Without
// this function every Type URI returns the SPA shell as text/html regardless of
// Accept, so both MUSTs are unimplementable against the published registry and
// §7.3 item 7.5 ("is served at its Type URI under content negotiation") is
// unmet by every specification.
//
// The schemas are already deployed; they are just at a different URL from the
// one the specification tells implementers to use. This maps one to the other.
//
//   /spec/trust-task/<M.m>   + application/schema+json -> /specs/_framework/<M.m>/trust-task.schema.json
//   /spec/<slug…>/<M.m>      + application/schema+json -> /specs/<slug…>/<M.m>/payload.schema.json
//   anything else                                      -> unchanged (the SPA renders prose)
//
// Fragments (#request, #response) never reach the server, so they need no
// handling: §4.4.1 publishes both variants in one schema document at the bare
// Type URI anyway.
//
// JSON-LD (application/ld+json, §7.3 item 10) is deliberately not handled yet —
// no specification in the registry publishes a context. Add a branch here when
// one does.
//
// Runtime note: CloudFront Functions run a constrained JS engine. Keep to ES5.1
// idioms — no optional chaining, no String.prototype.includes, no async.

function handler(event) {
  var request = event.request;
  var uri = request.uri;

  // Only /spec/... paths name a Type URI. /specs/... is the raw asset tree and
  // must pass through untouched, or the rewrite would loop.
  if (uri.indexOf('/spec/') !== 0) {
    return request;
  }

  var headers = request.headers;
  var accept = headers && headers.accept ? headers.accept.value : '';
  if (accept.indexOf('application/schema+json') === -1) {
    return request;
  }

  // Strip any trailing slash so /spec/acl/grant/0.1/ behaves as /spec/acl/grant/0.1.
  var path = uri;
  if (path.length > 1 && path.charAt(path.length - 1) === '/') {
    path = path.substring(0, path.length - 1);
  }

  var rest = path.substring('/spec/'.length);
  var segments = rest.split('/');
  if (segments.length < 2) {
    return request;
  }

  // The last segment is the MAJOR.MINOR version; everything before it is the slug.
  var version = segments[segments.length - 1];
  if (!/^[0-9]+\.[0-9]+$/.test(version)) {
    return request;
  }
  var slug = segments.slice(0, segments.length - 1).join('/');

  // `trust-task` is reserved for the framework itself (§6.1), and its envelope
  // schema lives outside the task tree.
  if (slug === 'trust-task') {
    request.uri = '/specs/_framework/' + version + '/trust-task.schema.json';
    return request;
  }

  // Reject anything that is not a plain slug path before rewriting — a stray
  // `.` or `..` would otherwise escape the specs tree.
  for (var i = 0; i < segments.length - 1; i++) {
    if (!/^[a-z][a-z0-9]*(-[a-z0-9]+)*$/.test(segments[i])) {
      return request;
    }
  }

  request.uri = '/specs/' + slug + '/' + version + '/payload.schema.json';
  return request;
}
