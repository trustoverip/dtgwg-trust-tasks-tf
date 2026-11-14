// Rewrite-logic tests for the CloudFront viewer-request function.
//
// The function itself cannot be exercised in CI — it runs on CloudFront's own
// constrained engine, against a distribution this repo does not manage. What is
// testable, and worth testing, is the routing decision: which requests get
// rewritten, to what, and which are left alone. A mistake there either breaks
// the human-facing site or silently stops serving schemas at the URIs SPEC §6.2
// promises, and neither shows up until someone hits the live site.
//
// The function is loaded as source and evaluated rather than imported, because
// CloudFront Functions declare a bare `function handler(event)` with no module
// system — the same reason it cannot use ESM idioms.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
// `TUN_BUILT=1` runs every case against the build that is deployed
// (build-function.mjs: comments and whitespace stripped, under CloudFront's
// size cap) instead of the source; `npm run test:infra` runs both.
const source = process.env.TUN_BUILT
  ? (await import("./build-function.mjs")).buildFunction()
  : readFileSync(join(here, "type-uri-negotiation.js"), "utf8");
const handler = new Function(`${source}; return handler;`)();

const SCHEMA = "application/schema+json";
const JSON_CT = "application/json";
const HTML = "text/html";

/** Run a request through the function and return the resulting URI, or
 *  "404" when the function answered with a generated not-found response. */
function route(uri, accept, userAgent) {
  const headers = accept === undefined ? {} : { accept: { value: accept } };
  if (userAgent !== undefined) headers["user-agent"] = { value: userAgent };
  const out = handler({ request: { uri, headers } });
  return out.statusCode ? String(out.statusCode) : out.uri;
}
const NOT_FOUND = "404";

describe("rewrites a Type URI to its schema", () => {
  it("maps the framework envelope out of the task tree", () => {
    // `trust-task` is reserved for the framework (§6.1) and its envelope schema
    // lives under _framework/, not alongside a task.
    assert.equal(
      route("/spec/trust-task/0.2", SCHEMA),
      "/specs/_framework/0.2/trust-task.schema.json",
    );
  });

  it("serves the framework at its three-part Type URI", () => {
    // SPEC §5.1.1: the framework's Type URI is MAJOR.MINOR.PATCH. A `.0`
    // release is stored under MAJOR.MINOR, which is also its two-part alias.
    assert.equal(
      route("/spec/trust-task/0.5.0", SCHEMA),
      "/specs/_framework/0.5/trust-task.schema.json",
    );
    assert.equal(
      route("/spec/trust-task/0.6.0", SCHEMA),
      "/specs/_framework/0.6/trust-task.schema.json",
    );
  });

  it("never substitutes a different PATCH for the one requested", () => {
    // A PATCH release is its own envelope (SPEC §5.1.1): 0.6.1 must not be
    // answered with 0.6.0's schema.
    assert.equal(
      route("/spec/trust-task/0.6.1", SCHEMA),
      "/specs/_framework/0.6.1/trust-task.schema.json",
    );
  });

  it("rejects a three-part version on a task slug", () => {
    // Only the framework's version is three-part; every task slug is MAJOR.MINOR.
    assert.notEqual(
      route("/spec/acl/grant/0.1.0", SCHEMA),
      "/specs/acl/grant/0.1.0/payload.schema.json",
    );
  });

  it("maps a single-segment slug", () => {
    assert.equal(
      route("/spec/acl/grant/0.1", SCHEMA),
      "/specs/acl/grant/0.1/payload.schema.json",
    );
  });

  it("maps a multi-segment slug", () => {
    assert.equal(
      route("/spec/did-management/did/delete/0.1", SCHEMA),
      "/specs/did-management/did/delete/0.1/payload.schema.json",
    );
  });

  it("tolerates a trailing slash", () => {
    assert.equal(
      route("/spec/acl/grant/0.1/", SCHEMA),
      "/specs/acl/grant/0.1/payload.schema.json",
    );
  });

  it("handles a 1.0-style version as readily as 0.x", () => {
    assert.equal(
      route("/spec/vta/did-templates/create/2.0", SCHEMA),
      "/specs/vta/did-templates/create/2.0/payload.schema.json",
    );
  });
});

