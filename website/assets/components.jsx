/* ============================================================
   Trust Tasks — Shared React components (Babel/JSX)
   ============================================================ */

const { useState, useEffect, useMemo, useRef } = React;

/* ---------- Brand mark (inline SVG, scales to font-size) -- */
function TTMark({ size = 28, withWord = true }) {
  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: "0.6em" }}>
      <svg width={size * 2} height={size} viewBox="0 0 280 140" aria-hidden="true">
        <path d="M 28 30 L 12 70 L 28 110" stroke="#8b5cf6" strokeWidth="3.5" fill="none" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M 252 30 L 268 70 L 252 110" stroke="#f59e0b" strokeWidth="3.5" fill="none" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M 60 80 L 115 115 L 215 45" stroke="#1e3a5f" strokeWidth="7" fill="none" strokeLinecap="round" strokeLinejoin="round" />
        <circle cx="60" cy="80" r="12" fill="#fb7185" />
        <circle cx="215" cy="45" r="12" fill="#0d9488" />
      </svg>
      {withWord && <span style={{ fontFamily: "var(--tt-font-display)", fontWeight: 450, fontSize: "var(--tt-text-md)", letterSpacing: "-0.01em" }}>Trust Tasks</span>}
    </span>
  );
}

/* ---------- Site sections ---------------------------------
 * One list for the nav and the footer, grouped so the footer (and the narrow-
 * width menu) can show every page without one long undifferentiated row. */
const TT_NAV_GROUPS = [
  { label: "Specification", links: [
    { id: "specification",   label: "Specification" },
    { id: "bindings",        label: "Bindings", also: ["binding"] },
    // Adjacent to Bindings deliberately: both are artifact types *beside* the
    // task registry rather than above it. A nav entry is discoverability, not
    // billing — the home page's framing is unchanged, because the ceremony
    // layer is optional (SPEC §4.11.1) and its evidence machinery is not yet
    // specified. Revisit the site's top-level story when that lands.
    { id: "ceremonies",      label: "Ceremonies", also: ["ceremony"] },
  ] },
  { label: "Build", links: [
    // `also` keeps the tab lit on the per-library deep dives, which route as
    // `implementation` (singular) rather than the index's `implementations`.
    { id: "implementations", label: "Implementations", also: ["implementation"] },
  ] },
  { label: "Registry", links: [
    { id: "registry",        label: "Registry", also: ["spec"] },
    { id: "schemas",         label: "Schemas", path: "schema", also: ["schema"] },
    { id: "categories",      label: "Categories" },
  ] },
  { label: "About", links: [
    { id: "ecosystem",       label: "Ecosystem" },
    { id: "about",           label: "About" },
    { id: "contributing",    label: "Contributing" },
    { id: "glossary",        label: "Glossary" },
  ] },
];
const TT_NAV_LINKS = TT_NAV_GROUPS.flatMap(g => g.links);

/* ---------- Top nav ---------------------------------------
 * At narrow widths the link row does not fit, so a disclosure button shows
 * and hides it as a panel (#1149). The button is the standard pattern: a
 * <button aria-expanded aria-controls>, closing on navigation and on Escape. */
