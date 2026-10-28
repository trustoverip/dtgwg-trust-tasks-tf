/// Forward-minor reading along a declared `extends` chain (SPEC §5.2,
/// "Declared extension").
///
/// A specification version may declare `extends: "M.K"`: it keeps everything
/// `M.K` defines and only adds. A consumer implementing `M.K` SHOULD then accept
/// a document of the later version by ignoring the payload members it does not
/// recognise. Without a declaration it SHOULD NOT — a draft may break in a MINOR.
///
/// [readForVersion] is that rule as a function. The application keeps passing
/// the [SpecPolicy] of the version it implements; the generated
/// `package:trust_tasks/specs/extends_index.dart` carries which versions extend
/// which, so a library upgrade alone teaches an application written against
/// `x/0.2` to read an `x/0.3` document.
///
/// ```dart
/// switch (readForVersion(json, acl_grant.spec, acl_grant.Payload.fromJson)) {
///   case VersionAccepted(:final doc, :final received):
///     final outcome = await consumeInbound<acl_grant.Payload, acl_grant.Response>(
///       doc: doc,
///       received: received, // proofs verify over this, never the pruned copy
///       spec: acl_grant.spec,
///       // ...
///     );
///   case VersionRefused(:final document, :final reason):
///     emit(reject(transport, document, uuid.v4(), reason));
/// }
/// ```
///
/// Hand-written. The Rust dispatcher, and the TypeScript and Go runtimes, carry
/// their own versions of the same rule; a document one of them routes forward
/// another must too.
library;

import 'dart:convert';

import 'package:trust_tasks/specs/extends_index.dart' as generated;

import 'canonical.dart';
import 'codes.dart';
import 'document.dart';
import 'type_uri.dart';

/// The versions [typeUri]'s declared `extends` chain reaches, nearest first,
/// excluding the version itself.
///
/// [typeUri] may be in any SPEC §6.1 form and may carry a fragment, which is
/// ignored; anything that is not a Type URI has an empty chain. Each link is
/// looked up under the *same* authority, form and slug with only the version
/// changed ([TypeUri.withVersion]), so a chain never leaves the authority it
/// started under — a third-party specification's chain can never reach a
/// registry version of the same slug.
///
/// [index] defaults to the generated one, which covers the public registry; a
/// consumer of third-party (§6.5) specifications passes its own, keyed the
/// same way, by bare request Type URI. A cycle — which the build refuses,
/// since `extends` must name an earlier version — ends the walk rather than
/// looping, as does a link that is not a `MAJOR.MINOR`.
List<String> extensionChain(
  String typeUri, {
  Map<String, String> index = generated.extendsIndex,
}) {
  final parsed = TypeUri.tryParse(typeUri);
  if (parsed == null) return const <String>[];
  final chain = <String>[];
  final seen = <String>{parsed.version};
  var current = parsed.bare;
  while (true) {
    final next = index[current.toString()];
    if (next == null) break;
    final version = parseTypeUriVersion(next);
    if (version == null) break;
    current = current.withVersion(version.$1, version.$2);
    if (!seen.add(current.version)) break;
    chain.add(current.version);
  }
  return chain;
}

/// Whether a document typed [receivedTypeUri] may be read as the version
/// [implementedTypeUri] names, because the former's declared `extends` chain
/// reaches the latter. Same specification only — the same authority, form and
/// slug ([TypeUri.isSameSpecification]); fragments are ignored.
///
/// False for the same version — that is not an extension, it is the version.
bool extendsTo(
  String receivedTypeUri,
  String implementedTypeUri, {
  Map<String, String> index = generated.extendsIndex,
}) {
  final received = TypeUri.tryParse(receivedTypeUri);
  final implemented = TypeUri.tryParse(implementedTypeUri);
  if (received == null || implemented == null) return false;
  if (!received.isSameSpecification(implemented)) return false;
  return extensionChain(receivedTypeUri, index: index)
      .contains(implemented.version);
}

/// The outcome of [readForVersion].
///
/// Sealed, so a `switch` over it is exhaustive. Produced by this library and
/// never parsed from a peer, so closing it costs no forward compatibility.
sealed class VersionRead<P> {
  const VersionRead();
}

