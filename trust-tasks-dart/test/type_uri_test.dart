// SPEC.md §6.1 Type URI parsing — every form, not only the public registry's.
//
// Mirrors the tests of trust-tasks-rs/src/type_uri.rs, whose vectors these are:
// the two parsers must accept and refuse the same strings.

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/trust_tasks.dart';

Matcher throwsTypeUri(TypeUriError kind) =>
    throwsA(isA<TypeUriFormatException>().having((e) => e.kind, 'kind', kind));

void main() {
  group('the registry form', () {
    test('parses a single-segment slug', () {
      final uri = TypeUri.parse(
        'https://trusttasks.org/spec/trust-task-error/0.5',
      );
      expect(uri.authority, 'https://trusttasks.org');
      expect(uri.isUrn, isFalse);
      expect(uri.isRegistry, isTrue);
      expect(uri.slug, 'trust-task-error');
      expect((uri.major, uri.minor), (0, 5));
      expect(uri.version, '0.5');
      expect(uri.variant, isNull);
      expect(
        uri.toString(),
        'https://trusttasks.org/spec/trust-task-error/0.5',
      );
    });

    test('parses a hierarchical slug', () {
      final uri = TypeUri.parse('https://trusttasks.org/spec/acl/grant/0.1');
      expect(uri.slug, 'acl/grant');
      expect(uri, TypeUri.canonical('acl/grant', 0, 1));
    });

    test('parses #request and #response', () {
      final req = TypeUri.parse(
        'https://trusttasks.org/spec/acl/grant/0.1#request',
      );
      expect(req.variant, TypeUriVariant.request);
      expect(req.isResponse, isFalse);
      final resp = TypeUri.parse(
        'https://trusttasks.org/spec/acl/grant/0.1#response',
      );
      expect(resp.variant, TypeUriVariant.response);
      expect(resp.isResponse, isTrue);
      expect(resp.bare.toString(), 'https://trusttasks.org/spec/acl/grant/0.1');
    });
  });

  test('every §6.1 form round-trips, with each variant', () {
    const forms = <(String, String)>[
      ('https://trusttasks.org/spec/acl/grant/0.1', 'acl/grant'),
      ('https://example.com/trust-tasks/spec/acl/grant/0.1', 'acl/grant'),
      ('did:example:123456789abcdefghi/spec/acl/grant/0.1', 'acl/grant'),
      ('did:web:tasks.example.com:x/spec/pay/0.1', 'pay'),
      ('did:web:acme.example/spec/payments/settle/0.1', 'payments/settle'),
      ('urn:example:tasks:spec:acl:grant:0.1', 'acl/grant'),
      ('urn:example:tasks:sub:spec:pay:3.10', 'pay'),
      ('tag:example.com,2026:tasks/spec/acl/grant/0.1', 'acl/grant'),
    ];
    for (final (uri, slug) in forms) {
      for (final fragment in const ['', '#request', '#response']) {
        final s = '$uri$fragment';
        final parsed = TypeUri.parse(s);
        expect(parsed.slug, slug, reason: s);
        expect(parsed.toString(), s);
        expect(parsed.bare.toString(), uri);
        expect(TypeUri.parse(parsed.toString()), parsed);
        expect(TypeUri.tryParse(s), parsed);
      }
    }
  });

  group('the URN form', () {
    test('splits slug segments and version on ":"', () {
      final urn = TypeUri.parse('urn:example:tasks:spec:payments:settle:3.10');
      expect(urn.isUrn, isTrue);
      expect(urn.authority, 'urn:example:tasks');
      expect(urn.slug, 'payments/settle');
      expect((urn.major, urn.minor), (3, 10));
      expect(urn.version, '3.10');
      expect(urn.isRegistry, isFalse);
    });

    test('keeps its form across with* copies', () {
      final urn = TypeUri.parse('urn:example:tasks:spec:acl:grant:0.1');
      expect(
        urn.withResponse().toString(),
        'urn:example:tasks:spec:acl:grant:0.1#response',
      );
      expect(
        urn.withVersion(0, 2).toString(),
        'urn:example:tasks:spec:acl:grant:0.2',
      );
      expect(
        TypeUri.urnForm('urn:example:tasks', 'acl/grant', 0, 1).toString(),
        'urn:example:tasks:spec:acl:grant:0.1',
      );
      expect(TypeUri.urnForm('urn:example:tasks:', 'acl/grant', 0, 1), urn);
    });

    test('a URN scheme is matched without regard to case', () {
      expect(TypeUri.parse('URN:example:tasks:spec:pay:0.1').isUrn, isTrue);
    });
  });

  test('pathForm builds a third-party Type URI', () {
    final uri = TypeUri.pathForm(
      'https://acme.example/tasks/',
      'payments/settle',
      1,
      2,
      variant: TypeUriVariant.response,
    );
    expect(
      uri.toString(),
      'https://acme.example/tasks/spec/payments/settle/1.2#response',
    );
    expect(
      () => TypeUri.pathForm('https://a.example', 'Bad', 0, 1),
      throwsTypeUri(TypeUriError.slug),
    );
    expect(
      () => TypeUri.pathForm('https://a.example', 'ok', -1, 1),
      throwsArgumentError,
    );
  });

  test('the authority is part of identity', () {
    final a = TypeUri.parse('https://trusttasks.org/spec/vault/get/0.3');
    final b = TypeUri.parse('https://tasks.example.com/spec/vault/get/0.3');
    final c = TypeUri.parse('urn:example:tasks:spec:vault:get:0.3');
    final d = TypeUri.parse('did:web:tasks.example.com/spec/vault/get/0.3');
    for (final (x, y) in [(a, b), (a, c), (b, c), (b, d), (c, d)]) {
      expect(x.slug, y.slug);
      expect(x, isNot(y));
      expect(x.isSameSpecification(y), isFalse);
    }
    expect(a.isSameSpecification(a.withVersion(1, 0).withResponse()), isTrue);
    expect(a.hashCode, TypeUri.parse(a.toString()).hashCode);
  });

  group('refusals', () {
    test('http, in every form', () {
      for (final uri in const [
        'http://trusttasks.org/spec/acl/grant/0.1',
        'HTTP://example.com/spec/acl/grant/0.1',
        'http://example.com/x/spec/acl/grant/0.1#response',
      ]) {
        expect(
          () => TypeUri.parse(uri),
          throwsTypeUri(TypeUriError.scheme),
          reason: uri,
        );
      }
    });

    test('malformed shapes', () {
      const cases = <(String, TypeUriError)>[
        ('trusttasks.org/spec/acl/grant/0.1', TypeUriError.scheme),
        ('1https://example.com/spec/a/0.1', TypeUriError.scheme),
        ('', TypeUriError.scheme),
        ('https:/spec/a/0.1', TypeUriError.shape),
        ('https:///spec/a/0.1', TypeUriError.shape),
        ('https:example.com/spec/a/0.1', TypeUriError.shape),
        ('did:/spec/a/0.1', TypeUriError.shape),
        // urn without :spec:, or without an NID and a prefix
        ('urn:example:tasks:a:0.1', TypeUriError.shape),
        ('urn:example:spec:a:0.1', TypeUriError.shape),
        ('urn::tasks:spec:a:0.1', TypeUriError.shape),
        ('urn:example:tasks:spec:acl/grant:0.1', TypeUriError.shape),
        ('urn:example:tasks/spec/acl/grant/0.1', TypeUriError.shape),
        ('urn:example:tasks:spec:0.1', TypeUriError.shape),
        ('did:example:123', TypeUriError.shape),
        ('https://example.com/spec/0.1', TypeUriError.shape),
      ];
      for (final (uri, kind) in cases) {
        expect(() => TypeUri.parse(uri), throwsTypeUri(kind), reason: uri);
        expect(TypeUri.tryParse(uri), isNull, reason: uri);
      }
    });

    test('an empty or malformed slug', () {
      for (final uri in const [
        'https://trusttasks.org/spec//0.1',
        'urn:example:tasks:spec::0.1',
        'urn:example:tasks:spec:Acl:0.1',
        'https://example.com/spec/acl//grant/0.1',
      ]) {
        expect(
          () => TypeUri.parse(uri),
          throwsTypeUri(TypeUriError.slug),
          reason: uri,
        );
      }
      for (final bad in const [
        'Acl',
        'acl--bad',
        '-leading',
        'trailing-',
        '1numeric-start',
        'with_underscore',
      ]) {
        expect(
          () => TypeUri.parse('https://trusttasks.org/spec/$bad/1.0'),
          throwsTypeUri(TypeUriError.slug),
          reason: bad,
        );
      }
    });

    test('a missing or garbage version', () {
      for (final bad in const [
        '01.0',
        '1.01',
        '1',
        '1.0.0',
        'a.b',
        '',
        '.1',
        '1.',
        '+1.0',
        '-1.0',
        '4294967296.0',
      ]) {
        expect(
          () => TypeUri.parse('https://trusttasks.org/spec/x/$bad'),
          throwsTypeUri(TypeUriError.version),
          reason: bad,
        );
      }
      expect(
        () => TypeUri.parse('urn:example:tasks:spec:acl:01.0'),
        throwsTypeUri(TypeUriError.version),
      );
      expect(
        () => TypeUri.parse('urn:example:tasks:spec:acl:grant'),
        throwsTypeUri(TypeUriError.version),
      );
    });

    test('a fragment other than #request / #response', () {
      expect(
        () => TypeUri.parse('https://trusttasks.org/spec/acl/grant/0.1#cancel'),
        throwsTypeUri(TypeUriError.fragment),
      );
      expect(
        () => TypeUri.parse('https://trusttasks.org/spec/acl/grant/0.1#'),
        throwsTypeUri(TypeUriError.fragment),
      );
    });
  });

  group('the §6.1 slug reservation', () {
    test('both halves are enforced, in every form', () {
      for (final uri in const [
        'https://example.com/spec/trust-task-custom/0.1',
        'https://trusttasks.org/spec/trust-ceremony-evil/0.1',
        'https://trusttasks.org/spec/trust-task/nested/0.1',
        'urn:example:tasks:spec:trust-task-custom:0.1',
      ]) {
        expect(
          () => TypeUri.parse(uri),
          throwsTypeUri(TypeUriError.reservedSlug),
          reason: uri,
        );
      }
      for (final slug in const ['trust-tasks', 'trusty-task']) {
        expect(
          TypeUri.parse('https://trusttasks.org/spec/$slug/0.1').slug,
          slug,
        );
      }
    });

    test('every framework slug the meta-schema allowlists parses', () {
      // The allowlist in type_uri.dart is a hand-maintained copy of the one
      // the registry build enforces; fail when they drift.
      final meta = File('../specs/spec.meta.schema.json');
      if (!meta.existsSync()) {
        markTestSkipped('the registry is not vendored alongside this package');
        return;
      }
      final json = jsonDecode(meta.readAsStringSync()) as Map<String, dynamic>;
      final slug = (json['properties'] as Map<String, dynamic>)['slug']
          as Map<String, dynamic>;
      final listed =
          ((slug['anyOf'] as List)[1] as Map<String, dynamic>)['enum'] as List;
      expect(listed, isNotEmpty);
      for (final s in listed.cast<String>()) {
        expect(
          TypeUri.tryParse('https://trusttasks.org/spec/$s/0.1'),
          isNotNull,
          reason: '$s is allowlisted by specs/spec.meta.schema.json',
        );
      }
    });
  });
}
