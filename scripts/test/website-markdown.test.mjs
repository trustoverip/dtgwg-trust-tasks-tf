// Tests for website/assets/markdown.js — the raw-HTML and URL handling the
// site applies to rendered spec, binding and framework-spec prose.
//
//   node --test scripts/test/website-markdown.test.mjs   (npm run test:website)
//
// `marked` is not an npm dependency here: the page loads it from a CDN. The
// end-to-end tests load the same file — the URL and `integrity` hash pinned in
// website/index.html — and refuse it unless its SHA-384 matches that hash. Set
// MARKED_JS to a local copy to run offline; the hash is checked either way.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const website = path.join(here, '..', '..', 'website');
const source = fs.readFileSync(path.join(website, 'assets', 'markdown.js'), 'utf8');

async function loadMarked() {
  const index = fs.readFileSync(path.join(website, 'index.html'), 'utf8');
  const tag = /<script src="(https:\/\/[^"]*\/marked@[^"]+)" integrity="(sha384-[^"]+)"/.exec(index);
  assert.ok(tag, 'index.html pins marked with an integrity hash');
  const [, url, integrity] = tag;
  let bytes;
  if (process.env.MARKED_JS) {
    bytes = fs.readFileSync(process.env.MARKED_JS);
  } else {
    const res = await fetch(url);
    assert.ok(res.ok, `fetching ${url}: HTTP ${res.status}`);
    bytes = Buffer.from(await res.arrayBuffer());
  }
  const actual = 'sha384-' + crypto.createHash('sha384').update(bytes).digest('base64');
  assert.equal(actual, integrity, `${url} does not match the integrity hash in index.html`);
  return bytes.toString('utf8');
}

const ctx = {};
ctx.window = ctx;
vm.createContext(ctx);
vm.runInContext(await loadMarked(), ctx);
vm.runInContext(source, ctx);
const { isSafeUrl, sanitizeRawHtml, renderMarkdown } = ctx.ttMarkdown;
const render = (md) => renderMarkdown(md, ctx.marked);

test('relative, fragment, http(s) and mailto URLs are kept', () => {
  for (const url of [
    '/SPEC.md#4-documents', '../../SPEC.md#a', 'spec.md', '#response', '',
    'https://trustoverip.github.io/x', 'http://example.com/a?b=c&d=e',
    'HTTPS://EXAMPLE.COM', 'mailto:someone@example.com', '/a//b', './/x',
  ]) assert.equal(isSafeUrl(url), true, url);
});

test('other schemes are refused, however they are spelled', () => {
  for (const url of [
    'javascript:alert(1)', 'JavaScript:alert(1)', ' javascript:alert(1)',
    'java\tscript:alert(1)', 'java\nscript:alert(1)', '\u0001javascript:x',
    'javascript&colon;alert(1)', 'javascript&#58;alert(1)', 'javascript&#x3a;x',
    'data:text/html,<p>x</p>', 'vbscript:x', 'file:///etc/hosts',
    'https&#58;//example.com',
    '//example.com/x', '\\\\example.com/x', '/\\example.com/x', '\\/example.com/x', ' //example.com',
  ]) assert.equal(isSafeUrl(url), false, JSON.stringify(url));
});

test('the raw HTML the prose uses today is unchanged', () => {
  for (const html of [
    '<a id="471-when-to-include-a-proof">', '</a>', '<a id="06">',
    '<a id="abstract"></a>', 'text <sup>1</sup> <br> <code>x</code>',
    '<a href="https://example.com/a">x</a>', '<a href="#frag" title="t">x</a>',
  ]) assert.equal(sanitizeRawHtml(html), html, html);
});

test('comments are dropped', () => {
  assert.equal(sanitizeRawHtml('<!-- a note -->'), '');
  assert.equal(sanitizeRawHtml('<!-- a\nmulti-line <b>note</b> -->\n'), '\n');
});

test('tags outside the allow-list are shown as text', () => {
  assert.equal(sanitizeRawHtml('<script>x()</script>'), '&lt;script&gt;x()&lt;/script&gt;');
  assert.equal(sanitizeRawHtml('<img src=x onerror=y()>'), '&lt;img src=x onerror=y()&gt;');
  assert.equal(sanitizeRawHtml('<iframe src="https://example.com"></iframe>'),
    '&lt;iframe src="https://example.com"&gt;&lt;/iframe&gt;');
  assert.equal(sanitizeRawHtml('<svg><a href="x">y</a></svg>'), '&lt;svg&gt;<a href="x">y</a>&lt;/svg&gt;');
});