/// The document may be consumed as the implemented version.
final class VersionAccepted<P> extends VersionRead<P> {
  const VersionAccepted({
    required this.doc,
    required this.received,
    required this.forward,
  });

  /// The document, its payload read as the implemented version's.
  ///
  /// Its `type` is the one **received**, never rewritten to the implemented
  /// version's: a response built from it with [respondWith] is therefore typed
  /// `<received type>#response`, the version the producer used. Because the
  /// received version only adds to the implemented one, a response shaped for
  /// the implemented version is valid for it.
  final TrustTaskDocument<P> doc;

  /// The document exactly as it was handed to [readForVersion], unpruned and
  /// unmodified. Pass it to `consumeInbound` as `received`, so a proof is
  /// verified over what the producer signed rather than over [doc]'s typed
  /// re-encoding — which, in the forward case, has had members removed.
  final Map<String, dynamic> received;

  /// True when the document is of a later version read as the implemented one
  /// through a declared `extends` chain; false for the implemented version
  /// itself.
  final bool forward;
}

/// The document is not one the implemented version may read.
final class VersionRefused<P> extends VersionRead<P> {
  const VersionRefused({required this.document, required this.reason});

  /// The received document with its payload left as raw JSON — enough to build
  /// the §8.1-routed error response with [reject].
  final TrustTaskDocument<Object?> document;

  /// `unsupportedVersion` when the specification — authority, form and slug —
  /// matches and the received version's `extends` chain does not reach the
  /// implemented one; `unsupportedType` when the document is not of the
  /// implemented specification at all, including the same slug under another
  /// authority.
  final RejectReason reason;
}

/// Read [received] as the version [spec] describes, following a declared
/// `extends` chain forward (SPEC §5.2).
///
/// * Same version (bare Type URI and variant equal): read as-is, unchanged from
///   calling `TrustTaskDocument.fromJson` yourself.
/// * A later version of the same specification — authority, form and slug —
///   whose `extends` chain, per [index] (which defaults to the generated one;
///   pass your own for third-party specifications), reaches [spec]'s: the
///   payload is pruned with [prunePayload] against `spec.payloadSchema`, then
///   read with [payloadFromJson]. The document keeps the received `type`.
/// * Anything else: [VersionRefused].
///
/// [received] is never modified, and [VersionAccepted.received] is that same
/// object — the pruned payload lives in a copy. That is how a proof stays over
/// the document as received: `consumeInbound` verifies over `received` when it
/// is passed.
///
/// A document variant (`#request` / none vs `#response`) must match [spec]'s:
/// pass a generated `responseSpec` to read a response.
///
/// Throws as `TrustTaskDocument.fromJson` does when the envelope itself is
/// malformed (no string `id` or `type`) — that is §7.2 item 1, the caller's.
VersionRead<P> readForVersion<P>(
  Map<String, dynamic> received,
  SpecPolicy spec,
  P Function(Object? payload) payloadFromJson, {
  Map<String, String> index = generated.extendsIndex,
}) {
  final type = received['type'];
  if (type is! String) {
    // Let the envelope decoder report it, exactly as it would without us.
    return VersionAccepted<P>(
      doc: TrustTaskDocument<P>.fromJson(received, payloadFromJson),
      received: received,
      forward: false,
    );
  }

  final receivedBare = bareTypeUri(type);
  final implementedBare = bareTypeUri(spec.typeUri);
  final sameVariant = _variantOf(type) == _variantOf(spec.typeUri);

  if (sameVariant && receivedBare == implementedBare) {
    return VersionAccepted<P>(
      doc: TrustTaskDocument<P>.fromJson(received, payloadFromJson),
      received: received,
      forward: false,
    );
  }

  VersionRefused<P> refuse(StandardCode code, String message) =>
      VersionRefused<P>(
        document: TrustTaskDocument<Object?>.fromJson(received, (p) => p),
        reason: RejectReason(code: code, message: message),
      );

  final r = TypeUri.tryParse(receivedBare);
  final i = TypeUri.tryParse(implementedBare);
  if (!sameVariant || r == null || i == null || !r.isSameSpecification(i)) {
    return refuse(
      StandardCode.unsupportedType,
      'this consumer does not implement $type',
    );
  }
  if (!extendsTo(receivedBare, implementedBare, index: index)) {
    return refuse(
      StandardCode.unsupportedVersion,
      'version ${r.version} is not supported, and does not declare that it '
      'extends ${i.version} (SPEC §5.2)',
    );
  }

  final schema = spec.payloadSchema;
  final payload = received['payload'];
  final pruned = <String, dynamic>{
    ...received,
    'payload': schema == null ? payload : prunePayload(payload, schema),
  };
  return VersionAccepted<P>(
    doc: TrustTaskDocument<P>.fromJson(pruned, payloadFromJson),
    received: received,
    forward: true,
  );
}