function TTNav({ route, setRoute }) {
  const [open, setOpen] = useState(false);
  const panelId = "tt-site-nav";
  useEffect(() => { setOpen(false); }, [route.name, route.slug, route.version, route.id, route.cat]);
  useEffect(() => {
    if (!open) return;
    const onKey = (e) => { if (e.key === "Escape") setOpen(false); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);
  return (
    <header className="nav">
      <div className="container nav__inner">
        <a className="nav__brand" href="/" aria-label="Trust Tasks home" onClick={(e) => { e.preventDefault(); setRoute({ name: "home" }); }}>
          <TTMark size={26} />
        </a>
        <button
          type="button"
          className="nav__toggle"
          aria-expanded={open}
          aria-controls={panelId}
          onClick={() => setOpen(!open)}
        >
          <span aria-hidden="true">{open ? "✕" : "☰"}</span> Menu
        </button>
        <nav aria-label="Site" id={panelId} className={open ? "nav__panel nav__panel--open" : "nav__panel"}>
          <ul className="nav__links">
            {TT_NAV_LINKS.map(l => {
              const active = route.name === l.id || (l.also || []).includes(route.name);
              return (
                <li key={l.id}>
                  <a
                    href={`/${l.path || l.id}`}
                    className={active ? "active" : ""}
                    aria-current={active ? "page" : undefined}
                    onClick={(e) => { e.preventDefault(); setOpen(false); setRoute({ name: l.id }); }}
                  >{l.label}</a>
                </li>
              );
            })}
          </ul>
        </nav>
        <a className="btn btn--primary nav__github" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">
          GitHub →
        </a>
      </div>
    </header>
  );
}

/* ---------- Footer ---------------------------------------- */
function TTFooter({ setRoute }) {
  const linkStyle = { borderBottom: 0, fontSize: "var(--tt-text-sm)", color: "var(--tt-text-muted)" };
  return (
    <footer className="footer">
      <div className="container">
        <div className="tt-footer-strip">
          <div className="tt-footer-strip__org">
            A <b style={{ color: "var(--tt-text)" }}>Trust Over IP</b> Digital Trust Graph Working Group task force
          </div>
          <a href="https://trustoverip.org" target="_blank" rel="noreferrer" style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-xs)", letterSpacing: "0.06em", textTransform: "uppercase" }}>
            trustoverip.org →
          </a>
        </div>
        <nav aria-label="Footer" className="tt-footer-nav">
          {TT_NAV_GROUPS.map(g => (
            <div key={g.label}>
              <h2 className="tt-footer-nav__heading">{g.label}</h2>
              <ul>
                {g.links.map(l => (
                  <li key={l.id}>
                    <a href={`/${l.path || l.id}`} onClick={(e) => { e.preventDefault(); setRoute({ name: l.id }); }} style={linkStyle}>{l.label}</a>
                  </li>
                ))}
                {g.label === "About" && (
                  <li><a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer" style={linkStyle}>GitHub</a></li>
                )}
                {g.label === "Registry" && (
                  <li><a href="/specs/index.json" style={linkStyle}>Bundle index (JSON)</a></li>
                )}
              </ul>
            </div>
          ))}
        </nav>
        <div className="footer__inner">
          <div className="nav__brand"><TTMark size={22} /></div>
          <small>Open spec · Apache 2.0 · 2026</small>
        </div>
      </div>
    </footer>
  );
}

/* ---------- Error boundary --------------------------------
 * Without one, any render exception — typically a data shape the JSX did not
 * expect from build-registry.mjs — unmounts the whole app, nav included, and
 * leaves a white page (#1153). index.html keys this on the route, so moving to
 * another page clears the error. */
class TTErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { error: null };
  }
  static getDerivedStateFromError(error) {
    return { error };
  }
  componentDidCatch(error, info) {
    console.error("Trust Tasks: page failed to render", error, info && info.componentStack);
  }
  render() {
    if (!this.state.error) return this.props.children;
    return (
      <section className="container" style={{ paddingBlock: "var(--tt-space-8)" }}>
        <div className="tt-empty" style={{ padding: "var(--tt-space-6)", textAlign: "left" }}>
          <h1 tabIndex={-1} style={{ fontSize: "var(--tt-text-xl)", marginTop: 0 }}>This page failed to render.</h1>
          <p>
            Something in this page's data was not in the shape the site expected. The rest of the
            site still works — use the navigation above — and the source of every specification is on GitHub.
          </p>
          <p style={{ fontFamily: "var(--tt-font-mono)", fontSize: "var(--tt-text-sm)", color: "var(--tt-text-muted)" }}>
            {String(this.state.error && this.state.error.message || this.state.error)}
          </p>
          <p>
            <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">Source on GitHub →</a>
            {"  ·  "}
            <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/new" target="_blank" rel="noreferrer">Report this page →</a>
          </p>
        </div>
      </section>
    );
  }
}

/* ---------- Not found ------------------------------------- */
function NotFoundPage({ setRoute }) {
  return (
    <section className="container" style={{ paddingBlock: "var(--tt-space-8)" }}>
      <div className="tt-empty" style={{ padding: "var(--tt-space-6)", textAlign: "left" }}>
        <h1 style={{ fontSize: "var(--tt-text-xl)", marginTop: 0 }}>Page not found</h1>
        <p>
          Nothing on this site lives at <code>{location.pathname}</code>.
        </p>
        <p>
          <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }}>Search the Trust Task registry →</a>
          {"  ·  "}
          <a href="/" onClick={(e) => { e.preventDefault(); setRoute({ name: "home" }); }}>Home →</a>
        </p>
      </div>
    </section>
  );
}

