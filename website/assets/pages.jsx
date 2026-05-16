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
  const counts = useM(() => {
    const c = {};
    window.TT_TASKS.forEach(t => { c[t.category] = (c[t.category] || 0) + 1; });
    return c;
  }, []);

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
    return window.TT_TASKS.filter(t => {
      if (activeCat && t.category !== activeCat) return false;
      if (activeKw && !t.keywords.includes(activeKw)) return false;
      if (!ql) return true;
      const hay = (t.title + " " + t.summary + " " + t.keywords.join(" ") + " " + t.slug).toLowerCase();
      return hay.includes(ql);
    });
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
                  placeholder="Search 6 specifications — try “credential”, “consent”, “payment”…"
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
      <div className="tt-task-card__num">{task.slug}<br /><span style={{ opacity: 0.7 }}>v{task.version}</span></div>
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

  const counts = useM(() => {
    const c = {};
    window.TT_TASKS.forEach(t => { c[t.category] = (c[t.category] || 0) + 1; });
    return c;
  }, []);

  const allKeywords = useM(() => {
    const kws = {};
    window.TT_TASKS.forEach(t => t.keywords.forEach(k => { kws[k] = (kws[k] || 0) + 1; }));
    return Object.entries(kws).sort((a, b) => b[1] - a[1]);
  }, []);

  const results = useM(() => {
    const ql = q.trim().toLowerCase();
    return window.TT_TASKS.filter(t => {
      if (activeCat && t.category !== activeCat) return false;
      if (activeKw && !t.keywords.includes(activeKw)) return false;
      if (activeStatus && t.status !== activeStatus) return false;
      if (!ql) return true;
      const hay = (t.title + " " + t.summary + " " + t.keywords.join(" ") + " " + t.slug).toLowerCase();
      return hay.includes(ql);
    });
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
        <div className="container">
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
                  <b>{results.length}</b> of {window.TT_TASKS.length} specifications
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
    const ids = ["metadata", ...proseToc.map(t => t.id), "schema", "related"];
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
    <section className="container tt-spec">
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
          <dt>Version</dt><dd>{task.version}</dd>
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
  const counts = useM(() => {
    const c = {};
    window.TT_TASKS.forEach(t => { c[t.category] = (c[t.category] || 0) + 1; });
    return c;
  }, []);
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
        <div className="container container--narrow">
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
        <div className="container container--narrow">
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
        <div className="container container--narrow">
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
        <div className="container container--narrow">
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
        <div className="container container--narrow">
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

Object.assign(window, {
  HomePage, RegistryPage, RegistryCard, SpecPage, CategoriesPage, AboutPage, ContributingPage, GlossaryPage, FrameworkSpecPage
});
