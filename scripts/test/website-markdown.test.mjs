// Tests for website/assets/markdown.js — the raw-HTML and URL handling the
// site applies to rendered spec, binding and framework-spec prose. `marked`
// itself is loaded from a CDN by the page and is not installed here, so these
// cover the two functions the renderer hooks into it.
//
//   node --test scripts/test/website-markdown.test.mjs   (npm run test:website)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const source = fs.readFileSync(path.join(here, '..', '..', 'website', 'assets', 'markdown.js'), 'utf8');
const ctx = {};
ctx.window = ctx;
vm.createContext(ctx);
vm.runInContext(source, ctx);
const { isSafeUrl, sanitizeRawHtml, renderMarkdown } = ctx.ttMarkdown;

test('relative, fragment, http(s) and mailto URLs are kept', () => {
  for (const url of [
    '/SPEC.md#4-documents', '../../SPEC.md#a', 'spec.md', '#response', '',
    'https://trustoverip.github.io/x', 'http://example.com/a?b=c&d=e',
    'HTTPS://EXAMPLE.COM', 'mailto:someone@example.com', '//example.com/x',
  ]) assert.equal(isSafeUrl(url), true, url);
});

test('other schemes are refused, however they are spelled', () => {
  for (const url of [
    'javascript:alert(1)', 'JavaScript:alert(1)', ' javascript:alert(1)',
    'java\tscript:alert(1)', 'java\nscript:alert(1)', '\u0001javascript:x',
    'javascript&colon;alert(1)', 'javascript&#58;alert(1)', 'javascript&#x3a;x',
    'data:text/html,<p>x</p>', 'vbscript:x', 'file:///etc/hosts',
    'https&#58;//example.com',
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
  const saved = ctx.marked;
  delete ctx.marked;
  assert.throws(() => renderMarkdown('x'), /unavailable/);
  ctx.marked = saved;
});