/* ---------- Status pill ----------------------------------- */
function TTStatus({ status }) {
  return (
    <span className={`tt-status tt-status--${status}`}>
      <span className="dot"></span>
      {status}
    </span>
  );
}

/* ---------- Category dot color helper -------------------- */
function catColor(catId) {
  const cat = window.TT_CATEGORIES.find(c => c.id === catId);
  if (!cat) return "var(--tt-navy)";
  return `var(--tt-${cat.color})`;
}
/* The bright palette tokens fail WCAG AA as small text on the cream page
 * (amber 2.0:1, sky 2.3:1, coral 2.5:1, teal 3.5:1, violet 4.0:1), so text
 * takes the darker `--tt-<color>-text` variant (tokens.css) and the bright one
 * stays on dots, borders and fills (#1156). */
function catTextColor(catId) {
  const cat = window.TT_CATEGORIES.find(c => c.id === catId);
  if (!cat) return "var(--tt-navy)";
  return `var(--tt-${cat.color}-text)`;
}
function accentText(color) {
  return `var(--tt-${color}-text)`;
}

/* Copy to the clipboard, resolving to whether it worked, so that a button says
 * "Copied" only when something was copied — there is no Clipboard API on an
 * insecure origin, and the permission can be denied (#1156). */
function copyText(text) {
  try {
    if (!navigator.clipboard || !navigator.clipboard.writeText) return Promise.resolve(false);
    return navigator.clipboard.writeText(text).then(() => true, () => false);
  } catch (e) {
    return Promise.resolve(false);
  }
}

function catName(catId) {
  const cat = window.TT_CATEGORIES.find(c => c.id === catId);
  return cat ? cat.name : catId;
}

/* ---------- Highlighted text ------------------------------ */
function Highlight({ text, query }) {
  if (!query) return text;
  const re = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, "ig");
  const parts = String(text).split(re);
  return parts.map((p, i) =>
    re.test(p) ? <mark key={i} className="tt-mark">{p}</mark> : <React.Fragment key={i}>{p}</React.Fragment>
  );
}

/* ---------- Code block w/ copy ---------------------------- */
function CodeBlock({ json, language = "json" }) {
  const [copied, setCopied] = useState(false);
  const text = typeof json === "string" ? json : JSON.stringify(json, null, 2);
  const onCopy = () => {
    copyText(text).then((ok) => {
      if (!ok) return;
      setCopied(true);
      setTimeout(() => setCopied(false), 1400);
    });
  };
  return (
    <div className="tt-codeblock">
      <button type="button" className={`tt-codeblock__copy ${copied ? "copied" : ""}`} onClick={onCopy}>
        {copied ? "Copied" : "Copy"}
      </button>
      <span className="sr-only" aria-live="polite">{copied ? "Copied to the clipboard" : ""}</span>
      <pre><code>{text}</code></pre>
    </div>
  );
}

/* ---------- Animated network glyph (hero) ---------------- */
function HeroGlyph() {
  return (
    <svg className="tt-glyph-anim" viewBox="0 0 480 360" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <circle cx="60"  cy="60"  r="4" fill="#8b5cf6" className="pulse-1" />
      <circle cx="420" cy="80"  r="4" fill="#f59e0b" className="pulse-2" />
      <circle cx="80"  cy="300" r="4" fill="#06b6d4" className="pulse-3" />
      <circle cx="400" cy="290" r="4" fill="#fb7185" className="pulse-1" />
      <circle cx="240" cy="40"  r="3" fill="#0d9488" className="pulse-2" />
      <path d="M 60 130 L 40 180 L 60 230" stroke="#8b5cf6" strokeWidth="3" fill="none" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M 420 130 L 440 180 L 420 230" stroke="#f59e0b" strokeWidth="3" fill="none" strokeLinecap="round" strokeLinejoin="round" />
      <path className="draw-tick" d="M 110 195 L 200 240 L 360 130" stroke="#1e3a5f" strokeWidth="6" fill="none" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="110" cy="195" r="11" fill="#fb7185" />
      <circle cx="360" cy="130" r="11" fill="#0d9488" />
    </svg>
  );
}

/* ---------- Page hero ------------------------------------- */
function PageHero({ eyebrow, title, lede, children }) {
  return (
    <section className="tt-page-hero">
      <div className="container">
        {eyebrow && <span className="eyebrow">{eyebrow}</span>}
        <h1 style={{ marginTop: "var(--tt-space-3)" }}>{title}</h1>
        {lede && <p>{lede}</p>}
        {children}
      </div>
    </section>
  );
}

