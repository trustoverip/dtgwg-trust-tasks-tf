// The comparison behind scripts/check-hcp-drift.mjs: the hcp/* schemas against
// the JSON schemas HCP publishes for the operations and objects they carry.
//
// Both sides are read through a loader (file -> parsed JSON), so the engine
// knows nothing of git or the file system: the CLI loads HCP's files with
// `git show <commit>:<path>` and ours from the tree, and the tests from
// in-memory fixtures. `$ref`s are followed on both sides, each relative to the
// file it appears in.
//
// What is compared, per mapped object and recursively through members, array
// items, map values and object `anyOf` branches:
//
//   ours-only / hcp-only  a member declared (in `properties` or `required`) on
//                         one side only
//   required              a member required on one side only
//   type                  the JSON types allowed, null aside
//   nullable              null allowed on one side only
//   enum                  closed value sets (`enum`, `const`) that differ, or
//                         a closed set on one side and an open string on the other
//   alternatives          `anyOf`/`oneOf` of `{ required: [...] }` branches
//                         (which members must be present) that differ
//
// Not compared: bounds (`minItems`, `maxLength`, ...), `pattern`, `format`,
// `additionalProperties: false` on our closed payloads, and the conditionals
// (`if`/`then`/`else`, `not`, `contains`) the framework's generators cannot
// express. A member a `then`/`else` branch declares counts as an optional
// member, since HCP declares some (`content.value`) only there. The bounds,
// patterns, formats and closed payloads are compared by the second mode,
// scripts/lib/hcp-drift-constraints.mjs, written independently of this one.
//
// A `$ref` pair that is itself a mapped object is compared once, as that
// object, and not again wherever it is used.

import path from 'node:path';

export const KINDS = ['ours-only', 'hcp-only', 'required', 'type', 'nullable', 'enum', 'alternatives'];

const pp = path.posix;

/** A resolver over one side's files. `load(file)` returns the parsed JSON. */
export function makeResolver(load) {
  const docs = new Map();
  const doc = (file) => {
    if (!docs.has(file)) docs.set(file, load(file));
    return docs.get(file);
  };
  function at(file, pointer = '') {
    let node = doc(file);
    for (const seg of pointer.split('/').slice(1)) {
      const k = decodeURIComponent(seg).replace(/~1/g, '/').replace(/~0/g, '~');
      if (node === null || typeof node !== 'object' || !(k in node)) throw new Error(`${file}#${pointer}: no such pointer`);
      node = node[k];
    }
    return node;
  }
  /** Locate `file#pointer`, as a mapping names it. */
  function locate(spec) {
    const [file, pointer = ''] = spec.split('#');
    return { file, pointer, node: at(file, pointer) };
  }
  function ref(from, r) {
    const [f, frag = ''] = r.split('#');
    if (frag && !frag.startsWith('/')) throw new Error(`${from.file}: $ref "${r}": only JSON-pointer fragments are supported`);
    const file = f ? pp.normalize(pp.join(pp.dirname(from.file), f)) : from.file;
    return { file, pointer: frag, node: at(file, frag) };
  }
  return { locate, ref };
}

export const locKey = (loc) => `${loc.file}#${loc.pointer}`;
const child = (loc, ...segs) => ({
  file: loc.file,
  pointer: `${loc.pointer}/${segs.map((s) => String(s).replace(/~/g, '~0').replace(/\//g, '~1')).join('/')}`,
  node: segs.reduce((n, s) => n[s], loc.node),
});

const emptyShape = () => ({
  types: null, // Set of JSON type names, or null when the schema does not say
  values: null, // Set of JSON-encoded allowed values (closed), or null
  openValues: null, // known values of an open set (an enum alongside a free string)
  props: null, // Map name -> loc, or null when no member is declared
  required: new Set(),
  items: null,
  addl: null,
  alts: null, // structured anyOf branches (array of loc)
  reqAlts: [], // `anyOf`/`oneOf` of { required } alternatives, each as text
  refKeys: new Set(),
});

