/* Markdown rendering for spec, binding and framework-spec prose.
 *
 * The prose comes from files in this repository and from the canonical
 * framework repository, and the HTML it produces is inserted into the page as
 * markup. `marked` passes raw HTML and any link scheme through unchanged, so
 * this file renders with a dedicated `marked` instance that:
 *
 *   - keeps raw HTML only for an allow-list of inline tags and attributes
 *     (today the prose uses `<a id="…">` anchors and `<!-- … -->` comments),
 *     and shows anything else as literal text;
 *   - drops comments, which never render anyway;
 *   - escapes text after an inline `<code>`, `<kbd>`, `<pre>` or `<script>`
 *     tag like any other text;
 *   - keeps a link or image only when its URL is relative (not starting with
 *     `//`), a fragment, or `http:`, `https:` or `mailto:` — otherwise a link
 *     renders as its text and an image as its alt text.
 *
 * Markdown the prose uses today renders exactly as before. Plain script (no
 * JSX) so `scripts/test/website-markdown.test.mjs` can load it under Node.
 */
(function (global) {
  "use strict";

  // Tags raw HTML in prose may use, each with the attributes it may carry.
  const ALLOWED_TAGS = {
    a: ["id", "name", "href", "title"],
    abbr: ["title"],
    b: [], br: [], code: [], del: [], em: [], i: [], ins: [], kbd: [],
    mark: [], s: [], samp: [], small: [], span: ["id"], strong: [],
    sub: [], sup: [], u: [], var: [],
  };
  const VOID_TAGS = new Set(["br"]);
  const URL_ATTRS = new Set(["href"]);
  const ID_ATTRS = new Set(["id", "name"]);

  /* True when `url` is relative, a fragment, or uses http, https or mailto.
   * Browsers ignore whitespace and control characters inside a scheme and
   * decode character references in attribute values before reading it, so
   * those are removed first and any `&` before the first `/`, `?` or `#`
   * refuses the URL rather than trying to decode it. A URL that starts with
   * two slashes names another host (browsers read `\` as `/`), so it is
   * refused too: an off-site link spells out its scheme. */
  function isSafeUrl(url) {
    const s = String(url).replace(/[\u0000- \u007f-\u009f]/g, "");
    if (/^[/\\]{2}/.test(s)) return false;
    const head = s.split(/[/?#]/, 1)[0];
    if (!/[:&]/.test(head)) return true;
    return /^(?:https?|mailto):/i.test(head) && !head.includes("&");
  }

  function escapeText(s) {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  function escapeAttr(s) {
    return s.replace(/&(?!(?:amp|lt|gt|quot|#39);)/g, "&amp;")
      .replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  const ATTR_RE = /([^\s"'<>\/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;

  /* Rebuilds one tag from the allow-list, or returns null to refuse it. */
  function rebuildTag(closing, name, attrText) {
    const tag = name.toLowerCase();
    const allowed = ALLOWED_TAGS[tag];
    if (!allowed) return null;
    if (closing) return VOID_TAGS.has(tag) ? "" : `</${tag}>`;
    let out = `<${tag}`;
    ATTR_RE.lastIndex = 0;
    let m;
    while ((m = ATTR_RE.exec(attrText))) {
      const attr = m[1].toLowerCase();
      if (!allowed.includes(attr)) continue;
      const value = m[2] ?? m[3] ?? m[4] ?? "";
      if (URL_ATTRS.has(attr) && !isSafeUrl(value)) continue;
      if (ID_ATTRS.has(attr) && !/^[A-Za-z0-9_.:-]+$/.test(value)) continue;
      out += ` ${attr}="${escapeAttr(value)}"`;
    }
    return out + ">";
  }

  const TAG_RE = /<!--[\s\S]*?-->|<(\/?)([A-Za-z][A-Za-z0-9-]*)((?:\s+(?:[^\s"'<>\/=]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?)*)\s*\/?>/g;

  /* Sanitises a raw-HTML fragment from the prose. Allowed tags are rebuilt from
   * their allowed attributes, comments are dropped, and every other `<` is
   * escaped, so the only markup in the output is markup rebuilt here. */
  function sanitizeRawHtml(html) {
    let out = "";
    let last = 0;
    TAG_RE.lastIndex = 0;
    let m;
    while ((m = TAG_RE.exec(html))) {
      out += html.slice(last, m.index).replace(/</g, "&lt;");
      last = TAG_RE.lastIndex;
      if (m[0].startsWith("<!--")) continue;
      const rebuilt = rebuildTag(m[1] === "/", m[2], m[3] || "");
      out += rebuilt === null ? escapeText(m[0]) : rebuilt;
    }
    return out + html.slice(last).replace(/</g, "&lt;");
  }

  let instance = null;

  function markdownInstance(markedLib) {
    if (instance) return instance;
    instance = new markedLib.Marked({ gfm: true, breaks: false });
    // marked's inline `tag` tokenizer turns on a raw-block state after an
    // opening `<code>`, `<kbd>`, `<pre>` or `<script>` tag, and while it is on
    // the text that follows is emitted without escaping — whatever the html
    // renderer below does with the tag itself. Keep the state off, so that
    // text is escaped like any other.
    const baseTag = markedLib.Tokenizer.prototype.tag;
    instance.use({
      tokenizer: {
        tag(src) {
          const token = baseTag.call(this, src);
          if (token) {
            this.lexer.state.inRawBlock = false;
            token.inRawBlock = false;
          }
          return token;
        },
      },
      renderer: {
        html(html) { return sanitizeRawHtml(html); },
        // `false` hands a safe link or image back to marked's own renderer.
        link(href, _title, text) { return isSafeUrl(href) ? false : text; },
        image(href, _title, text) { return isSafeUrl(href) ? false : escapeText(text || ""); },
      },
    });
    return instance;
  }

  /* Renders markdown to HTML that is safe to insert as markup. */
  function renderMarkdown(src, markedLib) {
    const lib = markedLib || global.marked;
    if (!lib) throw new Error("Markdown renderer is unavailable.");
    return markdownInstance(lib).parse(src);
  }

  global.ttMarkdown = { renderMarkdown, sanitizeRawHtml, isSafeUrl };
})(typeof window !== "undefined" ? window : globalThis);
