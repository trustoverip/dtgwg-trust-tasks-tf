/* ============================================================
   Trust Tasks — Page views (Home, Registry, Spec, Categories, About, Contributing, Glossary)
   ============================================================ */

const { useState: useS, useEffect: useE, useMemo: useM, useRef: useR } = React;

/* ============================================================
   Markdown helpers — used by SpecPage and (lightly) FrameworkSpecPage
   ============================================================ */
function stripFrontMatter(src) {
  if (!src.startsWith("---")) return src;
  const end = src.indexOf("\n---", 3);
  if (end < 0) return src;
  return src.slice(end + 4).replace(/^\r?\n/, "");
}

/* Render an author string. The convention used in spec front matter is
 *   "Display Name (https://url)"
 * — when that shape matches, the name is anchored to the URL. Plain strings
 * (no URL, or a URL-only entry) are rendered as text. */
function renderAuthor(author, key) {
  if (typeof author !== "string") return null;
  const named = author.match(/^(.+?)\s*\((https?:\/\/[^\s)]+)\)\s*$/);
  if (named) {
    return (
      <a key={key} href={named[2]} target="_blank" rel="noreferrer">
        {named[1].trim()}
      </a>
    );
  }
  const bare = author.match(/^(https?:\/\/\S+)$/);
  if (bare) {
    return (
      <a key={key} href={bare[1]} target="_blank" rel="noreferrer">
        {bare[1]}
      </a>
    );
  }
  return <React.Fragment key={key}>{author}</React.Fragment>;
}

function renderAuthorList(authors) {
  if (!Array.isArray(authors) || authors.length === 0) return null;
  return authors.flatMap((a, i) => {
    const node = renderAuthor(a, `author-${i}`);
    return i === 0 ? [node] : [<React.Fragment key={`sep-${i}`}>, </React.Fragment>, node];
  });
}

/* GitHub-style heading slug: strip HTML, strip punctuation (including `.`),
 * collapse whitespace into single hyphens. Matches the anchors that SPEC.md's
 * own cross-references use (e.g. "4.8.1 Precedence..." -> "481-precedence-..."). */