/// [payload] with every object member [schemaJson] does not know removed, where
/// the schema closes the object with `additionalProperties: false`.
///
/// Returns a new tree; [payload] is not modified. Members are only ever
/// removed, never added or rewritten, and only where the schema says the
/// object is closed — an open object keeps its unknown members, because the
/// implemented version accepts them anyway.
///
/// How the schema is followed:
///
/// * `properties` recurse; `additionalProperties` as a schema recurses into
///   the members it governs; `additionalProperties: false` drops the rest.
/// * `$ref` to a JSON Pointer within the schema (`#/$defs/X`, `#`) is followed.
///   Generated schemas have every cross-file `$ref` already inlined.
/// * `items` recurses into each array element.
/// * `allOf` branches are applied in turn.
/// * `oneOf` / `anyOf`: the value is pruned against a branch only when exactly
///   one branch is *plausible* for it — its `type`, `const`, `enum` and
///   `required` members, and the `const` / `enum` of the members it names, all
///   fit. With more than one plausible branch, or none, the value is left
///   untouched there: pruning against the wrong branch would remove members
///   the right one needs, and an unpruned extra member is refused by item 2
///   validation rather than silently lost.
///
/// Not followed: `patternProperties`, `prefixItems`, `unevaluatedProperties`,
/// `if`/`then`/`else`, `dependentSchemas` and remote `$ref`s — no registered
/// schema uses them. Members they would govern are left untouched.
Object? prunePayload(Object? payload, String schemaJson) {
  final root = _decodeSchema(schemaJson);
  return _Pruner(root).prune(payload, root, 0);
}

/* ── internals ──────────────────────────────────────────────────────────── */

/// `#request` is the request variant spelled out; both read as the request.
String _variantOf(String typeUri) {
  final hash = typeUri.indexOf('#');
  if (hash < 0) return '';
  final fragment = typeUri.substring(hash + 1);
  return fragment == 'request' ? '' : fragment;
}

Object? _decodeSchema(String json) {
  try {
    return jsonDecode(json);
  } on FormatException {
    return true;
  }
}

class _Pruner {
  _Pruner(this.root);

  final Object? root;

  /// Deep enough for any registered payload; a guard, not a limit anyone meets.
  static const int _maxDepth = 128;

  Object? prune(Object? value, Object? schema, int depth) {
    if (depth > _maxDepth) return value;
    final node = _resolve(schema);
    if (node is! Map<String, dynamic>) return value;

    var out = value;

    final allOf = node['allOf'];
    if (allOf is List) {
      for (final branch in allOf) {
        out = prune(out, branch, depth + 1);
      }
    }

    if (out is Map<String, dynamic>) {
      out = _pruneObject(out, node, depth);
    } else if (out is List) {
      final items = node['items'];
      if (items is Map<String, dynamic>) {
        out = <Object?>[for (final e in out) prune(e, items, depth + 1)];
      }
    }

    for (final keyword in const ['oneOf', 'anyOf']) {
      final branches = node[keyword];
      if (branches is! List) continue;
      final plausible = <Object?>[
        for (final b in branches)
          if (_plausible(out, b)) b,
      ];
      if (plausible.length == 1) out = prune(out, plausible.single, depth + 1);
    }
    return out;
  }