test('markup that does not parse as a tag cannot open one', () => {
  for (const html of ['<img/src=x onerror=y()>', '<a\nid=x', '<<script>>', '<!-- unterminated <b>']) {
    const out = sanitizeRawHtml(html);
    assert.doesNotMatch(out, /<(?!\/?b>)[a-z!/]/i, `${JSON.stringify(html)} -> ${out}`);
  }
});

test('attributes outside the allow-list, and unsafe URLs, are dropped', () => {
  assert.equal(sanitizeRawHtml('<a id="x" onclick="y()" style="z">t</a>'), '<a id="x">t</a>');
  assert.equal(sanitizeRawHtml('<a href="javascript:y()">t</a>'), '<a>t</a>');
  assert.equal(sanitizeRawHtml('<a href=" jav&#x61;script:y()">t</a>'), '<a>t</a>');
  assert.equal(sanitizeRawHtml('<a id="a b" name="ok">t</a>'), '<a name="ok">t</a>');
  assert.equal(sanitizeRawHtml('<span id="x" class="c">t</span>'), '<span id="x">t</span>');
});

test('attribute values are re-quoted so they cannot close the tag', () => {
  assert.equal(sanitizeRawHtml(`<a title='a"><b'>t</a>`), '<a title="a&quot;&gt;&lt;b">t</a>');
  assert.equal(sanitizeRawHtml('<a title=a&b>t</a>'), '<a title="a&amp;b">t</a>');
});

test('rendering needs the marked library', () => {
  const bare = {};
  bare.window = bare;
  vm.createContext(bare);
  vm.runInContext(source, bare);
  assert.throws(() => bare.ttMarkdown.renderMarkdown('x'), /unavailable/);
});

// ── End to end, through marked ──────────────────────────────────────────────

// The elements marked itself writes, plus the raw-HTML allow-list, each with
// the attributes it may carry.
const OUTPUT_TAGS = {
  a: ['href', 'title', 'id', 'name'], abbr: ['title'], b: [], blockquote: [], br: [],
  code: ['class'], del: [], em: [], h1: [], h2: [], h3: [], h4: [], h5: [], h6: [],
  hr: [], i: [], img: ['src', 'alt', 'title'], input: ['checked', 'disabled', 'type'],
  ins: [], kbd: [], li: [], mark: [], ol: ['start'], p: [], pre: [], s: [], samp: [],
  small: [], span: ['id'], strong: [], sub: [], sup: [], table: [], tbody: [],
  td: ['align'], th: ['align'], thead: [], tr: [], u: [], ul: [], var: [],
};

/* Splits rendered HTML into its tags, failing on any `<` that does not open a
 * well-formed tag with double-quoted attribute values — the only shape a
 * browser would read the same way this does. Returns [{ name, attrs }]. */