describe("rewrites a ceremony definition URI to its JSON", () => {
  it("maps /ceremony/<slug>/<M.m> into the ceremonies tree", () => {
    assert.equal(
      route("/ceremony/vtc/member-onboarding/0.1", JSON_CT),
      "/ceremonies/vtc/member-onboarding/0.1/ceremony.json",
    );
  });

  it("tolerates a trailing slash", () => {
    assert.equal(
      route("/ceremony/vtc/member-onboarding/0.1/", JSON_CT),
      "/ceremonies/vtc/member-onboarding/0.1/ceremony.json",
    );
  });

  it("serves the SPA to a browser", () => {
    // A person asking for the definition URI wants the page, not the document —
    // the same split /spec/ makes for Type URIs.
    assert.equal(route("/ceremony/vtc/member-onboarding/0.1", HTML), "/index.html");
  });

  it("leaves the raw ceremonies tree alone", () => {
    // /ceremonies/ is the asset tree. Rewriting it would loop; the singular and
    // plural forms are distinguishable only from the ninth character on.
    assert.equal(
      route("/ceremonies/vtc/member-onboarding/0.1/ceremony.json", JSON_CT),
      "/ceremonies/vtc/member-onboarding/0.1/ceremony.json",
    );
  });

  it("does not treat a Type URI as a definition", () => {
    // /spec/ and /ceremony/ are structurally disjoint (§6.7); a request for one
    // must never resolve into the other's tree. A JSON-only request for a Type
    // URI that is not a schema request names nothing, so it is a 404 — never
    // the ceremonies tree, and never the HTML shell.
    assert.equal(route("/spec/acl/grant/0.1", JSON_CT), NOT_FOUND);
  });

  it("refuses a slug segment that could escape the tree", () => {
    assert.equal(route("/ceremony/../secrets/0.1", JSON_CT), NOT_FOUND);
  });

  it("answers 404 for a path with no version segment", () => {
    assert.equal(route("/ceremony/vtc", JSON_CT), NOT_FOUND);
  });

  it("answers 404 for a definition URI asked for as a schema", () => {
    assert.equal(route("/ceremony/vtc/member-onboarding/0.1", SCHEMA), NOT_FOUND);
  });
});

describe("passes real files through untouched", () => {
  it("does not re-enter on the asset trees", () => {
    // /specs/ is where the schema rewrite points; rewriting again would loop.
    // These must also stay untouched so a missing object 404s honestly rather
    // than returning the SPA shell under a 200 — the bug this function exists
    // to fix.
    for (const uri of [
      "/specs/acl/grant/0.1/payload.schema.json",
      "/specs/_framework/0.2/trust-task.schema.json",
      "/bindings/didcomm/0.1/spec.md",
      "/assets/data.js",
    ]) {
      assert.equal(route(uri, SCHEMA), uri);
      assert.equal(route(uri, HTML), uri);
    }
  });

  it("passes root files through", () => {
    for (const uri of ["/index.html", "/registry.json", "/SPEC.md"]) {
      assert.equal(route(uri, HTML), uri);
    }
  });

  it("serves robots.txt and sitemap.xml as themselves", () => {
    // Generated by build-registry.mjs. Without a ROOT_FILES entry each would
    // be answered with the HTML shell, which is what crawlers saw (#1150).
    for (const uri of ["/robots.txt", "/sitemap.xml"]) {
      assert.equal(route(uri, undefined), uri);
      assert.equal(route(uri, "*/*"), uri);
      assert.equal(route(uri, HTML), uri);
    }
  });

  it("404s a missing asset rather than swallowing it", () => {
    // The point of the inverted rule: anything under an asset prefix is left
    // for the origin to answer, so a typo produces a real 404.
    const missing = "/specs/does/not/exist.schema.json";
    assert.equal(route(missing, SCHEMA), missing);
  });
});