const typeOf = (v) => (v === null ? 'null' : Array.isArray(v) ? 'array' : Number.isInteger(v) ? 'integer' : typeof v);
const intersect = (a, b) => new Set([...a].filter((x) => b.has(x)));

function merge(a, b) {
  const out = { ...a };
  out.types = a.types && b.types ? intersect(a.types, b.types) : a.types ?? b.types;
  out.values = a.values && b.values ? intersect(a.values, b.values) : a.values ?? b.values;
  out.openValues = a.openValues ?? b.openValues;
  if (b.props) {
    out.props = new Map(a.props ?? []);
    for (const [k, v] of b.props) if (!out.props.has(k)) out.props.set(k, v);
  }
  out.required = new Set([...a.required, ...b.required]);
  out.items = a.items ?? b.items;
  out.addl = a.addl ?? b.addl;
  out.alts = a.alts ?? b.alts;
  out.reqAlts = [...a.reqAlts, ...b.reqAlts];
  out.refKeys = new Set([...a.refKeys, ...b.refKeys]);
  return out;
}

const isRequiredOnly = (n) =>
  n && typeof n === 'object' && Array.isArray(n.required) && Object.keys(n).every((k) => ['required', 'description', '$comment'].includes(k));

/** The shape of a located schema node, with its `$ref`s, `allOf` and `anyOf` folded in. */
export function shapeOf(resolver, loc, depth = 0) {
  if (depth > 64) throw new Error(`${locKey(loc)}: $ref chain too deep`);
  let s = emptyShape();
  const n = loc.node;
  if (n === true || n === undefined || n === null || typeof n !== 'object') return s;
  if (typeof n.$ref === 'string') {
    const target = resolver.ref(loc, n.$ref);
    s = merge(s, shapeOf(resolver, target, depth + 1));
    s.refKeys.add(locKey(target));
  }
  if (n.type !== undefined) {
    const t = new Set([n.type].flat());
    s.types = s.types ? intersect(s.types, t) : t;
  }
  if ('const' in n) s.values = new Set([JSON.stringify(n.const)]);
  if (Array.isArray(n.enum)) s.values = new Set(n.enum.map((v) => JSON.stringify(v)));
  if (n.properties && typeof n.properties === 'object') {
    s.props = new Map(s.props ?? []);
    for (const k of Object.keys(n.properties)) s.props.set(k, child(loc, 'properties', k));
  }
  if (Array.isArray(n.required)) for (const k of n.required) s.required.add(k);
  if (n.items && typeof n.items === 'object') s.items = child(loc, 'items');
  if (n.additionalProperties && typeof n.additionalProperties === 'object') s.addl = child(loc, 'additionalProperties');
  if (Array.isArray(n.allOf)) n.allOf.forEach((_, i) => (s = merge(s, shapeOf(resolver, child(loc, 'allOf', i), depth + 1))));
  for (const k of ['then', 'else']) {
    if (!n[k] || typeof n[k] !== 'object') continue;
    const b = shapeOf(resolver, child(loc, k), depth + 1);
    if (b.props) {
      s.props = new Map(s.props ?? []);
      for (const [name, v] of b.props) if (!s.props.has(name)) s.props.set(name, v);
    }
  }
  for (const k of ['anyOf', 'oneOf']) {
    if (!Array.isArray(n[k])) continue;
    if (n[k].every(isRequiredOnly)) {
      s.reqAlts.push(`${k === 'oneOf' ? 'exactly' : 'at least'} one of (${n[k].map((b) => [...b.required].sort().join(' + ')).sort().join(' | ')})`);
      continue;
    }
    const branches = n[k].map((_, i) => ({ loc: child(loc, k, i), shape: shapeOf(resolver, child(loc, k, i), depth + 1) }));
    const isNull = (b) => b.shape.types?.size === 1 && b.shape.types.has('null');
    const nonNull = branches.filter((b) => !isNull(b));
    const hasNull = nonNull.length < branches.length;
    const structured = (b) => b.shape.props || b.shape.items || b.shape.alts;
    let union;
    if (nonNull.length === 1) {
      union = nonNull[0].shape;
    } else if (nonNull.some(structured)) {
      union = emptyShape();
      union.alts = nonNull.map((b) => b.loc);
      const ts = nonNull.map((b) => b.shape.types);
      union.types = ts.every(Boolean) ? new Set(ts.flatMap((t) => [...t])) : null;
    } else {
      union = emptyShape();
      const ts = nonNull.map((b) => b.shape.types ?? (b.shape.values ? new Set([...b.shape.values].map((v) => typeOf(JSON.parse(v)))) : null));
      union.types = ts.every(Boolean) ? new Set(ts.flatMap((t) => [...t])) : null;
      const vs = nonNull.map((b) => b.shape.values);
      if (vs.every(Boolean)) union.values = new Set(vs.flatMap((v) => [...v]));
      else if (vs.some(Boolean)) union.openValues = new Set(vs.filter(Boolean).flatMap((v) => [...v]));
    }
    if (hasNull) {
      if (union.types) union.types = new Set([...union.types, 'null']);
      if (union.values) union.values = new Set([...union.values, 'null']);
    }
    // A node's own `type` bounds its branches (common.json's `category` is a
    // string whose anyOf only adds patterns); otherwise the branches decide.
    const ownTypes = s.types;
    s = merge(s, { ...union, types: null });
    s.types = ownTypes && union.types ? intersect(ownTypes, union.types) : ownTypes ?? union.types;
  }
  if (!s.types && s.values) s.types = new Set([...s.values].map((v) => typeOf(JSON.parse(v))));
  // Null as an enum value is nullability, not a value.
  if (s.values?.has('null')) {
    s.values = new Set([...s.values].filter((v) => v !== 'null'));
    s.types = new Set([...(s.types ?? []), 'null']);
  }
  return s;
}