function parseTags(html) {
  const tags = [];
  const TAG = /<(\/?)([a-z][a-z0-9]*)((?:\s+[a-z-]+(?:="[^"<>]*")?)*)\s*\/?>/y;
  for (let i = html.indexOf('<'); i >= 0; i = html.indexOf('<', i + 1)) {
    TAG.lastIndex = i;
    const m = TAG.exec(html);
    assert.ok(m, `stray "<" at ${i}: ${JSON.stringify(html.slice(i, i + 60))}`);
    const attrs = {};
    for (const a of m[3].matchAll(/([a-z-]+)(?:="([^"]*)")?/g)) attrs[a[1]] = a[2] ?? '';
    tags.push({ closing: m[1] === '/', name: m[2], attrs });
  }
  return tags;
}

/* Asserts every element and attribute in `html` is one the site writes, and
 * every URL passes the scheme check. */
function assertOnlyExpectedMarkup(html) {
  for (const { closing, name, attrs } of parseTags(html)) {
    assert.ok(name in OUTPUT_TAGS, `unexpected <${closing ? '/' : ''}${name}> in ${html}`);
    for (const [attr, value] of Object.entries(attrs)) {
      assert.ok(OUTPUT_TAGS[name].includes(attr), `unexpected ${name}.${attr} in ${html}`);
      if (attr === 'href' || attr === 'src') assert.ok(isSafeUrl(value), `unsafe ${name}.${attr}=${value}`);
    }
  }
}

const anchors = (html) => parseTags(html).filter((t) => t.name === 'a' && !t.closing);
const named = (html, name) => parseTags(html).filter((t) => t.name === name && !t.closing);

test('text after an unclosed inline <code>, <kbd>, <pre> or <script> is escaped', () => {
  for (const md of [
    'text <code>x <img src=x onerror=alert(1) ',
    'a <code>pre <img src=x onerror=alert(1)//',
    'a <kbd>q <svg onload=alert(1)// b',
    'a <pre>q <img src=x onerror=alert(1)>',
    'a <script>q <img src=x onerror=alert(1)>',
    'a <code><a href=x onmouseover=alert(1) </code>',
  ]) {
    const html = render(md);
    assertOnlyExpectedMarkup(html);
    assert.equal(named(html, 'img').length + named(html, 'svg').length, 0, html);
  }
});

test('the escaping holds into later paragraphs, headings and table cells', () => {
  const html = render([
    'a <code>open', '',
    'para <img src=x onerror=alert(1) ', '',
    '## heading <svg onload=alert(1) ', '',
    '| a | b |', '|---|---|', '| x <img src=x onerror=alert(1) | c |',
  ].join('\n'));
  assertOnlyExpectedMarkup(html);
  assert.equal(named(html, 'img').length + named(html, 'svg').length, 0);
  assert.equal(named(html, 'h2').length, 1);
  assert.equal(named(html, 'td').length, 2);
  assert.match(html, /<td>x &lt;img src=x onerror=alert\(1\)<\/td>/);
});

test('a link or image with another scheme renders without its URL', () => {
  for (const md of [
    '[x](javascript:alert(1))', '[x](JaVaScRiPt:alert(1))', '[x]( javascript:alert(1))',
    '[x](javascript&colon;alert(1))', '[x](data:text/html,x)', '[x](//evil.example/x)',
    '<javascript:alert(1)>', '<JaVaScRiPt:alert(1)>',
    '[x][r]\n\n[r]: javascript:alert(1) "t"', '[x][r]\n\n[r]: <javascript:alert(1)>',
    '![i](javascript:alert(1))', '<a href="javascript:alert(1)">x</a>',
    '<a href="//evil.example/x">x</a>', '<a href=javascript:alert(1)>x</a>',
  ]) {
    const html = render(md);
    assertOnlyExpectedMarkup(html);
    for (const a of anchors(html)) assert.equal(a.attrs.href, undefined, `${md} -> ${html}`);
    assert.equal(named(html, 'img').length, 0, `${md} -> ${html}`);
  }
});

test('links the prose uses keep their URL', () => {
  const html = render('[a](https://example.com/x?y=1&z=2) [b](/SPEC.md#4-documents) [c](../x.md) [d](#frag) <https://example.com> [e][r]\n\n[r]: https://example.com/ref');
  assertOnlyExpectedMarkup(html);
  assert.deepEqual(anchors(html).map((a) => a.attrs.href), [
    'https://example.com/x?y=1&z=2', '/SPEC.md#4-documents', '../x.md', '#frag',
    'https://example.com', 'https://example.com/ref',
  ]);
});

test('a link title cannot add an attribute', () => {
  for (const md of ['[x](https://ok "a\\" onmouseover=\\"alert(1)")', `[x](https://ok 'a" onmouseover="alert(1)')`]) {
    const html = render(md);
    assertOnlyExpectedMarkup(html);
    assert.deepEqual(Object.keys(anchors(html)[0].attrs).sort(), ['href', 'title']);
  }
});

test('HTML in image alt text stays text', () => {
  const html = render('![<img src=x onerror=alert(1)>](https://ok/i.png)');
  assertOnlyExpectedMarkup(html);
  const imgs = named(html, 'img');
  assert.equal(imgs.length, 1);
  assert.equal(imgs[0].attrs.src, 'https://ok/i.png');
});

test('raw HTML outside the allow-list renders as text, block or inline', () => {
  for (const md of [
    '<script>alert(1)</script>', '<img src=x onerror=alert(1)>',
    '<svg><script>alert(1)</script></svg>', '<math><mtext><table><mglyph><style><img src=x onerror=alert(1)>',
    '<a href="https://ok"<img src=x onerror=alert(1)>>x</a>', '<a <img src=x onerror=alert(1)> id=x>t</a>',
    '<a/id=x/onclick=alert(1)>t</a>', '<a id=x\nonclick=alert(1)>t</a>', 'x <iframe src="https://ok"></iframe>',
  ]) assertOnlyExpectedMarkup(render(md));
});

test('anchors and comments the prose uses render as before', () => {
  assert.equal(render('<a id="abstract"></a>\n## Abstract'), '<p><a id="abstract"></a></p>\n<h2>Abstract</h2>\n');
  assert.equal(render('<!-- a note -->\ntext'), '\n<p>text</p>\n');
  assert.equal(render('see `<code>` and <code>x</code>'), '<p>see <code>&lt;code&gt;</code> and <code>x</code></p>\n');
});
