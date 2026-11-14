/* ============================================================
   Trust Tasks — the app shell: routing, page titles and the mount.
   Compiled with components.jsx and pages.jsx into one bundle by
   scripts/build-website.mjs (#1147). It used to be an inline
   <script type="text/babel"> in index.html.
   ============================================================ */

const { useState: useStateApp, useEffect: useEffectApp, useRef: useRefApp } = React;

/* Names of the pages that take no path segments after their own. */
const SINGLE_PAGES = new Set([
  "specification", "categories", "ecosystem", "about", "contributing", "glossary", "schemas",
]);

/* parse pathname: /, /registry, /registry/identity, /spec/<slug...>/<version>,
   /binding/<slug>/<version>, etc. Slugs may be hierarchical (e.g.
   /spec/acl/grant/0.1); the version is always the final segment, the slug
   is everything between /spec/ (or /binding/) and the version.

   Per SPEC §9.3 the canonical binding URI uses singular /binding/, which is
   what bindingURI fields advertise. We also accept legacy plural /bindings/
   so old in-app history entries continue to resolve. The bindings *index*
   page (no slug) is reached via /bindings (plural) — keeping that distinct
   from a binding URI with no version.

   Anything else is { name: "notfound" } — it used to fall through to the
   home page, so a mistyped URL showed no sign of an error (#1150). */
function parsePath() {
  const path = location.pathname.replace(/^\/+|\/+$/g, "");
  if (!path) return { name: "home" };
  const [name, ...rest] = path.split("/");
  if (name === "ceremony" || name === "ceremonies") {
    /* Per SPEC §6.7 the canonical definition URI uses singular /ceremony/,
       structurally disjoint from /spec/ and /binding/. The index page is
       /ceremonies (plural), keeping it distinct from a definition URI with
       no version — the same split the bindings routes make. */
    if (rest.length === 0) return { name: "ceremonies" };
    const last = rest[rest.length - 1];
    const versionLike = /^\d+\.\d+$/.test(last);
    const version = versionLike ? last : undefined;
    const slugSegs = versionLike ? rest.slice(0, -1) : rest;
    return { name: "ceremony", slug: slugSegs.join("/"), version };
  }
  if (name === "spec" || name === "binding" || name === "bindings") {
    const routeName = name === "spec" ? "spec" : "binding";
    if (rest.length === 0) {
      // /spec → spec index; /bindings → bindings index; /binding (with
      // no slug) is non-canonical but treated as the bindings index.
      return { name: name === "spec" ? "spec" : "bindings" };
    }
    // Heuristic: if the last segment looks like MAJOR.MINOR, it's the version.
    const last = rest[rest.length - 1];
    const versionLike = /^\d+\.\d+$/.test(last);
    const version = versionLike ? last : undefined;
    const slugSegs = versionLike ? rest.slice(0, -1) : rest;
    return { name: routeName, slug: slugSegs.join("/"), version };
  }
  if (name === "schema") {
    // /schema/<slug…> — the slug is the full remainder, including any version
    // segment. The version is part of the schema's $id, not a separate param.
    if (rest.length === 0) return { name: "schemas" };
    return { name: "schema", slug: rest.join("/") };
  }
  if (name === "implementations") {
    // /implementations is the index across every library; /implementations/rust
    // is one library's deep dive. Structurally the same split /bindings and
    // /binding/<slug> make, with the plural kept for both because the
    // singular carries no canonical URI here the way a bindingURI does.
    if (rest.length === 0) return { name: "implementations" };
    return { name: "implementation", id: rest[0] };
  }
  if (name === "registry") {
    // Filters live in the query string (#1154): /registry/<cat>?q=&kw=&status=
    if (rest.length > 1) return { name: "notfound" };
    const params = new URLSearchParams(location.search);
    return {
      name: "registry",
      cat: rest[0] || undefined,
      q: params.get("q") || undefined,
      kw: params.get("kw") || undefined,
      status: params.get("status") || undefined,
    };
  }
  if (SINGLE_PAGES.has(name) && rest.length === 0) return { name };
  return { name: "notfound" };
}