const fmt = (set) => [...set].sort().join(', ');
const nonNullTypes = (t) => new Set([...t].filter((x) => x !== 'null'));
const eqSet = (a, b) => a.size === b.size && [...a].every((x) => b.has(x));
const join = (p, name) => (p ? `${p}.${name}` : name);
const fmtValues = (set) => [...set].sort().join(', ') || '(none)';

/**
 * Compare every mapped object. `mapping.objects` entries are
 * `{ id, ours: 'file#pointer', hcp: 'file#pointer', renames?: { ourPath: hcpName } }`.
 * Returns `{ findings, compared, covered }`: findings `{ object, path, kind, detail }`.
 */
export function compareMapping({ mapping, ours, hcp }) {
  const findings = [];
  const covered = [];
  const compared = [];
  const objects = mapping.objects.map((o) => ({ ...o, oursLoc: ours.locate(o.ours), hcpLoc: hcp.locate(o.hcp) }));
  const pairs = new Map(objects.map((o) => [`${locKey(o.oursLoc)}|${locKey(o.hcpLoc)}`, o.id]));
  for (const o of objects) {
    const covering = coveredBy(o, objects, pairs, ours, hcp);
    if (covering) {
      covered.push({ object: o.id, by: covering });
      continue;
    }
    compared.push(o.id);
    const ctx = { object: o.id, renames: o.renames ?? {}, pairs, self: o.id, findings, ours, hcp, stack: new Set() };
    compareNode(ctx, o.oursLoc, o.hcpLoc, '');
  }
  return { findings, compared, covered };
}

