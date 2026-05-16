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

/* ---------- Top nav --------------------------------------- */
function TTNav({ route, setRoute }) {
  const links = [
    { id: "specification", label: "Specification" },
    { id: "registry",      label: "Registry" },
    { id: "categories",    label: "Categories" },
    { id: "ecosystem",     label: "Ecosystem" },
    { id: "about",         label: "About" },
    { id: "contributing",  label: "Contributing" },
    { id: "glossary",      label: "Glossary" },
  ];
  return (
    <header className="nav">
      <div className="container nav__inner">
        <a className="nav__brand" href="/" onClick={(e) => { e.preventDefault(); setRoute({ name: "home" }); }}>
          <TTMark size={26} />
        </a>
        <nav>
          <ul className="nav__links">
            {links.map(l => (
              <li key={l.id}>
                <a
                  href={l.id === "home" ? "/" : `/${l.id}`}
                  className={route.name === l.id ? "active" : ""}
                  onClick={(e) => { e.preventDefault(); setRoute({ name: l.id }); }}
                >{l.label}</a>
              </li>
            ))}
          </ul>
        </nav>
        <a className="btn btn--primary" href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer">
          GitHub →
        </a>
      </div>
    </header>
  );
}

/* ---------- Footer ---------------------------------------- */
function TTFooter({ setRoute }) {
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
        <div className="footer__inner">
          <div className="nav__brand"><TTMark size={22} /></div>
          <div style={{ display: "flex", gap: "var(--tt-space-5)", flexWrap: "wrap" }}>
            <a href="/registry" onClick={(e) => { e.preventDefault(); setRoute({ name: "registry" }); }} style={{ borderBottom: 0, fontSize: "var(--tt-text-sm)", color: "var(--tt-text-muted)" }}>Registry</a>
            <a href="/ecosystem" onClick={(e) => { e.preventDefault(); setRoute({ name: "ecosystem" }); }} style={{ borderBottom: 0, fontSize: "var(--tt-text-sm)", color: "var(--tt-text-muted)" }}>Ecosystem</a>
            <a href="/about" onClick={(e) => { e.preventDefault(); setRoute({ name: "about" }); }} style={{ borderBottom: 0, fontSize: "var(--tt-text-sm)", color: "var(--tt-text-muted)" }}>About</a>
            <a href="https://github.com/trustoverip/dtgwg-trust-tasks-tf" target="_blank" rel="noreferrer" style={{ borderBottom: 0, fontSize: "var(--tt-text-sm)", color: "var(--tt-text-muted)" }}>GitHub</a>
          </div>
          <small>Open spec · Apache 2.0 · 2026</small>
        </div>
      </div>
    </footer>
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
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1400);
    });
  };
  return (
    <div className="tt-codeblock">
      <button className={`tt-codeblock__copy ${copied ? "copied" : ""}`} onClick={onCopy}>
        {copied ? "Copied" : "Copy"}
      </button>
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

/* expose */
Object.assign(window, {
  TTMark, TTNav, TTFooter, TTStatus, Highlight, CodeBlock,
  HeroGlyph, PageHero, AnimNumber, catColor, catName
});
