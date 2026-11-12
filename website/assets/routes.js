/* ============================================================
   Trust Tasks — routing and version helpers (plain script)
   ------------------------------------------------------------
   Loaded before tasks.generated.js, whose TT_STATS footer uses
   `latestPerSlug`, and before the JSX, which uses all of it.
   A plain script rather than JSX so that Node can load it too:
   scripts/build-registry.mjs runs `resolveProseHref` over every
   relative link in spec and binding prose, and the website tests
   exercise it, so the build checks links from the location they
   are rendered at rather than from the file they are written in.

   Before this file, the version comparison was written inline in
   eight places, and prose links were resolved by the browser
   against the page URL (/spec/acl/grant/0.1) rather than the
   file they were written in (/specs/acl/grant/0.1/spec.md), which
   dropped a path segment and kept `spec.md` — every relative
   cross-reference between specs landed on "No Trust Task
   registered".
   ============================================================ */
(function (global) {
  "use strict";

  /* Numeric MAJOR.MINOR[.PATCH] comparison; ascending. */
  function cmpVersion(a, b) {
    const pa = String(a).split(".").map(Number), pb = String(b).split(".").map(Number);
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
      const d = (pa[i] || 0) - (pb[i] || 0);
      if (d) return d;
    }
    return 0;
  }

  /* Newest first. */
  function byVersionDesc(a, b) { return cmpVersion(b.version, a.version); }

  function isLive(t) { return t.status !== "retired" && t.status !== "withdrawn"; }

  /* Collapse a task list to one entry per slug — the latest live version, or
   * the latest of any status when a slug has no live version — so coexisting
   * 0.1/0.2 specs count and display as one distinct Trust Task. */
  function latestPerSlug(tasks) {
    const bySlug = new Map();
    for (const t of tasks) {
      const prev = bySlug.get(t.slug);
      if (!prev) { bySlug.set(t.slug, t); continue; }
      const pr = !isLive(prev), tr = !isLive(t);
      if (pr !== tr) { if (pr) bySlug.set(t.slug, t); continue; }
      if (cmpVersion(t.version, prev.version) > 0) bySlug.set(t.slug, t);
    }
    return [...bySlug.values()];
  }

  /* The latest version of `slug` in `list` (each entry has slug + version),
   * or undefined. */
  function latestOf(list, slug) {
    return list.filter(x => x.slug === slug).sort(byVersionDesc)[0];
  }

  const REPO_BLOB = "https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main";

  /* Trees the site serves as files (the CloudFront function's ASSET_PREFIXES). */
  const ASSET_PREFIXES = ["/assets/", "/specs/", "/bindings/", "/ceremonies/"];

  /* Whether a same-origin pathname is a client-side route rather than a file. */
  function isSpaPath(pathname) {
    for (const p of ASSET_PREFIXES) if (pathname.indexOf(p) === 0) return false;
    const last = pathname.split("/").pop();
    // Root files (/SPEC.md, /registry.json, /robots.txt …). A version segment
    // such as `0.1` is not a file extension.
    if (/\.[a-z]+$/i.test(last) && !/^\d+\.\d+(\.\d+)?$/.test(last)) return false;
    return true;
  }

  /*
   * Map one href found in rendered spec or binding prose to the URL a reader
   * should land on. `prosePath` is the served path of the markdown file
   * (`/specs/acl/grant/0.1/spec.md`), which is what a relative link in it is
   * relative to — on disk, on GitHub, and now here.
   *
   *   /SPEC.md#frag                         -> /specification#frag
   *   /specs/<slug>/<M.N>/spec.md#frag      -> /spec/<slug>/<M.N>#frag
   *   /bindings/<slug>/<M.N>/spec.md#frag   -> /binding/<slug>/<M.N>#frag
   *   a shared *.schema.json                -> /schema/<slug>
   *   anything else under a published tree  -> that file, as an absolute path
   *   anything outside the published trees  -> the file on GitHub
   *
   * Absolute URLs, fragments and other schemes are returned unchanged.
   * `sharedSlugByPath` maps a shared schema's served path to its /schema slug.
   * Returns { href, kind } with kind one of "unchanged", "framework", "spec",
   * "binding", "schema", "asset", "github".
   */
  function resolveProseHref(href, prosePath, sharedSlugByPath) {
    if (!href || href.charAt(0) === "#" || /^[a-z][a-z0-9+.-]*:/i.test(href) || href.indexOf("//") === 0) {
      return { href, kind: "unchanged" };
    }
    let u;
    try { u = new URL(href, "https://trusttasks.invalid" + prosePath); } catch (e) { return { href, kind: "unchanged" }; }
    const p = decodeURIComponent(u.pathname);
    const hash = u.hash;
    if (p === "/SPEC.md") return { href: "/specification" + hash, kind: "framework" };
    let m = /^\/specs\/(.+)\/(\d+\.\d+)(?:\/(?:spec\.md)?)?$/.exec(p);
    if (m && !m[1].split("/").some(s => s.charAt(0) === "_")) {
      return { href: `/spec/${m[1]}/${m[2]}${hash}`, kind: "spec" };
    }
    m = /^\/bindings\/([^/]+)\/(\d+\.\d+)(?:\/(?:spec\.md)?)?$/.exec(p);
    if (m) return { href: `/binding/${m[1]}/${m[2]}${hash}`, kind: "binding" };
    if (/\.schema\.json$/.test(p) && sharedSlugByPath) {
      const slug = typeof sharedSlugByPath.get === "function" ? sharedSlugByPath.get(p) : sharedSlugByPath[p];
      if (slug) return { href: `/schema/${slug}${hash}`, kind: "schema" };
    }
    for (const prefix of ASSET_PREFIXES) {
      if (p.indexOf(prefix) === 0) return { href: u.pathname + u.search + hash, kind: "asset" };
    }
    return { href: REPO_BLOB + u.pathname + hash, kind: "github" };
  }

  /* Apply resolveProseHref to every <a href> in a rendered HTML string, and
   * make every relative <img src> absolute against the prose file. The HTML
   * comes from markdown.js, which escapes `&` and `"` in attribute values. */
  function rewriteProseLinks(html, prosePath, sharedSlugByPath) {
    const unescape = (s) => s.replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&amp;/g, "&");
    const escape = (s) => s.replace(/&/g, "&amp;").replace(/"/g, "&quot;");
    return html
      .replace(/(<a\b[^>]*?\shref=")([^"]*)(")/g, (_m, pre, href, post) =>
        pre + escape(resolveProseHref(unescape(href), prosePath, sharedSlugByPath).href) + post)
      .replace(/(<img\b[^>]*?\ssrc=")([^"]*)(")/g, (_m, pre, src, post) => {
        const raw = unescape(src);
        if (!raw || /^[a-z][a-z0-9+.-]*:/i.test(raw) || raw.indexOf("//") === 0) return _m;
        try {
          const u = new URL(raw, "https://trusttasks.invalid" + prosePath);
          return pre + escape(u.pathname + u.search + u.hash) + post;
        } catch (e) { return _m; }
      });
  }

  global.ttRoutes = {
    cmpVersion, byVersionDesc, isLive, latestPerSlug, latestOf,
    isSpaPath, resolveProseHref, rewriteProseLinks, ASSET_PREFIXES, REPO_BLOB,
  };
})(typeof window !== "undefined" ? window : globalThis);