describe("falls back to the SPA for client-side routes", () => {
  it("serves the shell for app routes", () => {
    for (const uri of ["/", "/categories", "/about", "/registry", "/specification"]) {
      assert.equal(route(uri, HTML), "/index.html");
    }
  });

  it("serves the shell for a Type URI viewed in a browser", () => {
    // No schema Accept, so this is a human reading prose — the SPA renders it.
    assert.equal(route("/spec/acl/grant/0.1", HTML), "/index.html");
    assert.equal(route("/spec/acl/grant/0.1", undefined), "/index.html");
  });

  it("serves the shell for an unknown Type URI in a browser", () => {
    // The SPA shows its own not-found UI. Only the schema request 404s.
    assert.equal(route("/spec/does-not-exist/0.1", HTML), "/index.html");
  });
});

describe("refuses to rewrite anything that is not a plain slug", () => {
  // None of these builds a path into the specs tree. A schema request gets a
  // 404 (#1152) — before, it got the HTML shell with a 200, and a consumer
  // "validated" against a web page. A browser asking for the same URL still
  // gets the SPA, which shows its own not-found view.
  it("rejects path traversal", () => {
    assert.equal(route("/spec/../../etc/0.1", SCHEMA), NOT_FOUND);
    assert.equal(route("/spec/acl/../../../x/0.1", SCHEMA), NOT_FOUND);
  });

  it("rejects segments outside the §6.1 slug grammar", () => {
    assert.equal(route("/spec/AcL/grant/0.1", SCHEMA), NOT_FOUND);
    assert.equal(route("/spec/acl_grant/0.1", SCHEMA), NOT_FOUND);
    assert.equal(route("/spec/-leading/0.1", SCHEMA), NOT_FOUND);
    assert.equal(route("/spec/Nope/0.1", SCHEMA), NOT_FOUND);
  });

  it("answers 404 for a path with no version segment", () => {
    assert.equal(route("/spec/acl/grant", SCHEMA), NOT_FOUND);
  });

  it("answers 404 for a three-part version on a task slug", () => {
    assert.equal(route("/spec/acl/grant/0.1.0", SCHEMA), NOT_FOUND);
  });

  it("still serves the SPA for the same URLs in a browser", () => {
    for (const uri of ["/spec/Nope/0.1", "/spec/acl/grant", "/ceremony/vtc"]) {
      assert.equal(route(uri, HTML), "/index.html");
      assert.equal(route(uri, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"), "/index.html");
      assert.equal(route(uri, undefined), "/index.html");
      assert.equal(route(uri, "*/*"), "/index.html");
    }
  });
});

describe("answers JSON-only requests that name nothing with a 404", () => {
  it("treats every JSON media type as JSON", () => {
    for (const accept of [JSON_CT, SCHEMA, "application/ld+json", "application/schema+json, */*;q=0.1"]) {
      assert.equal(route("/spec/acl/grant", accept), NOT_FOUND, accept);
    }
  });

  it("does not answer 404 when text/html is acceptable too", () => {
    assert.equal(route("/spec/acl/grant", "application/json, text/html"), "/index.html");
  });

  it("returns a JSON body with a 404 status", () => {
    const out = handler({ request: { uri: "/spec/acl/grant", headers: { accept: { value: SCHEMA } } } });
    assert.equal(out.statusCode, 404);
    assert.equal(out.headers["content-type"].value, "application/json");
    assert.equal(out.body.encoding, "text");
    assert.equal(JSON.parse(out.body.data).error, "not found");
  });

  it("leaves the asset trees to the origin", () => {
    // /specs/ and /ceremonies/ are plural: a missing file there is the
    // origin's own 404, not this one.
    assert.equal(route("/specs/nope/0.1/payload.schema.json", SCHEMA), "/specs/nope/0.1/payload.schema.json");
    assert.equal(route("/ceremonies/nope/0.1/ceremony.json", JSON_CT), "/ceremonies/nope/0.1/ceremony.json");
  });

  it("leaves non-registry routes to the SPA", () => {
    assert.equal(route("/registry", JSON_CT), "/index.html");
  });
});

describe("serves crawlers the static copy of a page (#1151)", () => {
  const BROWSER_ACCEPT = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
  const CHROME = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
  const CRAWLERS = [
    "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
    "Mozilla/5.0 (compatible; bingbot/2.0; +http://www.bing.com/bingbot.htm)",
    "Slackbot-LinkExpanding 1.0 (+https://api.slack.com/robots)",
    "Twitterbot/1.0",
    "facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)",
    "LinkedInBot/1.0 (compatible; Mozilla/5.0; Apache-HttpClient +http://www.linkedin.com)",
    "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)",
    "WhatsApp/2.23.20.0",
    "TelegramBot (like TwitterBot)",
    "http.rb/5.1.1 (Mastodon/4.2.0; +https://mastodon.social/)",
    "Mozilla/5.0 (compatible; Bluesky Cardyb/1.1; +mailto:support@bsky.app)",
  ];

  it("maps each kind of page to its copy under /pages/", () => {
    const ua = CRAWLERS[0];
    assert.equal(route("/spec/acl/grant/0.1", BROWSER_ACCEPT, ua), "/pages/spec/acl/grant/0.1/index.html");
    assert.equal(route("/spec/did-management/did/delete/0.1/", BROWSER_ACCEPT, ua), "/pages/spec/did-management/did/delete/0.1/index.html");
    assert.equal(route("/binding/https/0.3", BROWSER_ACCEPT, ua), "/pages/binding/https/0.3/index.html");
    assert.equal(route("/schema/acl/_shared/0.1/acl-entry", BROWSER_ACCEPT, ua), "/pages/schema/acl/_shared/0.1/acl-entry/index.html");
    assert.equal(route("/schema/trust-task/0.5.0", BROWSER_ACCEPT, ua), "/pages/schema/trust-task/0.5.0/index.html");
    assert.equal(route("/ceremony/vtc/member-onboarding/0.1", BROWSER_ACCEPT, ua), "/pages/ceremony/vtc/member-onboarding/0.1/index.html");
  });

  it("recognises the common crawlers and link unfurlers, with any Accept", () => {
    for (const ua of CRAWLERS) {
      for (const accept of [BROWSER_ACCEPT, "*/*", undefined]) {
        assert.equal(route("/spec/acl/grant/0.1", accept, ua), "/pages/spec/acl/grant/0.1/index.html", `${ua} / ${accept}`);
      }
    }
  });

  it("leaves browsers on the SPA shell, so an unknown route keeps its not-found page", () => {
    for (const uri of ["/spec/acl/grant/0.1", "/spec/does-not-exist/0.1", "/binding/https/0.3", "/schema/acl/_shared/0.1/acl-entry"]) {
      assert.equal(route(uri, BROWSER_ACCEPT, CHROME), "/index.html", uri);
      assert.equal(route(uri, BROWSER_ACCEPT), "/index.html", uri);
    }
  });

  it("still negotiates a schema or definition for a crawler that asks for one", () => {
    assert.equal(route("/spec/acl/grant/0.1", SCHEMA, CRAWLERS[0]), "/specs/acl/grant/0.1/payload.schema.json");
    assert.equal(route("/ceremony/vtc/member-onboarding/0.1", JSON_CT, CRAWLERS[0]), "/ceremonies/vtc/member-onboarding/0.1/ceremony.json");
    assert.equal(route("/spec/acl/grant", JSON_CT, CRAWLERS[0]), NOT_FOUND);
  });

  it("leaves routes with no static copy to the SPA", () => {
    const ua = CRAWLERS[0];
    for (const uri of ["/", "/registry", "/spec/acl/grant", "/spec/trust-task/0.5.0", "/binding/https", "/schema",
      "/ceremony/vtc", "/bindings", "/specification"]) {
      assert.equal(route(uri, BROWSER_ACCEPT, ua), "/index.html", uri);
    }
  });

  it("refuses traversal and characters outside the slug grammar", () => {
    const ua = CRAWLERS[0];
    for (const uri of ["/spec/../../etc/0.1", "/spec/acl/../x/0.1", "/schema/..", "/schema/acl/./x", "/spec/AcL/0.1", "/schema/a%2f..%2fb"]) {
      assert.equal(route(uri, BROWSER_ACCEPT, ua), "/index.html", uri);
    }
  });

  it("leaves the asset trees alone for crawlers too", () => {
    const ua = CRAWLERS[0];
    assert.equal(route("/specs/acl/grant/0.1/spec.md", "*/*", ua), "/specs/acl/grant/0.1/spec.md");
    assert.equal(route("/sitemap.xml", "*/*", ua), "/sitemap.xml");
  });
});