  Map<String, dynamic> _pruneObject(
    Map<String, dynamic> value,
    Map<String, dynamic> node,
    int depth,
  ) {
    final properties = node['properties'];
    final additional = node['additionalProperties'];
    if (properties is! Map<String, dynamic> && additional == null) return value;
    final props = properties is Map<String, dynamic>
        ? properties
        : const <String, dynamic>{};

    final out = <String, dynamic>{};
    for (final entry in value.entries) {
      if (props.containsKey(entry.key)) {
        out[entry.key] = prune(entry.value, props[entry.key], depth + 1);
      } else if (additional == false) {
        continue; // unknown to this version, and the object is closed
      } else if (additional is Map<String, dynamic>) {
        out[entry.key] = prune(entry.value, additional, depth + 1);
      } else {
        out[entry.key] = entry.value;
      }
    }
    return out;
  }

  /// Whether [value] could be an instance of [schema], judged shallowly.
  bool _plausible(Object? value, Object? schema) {
    final node = _resolve(schema);
    if (node is bool) return node;
    if (node is! Map<String, dynamic>) return true;
    if (!_fitsShallow(value, node)) return false;
    if (value is Map<String, dynamic>) {
      final required = node['required'];
      if (required is List) {
        for (final r in required) {
          if (!value.containsKey(r)) return false;
        }
      }
      final properties = node['properties'];
      if (properties is Map<String, dynamic>) {
        for (final entry in properties.entries) {
          if (!value.containsKey(entry.key)) continue;
          final member = _resolve(entry.value);
          if (member is Map<String, dynamic> &&
              !_fitsShallow(value[entry.key], member)) {
            return false;
          }
        }
      }
    }
    return true;
  }

  /// `type`, `const` and `enum` of [node] against [value], nothing deeper.
  bool _fitsShallow(Object? value, Map<String, dynamic> node) {
    final type = node['type'];
    if (type != null) {
      final types = type is List ? type : <Object?>[type];
      if (!types.any((t) => _isType(value, t))) return false;
    }
    if (node.containsKey('const') && !_jsonEquals(node['const'], value)) {
      return false;
    }
    final enumValues = node['enum'];
    if (enumValues is List && !enumValues.any((e) => _jsonEquals(e, value))) {
      return false;
    }
    return true;
  }

  static bool _isType(Object? value, Object? type) => switch (type) {
        'object' => value is Map,
        'array' => value is List,
        'string' => value is String,
        'boolean' => value is bool,
        'null' => value == null,
        'number' => value is num,
        'integer' =>
          value is int || (value is double && value == value.floorToDouble()),
        _ => true,
      };

  static bool _jsonEquals(Object? a, Object? b) =>
      canonicalJson(a) == canonicalJson(b);

  /// Follow local `$ref`s from [schema] to the node they name.
  Object? _resolve(Object? schema) {
    var node = schema;
    for (var hops = 0; hops < 32; hops++) {
      if (node is! Map<String, dynamic>) return node;
      final ref = node[r'$ref'];
      if (ref is! String) return node;
      final target = _pointer(ref);
      if (target == null) return node;
      // A `$ref` with siblings: 2020-12 applies both. The siblings in the
      // registry are annotations (`description`), so the target is what counts.
      node = target;
    }
    return node;
  }

  Object? _pointer(String ref) {
    if (!ref.startsWith('#')) return null;
    var node = root;
    final pointer = Uri.decodeComponent(ref.substring(1));
    if (pointer.isEmpty) return node;
    if (!pointer.startsWith('/')) return null;
    for (final raw in pointer.substring(1).split('/')) {
      final token = raw.replaceAll('~1', '/').replaceAll('~0', '~');
      if (node is Map<String, dynamic>) {
        if (!node.containsKey(token)) return null;
        node = node[token];
      } else if (node is List) {
        final i = int.tryParse(token);
        if (i == null || i < 0 || i >= node.length) return null;
        node = node[i];
      } else {
        return null;
      }
    }
    return node;
  }
}
