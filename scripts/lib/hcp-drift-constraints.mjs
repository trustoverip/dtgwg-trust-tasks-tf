// The second mode of scripts/check-hcp-drift.mjs: the constraints the first
// mode (scripts/lib/hcp-drift.mjs) does not look at.
//
// The first mode compares members, required-ness, types, nullability and enum
// values. This one compares, for the same mapped objects and recursively
// through members, array items, map values and same-sized `anyOf`/`oneOf`
// branches:
//
//   minLength, maxLength,      a bound on one side only, or different values
//   minItems, maxItems, ...    (every keyword in BOUND_KEYWORDS; each is its
//                              own finding kind)
//   pattern                    the patterns differ (compared as text; the
//                              allowlist's `patternEquivalences` names the
//                              pairs that are the same language written
//                              differently)
//   format                     the formats differ, or one side has none
//   closed                     `additionalProperties: false` or
//                              `unevaluatedProperties: false` on one side only
//   union                      one side is a choice of two or more scalar
//                              shapes (`anyOf`/`oneOf`, null aside) and the
//                              other a single shape, or a choice of a
//                              different size
//
// Not compared: anything the first mode compares, the conditionals
// (`if`/`then`/`else`, `not`, `contains`, `dependent*`), and members on one
// side only (the first mode reports those). A member a `then`/`else` branch
// declares is walked, as the first mode treats it as a member.
//
// Written independently of scripts/lib/hcp-drift.mjs on purpose (docs/
// DEVELOPMENT.md, "Keep the generators and checks independent"): its own
// `$ref` resolution, walker and comparison, so that a mistake in one mode's
// reading of a schema is not silently repeated in the other. What the two
// modes share is data: scripts/lib/hcp-drift-map.json, which says which of our
// schemas corresponds to which of HCP's and which member HCP names differently
// (`renames`). Its own allowlist is scripts/lib/hcp-drift-constraints-allowlist.json.

export const BOUND_KEYWORDS = [
  'minLength',
  'maxLength',
  'minItems',
  'maxItems',
  'uniqueItems',
  'minProperties',
  'maxProperties',
  'minimum',
  'maximum',
  'exclusiveMinimum',
  'exclusiveMaximum',
  'multipleOf',
];
export const CONSTRAINT_KINDS = [...BOUND_KEYWORDS, 'pattern', 'format', 'closed', 'union'];

// Keywords that do not change what a `$ref` wrapper accepts.
const ANNOTATIONS = new Set(['description', '$comment', 'title', 'examples', 'default', 'deprecated', '$anchor', 'readOnly', 'writeOnly']);