function slugifyHeading(text) {
  return text
    .replace(/<[^>]+>/g, "")
    .replace(/&amp;/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9 \-]+/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/* Walks an already-rendered HTML string, gives every <h2>–<h6> an id, and
 * returns { html, toc: [{id, text}] }. Only <h2> elements are collected into
 * the TOC so sidebars stay shallow; the deeper levels just become anchorable
 * for fragment navigation. */
function injectHeadingIds(html) {
  const toc = [];
  const seen = new Set();
  const out = html.replace(/<h([2-6])(\s[^>]*)?>([\s\S]*?)<\/h\1>/g, (match, level, attrs, inner) => {
    const text = inner.replace(/<[^>]+>/g, "").replace(/&amp;/g, "").trim();
    if (!text) return match;
    let id = slugifyHeading(text);
    if (!id) return match;
    let i = 2;
    while (seen.has(id)) { id = `${slugifyHeading(text)}-${i++}`; }
    seen.add(id);
    if (level === "2") toc.push({ id, text });
    if (attrs && /\bid=/.test(attrs)) return match;
    return `<h${level}${attrs || ""} id="${id}">${inner}</h${level}>`;
  });
  return { html: out, toc };
}

/* ============================================================
   HOME
   ============================================================ */
// Collapse a task list to one entry per slug — the latest non-retired version
// — so coexisting 0.1/0.2 specs count and display as one distinct Trust Task.
function latestPerSlug(tasks) {
  const cmpVer = (a, b) => { const pa = a.split(".").map(Number), pb = b.split(".").map(Number); return (pa[0] - pb[0]) || (pa[1] - pb[1]); };
  const bySlug = new Map();
  for (const t of tasks) {
    const prev = bySlug.get(t.slug);
    if (!prev) { bySlug.set(t.slug, t); continue; }
    const pr = prev.status === "retired", tr = t.status === "retired";
    if (pr !== tr) { if (pr) bySlug.set(t.slug, t); continue; }
    if (cmpVer(t.version, prev.version) > 0) bySlug.set(t.slug, t);
  }
  return [...bySlug.values()];
}

// Distinct-slug count per category (not per version).
function countSlugsByCategory(tasks) {
  const sets = {};
  for (const t of tasks) (sets[t.category] ||= new Set()).add(t.slug);
  const out = {};
  for (const k in sets) out[k] = sets[k].size;
  return out;
}

function HomePage({ tweaks, setRoute }) {
  const stats = window.TT_STATS;
  const [q, setQ] = useS("");
  const [activeCat, setActiveCat] = useS(null);
  const [activeKw, setActiveKw] = useS(null);
  const inputRef = useR(null);

  useE(() => {
    const onKey = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // category counts
  const counts = useM(() => countSlugsByCategory(window.TT_TASKS), []);

  // 10 newest specs (by created date, desc)
  const newestSpecs = useM(() => {
    return [...window.TT_TASKS]
      .sort((a, b) => (b.created || "").localeCompare(a.created || ""))
      .slice(0, 10);
  }, []);

  // 10 most-recently-updated specs, excluding ones that haven't been touched since
  // creation (those belong in the "newest" list instead).
  const recentlyUpdatedSpecs = useM(() => {
    return [...window.TT_TASKS]
      .filter(t => t.created && t.updated && t.updated > t.created)
      .sort((a, b) => (b.updated || "").localeCompare(a.updated || ""))
      .slice(0, 10);
  }, []);

  // top keywords across registry
  const topKeywords = useM(() => {
    const kws = {};
    window.TT_TASKS.forEach(t => t.keywords.forEach(k => { kws[k] = (kws[k] || 0) + 1; }));
    return Object.entries(kws).sort((a, b) => b[1] - a[1]).slice(0, 10).map(([k]) => k);
  }, []);

  // filtered preview results
  const results = useM(() => {
    const ql = q.trim().toLowerCase();
    return latestPerSlug(window.TT_TASKS.filter(t => {
      if (activeCat && t.category !== activeCat) return false;
      if (activeKw && !t.keywords.includes(activeKw)) return false;
      if (!ql) return true;
      const hay = (t.title + " " + t.summary + " " + t.keywords.join(" ") + " " + t.slug).toLowerCase();
      return hay.includes(ql);
    }));
  }, [q, activeCat, activeKw]);

  const heroLayout = tweaks.heroLayout || "split";

  return (
    <React.Fragment>
      {/* HERO */}
      <section className={`tt-hero ${heroLayout === "split" ? "tt-hero--split" : "tt-hero--editorial"}`}>
        <div className="container tt-hero__inner">
          <div>
            <span className="eyebrow tt-hero__eyebrow tt-rise tt-rise-1">Trust Over IP · DTGWG · v1.0 draft</span>
            <h1 className="tt-rise tt-rise-2">
              The reference registry of <em style={{ whiteSpace: "nowrap" }}>trust task</em> specifications.
            </h1>
            <p className="tt-hero__lede tt-rise tt-rise-3">
              Trust Tasks are self-contained, transport-agnostic, JSON-based specifications for the verifiable work
              that happens between entities. Part of the verifiable-trust stack alongside&nbsp;
              <a href="https://github.com/OpenVTC/dtg-credentials" target="_blank" rel="noreferrer">DTG Credentials</a>,&nbsp;
              <a href="https://github.com/OpenVTC/verifiable-trust-infrastructure" target="_blank" rel="noreferrer">Verifiable Trust Infrastructure</a>,
              and <a href="https://github.com/OpenVTC/openvtc" target="_blank" rel="noreferrer">OpenVTC</a>.
            </p>

            <div className="tt-rise tt-rise-4" style={{ maxWidth: "640px" }}>
              <label className="tt-search" htmlFor="tt-home-search">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="11" cy="11" r="7" />
                  <path d="m21 21-4.3-4.3" />
                </svg>
                <input
                  id="tt-home-search"
                  ref={inputRef}
                  type="search"
                  placeholder="Search specifications — try “credential”, “consent”, “payment”…"
                  value={q}
                  onChange={(e) => setQ(e.target.value)}
                  onKeyDown={(e) => { if (e.key === "Enter") setRoute({ name: "registry", q, cat: activeCat, kw: activeKw }); }}
                />
                <kbd>⌘K</kbd>
              </label>

              <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--tt-space-3)", alignItems: "center", marginTop: "var(--tt-space-4)" }}>
                <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--tt-text-muted)" }}>Categories</span>
                <div className="tt-chips">
                  {window.TT_CATEGORIES.map(c => (
                    <button
                      key={c.id}
                      className="tt-chip"
                      aria-pressed={activeCat === c.id}
                      onClick={() => setActiveCat(activeCat === c.id ? null : c.id)}
                    >
                      <span className="dot" style={{ background: catColor(c.id) }}></span>
                      {c.name}
                      <span className="tt-chip__count">{counts[c.id] || 0}</span>
                    </button>
                  ))}
                </div>
              </div>

              <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--tt-space-2)", alignItems: "center", marginTop: "var(--tt-space-3)" }}>
                <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--tt-text-muted)", marginRight: "0.5em" }}>Keywords</span>
                {topKeywords.map(k => (
                  <button
                    key={k}
                    className="tt-keyword"
                    aria-pressed={activeKw === k}
                    onClick={() => setActiveKw(activeKw === k ? null : k)}
                  >{k}</button>
                ))}
              </div>

              {(q || activeCat || activeKw) && (
                <div style={{ marginTop: "var(--tt-space-5)", paddingTop: "var(--tt-space-4)", borderTop: "1px solid var(--tt-line)" }}>
                  <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--tt-text-muted)", marginBottom: "var(--tt-space-3)" }}>
                    {results.length} match{results.length === 1 ? "" : "es"}
                  </div>
                  <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-2)" }}>
                    {results.slice(0, 4).map(t => (
                      <a
                        key={t.id}
                        href={`/spec/${t.slug}/${t.version}`}
                        onClick={(e) => { e.preventDefault(); setRoute({ name: "spec", slug: t.slug, version: t.version }); }}
                        style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: "var(--tt-space-3)", padding: "var(--tt-space-3) var(--tt-space-4)", background: "var(--tt-surface)", border: "1px solid var(--tt-border)", borderRadius: "var(--tt-radius)", borderBottom: "1px solid var(--tt-border)", textDecoration: "none", color: "inherit" }}
                      >
                        <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", letterSpacing: "0.06em" }}>v{t.version}</span>
                        <span style={{ flex: 1, fontFamily: "var(--tt-font-display)", fontSize: "var(--tt-text-md)" }}><Highlight text={t.title} query={q} /></span>
                        <TTStatus status={t.status} />
                      </a>
                    ))}
                    {results.length > 4 && (
                      <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry", q, cat: activeCat, kw: activeKw }); }} style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--tt-text-muted)", borderBottom: 0, marginTop: "var(--tt-space-2)" }}>
                        See all {results.length} in the registry →
                      </a>
                    )}
                    {results.length === 0 && <div className="tt-empty" style={{ padding: "var(--tt-space-6) 0" }}>No matches. Try a broader term.</div>}
                  </div>
                </div>
              )}
            </div>
          </div>

          {heroLayout === "split" && (
            <div className="tt-hero__glyph tt-rise tt-rise-3" aria-hidden="true">
              <HeroGlyph />
            </div>
          )}
        </div>
      </section>

      {/* FRAMEWORK SPEC CTA */}
      <section style={{ paddingBlock: "var(--tt-space-6)", borderTop: "1px solid var(--tt-line)", borderBottom: "1px solid var(--tt-line)" }}>
        <div className="container">
          <a
            href="/specification"
            onClick={(e) => { e.preventDefault(); setRoute({ name: "specification" }); }}
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              gap: "var(--tt-space-5)",
              padding: "var(--tt-space-5) 0",
              textDecoration: "none",
              color: "inherit",
              borderBottom: 0,
              flexWrap: "wrap",
            }}
          >
            <div style={{ flex: "1 1 auto", minWidth: "240px" }}>
              <span className="eyebrow" style={{ display: "inline-flex", marginBottom: "var(--tt-space-2)" }}>v0.1 framework specification</span>
              <h3 style={{ margin: 0, marginBottom: "var(--tt-space-2)" }}>Read the framework specification.</h3>
              <p style={{ margin: 0, color: "var(--tt-text-muted)" }}>Document structure, version scheme, namespace, errors, and transport bindings — the contract every individual Trust Task conforms to.</p>
            </div>
            <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-sm)", letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--tt-accent, var(--tt-violet))", whiteSpace: "nowrap" }}>
              Open SPEC.md →
            </span>
          </a>
        </div>
      </section>

      {/* STATS */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow" style={{ marginBottom: "var(--tt-space-4)", display: "inline-flex" }}>Registry at a glance</span>
          <div className="tt-stats" style={{ marginTop: "var(--tt-space-4)" }}>
            <div className="tt-stat">
              <div className="tt-stat__accent" style={{ background: "var(--tt-coral)" }}></div>
              <div className="tt-stat__num"><AnimNumber value={stats.total} /></div>
              <div className="tt-stat__label">Specifications</div>
              <div className="tt-stat__sub">across {stats.categories} categories</div>
            </div>
            <div className="tt-stat">
              <div className="tt-stat__accent" style={{ background: "var(--tt-teal)" }}></div>
              <div className="tt-stat__num"><AnimNumber value={stats.byStatus.standard || 0} /><span className="unit">/ {stats.total}</span></div>
              <div className="tt-stat__label">Standard</div>
              <div className="tt-stat__sub">{stats.byStatus.candidate || 0} candidate · {stats.byStatus.draft || 0} draft</div>
            </div>
            <div className="tt-stat">
              <div className="tt-stat__accent" style={{ background: "var(--tt-violet)" }}></div>
              <div className="tt-stat__num"><AnimNumber value={stats.categories} /></div>
              <div className="tt-stat__label">Categories</div>
              <div className="tt-stat__sub">{window.TT_CATEGORIES.map(c => c.name).join(", ")}</div>
            </div>
            <div className="tt-stat">
              <div className="tt-stat__accent" style={{ background: "var(--tt-amber)" }}></div>
              <div className="tt-stat__num"><AnimNumber value={stats.orgs} /></div>
              <div className="tt-stat__label">Spec owners</div>
            </div>
            <div className="tt-stat">
              <div className="tt-stat__accent" style={{ background: "var(--tt-sky)" }}></div>
              <div className="tt-stat__num" style={{ fontSize: "var(--tt-text-xl)", letterSpacing: "-0.01em" }}>{stats.latest}</div>
              <div className="tt-stat__label">Latest update</div>
              <div className="tt-stat__sub">{stats.latestTitle}</div>
            </div>
          </div>
        </div>
      </section>

      <hr className="protocol-rule container" aria-hidden="true" />

      {/* NEWEST + RECENTLY UPDATED */}
      <section>
        <div className="container">
          <div className="section-head">
            <span className="eyebrow">What's new in the registry</span>
            <h2>Recently added and updated.</h2>
            <p className="lead">The latest activity across the registry. New specifications on the left; ones that have been revised since their initial publication on the right.</p>
          </div>
          <div className="tt-home-lists">
            <div>
              <h3 className="tt-home-list__heading">Newest specifications</h3>
              {newestSpecs.length === 0 ? (
                <p style={{ color: "var(--tt-text-muted)" }}>No specifications have been published yet.</p>
              ) : (
                <ul className="tt-home-list">
                  {newestSpecs.map(t => (
                    <li key={`new-${t.slug}-${t.version}`}>
                      <SpecListRow task={t} dateLabel="Added" dateValue={t.created} setRoute={setRoute} />
                    </li>
                  ))}
                </ul>
              )}
            </div>
            <div>
              <h3 className="tt-home-list__heading">Recently updated</h3>
              {recentlyUpdatedSpecs.length === 0 ? (
                <p style={{ color: "var(--tt-text-muted)" }}>No specifications have been revised since their initial publication.</p>
              ) : (
                <ul className="tt-home-list">
                  {recentlyUpdatedSpecs.map(t => (
                    <li key={`upd-${t.slug}-${t.version}`}>
                      <SpecListRow task={t} dateLabel="Updated" dateValue={t.updated} setRoute={setRoute} />
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
          <div style={{ marginTop: "var(--tt-space-6)", textAlign: "right" }}>
            <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }} className="btn btn--ghost">
              Open the registry →
            </a>
          </div>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   COMPACT SPEC ROW for HomePage's newest / updated lists
   ============================================================ */
function SpecListRow({ task, dateLabel, dateValue, setRoute }) {
  const cat = window.TT_CATEGORIES.find(c => c.id === task.category);
  return (
    <a
      className="tt-home-list__row"
      href={`/spec/${task.slug}/${task.version}`}
      onClick={(e) => { e.preventDefault(); setRoute({ name: "spec", slug: task.slug, version: task.version }); }}
      style={{ "--accent": catColor(task.category) }}
    >
      <span className="tt-home-list__slug">
        <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)" }}>{task.slug}</span>
        <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginLeft: "0.4em" }}>v{task.version}</span>
      </span>
      <span className="tt-home-list__title">{task.title}</span>
      <span className="tt-home-list__cat" style={{ color: catColor(task.category) }}>{cat ? cat.name : task.category}</span>
      <span className="tt-home-list__date" title={dateLabel}>{dateValue}</span>
    </a>
  );
}

/* ============================================================
   REGISTRY CARD (shared between Home featured + Registry list)
   ============================================================ */
function RegistryCard({ task, setRoute, query, activeKw, onKwToggle }) {
  return (
    <a
      className="tt-task-card"
      href={`/spec/${task.slug}/${task.version}`}
      onClick={(e) => { e.preventDefault(); setRoute({ name: "spec", slug: task.slug, version: task.version }); }}
      style={{ "--accent": catColor(task.category) }}
    >
      <div>
        <h3 className="tt-task-card__title"><Highlight text={task.title} query={query} /></h3>
        <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginBottom: "var(--tt-space-2)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          <span style={{ opacity: 0.6 }}>https://trusttasks.org</span>/spec/<span style={{ color: "var(--tt-text)" }}>{task.slug}</span>/<span style={{ color: catColor(task.category) }}>{task.version}</span>
        </div>
        <p className="tt-task-card__summary"><Highlight text={task.summary} query={query} /></p>
        <div className="tt-task-card__meta">
          <span className="pill" style={{ borderColor: catColor(task.category), color: catColor(task.category) }}>
            <span className="dot" style={{ background: catColor(task.category) }}></span>
            {catName(task.category)}
          </span>
          <div className="tt-task-card__keywords">
            {task.keywords.slice(0, 5).map(k => (
              <span
                key={k}
                className="tt-keyword"
                aria-pressed={activeKw === k}
                onClick={onKwToggle ? (e) => { e.preventDefault(); e.stopPropagation(); onKwToggle(k); } : undefined}
              >{k}</span>
            ))}
          </div>
        </div>
      </div>
      <div className="tt-task-card__status">
        <TTStatus status={task.status} />
      </div>
    </a>
  );
}

/* ============================================================
   REGISTRY
   ============================================================ */
function RegistryPage({ initial, setRoute }) {
  const [q, setQ] = useS(initial?.q || "");
  const [activeCat, setActiveCat] = useS(initial?.cat || null);
  const [activeKw, setActiveKw] = useS(initial?.kw || null);
  const [activeStatus, setActiveStatus] = useS(null);

  const counts = useM(() => countSlugsByCategory(window.TT_TASKS), []);

  const allKeywords = useM(() => {
    const kws = {};
    window.TT_TASKS.forEach(t => t.keywords.forEach(k => { kws[k] = (kws[k] || 0) + 1; }));
    return Object.entries(kws).sort((a, b) => b[1] - a[1]);
  }, []);

  const results = useM(() => {
    const ql = q.trim().toLowerCase();
    const filtered = window.TT_TASKS.filter(t => {
      if (activeCat && t.category !== activeCat) return false;
      if (activeKw && !t.keywords.includes(activeKw)) return false;
      if (activeStatus && t.status !== activeStatus) return false;
      if (!ql) return true;
      const hay = (t.title + " " + t.summary + " " + t.keywords.join(" ") + " " + t.slug).toLowerCase();
      return hay.includes(ql);
    });
    // Collapse to one card per slug — the latest non-retired version. Older and
    // retired versions stay reachable from the spec page's version switcher.
    return latestPerSlug(filtered);
  }, [q, activeCat, activeKw, activeStatus]);

  const onClear = () => { setQ(""); setActiveCat(null); setActiveKw(null); setActiveStatus(null); };

  return (
    <React.Fragment>
      <PageHero
        eyebrow="Registry"
        title="Trust Task Registry"
        lede="Every published Trust Task specification, searchable by category, keyword, and status. Each entry links to its full reference document and JSON schema."
      />

      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container container--wide">
          <label className="tt-search" htmlFor="tt-reg-search" style={{ marginBottom: "var(--tt-space-5)" }}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="11" cy="11" r="7" />
              <path d="m21 21-4.3-4.3" />
            </svg>
            <input
              id="tt-reg-search"
              type="search"
              placeholder="Search the registry…"
              value={q}
              onChange={(e) => setQ(e.target.value)}
            />
          </label>

          <div className="tt-filter-rail">
            <aside className="tt-filter-rail__side">
              <div className="tt-filter-group">
                <h5>Category</h5>
                <div className="tt-chips" style={{ flexDirection: "column", alignItems: "flex-start" }}>
                  {window.TT_CATEGORIES.map(c => (
                    <button key={c.id} className="tt-chip" aria-pressed={activeCat === c.id} onClick={() => setActiveCat(activeCat === c.id ? null : c.id)}>
                      <span className="dot" style={{ background: catColor(c.id) }}></span>
                      {c.name}<span className="tt-chip__count">{counts[c.id] || 0}</span>
                    </button>
                  ))}
                </div>
              </div>

              <div className="tt-filter-group">
                <h5>Status</h5>
                <div className="tt-chips" style={{ flexDirection: "column", alignItems: "flex-start" }}>
                  {["standard", "candidate", "draft"].map(s => (
                    <button key={s} className="tt-chip" aria-pressed={activeStatus === s} onClick={() => setActiveStatus(activeStatus === s ? null : s)}>
                      <span className="dot" style={{ background: s === "standard" ? "var(--tt-teal)" : s === "candidate" ? "var(--tt-amber)" : "var(--tt-violet)" }}></span>
                      {s}
                    </button>
                  ))}
                </div>
              </div>

              <div className="tt-filter-group">
                <h5>Keywords</h5>
                <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--tt-space-2)" }}>
                  {allKeywords.slice(0, 18).map(([k]) => (
                    <button key={k} className="tt-keyword" aria-pressed={activeKw === k} onClick={() => setActiveKw(activeKw === k ? null : k)}>{k}</button>
                  ))}
                </div>
              </div>
            </aside>

            <div>
              <div className="tt-results-bar">
                <div className="tt-results-bar__count">
                  <b>{results.length}</b> of {new Set(window.TT_TASKS.map(t => t.slug)).size} specifications
                  {activeCat && <> · category <b>{catName(activeCat)}</b></>}
                  {activeStatus && <> · status <b>{activeStatus}</b></>}
                  {activeKw && <> · keyword <b>{activeKw}</b></>}
                  {q && <> · matching <b>“{q}”</b></>}
                </div>
                {(q || activeCat || activeKw || activeStatus) && (
                  <button className="tt-clear" onClick={onClear}>Clear filters</button>
                )}
              </div>

              {results.length === 0 ? (
                <div className="tt-empty">No specifications match these filters.</div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-4)" }}>
                  {results.map(t => (
                    <RegistryCard
                      key={t.id}
                      task={t}
                      setRoute={setRoute}
                      query={q}
                      activeKw={activeKw}
                      onKwToggle={(k) => setActiveKw(activeKw === k ? null : k)}
                    />
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   SHARED SCHEMA HELPERS — used by SpecPage's Uses panel and by SchemaPage
   ============================================================ */

/* Human label for the kind of shared schema. Kept short for chip layout. */
function sharedKindLabel(kind) {
  switch (kind) {
    case "framework":         return "Framework";
    case "method-extension":  return "Method Extension";
    case "shared":            return "Shared";
    default:                  return kind || "Shared";
  }
}

/* Accent color: framework gets neutral, method-extension gets amber, otherwise
   follow the family's category color when one exists. */
function sharedAccent(rec) {
  if (rec.kind === "framework") return "var(--tt-text-muted)";
  if (rec.kind === "method-extension") return "var(--tt-amber, var(--tt-coral))";
  const cat = window.TT_CATEGORIES.find(c => c.id === rec.family);
  return cat ? `var(--tt-${cat.color})` : "var(--tt-navy)";
}

/* Compact card describing one shared-schema dependency. Clicking navigates
   to /schema/<slug>; the optional def name (when the parent task referenced
   a specific $defs entry) becomes a deep-link fragment. The optional
   `method`/`requirement` props are populated for method-extension entries. */
function SharedChip({ rec, def, occurrences, method, requirement, setRoute }) {
  const accent = sharedAccent(rec);
  const onGo = (e) => {
    e.preventDefault();
    setRoute({ name: "schema", slug: rec.slug, hash: def || undefined });
  };
  const href = `/schema/${rec.slug}${def ? `#${def}` : ""}`;
  return (
    <a
      href={href}
      onClick={onGo}
      style={{
        display: "block",
        padding: "var(--tt-space-4)",
        border: "1px solid var(--tt-border)",
        borderLeft: `3px solid ${accent}`,
        borderRadius: "var(--tt-radius)",
        textDecoration: "none",
        color: "inherit",
        background: "var(--tt-surface)"
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", gap: "var(--tt-space-3)", marginBottom: "var(--tt-space-2)" }}>
        <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: "var(--tt-text-muted)" }}>
          {sharedKindLabel(rec.kind)}
          {rec.family && rec.kind !== "framework" ? ` · ${rec.family}` : ""}
        </span>
        {occurrences > 1 && (
          <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)" }}>
            referenced {occurrences}×
          </span>
        )}
      </div>
      <div style={{ fontFamily: "var(--tt-font-display)", fontSize: "var(--tt-text-md)", marginBottom: "var(--tt-space-2)" }}>
        {def ? <code style={{ fontSize: "0.92em" }}>{def}</code> : rec.title}
        {def && <span style={{ color: "var(--tt-text-muted)" }}> — {rec.title}</span>}
      </div>
      {method && (
        <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginBottom: "var(--tt-space-2)" }}>
          when <code>method = "{method}"</code>
          {requirement && requirement !== "OPTIONAL" && (
            <span> · <b style={{ color: "var(--tt-text)" }}>{requirement}</b></span>
          )}
        </div>
      )}
      <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {rec.schemaId || rec.sourcePath}
      </div>
    </a>
  );
}

/* Render one labeled group of chips. Pulled out so UsesPanel can stack the
 * "From payload schema" and "Method extensions" sections with consistent
 * styling and the unresolved-shared fallback. */
function UsesGroup({ heading, hint, entries, setRoute }) {
  if (!entries || entries.length === 0) return null;
  const catalog = window.TT_SHARED || [];
  return (
    <div style={{ marginBottom: "var(--tt-space-5)" }}>
      <h3 style={{ marginTop: 0, marginBottom: "var(--tt-space-2)", fontSize: "var(--tt-text-md)" }}>{heading}</h3>
      {hint && (
        <p style={{ color: "var(--tt-text-muted)", marginTop: 0, marginBottom: "var(--tt-space-4)", fontSize: "var(--tt-text-sm)" }}>{hint}</p>
      )}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: "var(--tt-space-3)" }}>
        {entries.map((u, i) => {
          const rec = catalog.find(s => s.slug === u.schemaSlug);
          if (!rec) {
            return (
              <div key={i} style={{ padding: "var(--tt-space-4)", border: "1px dashed var(--tt-border)", borderRadius: "var(--tt-radius)", color: "var(--tt-text-muted)" }}>
                <code style={{ fontSize: "0.9em" }}>{u.schemaSlug}</code><br />
                <small>shared schema not indexed</small>
              </div>
            );
          }
          return (
            <SharedChip
              key={i}
              rec={rec}
              def={u.def}
              occurrences={u.occurrences}
              method={u.method}
              requirement={u.requirement}
              setRoute={setRoute}
            />
          );
        })}
      </div>
    </div>
  );
}

function UsesPanel({ uses, setRoute }) {
  if (!uses || uses.length === 0) {
    return <p style={{ color: "var(--tt-text-muted)" }}>This task's payload schema has no cross-document references and declares no method extensions — it's self-contained.</p>;
  }
  const refs = uses.filter(u => (u.via || "ref") === "ref");
  const methodExts = uses.filter(u => u.via === "methodExtension");
  return (
    <React.Fragment>
      <UsesGroup
        heading="From payload schema"
        hint="Resolved by walking $ref pointers in this task's payload.schema.json."
        entries={refs}
        setRoute={setRoute}
      />
      <UsesGroup
        heading="Method extensions"
        hint="Vendor-namespaced extension shapes that producers MAY include in the payload's `ext` member when their declared method matches. Declared in the spec's frontmatter."
        entries={methodExts}
        setRoute={setRoute}
      />
    </React.Fragment>
  );
}

/* ============================================================
   SCHEMA PAGE — standalone view for a shared/framework/method-extension schema
   ============================================================ */
function SchemaPage({ slug, setRoute }) {
  const catalog = window.TT_SHARED || [];
  const rec = catalog.find(s => s.slug === slug);
  // Always call the fragment-scroll effect regardless of whether the schema
  // resolves, so hook order is stable across renders (Rules of Hooks).
  useE(() => {
    if (!rec || !location.hash) return;
    const target = document.getElementById(`def-${location.hash.slice(1)}`);
    if (target) requestAnimationFrame(() => target.scrollIntoView({ block: "start" }));
  }, [slug, !!rec]);

  if (!rec) {
    return (
      <section className="container">
        <div className="tt-empty" style={{ padding: "var(--tt-space-6)", marginTop: "var(--tt-space-6)" }}>
          <b>No shared schema matches <code>/schema/{slug}</code>.</b>
          <div style={{ marginTop: "var(--tt-space-4)" }}>
            <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }}>← Back to registry</a>
          </div>
        </div>
      </section>
    );
  }
  const usedBy = (window.TT_SHARED_USED_BY && window.TT_SHARED_USED_BY[slug]) || [];
  const accent = sharedAccent(rec);
  const schemaIdLink = rec.schemaId;

  return (
    <section className="container container--wide tt-spec">
      <div>
        <div style={{ marginBottom: "var(--tt-space-4)" }}>
          <span className="tt-spec__num">
            <span style={{ color: accent }}>{sharedKindLabel(rec.kind)}</span>
            {rec.family && rec.kind !== "framework" ? ` · ${rec.family}` : ""}
            {" · "}{rec.slug}
          </span>
        </div>
        <h1 className="tt-spec__title">{rec.title}</h1>
        {rec.description && (
          <p className="lead" style={{ marginBottom: "var(--tt-space-5)" }}>{rec.description}</p>
        )}

        {(() => {
          // Sibling versions of this shared component: same base identity
          // (everything but the MAJOR.MINOR segment), different version.
          const parse = (sl) => { const m = sl.match(/^(.*)\/(\d+\.\d+)\/(.*)$/); return m ? { base: m[1] + "//" + m[3], ver: m[2] } : null; };
          const self = parse(rec.slug);
          if (!self) return null;
          const versions = (window.TT_SHARED || [])
            .map(s => { const p = parse(s.slug); return p && p.base === self.base ? { ver: p.ver, slug: s.slug } : null; })
            .filter(Boolean)
            .sort((a, b) => { const pa = a.ver.split(".").map(Number), pb = b.ver.split(".").map(Number); return (pb[0] - pa[0]) || (pb[1] - pa[1]); });
          if (versions.length < 2) return null;
          return (
            <div style={{ marginBottom: "var(--tt-space-5)", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-sm)" }}>
              <span style={{ color: "var(--tt-text-muted)", textTransform: "uppercase", letterSpacing: "0.06em", fontSize: "var(--tt-text-xs)", marginRight: "var(--tt-space-3)" }}>Versions</span>
              {versions.map((v, i) => (
                <React.Fragment key={v.ver}>
                  {i > 0 && <span style={{ color: "var(--tt-text-muted)" }}> · </span>}
                  {v.ver === self.ver
                    ? <b>v{v.ver}</b>
                    : <a href={`/schema/${v.slug}`} onClick={(e) => { e.preventDefault(); setRoute({ name: "schema", slug: v.slug }); window.scrollTo(0, 0); }}>v{v.ver}</a>}
                </React.Fragment>
              ))}
            </div>
          );
        })()}

        {schemaIdLink && (
          <div
            className="tt-type-uri"
            style={{
              display: "flex", alignItems: "stretch",
              border: "1px solid var(--tt-border)",
              borderLeft: `3px solid ${accent}`,
              background: "var(--tt-surface-elev)",
              marginBottom: "var(--tt-space-6)",
              fontFamily: "var(--tt-font-mono)",
              fontSize: "var(--tt-text-sm)",
            }}
          >
            <div style={{ padding: "var(--tt-space-3) var(--tt-space-4)", borderRight: "1px solid var(--tt-border)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", fontSize: "var(--tt-text-xs)", textTransform: "uppercase", display: "flex", alignItems: "center" }}>Schema $id</div>
            <code style={{ flex: 1, padding: "var(--tt-space-3) var(--tt-space-4)", overflow: "auto", whiteSpace: "nowrap", color: "var(--tt-text)" }}>{schemaIdLink}</code>
          </div>
        )}

        <h2 id="source">Source</h2>
        <p>
          <code>{rec.sourcePath}</code>
          {" — "}
          <a href={`https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main${rec.sourcePath}`} target="_blank" rel="noreferrer">View on GitHub →</a>
        </p>

        {rec.defs.length > 0 && (
          <React.Fragment>
            <h2 id="defs">Definitions</h2>
            <p style={{ color: "var(--tt-text-muted)", marginTop: "calc(-1 * var(--tt-space-3))" }}>
              Each definition can be referenced from a payload schema as
              {" "}<code>{rec.sourcePath.replace("/specs/", "")}#/$defs/&lt;Name&gt;</code>.
            </p>
            <ul style={{ paddingLeft: "var(--tt-space-5)" }}>
              {rec.defs.map(d => (
                <li key={d} id={`def-${d}`}>
                  <code>{d}</code>
                  {rec.schema.$defs && rec.schema.$defs[d] && rec.schema.$defs[d].description && (
                    <span style={{ color: "var(--tt-text-muted)" }}> — {rec.schema.$defs[d].description}</span>
                  )}
                </li>
              ))}
            </ul>
          </React.Fragment>
        )}

        <h2 id="schema">JSON Schema</h2>
        <CodeBlock json={rec.schema} />

        <h2 id="used-by">Used by</h2>
        {usedBy.length === 0 ? (
          <p style={{ color: "var(--tt-text-muted)" }}>
            No payload schema currently references this document via <code>$ref</code>. It is published as a building block for ecosystem use.
          </p>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-3)" }}>
            {usedBy.map((u, i) => (
              <a
                key={i}
                href={`/spec/${u.slug}/${u.version}`}
                onClick={(e) => { e.preventDefault(); setRoute({ name: "spec", slug: u.slug, version: u.version }); }}
                style={{ padding: "var(--tt-space-4)", border: "1px solid var(--tt-border)", borderRadius: "var(--tt-radius)", display: "flex", justifyContent: "space-between", alignItems: "center", textDecoration: "none", color: "inherit" }}
              >
                <span>
                  <span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", marginRight: "var(--tt-space-3)" }}>
                    {u.slug} · v{u.version}
                  </span>
                  {u.title}
                  {u.def && (
                    <span style={{ marginLeft: "var(--tt-space-3)", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)" }}>
                      → <code>{u.def}</code>
                    </span>
                  )}
                  {u.via === "methodExtension" && (
                    <span style={{ marginLeft: "var(--tt-space-3)", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)" }}>
                      via <code>method = "{u.method}"</code>
                      {u.requirement && u.requirement !== "OPTIONAL" && (
                        <span> · {u.requirement}</span>
                      )}
                    </span>
                  )}
                </span>
                <TTStatus status={u.status} />
              </a>
            ))}
          </div>
        )}

        <div style={{ marginTop: "var(--tt-space-8)", paddingTop: "var(--tt-space-5)", borderTop: "1px solid var(--tt-line)", display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--tt-space-4)" }}>
          <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }} className="btn btn--ghost">← Back to registry</a>
          <a href={`https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main${rec.sourcePath}`} target="_blank" rel="noreferrer" className="btn btn--ghost">Edit on GitHub →</a>
        </div>
      </div>

      <aside className="tt-spec__sidebar">
        <div className="tt-toc-title">On this page</div>
        <ol className="tt-toc">
          {[
            { id: "source", text: "Source" },
            ...(rec.defs.length > 0 ? [{ id: "defs", text: "Definitions" }] : []),
            { id: "schema", text: "JSON Schema" },
            { id: "used-by", text: "Used by" }
          ].map(({ id: sid, text }) => (
            <li key={sid}><a href={`#${sid}`}>{text}</a></li>
          ))}
        </ol>
      </aside>
    </section>
  );
}