function coveredBy(o, objects, pairs, ours, hcp) {
  const a = shapeOf(ours, o.oursLoc);
  const b = shapeOf(hcp, o.hcpLoc);
  const own = [locKey(o.oursLoc)];
  const their = [locKey(o.hcpLoc)];
  for (const ok of [...own, ...a.refKeys]) {
    for (const hk of [...their, ...b.refKeys]) {
      const id = pairs.get(`${ok}|${hk}`);
      if (id && id !== o.id) return id;
    }
  }
  return null;
}

function compareNode(ctx, oLoc, hLoc, p) {
  const pairKey = `${locKey(oLoc)}|${locKey(hLoc)}`;
  if (ctx.stack.has(pairKey)) return;
  const a = shapeOf(ctx.ours, oLoc);
  const b = shapeOf(ctx.hcp, hLoc);
  if (p !== '') {
    for (const ok of a.refKeys) for (const hk of b.refKeys) if (ctx.pairs.has(`${ok}|${hk}`)) return; // compared as that object
  }
  ctx.stack.add(pairKey);
  const add = (kind, path, detail) => ctx.findings.push({ object: ctx.object, path: path || '(root)', kind, detail });

  if (a.types && b.types) {
    const at = nonNullTypes(a.types);
    const bt = nonNullTypes(b.types);
    if (at.size && bt.size && !eqSet(at, bt)) add('type', p, `ours ${fmt(at)}, HCP ${fmt(bt)}`);
    const an = a.types.has('null');
    const bn = b.types.has('null');
    if (an !== bn) add('nullable', p, an ? 'null allowed in ours only' : 'null allowed in HCP only');
  }
  if (a.values && b.values) {
    if (!eqSet(a.values, b.values)) {
      const oo = [...a.values].filter((v) => !b.values.has(v));
      const ho = [...b.values].filter((v) => !a.values.has(v));
      add('enum', p, `values in ours only: ${fmtValues(oo)}; in HCP only: ${fmtValues(ho)}`);
    }
  } else if (a.values || b.values) {
    const [closed, side, other] = a.values ? [a.values, 'ours', b] : [b.values, 'HCP', a];
    if (other.types || other.openValues) add('enum', p, `closed in ${side} only (${fmtValues(closed)})${other.openValues ? `; the other side also allows any other value besides ${fmtValues(other.openValues)}` : ''}`);
  } else if (a.openValues && b.openValues && !eqSet(a.openValues, b.openValues)) {
    add('enum', p, `named values: ours ${fmtValues(a.openValues)}; HCP ${fmtValues(b.openValues)}`);
  }
  const ra = [...a.reqAlts].sort();
  const rb = [...b.reqAlts].sort();
  if (ra.join(';') !== rb.join(';')) add('alternatives', p, `members present: ours ${ra.join(', ') || 'no rule'}; HCP ${rb.join(', ') || 'no rule'}`);

  if (a.alts && b.alts) {
    if (a.alts.length !== b.alts.length) add('type', p, `ours has ${a.alts.length} object alternatives, HCP ${b.alts.length}`);
    else a.alts.forEach((al, i) => compareNode(ctx, al, b.alts[i], `${p}|${i}`));
  } else if ((a.alts && b.props) || (b.alts && a.props)) {
    add('type', p, a.alts ? `ours is a choice of ${a.alts.length} shapes, HCP a single object` : `HCP is a choice of ${b.alts.length} shapes, ours a single object`);
  }

  if (a.props && b.props) {
    const oursNames = new Set([...a.props.keys(), ...a.required]);
    const hcpNames = new Set([...b.props.keys(), ...b.required]);
    const toHcp = new Map([...oursNames].map((n) => [n, ctx.renames[join(p, n)] ?? n]));
    const matched = new Set();
    for (const n of [...oursNames].sort()) {
      const hn = toHcp.get(n);
      const mp = join(p, n);
      const label = hn === n ? '' : ` (HCP \`${hn}\`)`;
      if (!hcpNames.has(hn)) {
        add('ours-only', mp, `${a.required.has(n) ? 'required' : 'optional'} in ours${label}; HCP does not declare it`);
        continue;
      }
      matched.add(hn);
      const ar = a.required.has(n);
      const br = b.required.has(hn);
      if (ar !== br) add('required', mp, `${ar ? 'required' : 'optional'} in ours, ${br ? 'required' : 'optional'} in HCP${label}`);
      const ol = a.props.get(n);
      const hl = b.props.get(hn);
      if (ol && hl) compareNode(ctx, ol, hl, mp);
    }
    for (const hn of [...hcpNames].sort()) {
      if (!matched.has(hn)) add('hcp-only', join(p, hn), `${b.required.has(hn) ? 'required' : 'optional'} in HCP; ours does not declare it`);
    }
  } else if ((a.props || b.props) && !a.alts && !b.alts) {
    // One side details an object the other leaves as a bare `{ type: object }`.
    const [detailed, bare, kind, side] = a.props ? [a, b, 'ours-only', 'ours'] : [b, a, 'hcp-only', 'HCP'];
    if (bare.types?.has('object') && !bare.items) {
      for (const n of [...new Set([...detailed.props.keys(), ...detailed.required])].sort()) {
        add(kind, join(p, n), `${detailed.required.has(n) ? 'required' : 'optional'} in ${side}; the other side declares this object without members`);
      }
    }
  }
  if (a.items && b.items) compareNode(ctx, a.items, b.items, `${p}[]`);
  if (a.addl && b.addl) compareNode(ctx, a.addl, b.addl, `${p}{}`);
  ctx.stack.delete(pairKey);
}