/** One side's schemas: `load(file)` returns the parsed JSON of a file. */
export function schemaSide(load) {
  const cache = new Map();
  const file = (f) => {
    if (!cache.has(f)) cache.set(f, load(f));
    return cache.get(f);
  };
  const unescape = (s) => decodeURIComponent(s).replace(/~1/g, '/').replace(/~0/g, '~');
  const escape = (s) => String(s).replace(/~/g, '~0').replace(/\//g, '~1');
  function node(f, ptr) {
    let n = file(f);
    for (const seg of ptr ? ptr.split('/').slice(1) : []) {
      const k = unescape(seg);
      if (n === null || typeof n !== 'object' || !Object.hasOwn(n, k)) throw new Error(`${f}#${ptr}: no such pointer`);
      n = n[k];
    }
    return n;
  }
  const at = (f, ptr) => ({ f, ptr, n: node(f, ptr) });
  const dir = (f) => (f.includes('/') ? f.slice(0, f.lastIndexOf('/') + 1) : '');
  function normalize(p) {
    const out = [];
    for (const seg of p.split('/')) {
      if (seg === '..') out.pop();
      else if (seg !== '.' && seg !== '') out.push(seg);
    }
    return out.join('/');
  }
  return {
    /** `file#pointer` as the mapping writes it. */
    open(spec) {
      const i = spec.indexOf('#');
      return i < 0 ? at(spec, '') : at(spec.slice(0, i), spec.slice(i + 1));
    },
    follow(loc) {
      const r = loc.n.$ref;
      const i = r.indexOf('#');
      const target = i < 0 ? r : r.slice(0, i);
      const frag = i < 0 ? '' : r.slice(i + 1);
      if (frag && !frag.startsWith('/')) throw new Error(`${loc.f}#${loc.ptr}: $ref "${r}" is not a JSON pointer`);
      return at(target ? normalize(dir(loc.f) + target) : loc.f, frag);
    },
    down: (loc, ...segs) => at(loc.f, loc.ptr + segs.map((s) => `/${escape(s)}`).join('')),
  };
}

const key = (loc) => `${loc.f}#${loc.ptr}`;
const isObj = (n) => n !== null && typeof n === 'object' && !Array.isArray(n);

/** Skip `$ref` wrappers that add nothing but annotations. */
function canonical(side, loc) {
  let l = loc;
  for (let i = 0; i < 64 && isObj(l.n) && typeof l.n.$ref === 'string' && Object.keys(l.n).every((k) => k === '$ref' || ANNOTATIONS.has(k)); i++) {
    l = side.follow(l);
  }
  return l;
}

const onlyNull = (n) =>
  isObj(n) && ((n.type === 'null') || (Array.isArray(n.type) && n.type.length === 1 && n.type[0] === 'null') || ('const' in n && n.const === null) || (Array.isArray(n.enum) && n.enum.length === 1 && n.enum[0] === null));

function blank() {
  return { enumText: null, bounds: new Map(), patterns: new Set(), formats: new Set(), closed: false, props: new Map(), items: null, values: null, choice: null };
}

function absorb(into, from) {
  for (const [k, v] of from.bounds) {
    const cur = into.bounds.get(k);
    if (cur === undefined) into.bounds.set(k, v);
    else if (k.startsWith('min') || k.startsWith('exclusiveMin')) into.bounds.set(k, Math.max(cur, v));
    else if (k.startsWith('max') || k.startsWith('exclusiveMax')) into.bounds.set(k, Math.min(cur, v));
    else if (k === 'uniqueItems') into.bounds.set(k, cur || v);
  }
  for (const p of from.patterns) into.patterns.add(p);
  for (const f of from.formats) into.formats.add(f);
  into.closed ||= from.closed;
  for (const [k, v] of from.props) if (!into.props.has(k)) into.props.set(k, v);
  into.items ??= from.items;
  into.values ??= from.values;
  into.choice ??= from.choice;
  into.enumText ??= from.enumText;
}

/** What a schema node constrains, with `$ref`s and `allOf` folded in. */
export function constraintsOf(side, loc, depth = 0) {
  if (depth > 64) throw new Error(`${key(loc)}: schema nests too deep`);
  const out = blank();
  const n = loc.n;
  if (!isObj(n)) return out;
  if (typeof n.$ref === 'string') absorb(out, constraintsOf(side, side.follow(loc), depth + 1));
  for (const k of BOUND_KEYWORDS) if (k in n && (typeof n[k] === 'number' || typeof n[k] === 'boolean')) absorb(out, { ...blank(), bounds: new Map([[k, n[k]]]) });
  if (typeof n.pattern === 'string') out.patterns.add(n.pattern);
  // Only for describing a union's branches: closed value sets are the first mode's to compare.
  if (Array.isArray(n.enum)) out.enumText = n.enum.map((v) => JSON.stringify(v)).join(', ');
  else if ('const' in n) out.enumText = JSON.stringify(n.const);
  if (typeof n.format === 'string') out.formats.add(n.format);
  if (n.additionalProperties === false || n.unevaluatedProperties === false) out.closed = true;
  if (isObj(n.properties)) for (const name of Object.keys(n.properties)) if (!out.props.has(name)) out.props.set(name, side.down(loc, 'properties', name));
  if (isObj(n.items)) out.items ??= side.down(loc, 'items');
  if (isObj(n.additionalProperties)) out.values ??= side.down(loc, 'additionalProperties');
  if (Array.isArray(n.allOf)) n.allOf.forEach((_, i) => absorb(out, constraintsOf(side, side.down(loc, 'allOf', i), depth + 1)));
  // Members a conditional branch declares are walked; the branch's other constraints are conditional and not compared.
  for (const b of ['then', 'else']) {
    if (!isObj(n[b]) || !isObj(n[b].properties)) continue;
    for (const name of Object.keys(n[b].properties)) if (!out.props.has(name)) out.props.set(name, side.down(loc, b, 'properties', name));
  }
  for (const k of ['anyOf', 'oneOf']) {
    if (!Array.isArray(n[k])) continue;
    // A branch of bare `required` lists says which members must be present, not what they hold.
    if (n[k].every((b) => isObj(b) && Object.keys(b).every((x) => x === 'required' || ANNOTATIONS.has(x)))) continue;
    const branches = n[k].map((_, i) => side.down(loc, k, i)).filter((b) => !onlyNull(canonical(side, b).n));
    if (branches.length === 1) absorb(out, constraintsOf(side, branches[0], depth + 1));
    else if (branches.length > 1 && !out.choice) out.choice = branches.map((b) => ({ loc: b, c: constraintsOf(side, b, depth + 1) }));
  }
  return out;
}

const structured = (c) => c.props.size > 0 || c.items || c.values;
const show = (v) => (v === undefined ? 'none' : JSON.stringify(v));
const listOf = (set) => (set.size ? [...set].sort().map((x) => JSON.stringify(x)).join(' + ') : 'none');
function describe(c) {
  const bits = [];
  for (const [k, v] of c.bounds) bits.push(`${k} ${v}`);
  if (c.patterns.size) bits.push(`pattern ${listOf(c.patterns)}`);
  if (c.formats.size) bits.push(`format ${listOf(c.formats)}`);
  if (c.enumText) bits.push(`one of ${c.enumText}`);
  return bits.join(', ') || 'unconstrained';
}
const join = (p, name) => (p ? `${p}.${name}` : name);

/**
 * Compare every mapped object's constraints. `mapping.objects` are
 * `{ id, ours, hcp, renames? }` as in scripts/lib/hcp-drift-map.json.
 * Returns `{ findings, objects }`: each finding is `{ object, path, kind, detail }`,
 * and a pattern finding also carries `ours` and `hcp` (the pattern lists).
 * A pair of schema nodes reached from several objects is reported once, under
 * the first; a pair that is itself a mapped object is reported under that object.
 */
export function compareConstraints({ mapping, ours, hcp }) {
  const findings = [];
  const roots = mapping.objects.map((o) => ({ o, a: canonical(ours, ours.open(o.ours)), b: canonical(hcp, hcp.open(o.hcp)) }));
  const mapped = new Map(roots.map((r) => [`${key(r.a)}|${key(r.b)}`, r.o.id]));
  const seen = new Set();
  for (const { o, a, b } of roots) {
    const pairKey = `${key(a)}|${key(b)}`;
    if (seen.has(pairKey)) continue;
    walk({ object: o.id, renames: o.renames ?? {}, findings, mapped, seen }, a, b, '');
  }
  return { findings, objects: roots.length };

  function walk(ctx, oLoc, hLoc, p) {
    const a0 = canonical(ours, oLoc);
    const b0 = canonical(hcp, hLoc);
    const pairKey = `${key(a0)}|${key(b0)}`;
    if (p !== '' && mapped.has(pairKey)) return; // reported as that object
    if (ctx.seen.has(pairKey)) return;
    ctx.seen.add(pairKey);
    const a = constraintsOf(ours, a0);
    const b = constraintsOf(hcp, b0);
    const add = (kind, detail, extra = {}) => ctx.findings.push({ object: ctx.object, path: p || '(root)', kind, detail, at: { ours: key(a0), hcp: key(b0) }, ...extra });

    for (const k of BOUND_KEYWORDS) {
      const x = a.bounds.get(k);
      const y = b.bounds.get(k);
      if (x !== y) add(k, `ours ${show(x)}, HCP ${show(y)}`);
    }
    if (a.closed !== b.closed) add('closed', a.closed ? 'closed in ours, open in HCP' : 'closed in HCP, open in ours');

    const aScalarChoice = a.choice && !a.choice.some((x) => structured(x.c));
    const bScalarChoice = b.choice && !b.choice.some((x) => structured(x.c));
    if (a.choice && b.choice && a.choice.length === b.choice.length) {
      a.choice.forEach((x, i) => walk(ctx, x.loc, b.choice[i].loc, `${p}|${i}`));
    } else if (aScalarChoice || bScalarChoice) {
      const side = (c) => (c.choice ? `a choice of [${c.choice.map((x) => describe(x.c)).join(' | ')}]` : `a single shape [${describe(c)}]`);
      add('union', `ours is ${side(a)}; HCP is ${side(b)}`);
      // The patterns and formats are inside the union; they are not compared again.
    }
    if (!(aScalarChoice || bScalarChoice) || (a.choice && b.choice && a.choice.length === b.choice.length)) {
      const same = (x, y) => x.size === y.size && [...x].every((v) => y.has(v));
      if (!same(a.patterns, b.patterns)) add('pattern', `ours ${listOf(a.patterns)}, HCP ${listOf(b.patterns)}`, { ours: [...a.patterns].sort(), hcp: [...b.patterns].sort() });
      if (!same(a.formats, b.formats)) add('format', `ours ${listOf(a.formats)}, HCP ${listOf(b.formats)}`);
    }

    for (const [name, loc] of [...a.props].sort((x, y) => x[0].localeCompare(y[0]))) {
      const mp = join(p, name);
      const theirs = b.props.get(ctx.renames[mp] ?? name);
      if (theirs) walk(ctx, loc, theirs, mp);
    }
    if (a.items && b.items) walk(ctx, a.items, b.items, `${p}[]`);
    if (a.values && b.values) walk(ctx, a.values, b.values, `${p}{}`);
  }
}

const glob = (g) => new RegExp(`^${String(g).split('*').map((s) => s.replace(/[.+?^${}()|[\]\\]/g, '\\$&')).join('.*')}$`);

/**
 * Split findings by this mode's allowlist: `{ entries, patternEquivalences }`.
 * An entry is `{ object, path, kind, detail?, reason, ref }` (`object`, `path`
 * and `detail` may use `*`); the first entry that matches a finding takes it.
 * A pattern equivalence is `{ ours, hcp, reason, ref }`, exact pattern text: a
 * `pattern` finding with one pattern on each side, equal to a listed pair, is
 * allowed by it. Entries and equivalences that match nothing are stale.
 */
export function applyConstraintsAllowlist(findings, allowlist) {
  const invalid = [];
  const need = (e, where, keys) => {
    for (const k of keys) if (typeof e?.[k] !== 'string' || !e[k].trim()) invalid.push(`${where}: \`${k}\` is missing`);
  };
  const entries = (allowlist.entries ?? []).map((e, i) => {
    need(e, `entries[${i}]`, ['object', 'path', 'kind', 'reason', 'ref']);
    if (e.kind && !CONSTRAINT_KINDS.includes(e.kind)) invalid.push(`entries[${i}]: unknown kind "${e.kind}"`);
    if ('detail' in e && typeof e.detail !== 'string') invalid.push(`entries[${i}]: \`detail\` must be a string`);
    return { e, object: glob(e.object), path: glob(e.path), detail: 'detail' in e ? glob(e.detail) : null, hits: 0 };
  });
  const equivalences = (allowlist.patternEquivalences ?? []).map((e, i) => {
    need(e, `patternEquivalences[${i}]`, ['ours', 'hcp', 'reason', 'ref']);
    return { e, hits: 0 };
  });
  const allowed = [];
  const unexpected = [];
  for (const f of findings) {
    let by = null;
    if (f.kind === 'pattern' && f.ours?.length === 1 && f.hcp?.length === 1) {
      const q = equivalences.find((q) => q.e.ours === f.ours[0] && q.e.hcp === f.hcp[0]);
      if (q) {
        q.hits++;
        by = { ...q.e, ref: q.e.ref };
      }
    }
    if (!by) {
      const m = entries.find((m) => m.e.kind === f.kind && m.object.test(f.object) && m.path.test(f.path) && (!m.detail || m.detail.test(f.detail)));
      if (m) {
        m.hits++;
        by = m.e;
      }
    }
    if (by) allowed.push({ ...f, entry: by });
    else unexpected.push(f);
  }
  const stale = [
    ...entries.filter((m) => !m.hits).map((m) => ({ ...m.e, label: `${m.e.object} ${m.e.path} ${m.e.kind}${m.e.detail ? ` (${m.e.detail})` : ''}` })),
    ...equivalences.filter((q) => !q.hits).map((q) => ({ ...q.e, label: `pattern ${JSON.stringify(q.e.ours)} ~ ${JSON.stringify(q.e.hcp)}` })),
  ];
  return { allowed, unexpected, stale, invalid };
}