/* The path (and query) a route object lives at. */
function routePath(r) {
  if (r.name === "spec") {
    const v = r.version || (window.ttRoutes.latestOf(window.TT_TASKS, r.slug || r.id) || {}).version || "";
    return `/spec/${r.slug || r.id}${v ? "/" + v : ""}`;
  }
  if (r.name === "schema") return `/schema/${r.slug}`;
  if (r.name === "schemas") return "/schema";
  if (r.name === "ceremonies") return "/ceremonies";
  // Singular /ceremony/ per SPEC §6.7; matches the definitionURI a step
  // carries in its ceremony.definition member.
  if (r.name === "ceremony") return `/ceremony/${r.slug}${r.version ? "/" + r.version : ""}`;
  if (r.name === "binding") {
    // A versionless /binding/<slug> resolves to the latest registered
    // version. Singular /binding/ per SPEC §9.3; matches the canonical
    // bindingURI shown on each binding's spec page.
    const v = r.version || (window.ttRoutes.latestOf(window.TT_BINDINGS || [], r.slug) || {}).version || "";
    return `/binding/${r.slug}${v ? "/" + v : ""}`;
  }
  if (r.name === "implementation") return `/implementations/${r.id}`;
  if (r.name === "registry") {
    const params = new URLSearchParams();
    if (r.q) params.set("q", r.q);
    if (r.kw) params.set("kw", r.kw);
    if (r.status) params.set("status", r.status);
    const qs = params.toString();
    return `/registry${r.cat ? "/" + r.cat : ""}${qs ? "?" + qs : ""}`;
  }
  if (r.name === "home") return "/";
  return `/${r.name}`;
}

/* Title and description for a route (#1151): every tab, history entry and
   bookmark used to read the same site title. The static pages
   scripts/build-website.mjs writes for specifications, bindings, shared
   schemas and ceremonies carry the same values (staticPages() there). */
const SITE_TITLE = "Trust Tasks — Reference registry of trust task specifications";
const SITE_DESCRIPTION = document.querySelector('meta[name="description"]').getAttribute("content");
function routeMeta(r) {
  const t = (s) => `${s} · Trust Tasks`;
  const label = (id) => ((window.TT_NAV_GROUPS || []).flatMap(g => g.links).find(l => l.id === id) || {}).label;
  switch (r.name) {
    case "home": return { title: SITE_TITLE, description: SITE_DESCRIPTION };
    case "spec": {
      const all = window.TT_TASKS || [];
      const task = r.version ? all.find(x => x.slug === r.slug && x.version === r.version) : window.ttRoutes.latestOf(all, r.slug);
      return task
        ? { title: t(`${task.title} (${task.slug} v${task.version})`), description: task.summary }
        : { title: t("Specification not found") };
    }
    case "binding": {
      const b = (window.TT_BINDINGS || []).find(x => x.slug === r.slug && (!r.version || x.version === r.version));
      return b ? { title: t(`${b.title} binding v${b.version}`), description: b.summary } : { title: t("Binding not found") };
    }
    case "ceremony": {
      const c = (window.TT_CEREMONIES || []).find(x => x.slug === r.slug && (!r.version || x.version === r.version));
      return c ? { title: t(`${c.title} ceremony v${c.version}`), description: c.summary } : { title: t("Ceremony not found") };
    }
    case "schema": {
      const rec = (window.TT_SHARED || []).find(x => x.slug === r.slug);
      return { title: t(`${r.slug} — shared schema`), description: rec ? rec.description : undefined };
    }
    case "implementation": {
      const lib = (window.TT_LIBRARIES || []).find(l => l.id === r.id);
      return { title: t(lib ? `${lib.name} (${lib.language})` : "Implementations"), description: lib ? lib.tagline : undefined };
    }
    case "registry": {
      const cat = r.cat && (window.TT_CATEGORIES || []).find(c => c.id === r.cat);
      return { title: t(cat ? `${cat.name} — Registry` : "Registry") };
    }
    case "notfound": return { title: t("Page not found") };
    default: return { title: t(label(r.name) || r.name) };
  }
}