/* ---------- Animated number (for stats) ------------------ */
function AnimNumber({ value, duration = 1100, suffix = "" }) {
  const [n, setN] = useState(0);
  const startedRef = useRef(false);
  const ref = useRef(null);
  useEffect(() => {
    if (startedRef.current) return;
    const obs = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting && !startedRef.current) {
        startedRef.current = true;
        const start = performance.now();
        const tick = (t) => {
          const p = Math.min(1, (t - start) / duration);
          const eased = 1 - Math.pow(1 - p, 3);
          setN(Math.round(eased * value));
          if (p < 1) requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      }
    }, { threshold: 0.3 });
    if (ref.current) obs.observe(ref.current);
    return () => obs.disconnect();
  }, [value, duration]);
  return <span ref={ref}>{n}{suffix}</span>;
}

/* ---------- SPEC.md cross-reference ---------------------- */
/* Map §-section to the anchor that `injectHeadingIds` produces when rendering
 * SPEC.md prose — the GitHub-flavoured slugify of the heading line (digits
 * collapsed via dot removal, lowercased, hyphenated).
 *
 * SPEC.md is generated from the canonical framework repository, which assigns
 * the section numbers, so this map goes stale whenever a canonical change
 * renumbers a section. `checkSiteFrameworkLinks` in scripts/build-registry.mjs
 * therefore fails the build on any entry that is not the heading anchor its
 * section number has now. To refresh it, regenerate the entries from SPEC.md's
 * numbered h2–h4 headings. */
const SPEC_ANCHORS = {
  "1":     "1-introduction",
  "1.1":   "11-design-goals",
  "2":     "2-requirements-language",
  "3":     "3-terminology",
  "4":     "4-trust-task-documents",
  "4.1":   "41-encoding",
  "4.2":   "42-top-level-members",
  "4.3":   "43-the-id-member",
  "4.4":   "44-the-type-member",
  "4.4.1": "441-request-and-response-variants",
  "4.4.2": "442-acknowledging-a-fire-and-forget-task",
  "4.5":   "45-the-payload-member",
  "4.5.1": "451-the-ext-extension-member",
  "4.5.2": "452-marking-a-namespace-critical",
  "4.6":   "46-json-ld-compatibility",
  "4.7":   "47-proof",
  "4.7.1": "471-cryptosuites",
  "4.7.2": "472-when-to-include-a-proof",
  "4.7.3": "473-proof-purpose-and-verification-relationship",
  "4.8":   "48-the-issuer-and-recipient-members",
  "4.8.1": "481-precedence-of-in-band-over-transport-derived-identity",
  "4.8.2": "482-audience-binding",
  "4.8.3": "483-bearer-specifications",
  "4.9":   "49-the-threadid-member",
  "4.9.1": "491-naming-an-exchange-from-outside-the-framework",
  "4.9.2": "492-the-parentthreadid-member",
  "4.9.3": "493-binding-a-citation-to-the-document-it-names",
  "4.9.4": "494-evidence-that-a-cited-exchange-completed",
  "4.10":  "410-naming-conventions",
  "4.11":  "411-the-ceremony-member",
  "4.11.1":"4111-ceremony-membership-is-optional",
  "4.11.2":"4112-the-identifiers-are-orthogonal",
  "4.11.3":"4113-ceremony-integrity",
  "4.11.4":"4114-membership-is-a-claim-not-a-permission",
  "4.12":  "412-document-lifecycle",
  "5":     "5-versioning",
  "5.1":   "51-version-scheme",
  "5.1.1": "511-versioning-of-this-framework-specification",
  "5.1.2": "512-ratification-and-the-stable-release-line",
  "5.2":   "52-compatibility-rules",
  "5.3":   "53-maturity-levels",
  "5.3.1": "531-permitted-transitions",
  "5.3.2": "532-behaviour-at-each-status",
  "5.3.3": "533-superseded-and-withdrawn-versions",
  "5.4":   "54-migrating-between-versions",
  "6":     "6-namespace",
  "6.1":   "61-type-uri",
  "6.2":   "62-content-negotiation",
  "6.3":   "63-schema-scope",
  "6.4":   "64-stability",
  "6.5":   "65-private-and-unpublished-trust-task-specifications",
  "6.6":   "66-shared-schema-components",
  "6.7":   "67-ceremony-namespace",
  "7":     "7-minimum-requirements",
  "7.1":   "71-producer-requirements",
  "7.2":   "72-consumer-requirements",
  "7.3":   "73-specification-requirements",
  "8":     "8-error-responses",
  "8.1":   "81-the-trust-task-error-specification",
  "8.2":   "82-error-payload",
  "8.2.1": "821-what-a-message-may-not-say",
  "8.2.2": "822-bounding-details",
  "8.3":   "83-standard-error-codes",
  "8.3.1": "831-effects-reported-with-cancelled",
  "8.4":   "84-retry-semantics",
  "8.5":   "85-extension-by-individual-trust-task-specifications",
  "8.6":   "86-reserved-response-type-slugs",
  "9":     "9-transport-bindings",
  "9.1":   "91-what-a-transport-binding-specifies",
  "9.1.1": "911-permitting-proof-to-be-omitted",
  "9.2":   "92-the-transport-handler",
  "9.3":   "93-binding-namespace",
  "9.4":   "94-carried-protocols",
  "10":    "10-discovery-and-capability-negotiation",
  "10.1":  "101-discovery-request",
  "10.2":  "102-pattern-grammar",
  "10.3":  "103-discovery-response",
  "10.4":  "104-status-of-the-discovery-response",
  "10.5":  "105-authenticity-of-a-discovery-response",
  "10.6":  "106-privacy-of-discovery-responses",
  "11":    "11-task-control",
  "11.1":  "111-control-authorization",
  "11.2":  "112-identifying-the-target",
  "11.3":  "113-when-a-control-operation-takes-effect",
  "11.4":  "114-control-does-not-roll-back",
  "11.5":  "115-suspension-and-resumption",
  "11.6":  "116-notifications-and-the-meaning-of-silence",
  "11.7":  "117-transport-level-cancellation-is-not-semantic-cancellation",
  "11.8":  "118-support-is-optional",
  "12":    "12-security-considerations",
  "12.1":  "121-cross-recipient-replay",
  "12.2":  "122-cryptosuite-downgrade",
  "12.3":  "123-stripping-a-criticality-marking",
  "12.4":  "124-parser-hardening",
  "12.5":  "125-schema-validation-dos",
  "12.6":  "126-error-response-identity-leakage",
  "13":    "13-privacy-considerations",
  "13.1":  "131-identifier-correlation-and-linkability",
  "14":    "14-governance-considerations",
  "15":    "15-internationalization-considerations",
  "16":    "16-accessibility-considerations",
  "17":    "17-conformance",
  "17.1":  "171-conformance-targets",
  "17.2":  "172-conformance-tests",
  "18":    "18-references",
  "18.1":  "181-normative-references",
  "18.2":  "182-informative-references",
};