const globRe = (g) => new RegExp(`^${g.split('*').map((s) => s.replace(/[.+?^${}()|[\]\\]/g, '\\$&')).join('.*')}$`);

/**
 * Split findings by the allowlist. Each entry is `{ object, path, kind, reason, ref }`;
 * `object` and `path` may use `*` as a wildcard. Returns `{ allowed, unexpected, stale, invalid }`.
 */
export function applyAllowlist(findings, entries) {
  const invalid = [];
  const compiled = entries.map((e, i) => {
    for (const k of ['object', 'path', 'kind', 'reason', 'ref']) {
      if (typeof e[k] !== 'string' || !e[k].trim()) invalid.push(`entry ${i}: missing \`${k}\``);
    }
    if (e.kind && !KINDS.includes(e.kind)) invalid.push(`entry ${i}: unknown kind "${e.kind}"`);
    return { e, o: globRe(e.object ?? ''), p: globRe(e.path ?? ''), hits: 0 };
  });
  const allowed = [];
  const unexpected = [];
  for (const f of findings) {
    const c = compiled.find((c) => c.e.kind === f.kind && c.o.test(f.object) && c.p.test(f.path));
    if (c) {
      c.hits++;
      allowed.push({ ...f, entry: c.e });
    } else unexpected.push(f);
  }
  const stale = compiled.filter((c) => c.hits === 0).map((c) => c.e);
  return { allowed, unexpected, stale, invalid };
}

/**
 * The carried specs' sides that the mapping must account for: each spec with a
 * `carries` block has a `request` and, when its schema has `$defs.Response`, a
 * `response`. Returns the `slug:side` ids neither mapped nor listed in `notCompared`,
 * and the mapping/notCompared ids that name no carried side.
 */
export function coverage({ carried, mapping }) {
  const want = new Set(carried.flatMap((c) => [`${c.slug}:request`, ...(c.hasResponse ? [`${c.slug}:response`] : [])]));
  const have = new Set([...mapping.objects.map((o) => o.id), ...Object.keys(mapping.notCompared ?? {})]);
  const missing = [...want].filter((id) => !have.has(id)).sort();
  const unknown = [...have].filter((id) => !want.has(id) && !id.startsWith(`${mapping.sharedSlug ?? 'hcp/_shared'}:`)).sort();
  return { missing, unknown };
}