/* ============================================================
   SCHEMAS INDEX PAGE — browse every shared/framework schema
   ============================================================ */
function SchemasIndexPage({ setRoute }) {
  const catalog = window.TT_SHARED || [];
  const usedBy = window.TT_SHARED_USED_BY || {};
  // Group by kind so the framework primitives sit at the top, then family-shared,
  // then method extensions. Within each group, alphabetize by slug.
  const groups = [
    { id: "framework",         label: "Framework primitives",  desc: "Reusable $defs cross-referenced by every Trust Task specification." },
    { id: "shared",            label: "Family-shared schemas", desc: "Per-category shared $defs (e.g. did-management, acl, auth) referenced by every task in that family." },
    { id: "method-extension",  label: "Method extensions",     desc: "Optional vendor-namespaced extension shapes (e.g. did:webvh) that producers MAY include via the ext member." }
  ];
  return (
    <React.Fragment>
      <PageHero
        eyebrow="Shared schemas"
        title="Reusable building blocks"
        lede="Trust Task payload schemas don't duplicate common shapes — they reference these documents via $ref. Each shared schema is independently versioned and citable."
      />
      <section className="container">
        {groups.map(g => {
          // Collapse to one card per shared component — the latest version.
          // Older versions stay reachable from the detail page's version switcher.
          const parseV = (sl) => { const m = sl.match(/^(.*)\/(\d+\.\d+)\/(.*)$/); return m ? { base: m[1] + "//" + m[3], ver: m[2] } : { base: sl, ver: "0.0" }; };
          const cmpV = (a, b) => { const pa = a.split(".").map(Number), pb = b.split(".").map(Number); return (pa[0] - pb[0]) || (pa[1] - pb[1]); };
          const byBase = new Map();
          for (const s of catalog.filter(x => x.kind === g.id)) {
            const p = parseV(s.slug), prev = byBase.get(p.base);
            if (!prev || cmpV(p.ver, parseV(prev.slug).ver) > 0) byBase.set(p.base, s);
          }
          const items = [...byBase.values()].sort((a, b) => (a.slug < b.slug ? -1 : a.slug > b.slug ? 1 : 0));
          if (items.length === 0) return null;
          return (
            <div key={g.id} style={{ marginBottom: "var(--tt-space-7)" }}>
              <h2 style={{ marginBottom: "var(--tt-space-2)" }}>{g.label}</h2>
              <p style={{ color: "var(--tt-text-muted)", marginTop: 0, marginBottom: "var(--tt-space-5)" }}>{g.desc}</p>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(440px, 1fr))", gap: "var(--tt-space-4)" }}>
                {items.map(rec => {
                  const refs = (usedBy[rec.slug] || []).length;
                  return (
                    <a
                      key={rec.slug}
                      href={`/schema/${rec.slug}`}
                      onClick={(e) => { e.preventDefault(); setRoute({ name: "schema", slug: rec.slug }); }}
                      style={{
                        padding: "var(--tt-space-4)",
                        border: "1px solid var(--tt-border)",
                        borderLeft: `3px solid ${sharedAccent(rec)}`,
                        borderRadius: "var(--tt-radius)",
                        textDecoration: "none",
                        color: "inherit",
                        background: "var(--tt-surface)"
                      }}
                    >
                      <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", textTransform: "uppercase", marginBottom: "var(--tt-space-2)" }}>
                        {rec.family || sharedKindLabel(rec.kind)}
                      </div>
                      <div style={{ fontFamily: "var(--tt-font-display)", fontSize: "var(--tt-text-md)", marginBottom: "var(--tt-space-2)" }}>{rec.title}</div>
                      <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginBottom: "var(--tt-space-2)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {rec.slug}
                      </div>
                      <div style={{ display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)" }}>
                        {rec.defs.length > 0 && <span>{rec.defs.length} def{rec.defs.length === 1 ? "" : "s"}</span>}
                        <span>{refs} task{refs === 1 ? "" : "s"} reference{refs === 1 ? "s" : ""} this</span>
                      </div>
                    </a>
                  );
                })}
              </div>
            </div>
          );
        })}
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   SPEC PAGE
   ============================================================ */
function SpecPage({ slug, version, id, setRoute }) {
  const task = (slug && window.TT_TASKS.find(t => t.slug === slug && (!version || t.version === version)))
            || (id && window.TT_TASKS.find(t => t.id === id))
            || window.TT_TASKS[0];
  if (!task) {
    return (
      <section className="container">
        <div className="tt-empty" style={{ padding: "var(--tt-space-6)" }}>
          <b>No specifications are currently registered.</b><br />
          Add one under <code>specs/&lt;slug&gt;/&lt;version&gt;/</code> and run <code>npm run build</code>.
        </div>
      </section>
    );
  }
  const typeURI = `https://trusttasks.org/spec/${task.slug}/${task.version}`;
  const [copied, setCopied] = useS(false);
  const [proseHtml, setProseHtml] = useS("");
  const [proseToc, setProseToc] = useS([]);
  const [proseError, setProseError] = useS(null);
  const [activeSection, setActiveSection] = useS("metadata");

  useE(() => {
    if (!task.prosePath) {
      setProseError("This spec has no prosePath; was it built with the registry script?");
      return;
    }
    let cancelled = false;
    setProseHtml(""); setProseToc([]); setProseError(null);
    fetch(task.prosePath, { headers: { "Accept": "text/markdown, text/plain" } })
      .then(r => { if (!r.ok) throw new Error(`Failed to load spec.md (${r.status})`); return r.text(); })
      .then(src => {
        if (cancelled) return;
        if (typeof marked === "undefined") throw new Error("Markdown renderer is unavailable.");
        const body = stripFrontMatter(src);
        marked.setOptions({ gfm: true, breaks: false });
        const rawHtml = marked.parse(body);
        const { html, toc } = injectHeadingIds(rawHtml);
        setProseHtml(html);
        setProseToc(toc);
      })
      .catch(e => { if (!cancelled) setProseError(e.message); });
    return () => { cancelled = true; };
  }, [task.prosePath]);

  // Once the prose is rendered, honor any #fragment in the URL — used by request/response
  // navigation (e.g. /spec/acl/grant/1.0#response scrolls to the Response section).
  useE(() => {
    if (!proseHtml || !location.hash) return;
    const id = location.hash.slice(1);
    const el = document.getElementById(id);
    if (el) {
      // Defer one frame so the layout settles before scrollIntoView.
      requestAnimationFrame(() => el.scrollIntoView({ block: "start" }));
    }
  }, [proseHtml]);

  useE(() => {
    const ids = ["metadata", ...proseToc.map(t => t.id), "schema", "uses", "related"];
    const onScroll = () => {
      for (const sid of ids) {
        const el = document.getElementById(sid);
        if (!el) continue;
        const rect = el.getBoundingClientRect();
        if (rect.top > 100) { setActiveSection(sid); return; }
      }
      setActiveSection(ids[ids.length - 1]);
    };
    window.addEventListener("scroll", onScroll);
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, [task.id, proseToc.length]);

  const cat = window.TT_CATEGORIES.find(c => c.id === task.category);

  return (
    <section className="container container--wide tt-spec">
      <div>
        <div style={{ marginBottom: "var(--tt-space-4)" }}>
          <span className="tt-spec__num">{task.slug} · v{task.version}</span>
        </div>
        <h1 className="tt-spec__title">{task.title}</h1>
        <p className="lead" style={{ marginBottom: "var(--tt-space-5)" }}>{task.summary}</p>

        <div
          className="tt-type-uri"
          style={{
            display: "flex", alignItems: "stretch",
            border: "1px solid var(--tt-border)",
            borderLeft: `3px solid ${catColor(task.category)}`,
            background: "var(--tt-surface-elev)",
            marginBottom: "var(--tt-space-6)",
            fontFamily: "var(--tt-font-mono)",
            fontSize: "var(--tt-text-sm)",
          }}
        >
          <div style={{ padding: "var(--tt-space-3) var(--tt-space-4)", borderRight: "1px solid var(--tt-border)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", fontSize: "var(--tt-text-xs)", textTransform: "uppercase", display: "flex", alignItems: "center" }}>Type URI</div>
          <code style={{ flex: 1, padding: "var(--tt-space-3) var(--tt-space-4)", overflow: "auto", whiteSpace: "nowrap", color: "var(--tt-text)" }}>{typeURI}</code>
          <button
            type="button"
            onClick={() => { navigator.clipboard?.writeText(typeURI); setCopied(true); setTimeout(() => setCopied(false), 1400); }}
            style={{ borderLeft: "1px solid var(--tt-border)", background: "transparent", padding: "0 var(--tt-space-4)", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", textTransform: "uppercase", letterSpacing: "0.06em", color: copied ? catColor(task.category) : "var(--tt-text-muted)", cursor: "pointer" }}
          >
            {copied ? "Copied" : "Copy"}
          </button>
        </div>

        <div className="tt-spec__banner">
          <span><b>Status</b> &nbsp; <TTStatus status={task.status} /></span>
          <span><b>Category</b> &nbsp; <a href="/categories" onClick={(e) => { e.preventDefault(); setRoute({ name: "categories" }); }} style={{ color: catColor(task.category), borderBottom: 0 }}>{cat.name}</a></span>
          <span><b>Updated</b> &nbsp; {task.updated}</span>
          <span><b>Editors</b> &nbsp; {renderAuthorList(task.authors)}</span>
        </div>

        <h2 id="metadata">Metadata</h2>
        <dl className="tt-meta-grid">
          <dt>Slug</dt><dd>{task.slug}</dd>
          <dt>Version</dt>
          <dd>
            {window.TT_TASKS
              .filter(t => t.slug === task.slug)
              .sort((a, b) => { const pa = a.version.split(".").map(Number), pb = b.version.split(".").map(Number); return (pb[0] - pa[0]) || (pb[1] - pa[1]); })
              .map((v, i) => (
                <React.Fragment key={v.version}>
                  {i > 0 && <span style={{ color: "var(--tt-text-muted)" }}> · </span>}
                  {v.version === task.version
                    ? <b>v{v.version}</b>
                    : <a href={`/spec/${v.slug}/${v.version}`} onClick={(e) => { e.preventDefault(); setRoute({ name: "spec", slug: v.slug, version: v.version }); window.scrollTo(0, 0); }}>v{v.version}</a>}
                  {v.status === "retired" && <span style={{ fontSize: "0.8em", color: "var(--tt-text-muted)" }}> (retired)</span>}
                </React.Fragment>
              ))}
          </dd>
          <dt>Type URI</dt><dd><code style={{ fontFamily: "var(--tt-font-mono)", fontSize: "0.95em" }}>{typeURI}</code></dd>
          <dt>Target framework</dt><dd>{task.targetFrameworkVersion ? `trust-task/${task.targetFrameworkVersion}` : "—"}</dd>
          <dt>Parties</dt><dd>{task.parties.join(" ↔ ")}</dd>
          <dt>Proof requirement</dt><dd>{task.proofRequirement ? task.proofRequirement.requirement : "—"}</dd>
          <dt>Category</dt><dd>{cat ? cat.name : task.category}</dd>
          <dt>Keywords</dt><dd>{task.keywords.join(", ")}</dd>
        </dl>

        {proseError && (
          <div className="tt-empty" style={{ padding: "var(--tt-space-5)", margin: "var(--tt-space-5) 0" }}>
            <b>Couldn't load this specification's prose.</b><br />
            {proseError}<br />
            <a href={`https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/specs/${task.slug}/${task.version}/spec.md`} target="_blank" rel="noreferrer">Read it on GitHub →</a>
          </div>
        )}
        {!proseError && !proseHtml && (
          <p style={{ color: "var(--tt-text-muted)" }}>Loading specification…</p>
        )}
        {!proseError && proseHtml && (
          <article className="tt-prose" dangerouslySetInnerHTML={{ __html: proseHtml }} />
        )}

        <h2 id="schema">JSON Schema</h2>
        <p>The normative JSON Schema for this Trust Task's <code>payload</code> member (Draft 2020-12). The outer document structure (<code>id</code>, <code>type</code>, <code>issuer</code>, <code>recipient</code>, <code>issuedAt</code>, <code>expiresAt</code>, <code>proof</code>) is defined by the framework specification.</p>
        <CodeBlock json={task.schema} />

        <h2 id="uses">Shared Schemas Used</h2>
        <p style={{ color: "var(--tt-text-muted)", marginTop: "calc(-1 * var(--tt-space-3))" }}>
          Reusable building blocks this task's payload schema references via <code>$ref</code> — framework primitives, family-level shared definitions, and method extensions.
        </p>
        <UsesPanel uses={task.uses} setRoute={setRoute} />

        <h2 id="related">Related Trust Tasks</h2>
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-3)" }}>
          {(task.related || []).map(rid => {
            const r = window.TT_TASKS.find(t => t.id === rid);
            if (!r) return null;
            return (
              <a key={rid} href={`/spec/${(window.TT_TASKS.find(x=>x.id===rid)||{}).slug}/${(window.TT_TASKS.find(x=>x.id===rid)||{}).version}`} onClick={(e) => { e.preventDefault(); (() => { const rr = window.TT_TASKS.find(x => x.id === rid); setRoute({ name: "spec", slug: rr?.slug, version: rr?.version }); })(); window.scrollTo(0, 0); }}
                 style={{ padding: "var(--tt-space-4)", border: "1px solid var(--tt-border)", borderRadius: "var(--tt-radius)", display: "flex", justifyContent: "space-between", borderBottom: "1px solid var(--tt-border)", textDecoration: "none", color: "inherit" }}>
                <span><span style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", marginRight: "var(--tt-space-3)" }}>{r.slug}</span>{r.title}</span>
                <TTStatus status={r.status} />
              </a>
            );
          })}
          {(!task.related || task.related.length === 0) && <p style={{ color: "var(--tt-text-muted)" }}>No related tasks recorded.</p>}
        </div>

        <div style={{ marginTop: "var(--tt-space-8)", paddingTop: "var(--tt-space-5)", borderTop: "1px solid var(--tt-line)", display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--tt-space-4)" }}>
          <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }} className="btn btn--ghost">← Back to registry</a>
          <a href={`https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/specs/${task.slug}/${task.version}/spec.md`} target="_blank" rel="noreferrer" className="btn btn--ghost">Edit on GitHub →</a>
        </div>
      </div>

      <aside className="tt-spec__sidebar">
        <div className="tt-toc-title">On this page</div>
        <ol className="tt-toc">
          {[
            { id: "metadata", text: "Metadata" },
            ...proseToc,
            { id: "schema", text: "JSON Schema" },
            { id: "uses", text: "Shared Schemas Used" },
            { id: "related", text: "Related Trust Tasks" }
          ].map(({ id: sid, text }) => (
            <li key={sid}><a href={`#${sid}`} className={activeSection === sid ? "active" : ""}>{text}</a></li>
          ))}
        </ol>
      </aside>
    </section>
  );
}

/* ============================================================
   CATEGORIES
   ============================================================ */
function CategoriesPage({ setRoute }) {
  const counts = useM(() => countSlugsByCategory(window.TT_TASKS), []);
  return (
    <React.Fragment>
      <PageHero
        eyebrow="Browse"
        title="Categories"
        lede="Every Trust Task belongs to a category — the broad domain in which the task makes sense. Pick a category to scope the registry."
      />
      <section>
        <div className="container">
          <div className="tt-cat-grid">
            {window.TT_CATEGORIES.map(c => (
              <a
                key={c.id}
                className="tt-cat-tile"
                style={{ "--accent": catColor(c.id) }}
                href={`/registry/${c.id}`}
                onClick={(e) => { e.preventDefault(); setRoute({ name: "registry", cat: c.id }); }}
              >
                <span className="tt-cat-tile__count">{counts[c.id] || 0} {(counts[c.id] || 0) === 1 ? "spec" : "specs"}</span>
                <h3>{c.name}</h3>
                <p style={{ margin: 0 }}>{c.blurb}</p>
              </a>
            ))}
          </div>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   ABOUT
   ============================================================ */
function AboutPage() {
  return (
    <React.Fragment>
      <PageHero
        eyebrow="About"
        title="What is a Trust Task?"
        lede="A specification framework for the verifiable work that happens between two or more parties — self-contained, transport-agnostic, JSON-based."
      />
      <section>
        <div className="container">
          <h2>Three properties.</h2>
          <p className="lead">Every Trust Task definition adheres to three properties. They are non-negotiable; together they make the task portable, durable, and unambiguous.</p>

          <div className="grid grid--3" style={{ marginTop: "var(--tt-space-6)" }}>
            <article className="card card--accent card--coral">
              <div className="card__index">01 · Self-contained</div>
              <h4>Everything in one document.</h4>
              <p>A Trust Task contains all relevant information needed to complete the task within the definition itself — parties, scope, criteria, schema. No hidden context.</p>
            </article>
            <article className="card card--accent card--teal">
              <div className="card__index">02 · Transport-agnostic</div>
              <h4>Indifferent to delivery.</h4>
              <p>The definition does not assume any particular protocol or channel. DIDComm, HTTP, message queue, paper — the task is the task.</p>
            </article>
            <article className="card card--accent card--violet">
              <div className="card__index">03 · JSON-based</div>
              <h4>One canonical encoding.</h4>
              <p>JSON is the single normative serialization. Other encodings MAY be derived; the JSON document is authoritative.</p>
            </article>
          </div>

          <hr className="protocol-rule" aria-hidden="true" />

          <h2>Why a registry?</h2>
          <p>
            Two parties only achieve interoperability when they agree on the shape of the task they're cooperating on.
            A central registry of Trust Task specifications gives implementers a finite, well-known vocabulary —
            the same way IANA registries serve the broader internet.
          </p>
          <p>
            This site is the canonical reference. Each specification is editable on GitHub; the registry index here
            is generated from the source repository.
          </p>

          <h2>Who runs this?</h2>
          <p>
            The Trust Tasks specification is developed under the <a href="https://trustoverip.org" target="_blank" rel="noreferrer">Trust Over IP</a> Digital Trust Graph Working Group (DTGWG), as a task force.
            Membership is open; contribution happens in the open via the GitHub repository.
          </p>
          <p>
            Trust Tasks is one layer of a wider verifiable-trust stack alongside DTG Credentials, Verifiable Trust
            Infrastructure, and OpenVTC. See the <a href="/ecosystem" onClick={(e) => { e.preventDefault(); window.history.pushState(null, "", "/ecosystem"); window.dispatchEvent(new PopStateEvent("popstate")); }}>ecosystem page</a> for how they fit together.
          </p>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   ECOSYSTEM
   ============================================================ */
function EcosystemPage({ setRoute }) {
  const projects = window.TT_ECOSYSTEM || [];
  const accentColor = (a) => `var(--tt-${a})`;

  return (
    <React.Fragment>
      <PageHero
        eyebrow="Ecosystem"
        title="The verifiable-trust stack."
        lede="Trust Tasks doesn't stand alone. It composes with a small set of related specifications and reference implementations that together let two parties exchange verifiable work — and a community of parties build a graph of trust around it."
      />

      <section>
        <div className="container">
          <h2>How the layers fit.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            Each project is independently usable; together they're a complete stack.
          </p>

          <div className="grid grid--4" style={{ marginTop: "var(--tt-space-5)", gap: "var(--tt-space-4)" }}>
            <article className="card" style={{ borderTop: `3px solid ${accentColor("violet")}` }}>
              <div className="card__index" style={{ color: accentColor("violet") }}>Layer 01</div>
              <h4>Vocabulary</h4>
              <p>What two parties exchange. Typed, JSON, transport-agnostic.</p>
              <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginTop: "var(--tt-space-2)" }}>Trust Tasks</div>
            </article>
            <article className="card" style={{ borderTop: `3px solid ${accentColor("teal")}` }}>
              <div className="card__index" style={{ color: accentColor("teal") }}>Layer 02</div>
              <h4>Credentials</h4>
              <p>What participants carry. Six W3C VC types from the DTG.</p>
              <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginTop: "var(--tt-space-2)" }}>DTG Credentials</div>
            </article>
            <article className="card" style={{ borderTop: `3px solid ${accentColor("coral")}` }}>
              <div className="card__index" style={{ color: accentColor("coral") }}>Layer 03</div>
              <h4>Infrastructure</h4>
              <p>What holds keys and authorizes operations. The VTA.</p>
              <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginTop: "var(--tt-space-2)" }}>Verifiable Trust Infrastructure</div>
            </article>
            <article className="card" style={{ borderTop: `3px solid ${accentColor("amber")}` }}>
              <div className="card__index" style={{ color: accentColor("amber") }}>Layer 04</div>
              <h4>Tooling</h4>
              <p>What developers use to participate in a community.</p>
              <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginTop: "var(--tt-space-2)" }}>OpenVTC</div>
            </article>
          </div>

          <hr className="protocol-rule" aria-hidden="true" />

          <h2>Projects.</h2>

          <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-5)", marginTop: "var(--tt-space-5)" }}>
            {projects.map(p => (
              <article
                key={p.id}
                style={{
                  display: "grid",
                  gridTemplateColumns: "minmax(0, 1fr)",
                  border: "1px solid var(--tt-border)",
                  borderLeft: `3px solid ${accentColor(p.accent)}`,
                  background: p.self ? "var(--tt-surface-elev)" : "transparent",
                  padding: "var(--tt-space-5) var(--tt-space-6)",
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline", flexWrap: "wrap", gap: "var(--tt-space-3)", marginBottom: "var(--tt-space-3)" }}>
                  <div>
                    <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: accentColor(p.accent), marginBottom: "var(--tt-space-1)" }}>
                      {p.role}{p.self ? " · this site" : ""}
                    </div>
                    <h3 style={{ margin: 0 }}>{p.name}</h3>
                    <div style={{ fontFamily: "var(--tt-font-serif, var(--tt-font-display))", fontStyle: "italic", color: "var(--tt-text-muted)", marginTop: "var(--tt-space-1)" }}>{p.tagline}</div>
                  </div>
                  <span className="pill" style={{ borderColor: accentColor(p.accent), color: accentColor(p.accent), textTransform: "capitalize" }}>
                    <span className="dot" style={{ background: accentColor(p.accent) }}></span>
                    {p.tier}
                  </span>
                </div>

                <p style={{ color: "var(--tt-text-muted)", marginTop: 0 }}>{p.summary}</p>

                {p.bullets && p.bullets.length > 0 && (
                  <ul style={{ margin: "var(--tt-space-3) 0 var(--tt-space-4)", paddingLeft: "1.1em", color: "var(--tt-text-muted)", lineHeight: 1.7 }}>
                    {p.bullets.map(b => <li key={b}>{b}</li>)}
                  </ul>
                )}

                <div style={{ display: "flex", gap: "var(--tt-space-4)", flexWrap: "wrap", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase" }}>
                  {p.primary && (
                    <a href={p.primary.href} target="_blank" rel="noreferrer" style={{ color: accentColor(p.accent), borderBottom: 0 }}>
                      {p.primary.label} →
                    </a>
                  )}
                  {p.repo && (
                    <a href={p.repo} target="_blank" rel="noreferrer" style={{ color: "var(--tt-text-muted)", borderBottom: 0 }}>
                      {p.repo.replace("https://", "")} →
                    </a>
                  )}
                  {p.spec && (
                    <a href={p.spec.href} target="_blank" rel="noreferrer" style={{ color: "var(--tt-text-muted)", borderBottom: 0 }}>
                      {p.spec.label} →
                    </a>
                  )}
                </div>
              </article>
            ))}
          </div>

          <hr className="protocol-rule" aria-hidden="true" />

          <h2>Adding a project.</h2>
          <p>
            The ecosystem grows. If you maintain a specification, library, or service that builds on
            Trust Tasks — or that Trust Tasks builds on — open a PR against{" "}
            <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">the registry repository</a>{" "}
            adding your project to <code style={{ fontFamily: "var(--tt-font-mono)", fontSize: "0.95em" }}>assets/ecosystem.js</code>.
            Listings are reviewed by the DTGWG task force; the bar is alignment with the
            self-contained, transport-agnostic, JSON-based principles.
          </p>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   CONTRIBUTING
   ============================================================ */
function ContributingPage() {
  return (
    <React.Fragment>
      <PageHero
        eyebrow="Contributing"
        title="Propose a new Trust Task."
        lede="The registry grows through proposal, review, and ratification — entirely in the open, on GitHub. Here's the path from idea to published spec."
      />
      <section>
        <div className="container">
          <h2>The lifecycle.</h2>
          <ol style={{ paddingLeft: "1.2em", lineHeight: 1.9, color: "var(--tt-text-muted)" }}>
            <li><b style={{ color: "var(--tt-text)" }}>Propose.</b> Open an issue on the <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues" target="_blank" rel="noreferrer">repository</a> describing the task: parties, motivation, prior art. The task force triages within two weeks.</li>
            <li><b style={{ color: "var(--tt-text)" }}>Draft.</b> Once accepted, fork the spec template and submit a pull request. The task is assigned a slug (e.g. <code>kyc-handoff</code>) and lands in the registry as <i>draft</i>.</li>
            <li><b style={{ color: "var(--tt-text)" }}>Review.</b> Public review during DTGWG meetings. Two implementations from independent parties are required before promotion.</li>
            <li><b style={{ color: "var(--tt-text)" }}>Candidate.</b> Once implementations are demonstrated, the spec moves to <i>candidate</i> status. The schema is frozen except for clarifications.</li>
            <li><b style={{ color: "var(--tt-text)" }}>Standard.</b> After a 90-day stability window with no breaking changes, the candidate is promoted to <i>standard</i>. Future revisions follow a <code style={{ fontFamily: "var(--tt-font-mono)" }}>major.minor</code> version scheme: minor bumps are backwards-compatible additions, major bumps indicate breaking changes.</li>
          </ol>

          <hr className="protocol-rule" aria-hidden="true" />

          <h2>What makes a good Trust Task?</h2>
          <ul style={{ lineHeight: 1.9, color: "var(--tt-text-muted)" }}>
            <li>Two or more identifiable parties with a clear role in the task.</li>
            <li>A specific, finite outcome — &ldquo;the task is complete when X.&rdquo;</li>
            <li>A JSON schema small enough to be implemented in an afternoon.</li>
            <li>No assumptions about transport, framework, or vendor.</li>
            <li>An obvious place in the existing taxonomy (or a strong argument for a new category).</li>
          </ul>

          <div style={{ marginTop: "var(--tt-space-7)", display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap" }}>
            <a className="btn btn--primary" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">Open the repository →</a>
            <a className="btn btn--ghost" href="https://trustoverip.org/get-involved/membership/" target="_blank" rel="noreferrer">Join ToIP</a>
          </div>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   GLOSSARY
   ============================================================ */
function GlossaryPage() {
  const terms = [
    ["Trust Task", "A self-contained, transport-agnostic, JSON-based specification for verifiable work between two or more parties."],
    ["Entity", "A participant in a Trust Task. Identified by a Verifiable Identifier (VID); may be a natural person, legal person, or autonomous agent."],
    ["Verifiable Identifier (VID)", "A string identifier whose controller is verifiable under a trust framework. DIDs are one realization; others include X.509 subjects, OIDC subject identifiers, and key thumbprints. The framework does not constrain the VID scheme."],
    ["Document identifier", "The globally unique string carried in a Trust Task document's id member. UUIDv4 is the recommended default; any unique string is permitted."],
    ["Thread identifier", "The optional string carried in a Trust Task document's threadId member that correlates the document with others in the same logical exchange (e.g. a response back to its originating request)."],
    ["Proof", "An optional W3C Data Integrity Proof attached to a Trust Task document via the proof member, binding the document's content to its issuer."],
    ["Initiator", "The entity that proposes a task. Responsible for drafting scope, criteria, and deadline."],
    ["Counterparty", "The entity that accepts and performs a task proposed by an initiator."],
    ["Schema", "A JSON Schema (Draft 2020-12) document that constrains the shape of a Trust Task instance. Normative."],
    ["Conformance", "A producer or consumer of a Trust Task is conformant if it adheres to the requirements stated in §4 of the relevant specification."],
    ["Status", "The maturity level of a specification: draft (working), candidate (frozen, two implementations), or standard (stable, 90 days unchanged)."],
    ["Verification", "The act of confirming a task is complete. Both parties co-sign the result; the verification is itself portable evidence."],
    ["Transport", "Any channel that conveys a Trust Task instance from one party to another. The specification is indifferent to the choice."],
    ["Trust Registry", "A queryable directory of authorized entities and their roles within a governance framework. See the trust-registry-query specification."],
  ];
  return (
    <React.Fragment>
      <PageHero
        eyebrow="Glossary"
        title="Terms used across the registry."
        lede="Working definitions of recurring concepts. Where these conflict with a specification's local definition, the local definition wins."
      />
      <section>
        <div className="container">
          <dl className="tt-glossary">
            {terms.map(([term, def]) => (
              <div key={term}>
                <dt>{term}</dt>
                <dd>{def}</dd>
              </div>
            ))}
          </dl>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   FRAMEWORK SPEC (renders SPEC.md as HTML)
   ============================================================ */
function FrameworkSpecPage({ setRoute }) {
  const [html, setHtml] = useS("");
  const [error, setError] = useS(null);

  useE(() => {
    let cancelled = false;
    fetch("/SPEC.md", { headers: { "Accept": "text/markdown, text/plain" } })
      .then(r => {
        if (!r.ok) throw new Error(`Failed to load specification (${r.status})`);
        return r.text();
      })
      .then(text => {
        if (cancelled) return;
        if (typeof marked === "undefined") throw new Error("Markdown renderer is unavailable.");
        marked.setOptions({ gfm: true, breaks: false });
        const rawHtml = marked.parse(text);
        const { html: withIds } = injectHeadingIds(rawHtml);
        setHtml(withIds);
      })
      .catch(e => { if (!cancelled) setError(e.message); });
    return () => { cancelled = true; };
  }, []);

  useE(() => {
    if (!html || !location.hash) return;
    const id = location.hash.slice(1);
    const el = document.getElementById(id);
    if (el) {
      // Defer one frame so layout settles before we scroll.
      requestAnimationFrame(() => el.scrollIntoView({ block: "start" }));
    }
  }, [html]);

  return (
    <React.Fragment>
      <PageHero
        eyebrow="Framework specification"
        title="The Trust Tasks framework"
        lede="The normative framework specification — document structure, version scheme, namespace, error responses, and transport bindings — rendered from SPEC.md in the repository."
      >
        <div style={{ display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap", marginTop: "var(--tt-space-4)" }}>
          <a className="btn btn--ghost" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md" target="_blank" rel="noreferrer">View on GitHub →</a>
          <a className="btn btn--ghost" href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }}>Browse the registry →</a>
        </div>
      </PageHero>

      <section style={{ paddingBlock: "var(--tt-space-6)" }}>
        <div className="container">
          {error && (
            <div className="tt-empty" style={{ padding: "var(--tt-space-6)" }}>
              <b>Couldn't load the specification.</b><br />
              {error}<br />
              <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md" target="_blank" rel="noreferrer">Read it on GitHub →</a>
            </div>
          )}
          {!error && !html && (
            <div className="tt-empty" style={{ padding: "var(--tt-space-6)" }}>Loading specification…</div>
          )}
          {!error && html && (
            <article className="tt-prose" dangerouslySetInnerHTML={{ __html: html }} />
          )}
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   IMPLEMENTATIONS — reference Rust crates + quickstart
   ============================================================ */
function ImplementationsPage({ setRoute }) {
  const accent = (c) => `var(--tt-${c})`;

  const crates = [
    {
      name: "trust-tasks-rs",
      accent: "violet",
      role: "Core library",
      tagline: "Framework primitives, types, dispatcher.",
      summary:
        "The reference implementation of SPEC.md §4–§11 — TrustTask<P> envelopes, TypeUri parser, Proof/ProofVerifier traits, the typed error pipeline, TransportHandler with §4.8.1 identity precedence baked in, a discovery module, and generated payload types for every spec in the registry.",
      bullets: [
        "TrustTask<P>, TypeUri, Proof, RejectReason, ErrorPayload",
        "TransportHandler trait + NoopHandler / InMemoryHandler",
        "Dispatcher<R> for typed multi-spec consumers",
        "specs::* — generated payload types for every registry spec",
        "Optional validate feature — JSON Schema check against embedded schemas",
      ],
      repo: "https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-rs",
    },
    {
      name: "trust-tasks-https",
      accent: "teal",
      role: "HTTPS binding",
      tagline: "Typed server + client over HTTP.",
      summary:
        "An axum-based HttpsServer with a builder API and bearer-token authentication, plus a reqwest-based HttpsClient that produces typed responses or trust-task-error/0.1 documents. Runs the full SPEC §7.2 pipeline per request (resolve_parties → validate_basic → enforce_audience_binding → dispatch → handler) and applies §8.1 routing on rejection.",
      bullets: [
        "HttpsServer builder — .on::<Payload, Response, _>(handler)",
        "Optional discovery wiring — .enable_discovery() snapshots the route table",
        "HttpsClient::send::<Req, Resp>() — typed end-to-end",
        "Bearer auth via pluggable Auth trait; BearerAuth for demos",
        "ClientError separates transport / framework error / non-2xx fallback",
      ],
      repo: "https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-https",
    },
    {
      name: "trust-tasks-didcomm",
      accent: "coral",
      role: "DIDComm v2.1 binding",
      tagline: "pack/unpack over authcrypt'd JWEs.",
      summary:
        "A DIDComm v2.1 binding on top of affinidi-messaging-didcomm. pack_trust_task wraps a TrustTask in an authcrypt envelope; unpack_trust_task returns the document together with a DidcommHandler whose authenticated peer is the verified sender_kid — the framework's transport-authenticated identity.",
      bullets: [
        "pack_trust_task / unpack_trust_task helpers",
        "DidcommHandler maps verified sender_kid → transport peer",
        "Envelope type: https://trusttasks.org/binding/didcomm/0.1/envelope",
        "Mediator end-to-end test against affinidi-messaging-test-mediator",
      ],
      repo: "https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-didcomm",
    },
    {
      name: "trust-tasks-proof",
      accent: "amber",
      role: "Proof verifiers",
      tagline: "Pluggable ProofVerifier impls behind Cargo features.",
      summary:
        "Umbrella crate hosting concrete ProofVerifier implementations for the framework's seam. The default-enabled `affinidi` feature ships a W3C Data Integrity backend on top of affinidi-data-integrity (EdDSA suites: eddsa-rdfc-2022, eddsa-jcs-2022) plus a CachedDidResolver that bridges affinidi-did-resolver-cache-sdk so did:web / did:webvh / did:peer / did:jwk / did:key all resolve through one adapter. Future backends slot in as siblings under `trust_tasks_proof::<backend>::Verifier`.",
      bullets: [
        "`affinidi` feature (default) — affinidi::Verifier + affinidi::CachedDidResolver",
        "affinidi::Verifier::for_did_key() — offline, no I/O",
        "affinidi::Verifier::with_resolver(...) — pluggable DID resolution",
        "DataIntegrityError → VerificationError → SPEC §8.3 proof_invalid",
        "Round-trip-tested against affinidi-data-integrity's sign path",
      ],
      repo: "https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-proof",
    },
  ];

  const cargoToml = `[dependencies]
trust-tasks-rs = { git = "https://github.com/trustoverip/dtgwg-trust-tasks-tf" }

# Pick the transport binding(s) you need:
trust-tasks-https   = { git = "https://github.com/trustoverip/dtgwg-trust-tasks-tf" }
trust-tasks-didcomm = { git = "https://github.com/trustoverip/dtgwg-trust-tasks-tf" }

# Optional: W3C Data Integrity proof verification. The default feature
# pulls in the Affinidi backend; use default-features = false for a bare
# umbrella ready to receive other backends.
trust-tasks-proof = { git = "https://github.com/trustoverip/dtgwg-trust-tasks-tf" }`;

  const loopbackSnippet = `use chrono::Utc;
use trust_tasks_rs::{
    handlers::InMemoryHandler, RejectReason, TransportHandler, TrustTask, TypeUri,
};

const VERIFIER: &str = "did:web:verifier.example";
const BANK: &str = "did:web:bank.example";

// Both ends know who they are; the in-memory handler conveys this as
// transport-authenticated identity.
let producer = InMemoryHandler::new().with_local(VERIFIER).with_peer(BANK);
let consumer = InMemoryHandler::new().with_local(BANK).with_peer(VERIFIER);

// Producer issues a kyc-handoff/1.0 request.
let mut request = TrustTask::new(
    "req-001",
    TypeUri::canonical("kyc-handoff", 1, 0)?,
    KycHandoff { /* payload */ },
);
request.issuer    = Some(VERIFIER.into());
request.recipient = Some(BANK.into());
request.issued_at = Some(Utc::now());
producer.prepare_outbound(&mut request);

// Consumer applies §4.8.1 + §7.2 and either responds or rejects.
let resolved = consumer.resolve_parties(&request)
    .map_err(|e| request.reject_with("err-001", RejectReason::from(e)))?;
request.validate_basic(Utc::now(), BANK)
    .map_err(|reason| request.reject_with("err-001", reason))?;

let response = request.respond_with("resp-001", KycReceipt { /* ... */ });`;

  const httpsServerSnippet = `use trust_tasks_https::{BearerAuth, HttpsServer};
use trust_tasks_rs::specs::acl::{grant, revoke};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = BearerAuth::from_pairs([
        ("alice", "did:web:alice.example"),
        ("bob",   "did:web:bob.example"),
    ]);

    let server = HttpsServer::builder()
        .local_vid("did:web:maintainer.example")
        .with_auth(auth)
        .on::<grant::v0_1::Payload, grant::v0_1::Response, _>(|req, ctx| {
            // Typed payload, authenticated sender in ctx.
            Ok(grant::v0_1::Response { entry: req.payload.entry.clone() })
        })
        .on::<revoke::v0_1::Payload, revoke::v0_1::Response, _>(|req, _ctx| {
            Ok(revoke::v0_1::Response { entry: None })
        })
        .enable_discovery()    // serves trust-task-discovery/0.1 on the same endpoint
        .build();

    server.serve("127.0.0.1:3000").await?;
    Ok(())
}`;

  const httpsClientSnippet = `use trust_tasks_https::{ClientError, HttpsClient};
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, TrustTask};

let client = HttpsClient::builder()
    .server_url("http://localhost:3000")
    .server_vid("did:web:maintainer.example")
    .my_vid("did:web:alice.example")
    .my_token("alice")
    .build()?;

let request = TrustTask::for_payload(
    format!("urn:uuid:{}", uuid::Uuid::new_v4()),
    grant::Payload { /* ... */ },
);

match client.send::<grant::Payload, grant::Response>(request).await {
    Ok(resp) => { /* typed response */ }
    Err(ClientError::TrustTaskError { http_status, error }) => {
        // Framework trust-task-error/0.1 with typed code + retryable flag.
    }
    Err(other) => return Err(other.into()),
}`;

  const didcommSnippet = `use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent};
use trust_tasks_didcomm::{pack_trust_task, unpack_trust_task};
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, TransportHandler, TrustTask};

// 1. Two fresh peer identities; cross-register them.
let alice = PrivateIdentity::generate("did:peer:alice");
let bob   = PrivateIdentity::generate("did:peer:bob");
let mut alice_agent = DIDCommAgent::new();
alice_agent.add_identity(alice.clone());
alice_agent.add_peer(bob.to_resolved());

// 2. Alice packs an acl/grant request as an authcrypt'd JWE.
let request = TrustTask::for_payload("urn:uuid:...", grant::Payload { /* ... */ });
let wire = pack_trust_task(&request, &alice_agent, &alice.did, &bob.did)?;

// 3. Bob unpacks; handler.peer() is the verified sender_kid.
let (received, handler) = unpack_trust_task::<grant::Payload>(
    &wire, &bob_agent, Some(&alice.did),
)?;
let resolved = handler.resolve_parties(&received)?;
received.validate_basic(chrono::Utc::now(), &bob.did)?;
received.enforce_audience_binding()?;`;

  const proofSnippet = `use trust_tasks_proof::affinidi::Verifier;
use trust_tasks_rs::ProofVerifier;

// did:key only — offline, no I/O. Good for tests and self-issued documents.
let verifier = Verifier::for_did_key();
verifier.verify(&inbound_doc).await?;

// For did:web / did:webvh / did:peer / did:jwk, plug in the resolver cache:
use std::sync::Arc;
use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use trust_tasks_proof::affinidi::CachedDidResolver;

let client   = DIDCacheClient::new(DIDCacheConfigBuilder::default().build()).await?;
let resolver = Arc::new(CachedDidResolver::new(Arc::new(client)));
let verifier = Verifier::with_resolver(resolver);`;

  return (
    <React.Fragment>
      <PageHero
        eyebrow="Reference implementation · Rust"
        title="trust-tasks for Rust."
        lede="A reference Rust implementation of the Trust Tasks framework — five crates that together cover the framework envelope, two transport bindings, a W3C Data Integrity proof verifier, and a codegen tool that turns registry specs into typed payload modules. Pre-publication 0.1.0, tracking SPEC.md 0.1."
      >
        <div style={{ display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap", marginTop: "var(--tt-space-4)" }}>
          <a className="btn btn--primary" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">Source on GitHub →</a>
          <a className="btn btn--ghost" href="/specification" onClick={(e) => { e.preventDefault(); setRoute({ name: "specification" }); }}>Read the framework specification →</a>
        </div>
      </PageHero>

      {/* CRATE OVERVIEW */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow" style={{ marginBottom: "var(--tt-space-4)", display: "inline-flex" }}>Workspace at a glance</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>Four publishable crates, one codegen tool.</h2>
          <p style={{ color: "var(--tt-text-muted)", maxWidth: "60ch" }}>
            Each crate is independently usable. Start with <code>trust-tasks-rs</code>;
            add the transport binding(s) and proof verifier you need.
          </p>

          <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-4)", marginTop: "var(--tt-space-6)" }}>
            {crates.map(c => (
              <article
                key={c.name}
                style={{
                  border: "1px solid var(--tt-border)",
                  borderLeft: `3px solid ${accent(c.accent)}`,
                  padding: "var(--tt-space-5) var(--tt-space-6)",
                  background: "var(--tt-surface-elev)",
                }}
              >
                <div style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: "var(--tt-space-4)", flexWrap: "wrap", marginBottom: "var(--tt-space-3)" }}>
                  <div>
                    <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: accent(c.accent), marginBottom: "var(--tt-space-1)" }}>
                      {c.role}
                    </div>
                    <h3 style={{ margin: 0, fontFamily: "var(--tt-font-mono)" }}>{c.name}</h3>
                    <div style={{ fontFamily: "var(--tt-font-serif, var(--tt-font-display))", fontStyle: "italic", color: "var(--tt-text-muted)", marginTop: "var(--tt-space-1)" }}>
                      {c.tagline}
                    </div>
                  </div>
                  <a href={c.repo} target="_blank" rel="noreferrer" style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: accent(c.accent), borderBottom: 0, whiteSpace: "nowrap" }}>
                    Source →
                  </a>
                </div>
                <p style={{ color: "var(--tt-text-muted)", marginTop: 0 }}>{linkifySpec(c.summary, setRoute)}</p>
                <ul style={{ margin: "var(--tt-space-3) 0 0", paddingLeft: "1.1em", color: "var(--tt-text-muted)", lineHeight: 1.7 }}>
                  {c.bullets.map(b => <li key={b}>{linkifySpec(b, setRoute)}</li>)}
                </ul>
              </article>
            ))}
          </div>

          <p style={{ color: "var(--tt-text-muted)", marginTop: "var(--tt-space-5)", fontSize: "var(--tt-text-sm)" }}>
            <b style={{ color: "var(--tt-text)" }}>trust-tasks-codegen</b> — internal-only build tool that reads <code>specs/&lt;slug&gt;/&lt;version&gt;/payload.schema.json</code>, runs each through <a href="https://crates.io/crates/typify" target="_blank" rel="noreferrer">typify</a>, and writes per-spec Rust modules into <code>trust-tasks-rs/src/specs/</code>. Output is committed; a CI gate runs the generator and <code>git diff --exit-code</code> to prevent drift. You don't depend on this crate directly — it runs to refresh the generated tree when a new spec lands in the registry.
          </p>
        </div>
      </section>

      <hr className="protocol-rule container" aria-hidden="true" />

      {/* QUICKSTART */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow">Quickstart</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>Add to your Cargo.toml.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            The crates aren't on crates.io yet — pull them as git dependencies until the first publish. MSRV is 1.94.
          </p>
          <CodeBlock json={cargoToml} language="toml" />

          <h2 style={{ marginTop: "var(--tt-space-7)" }}>Minimal loopback: producer ↔ consumer in-process.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            The example below uses <code>InMemoryHandler</code> to convey transport-authenticated identity without an actual transport.
            It exercises both branches of SPEC <SpecRef section="4.4.1" setRoute={setRoute}>§4.4.1</SpecRef> — success and rejection — and applies the <SpecRef section="4.8.1" setRoute={setRoute}>§4.8.1</SpecRef> / <SpecRef section="7.2" setRoute={setRoute}>§7.2</SpecRef> pipeline.
            See <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-rs/examples/loopback.rs" target="_blank" rel="noreferrer"><code>trust-tasks-rs/examples/loopback.rs</code></a> for the full file (runs with <code>cargo run --example loopback -p trust-tasks-rs</code>).
          </p>
          <CodeBlock json={loopbackSnippet} language="rust" />
        </div>
      </section>

      <hr className="protocol-rule container" aria-hidden="true" />

      {/* HTTPS BINDING */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow">HTTPS transport</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>Typed server and client.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            <code>HttpsServer</code> exposes a single <code>POST /trust-tasks</code> endpoint, runs the full SPEC <SpecRef section="7.2" setRoute={setRoute}>§7.2</SpecRef> pipeline per request,
            and routes by canonical Type URI. Handlers receive a typed <code>TrustTask&lt;Payload&gt;</code> and return either a typed
            response or a <code>RejectReason</code> that maps to a <code>trust-task-error/0.1</code> document with <SpecRef section="8.1" setRoute={setRoute}>§8.1</SpecRef> routing applied.
          </p>
          <CodeBlock json={httpsServerSnippet} language="rust" />

          <h3 style={{ marginTop: "var(--tt-space-6)" }}>Client side.</h3>
          <p style={{ color: "var(--tt-text-muted)" }}>
            <code>HttpsClient::send::&lt;Req, Resp&gt;()</code> returns a typed response; <code>ClientError</code> distinguishes
            transport-level failures from framework <code>trust-task-error/0.1</code> documents from untyped non-2xx fallbacks.
          </p>
          <CodeBlock json={httpsClientSnippet} language="rust" />

          <p style={{ color: "var(--tt-text-muted)", marginTop: "var(--tt-space-4)", fontSize: "var(--tt-text-sm)" }}>
            Pair the two examples: <code>cargo run -p trust-tasks-https --example server_demo</code> in one terminal,
            then <code>cargo run -p trust-tasks-https --example client_demo</code> in another.
          </p>
        </div>
      </section>

      <hr className="protocol-rule container" aria-hidden="true" />

      {/* DIDCOMM BINDING */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow">DIDComm v2.1 transport</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>Authcrypt'd over JWEs.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            <code>pack_trust_task</code> wraps a <code>TrustTask</code> in a DIDComm v2.1 authcrypt envelope.
            <code>unpack_trust_task</code> returns the document together with a <code>DidcommHandler</code> whose
            authenticated peer is the verified <code>sender_kid</code> — exactly what the framework's
            <SpecRef section="4.8.1" setRoute={setRoute}>§4.8.1</SpecRef> precedence rule consumes. Full mediator-routed flow is covered by the crate's
            <code>mediator_e2e.rs</code> integration test.
          </p>
          <CodeBlock json={didcommSnippet} language="rust" />

          <p style={{ color: "var(--tt-text-muted)", marginTop: "var(--tt-space-4)", fontSize: "var(--tt-text-sm)" }}>
            Run the full in-process example: <code>cargo run -p trust-tasks-didcomm --example local_roundtrip</code>.
          </p>
        </div>
      </section>

      <hr className="protocol-rule container" aria-hidden="true" />

      {/* PROOF VERIFICATION */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow">Proof verification</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>W3C Data Integrity via Affinidi.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            The core <code>trust-tasks-rs</code> crate intentionally ships <em>no</em> cryptosuites — verification
            is a trait seam. <code>trust-tasks-proof</code> is the umbrella that hosts concrete implementations behind Cargo features;
            its default-enabled <code>affinidi</code> feature exposes <code>trust_tasks_proof::affinidi::Verifier</code>,
            backed by <a href="https://crates.io/crates/affinidi-data-integrity" target="_blank" rel="noreferrer"><code>affinidi-data-integrity</code></a> for EdDSA cryptosuites
            (<code>eddsa-rdfc-2022</code>, <code>eddsa-jcs-2022</code>) and a <code>CachedDidResolver</code> adapter that
            covers <code>did:web</code>, <code>did:webvh</code>, <code>did:peer</code>, <code>did:jwk</code>, and <code>did:key</code>.
          </p>
          <CodeBlock json={proofSnippet} language="rust" />

          <p style={{ color: "var(--tt-text-muted)", marginTop: "var(--tt-space-4)", fontSize: "var(--tt-text-sm)" }}>
            All <code>DataIntegrityError</code> variants map cleanly into the SPEC <SpecRef section="8.3" setRoute={setRoute}>§8.3</SpecRef> <code>proof_invalid</code> standard code path.
          </p>
        </div>
      </section>

      <hr className="protocol-rule container" aria-hidden="true" />

      {/* ADDING A SPEC */}
      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow">Extending the library</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>Adding your own Trust Task spec.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            Drop a <code>specs/&lt;slug&gt;/&lt;version&gt;/payload.schema.json</code> + <code>spec.md</code> into the registry,
            then run the codegen:
          </p>
          <CodeBlock
            json={`cargo run -p trust-tasks-codegen
git diff --exit-code trust-tasks-rs/src/specs   # CI gate: codegen is idempotent`}
            language="bash"
          />
          <p style={{ color: "var(--tt-text-muted)", marginTop: "var(--tt-space-4)" }}>
            The generator walks <code>specs/</code>, normalises <code>$defs</code> → <code>definitions</code> for typify,
            emits one module per <code>(slug, version)</code>, and harvests <code>## Request</code> / <code>## Response</code>
            JSON code fences from the prose into a <code>#[cfg(test)] mod conformance</code> per module. Front matter
            (e.g. <code>bearer: true</code>) is reflected in trait impls (<code>const IS_BEARER: bool = true;</code>).
          </p>
        </div>
      </section>

      {/* STATUS */}
      <section style={{ paddingBlock: "var(--tt-space-7)", borderTop: "1px solid var(--tt-line)" }}>
        <div className="container">
          <span className="eyebrow">Status</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>Pre-publication.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            All four crates are at <code>0.1.0</code>, tracking <code>SPEC.md 0.1</code>. Neither the framework nor the
            implementation has gone through external review yet — interfaces and on-the-wire behaviour are subject
            to change. The repository ships full unit + integration suites (125 tests workspace-wide),
            a CI matrix on stable Rust (MSRV 1.94), and runnable examples for every binding.
          </p>
          <div style={{ marginTop: "var(--tt-space-5)", display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap" }}>
            <a className="btn btn--primary" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">Open the repository →</a>
            <a className="btn btn--ghost" href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }}>Browse the registry →</a>
            <a className="btn btn--ghost" href="/ecosystem" onClick={(e) => { e.preventDefault(); setRoute({ name: "ecosystem" }); }}>See the ecosystem →</a>
          </div>
        </div>
      </section>
    </React.Fragment>
  );
}

/* ============================================================
   BINDINGS — transport-binding hub + per-binding detail page
   ============================================================ */
function BindingsPage({ setRoute }) {
  const bindings = window.TT_BINDINGS || [];
  const accent = (c) => `var(--tt-${c})`;

  return (
    <React.Fragment>
      <PageHero
        eyebrow="Transport bindings"
        title="How Trust Tasks travel."
        lede="A transport binding is the integration layer between the framework's transport-agnostic semantics and a particular wire. Each binding specifies its identity-mapping rules, error mapping, and (where the transport needs one) envelope grammar — so the framework's validation pipeline stays the same regardless of the transport carrying the document."
      >
        <div style={{ display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap", marginTop: "var(--tt-space-4)" }}>
          <a className="btn btn--ghost" href="/specification" onClick={(e) => { e.preventDefault(); setRoute({ name: "specification" }); }}>Read SPEC §9 →</a>
          <a className="btn btn--ghost" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/bindings" target="_blank" rel="noreferrer">Source on GitHub →</a>
        </div>
      </PageHero>

      <section style={{ paddingBlock: "var(--tt-space-7)" }}>
        <div className="container">
          <span className="eyebrow" style={{ marginBottom: "var(--tt-space-4)", display: "inline-flex" }}>Published bindings</span>
          <h2 style={{ marginTop: "var(--tt-space-2)" }}>{bindings.length} {bindings.length === 1 ? "binding" : "bindings"}.</h2>
          <p style={{ color: "var(--tt-text-muted)", maxWidth: "60ch" }}>
            Each entry is a normative document describing how to carry a Trust Task over a specific transport. The list is open — new bindings <strong>MAY</strong> be published independently (<a href="/specification" onClick={(e) => { e.preventDefault(); setRoute({ name: "specification" }); }}>SPEC §9.2</a>).
          </p>

          {bindings.length === 0 ? (
            <div className="tt-empty" style={{ padding: "var(--tt-space-6)", marginTop: "var(--tt-space-5)" }}>
              No transport bindings are currently registered.
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--tt-space-4)", marginTop: "var(--tt-space-6)" }}>
              {bindings.map(b => (
                <a
                  key={b.id}
                  href={`/binding/${b.slug}/${b.version}`}
                  onClick={(e) => { e.preventDefault(); setRoute({ name: "binding", slug: b.slug, version: b.version }); }}
                  style={{
                    border: "1px solid var(--tt-border)",
                    borderLeft: `3px solid ${accent(b.accent)}`,
                    padding: "var(--tt-space-5) var(--tt-space-6)",
                    background: "var(--tt-surface-elev)",
                    textDecoration: "none",
                    color: "inherit",
                    borderBottom: "1px solid var(--tt-border)",
                  }}
                >
                  <div style={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: "var(--tt-space-4)", flexWrap: "wrap", marginBottom: "var(--tt-space-3)" }}>
                    <div>
                      <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase", color: accent(b.accent), marginBottom: "var(--tt-space-1)" }}>
                        Transport binding · v{b.version}
                      </div>
                      <h3 style={{ margin: 0 }}>{b.title}</h3>
                    </div>
                    <TTStatus status={b.status} />
                  </div>

                  <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginBottom: "var(--tt-space-3)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    <span style={{ color: "var(--tt-text-muted)" }}>Binding URI · </span>
                    <span style={{ color: "var(--tt-text)" }}>{b.bindingURI}</span>
                  </div>
                  {b.envelopeType && (
                    <div style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", marginBottom: "var(--tt-space-3)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      <span style={{ color: "var(--tt-text-muted)" }}>Envelope type · </span>
                      <span style={{ color: "var(--tt-text)" }}>{b.envelopeType}</span>
                    </div>
                  )}

                  <p style={{ color: "var(--tt-text-muted)", marginTop: 0, marginBottom: "var(--tt-space-3)" }}>{b.summary}</p>

                  {b.implementations && b.implementations.length > 0 && (
                    <div style={{ display: "flex", gap: "var(--tt-space-3)", flexWrap: "wrap", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", color: "var(--tt-text-muted)", letterSpacing: "0.06em" }}>
                      <span style={{ textTransform: "uppercase" }}>Reference impl{b.implementations.length === 1 ? "" : "s"}:</span>
                      {b.implementations.map(impl => (
                        <span key={impl.name}>{impl.name} ({impl.language})</span>
                      ))}
                    </div>
                  )}
                </a>
              ))}
            </div>
          )}

          <hr className="protocol-rule" aria-hidden="true" style={{ marginTop: "var(--tt-space-7)" }} />

          <h2>Publishing a new binding.</h2>
          <p style={{ color: "var(--tt-text-muted)" }}>
            A transport binding is published as a markdown document under <code>bindings/&lt;slug&gt;/&lt;MAJOR.MINOR&gt;/spec.md</code> in the framework repository,
            following the requirements in <a href="/specification" onClick={(e) => { e.preventDefault(); setRoute({ name: "specification" }); }}>SPEC §9.1</a>. The binding identifier itself lives under
            the <code>/binding/</code> subtree of the framework authority — see <a href="/specification" onClick={(e) => { e.preventDefault(); setRoute({ name: "specification" }); }}>SPEC §9.3</a> for the URI grammar
            and the rule that nothing under <code>/binding/</code> may appear in a Trust Task document's <code>type</code> member.
          </p>
        </div>
      </section>
    </React.Fragment>
  );
}

function BindingSpecPage({ slug, version, setRoute }) {
  const all = window.TT_BINDINGS || [];
  // Strict lookup — no fallback to the first registered binding. The
  // previous behavior silently rendered the wrong spec when a route
  // arrived with no slug (e.g. when the parser failed to recognize a
  // URL form) or with an unknown slug.
  const binding = all.find(b => b.slug === slug && (!version || b.version === version));
  if (!binding) {
    return (
      <section className="container">
        <div className="tt-empty" style={{ padding: "var(--tt-space-6)" }}>
          {all.length === 0 ? (
            <>
              <b>No transport bindings are currently registered.</b><br />
              Add one under <code>bindings/&lt;slug&gt;/&lt;version&gt;/</code>.
            </>
          ) : (
            <>
              <b>No binding registered for <code>{slug || "(missing slug)"}{version ? `/${version}` : ""}</code>.</b><br />
              <a href="/bindings" onClick={(e) => { e.preventDefault(); setRoute({ name: "bindings" }); }}>
                See the binding registry →
              </a>
            </>
          )}
        </div>
      </section>
    );
  }
  const accent = (c) => `var(--tt-${c})`;
  const [copied, setCopied] = useS(false);
  const [proseHtml, setProseHtml] = useS("");
  const [proseToc, setProseToc] = useS([]);
  const [proseError, setProseError] = useS(null);
  const [activeSection, setActiveSection] = useS("metadata");

  useE(() => {
    if (!binding.prosePath) {
      setProseError("This binding has no prosePath registered in TT_BINDINGS.");
      return;
    }
    let cancelled = false;
    setProseHtml(""); setProseToc([]); setProseError(null);
    fetch(binding.prosePath, { headers: { "Accept": "text/markdown, text/plain" } })
      .then(r => { if (!r.ok) throw new Error(`Failed to load spec.md (${r.status})`); return r.text(); })
      .then(src => {
        if (cancelled) return;
        if (typeof marked === "undefined") throw new Error("Markdown renderer is unavailable.");
        const body = stripFrontMatter(src);
        marked.setOptions({ gfm: true, breaks: false });
        const rawHtml = marked.parse(body);
        const { html, toc } = injectHeadingIds(rawHtml);
        setProseHtml(html);
        setProseToc(toc);
      })
      .catch(e => { if (!cancelled) setProseError(e.message); });
    return () => { cancelled = true; };
  }, [binding.prosePath]);

  useE(() => {
    if (!proseHtml || !location.hash) return;
    const id = location.hash.slice(1);
    const el = document.getElementById(id);
    if (el) requestAnimationFrame(() => el.scrollIntoView({ block: "start" }));
  }, [proseHtml]);

  useE(() => {
    const ids = ["metadata", ...proseToc.map(t => t.id)];
    const onScroll = () => {
      for (const sid of ids) {
        const el = document.getElementById(sid);
        if (!el) continue;
        const rect = el.getBoundingClientRect();
        if (rect.top > 100) { setActiveSection(sid); return; }
      }
      setActiveSection(ids[ids.length - 1]);
    };
    window.addEventListener("scroll", onScroll);
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, [binding.id, proseToc.length]);

  return (
    <section className="container container--wide tt-spec">
      <div>
        <div style={{ marginBottom: "var(--tt-space-4)" }}>
          <span className="tt-spec__num">binding/{binding.slug} · v{binding.version}</span>
        </div>
        <h1 className="tt-spec__title">{binding.title}</h1>
        <p className="lead" style={{ marginBottom: "var(--tt-space-5)" }}>{binding.summary}</p>

        <div
          style={{
            display: "flex", alignItems: "stretch",
            border: "1px solid var(--tt-border)",
            borderLeft: `3px solid ${accent(binding.accent)}`,
            background: "var(--tt-surface-elev)",
            marginBottom: "var(--tt-space-4)",
            fontFamily: "var(--tt-font-mono)",
            fontSize: "var(--tt-text-sm)",
          }}
        >
          <div style={{ padding: "var(--tt-space-3) var(--tt-space-4)", borderRight: "1px solid var(--tt-border)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", fontSize: "var(--tt-text-xs)", textTransform: "uppercase", display: "flex", alignItems: "center" }}>Binding URI</div>
          <code style={{ flex: 1, padding: "var(--tt-space-3) var(--tt-space-4)", overflow: "auto", whiteSpace: "nowrap", color: "var(--tt-text)" }}>{binding.bindingURI}</code>
          <button
            type="button"
            onClick={() => { navigator.clipboard?.writeText(binding.bindingURI); setCopied(true); setTimeout(() => setCopied(false), 1400); }}
            style={{ borderLeft: "1px solid var(--tt-border)", background: "transparent", padding: "0 var(--tt-space-4)", fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", textTransform: "uppercase", letterSpacing: "0.06em", color: copied ? accent(binding.accent) : "var(--tt-text-muted)", cursor: "pointer" }}
          >
            {copied ? "Copied" : "Copy"}
          </button>
        </div>

        {binding.envelopeType && (
          <div
            style={{
              display: "flex", alignItems: "stretch",
              border: "1px solid var(--tt-border)",
              background: "var(--tt-surface-elev)",
              marginBottom: "var(--tt-space-6)",
              fontFamily: "var(--tt-font-mono)",
              fontSize: "var(--tt-text-sm)",
            }}
          >
            <div style={{ padding: "var(--tt-space-3) var(--tt-space-4)", borderRight: "1px solid var(--tt-border)", color: "var(--tt-text-muted)", letterSpacing: "0.06em", fontSize: "var(--tt-text-xs)", textTransform: "uppercase", display: "flex", alignItems: "center" }}>Envelope type</div>
            <code style={{ flex: 1, padding: "var(--tt-space-3) var(--tt-space-4)", overflow: "auto", whiteSpace: "nowrap", color: "var(--tt-text)" }}>{binding.envelopeType}</code>
          </div>
        )}

        <div className="tt-spec__banner">
          <span><b>Status</b> &nbsp; <TTStatus status={binding.status} /></span>
          <span><b>Target framework</b> &nbsp; trust-task/0.1</span>
          {binding.implementations && binding.implementations.length > 0 && (
            <span><b>Reference impl</b> &nbsp; {binding.implementations.map((impl, i) => (
              <React.Fragment key={impl.name}>
                {i > 0 && ", "}
                <a href={impl.href} target="_blank" rel="noreferrer">{impl.name}</a>
              </React.Fragment>
            ))}</span>
          )}
        </div>

        <h2 id="metadata">Metadata</h2>
        <dl className="tt-meta-grid">
          <dt>Slug</dt><dd>{binding.slug}</dd>
          <dt>Version</dt><dd>{binding.version}</dd>
          <dt>Binding URI</dt><dd><code style={{ fontFamily: "var(--tt-font-mono)", fontSize: "0.95em" }}>{binding.bindingURI}</code></dd>
          {binding.envelopeType && (<React.Fragment><dt>Envelope type</dt><dd><code style={{ fontFamily: "var(--tt-font-mono)", fontSize: "0.95em" }}>{binding.envelopeType}</code></dd></React.Fragment>)}
          <dt>Status</dt><dd>{binding.status}</dd>
        </dl>

        {proseError && (
          <div className="tt-empty" style={{ padding: "var(--tt-space-5)", margin: "var(--tt-space-5) 0" }}>
            <b>Couldn't load this binding's prose.</b><br />
            {proseError}<br />
            <a href={`https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/bindings/${binding.slug}/${binding.version}/spec.md`} target="_blank" rel="noreferrer">Read it on GitHub →</a>
          </div>
        )}
        {!proseError && !proseHtml && (
          <p style={{ color: "var(--tt-text-muted)" }}>Loading binding specification…</p>
        )}
        {!proseError && proseHtml && (
          <article className="tt-prose" dangerouslySetInnerHTML={{ __html: proseHtml }} />
        )}

        <div style={{ marginTop: "var(--tt-space-8)", paddingTop: "var(--tt-space-5)", borderTop: "1px solid var(--tt-line)", display: "flex", justifyContent: "space-between", flexWrap: "wrap", gap: "var(--tt-space-4)" }}>
          <a href="/bindings" onClick={(e) => { e.preventDefault(); setRoute({ name: "bindings" }); }} className="btn btn--ghost">← Back to bindings</a>
          <a href={`https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/bindings/${binding.slug}/${binding.version}/spec.md`} target="_blank" rel="noreferrer" className="btn btn--ghost">Edit on GitHub →</a>
        </div>
      </div>

      <aside className="tt-spec__sidebar">
        <div className="tt-toc-title">On this page</div>
        <ol className="tt-toc">
          {[{ id: "metadata", text: "Metadata" }, ...proseToc].map(({ id: sid, text }) => (
            <li key={sid}><a href={`#${sid}`} className={activeSection === sid ? "active" : ""}>{text}</a></li>
          ))}
        </ol>
      </aside>
    </section>
  );
}

Object.assign(window, {
  HomePage, RegistryPage, RegistryCard, SpecPage, CategoriesPage, AboutPage, ContributingPage, GlossaryPage, FrameworkSpecPage, ImplementationsPage, BindingsPage, BindingSpecPage
});