/* A clickable cross-reference into SPEC.md, resolved through the SPA router. */
function SpecRef({ section, setRoute, children }) {
  const anchor = SPEC_ANCHORS[section];
  const href = anchor ? `/specification#${anchor}` : "/specification";
  return (
    <a
      href={href}
      onClick={(e) => {
        e.preventDefault();
        setRoute({ name: "specification", hash: anchor });
      }}
    >
      {children}
    </a>
  );
}

/* Turn `§X.Y[.Z]` occurrences in a plain string into clickable `SpecRef`s,
 * leaving everything else untouched. Returns either the original string (when
 * nothing matched) or an array of mixed strings + JSX nodes suitable for
 * dropping into a React child. */
function linkifySpec(text, setRoute) {
  if (typeof text !== "string") return text;
  const re = /§(\d+(?:\.\d+)*)/g;
  const parts = [];
  let lastIndex = 0;
  let m;
  let key = 0;
  while ((m = re.exec(text)) !== null) {
    if (m.index > lastIndex) parts.push(text.slice(lastIndex, m.index));
    const section = m[1];
    parts.push(
      <SpecRef key={`s${key++}`} section={section} setRoute={setRoute}>
        §{section}
      </SpecRef>
    );
    lastIndex = re.lastIndex;
  }
  if (parts.length === 0) return text;
  if (lastIndex < text.length) parts.push(text.slice(lastIndex));
  return parts;
}

/* expose */
Object.assign(window, {
  TTMark, TTNav, TTFooter, TTStatus, Highlight, CodeBlock,
  HeroGlyph, PageHero, AnimNumber, catColor, catTextColor, accentText, copyText, catName,
  SPEC_ANCHORS, SpecRef, linkifySpec, TTErrorBoundary, NotFoundPage, TT_NAV_GROUPS
});