/* The canonical URL of a route (#1151): the production origin, whatever host
   served the page, and the path without the registry's filters — they narrow
   one page rather than name another. A versionless /spec/<slug> or
   /binding/<slug> canonicalises to the version it shows. */
const SITE_ORIGIN = "https://trusttasks.org";
function canonicalPath(r) {
  if (r.name === "registry") return routePath({ name: "registry", cat: r.cat });
  if (r.name === "notfound" || ((r.name === "spec" || r.name === "binding") && !r.slug)) {
    return "/" + location.pathname.replace(/^\/+|\/+$/g, "");
  }
  return routePath(r);
}

function setHead(selector, attr, value) {
  const el = document.querySelector(selector);
  if (el) el.setAttribute(attr, value);
}

/* Remounts the page — and clears a caught render error — whenever the page
   changes, but not when only the registry's filters do. */
function routeKey(r) {
  if (r.name === "registry") return "registry";
  return [r.name, r.slug, r.version, r.id].join("|");
}

// The site's presentation settings: hero layout, type pairing, background,
// density and accent colour.
const SITE_THEME = {
  heroLayout: "editorial",
  type: "technical",
  background: "paper",
  density: "compact",
  accent: "violet"
};

function App() {
  const [route, setRouteRaw] = useStateApp(parsePath());
  // Bumped on every navigation that moves to a new page (not a filter
  // change), so focus can follow it.
  const [navSeq, setNavSeq] = useStateApp(0);
  const tweaks = SITE_THEME;

  /* Navigate to a route object. `{ replace: true }` rewrites the current
     history entry instead of adding one, and leaves scroll and focus alone
     — the registry's filters use it. */
  const setRoute = (r, opts) => {
    const replace = !!(opts && opts.replace);
    setRouteRaw(r);
    const path = routePath(r);
    const wantHash = r.hash ? `#${r.hash}` : "";
    const url = path + wantHash;
    if (location.pathname + location.search + location.hash !== url) {
      if (replace) history.replaceState(null, "", url);
      else history.pushState(null, "", url);
    }
    if (replace) return;
    setNavSeq(n => n + 1);
    // When navigating with a hash, let the destination page's effect
    // scroll to the anchor once the prose has rendered — top-of-page
    // scrolling here would just be visual noise.
    if (!r.hash) window.scrollTo(0, 0);
  };

  // Navigate to a same-origin URL — for links inside rendered prose, which
  // are plain hrefs rather than route objects (pages.jsx onProseClick).
  useEffectApp(() => {
    window.ttNavigate = (url) => {
      history.pushState(null, "", url);
      setRouteRaw(parsePath());
      setNavSeq(n => n + 1);
      if (!new URL(url, location.origin).hash) window.scrollTo(0, 0);
    };
    return () => { delete window.ttNavigate; };
  }, []);

  useEffectApp(() => {
    const onPop = () => { setRouteRaw(parsePath()); setNavSeq(n => n + 1); };
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

  // Title, description, canonical link and link-preview tags follow the route.
  useEffectApp(() => {
    const meta = routeMeta(route);
    const title = meta.title || SITE_TITLE;
    const description = meta.description || SITE_DESCRIPTION;
    const url = SITE_ORIGIN + canonicalPath(route);
    document.title = title;
    setHead('meta[name="description"]', "content", description);
    setHead('link[rel="canonical"]', "href", url);
    setHead('meta[property="og:url"]', "content", url);
    setHead('meta[property="og:title"]', "content", title);
    setHead('meta[property="og:description"]', "content", description);
    setHead('meta[name="twitter:title"]', "content", title);
    setHead('meta[name="twitter:description"]', "content", description);
  }, [route]);

  // Move focus to the new page's heading, so a keyboard or screen-reader
  // user lands on the page they asked for rather than on the link they
  // clicked (#1156). Not on first load: the browser handles that.
  const firstNav = useRefApp(true);
  useEffectApp(() => {
    if (firstNav.current) { firstNav.current = false; return; }
    requestAnimationFrame(() => {
      const main = document.getElementById("main");
      if (!main) return;
      const h1 = main.querySelector("h1");
      const target = h1 || main;
      if (!target.hasAttribute("tabindex")) target.setAttribute("tabindex", "-1");
      target.focus({ preventScroll: true });
    });
  }, [navSeq]);

  // apply the theme to the body data attrs
  useEffectApp(() => {
    document.body.dataset.bg = tweaks.background;
    document.body.dataset.density = tweaks.density;
    document.body.dataset.type = tweaks.type;
    // accent override — the `-text` variants, because the accent fills the
    // primary button behind white text (white on bright violet is 4.2:1).
    const accentMap = {
      navy: "var(--tt-navy)",
      teal: "var(--tt-teal-text)",
      coral: "var(--tt-coral-text)",
      violet: "var(--tt-violet-text)"
    };
    document.documentElement.style.setProperty("--tt-accent", accentMap[tweaks.accent] || accentMap.navy);
    document.documentElement.style.setProperty("--tt-link-hover", tweaks.accent === "teal" ? "var(--tt-amber-text)" : "var(--tt-teal-text)");
  }, []);

  let view;
  switch (route.name) {
    case "registry":      view = <RegistryPage route={route} setRoute={setRoute} />; break;
    case "spec":          view = <SpecPage slug={route.slug} version={route.version} setRoute={setRoute} />; break;
    case "schema":        view = <SchemaPage slug={route.slug} setRoute={setRoute} />; break;
    case "schemas":       view = <SchemasIndexPage setRoute={setRoute} />; break;
    case "specification": view = <FrameworkSpecPage setRoute={setRoute} />; break;
    case "bindings":      view = <BindingsPage setRoute={setRoute} />; break;
    case "binding":       view = <BindingSpecPage slug={route.slug} version={route.version} setRoute={setRoute} />; break;
    case "ceremonies":    view = <CeremoniesPage setRoute={setRoute} />; break;
    case "ceremony":      view = <CeremonyPage slug={route.slug} version={route.version} setRoute={setRoute} />; break;
    case "implementations": view = <ImplementationsPage setRoute={setRoute} />; break;
    // Rust has a bespoke deep dive; Go and Dart use the generic
    // package walkthrough. An unknown id falls back to the index rather
    // than rendering a blank page (ImplementationDetailPage does that).
    case "implementation":
      view = route.id === "rust"
        ? <RustImplementationPage setRoute={setRoute} />
        : <ImplementationDetailPage id={route.id} setRoute={setRoute} />;
      break;
    case "categories":    view = <CategoriesPage setRoute={setRoute} />; break;
    case "ecosystem":     view = <EcosystemPage setRoute={setRoute} />; break;
    case "about":         view = <AboutPage />; break;
    case "contributing":  view = <ContributingPage />; break;
    case "glossary":      view = <GlossaryPage />; break;
    case "home":          view = <HomePage tweaks={tweaks} setRoute={setRoute} />; break;
    default:              view = <NotFoundPage setRoute={setRoute} />; break;
  }

  return (
    <React.Fragment>
      <TTNav route={route} setRoute={setRoute} />
      <main id="main" tabIndex={-1}>
        <TTErrorBoundary key={routeKey(route)}>{view}</TTErrorBoundary>
      </main>
      <TTFooter setRoute={setRoute} />
    </React.Fragment>
  );
}

/* The skip link's focus move. It was an inline onclick in index.html, which
   the site's Content-Security-Policy (no inline script) would not run. Until
   this bundle runs the link is a plain #main anchor. */
function ttSkipToMain(event) {
  const link = event.target && event.target.closest ? event.target.closest(".tt-skip-link") : null;
  if (!link) return;
  const main = document.getElementById("main");
  if (!main) return;
  event.preventDefault();
  main.focus();
  main.scrollIntoView();
}

/* Mounted only where there is a #root to mount into: the website test
   (scripts/test/website-jsx.test.mjs) runs this bundle with no DOM and renders
   App through React's server renderer instead. */
const ttRootElement = document.getElementById("root");
if (ttRootElement) {
  document.addEventListener("click", ttSkipToMain);
  ReactDOM.createRoot(ttRootElement).render(<App />);
}
