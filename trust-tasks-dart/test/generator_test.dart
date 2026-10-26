// `dart run trust_tasks:generate` — the consumer-side generator.
//
// Its output is held to the prebuilt tree by
// scripts/check-dart-consumer-generation.mjs (every bundle in the registry);
// these pin the contract around it: the digest check, the lock and bundle
// versions it refuses, the output directory it owns, determinism, and the
// `$defs` order it takes from a bundle's `defsOrder` (or does without).

@TestOn('vm')
library;

import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:trust_tasks/src/generator/emit.dart';
import 'package:trust_tasks/src/generator/generate.dart';
import 'package:trust_tasks/src/generator/lock.dart';
import 'package:trust_tasks/src/generator/sha256.dart';

/// The Dart example application's lock and vendored bundles: acl/grant 0.2,
/// keys/list 0.2 and vault/get 0.3 from the registry, and payments/settle 1.0
/// from the third-party source `example` (https://tasks.example.com).
final exampleDir = Directory('../examples/consumer-app-dart');

String hex(List<int> bytes) =>
    bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

/// A copy of the example's lock + bundles in a fresh directory.
Directory workspace() {
  final dir = Directory.systemTemp.createTempSync('tt-gen-test-');
  addTearDown(() => dir.deleteSync(recursive: true));
  File('${exampleDir.path}/trust-tasks.lock')
      .copySync('${dir.path}/trust-tasks.lock');
  for (final f in Directory('${exampleDir.path}/trust-tasks')
      .listSync(recursive: true)
      .whereType<File>()) {
    final rel = f.path.substring(exampleDir.path.length + 1);
    final target = File('${dir.path}/$rel')..parent.createSync(recursive: true);
    f.copySync(target.path);
  }
  return dir;
}

/// Add a lock entry for [bundle] (written canonically enough for the
/// generator, which hashes bytes and does not re-canonicalize).
void pin(Directory dir, Map<String, dynamic> bundle, {String? source}) {
  final slug = bundle['slug'] as String, version = bundle['version'] as String;
  final bytes = utf8.encode(jsonEncode(bundle));
  final at = source == null ? '' : '@$source/';
  File('${dir.path}/trust-tasks/$at$slug/$version.bundle.json')
    ..parent.createSync(recursive: true)
    ..writeAsBytesSync(bytes);
  final lock = File('${dir.path}/trust-tasks.lock');
  final declare =
      source != null && !lock.readAsStringSync().contains('[sources.$source]');
  lock.writeAsStringSync(
    '${declare ? '\n[sources.$source]\nurl = "https://$source.example"\n' : ''}'
    '\n[[task]]\n${source == null ? '' : 'source = "$source"\n'}'
    'slug = "$slug"\nversion = "$version"\n'
    'type_uri = "${bundle['typeUri']}"\ndigest = "${sha256Digest(bytes)}"\n',
    mode: FileMode.append,
  );
}

/// The example's third-party fixture: payments/settle 1.0 from
/// https://tasks.example.com, vendored under `trust-tasks/@example/`.
Map<String, dynamic> settle() => jsonDecode(File(
        '${exampleDir.path}/trust-tasks/@example/payments/settle/1.0.bundle.json')
    .readAsStringSync()) as Map<String, dynamic>;

Matcher refusal(String text) => throwsA(
    isA<GenerateError>().having((e) => e.message, 'message', contains(text)));

Map<String, dynamic> vaultGet() =>
    jsonDecode(File('${exampleDir.path}/trust-tasks/vault/get/0.3.bundle.json')
        .readAsStringSync()) as Map<String, dynamic>;

Map<String, String> snapshot(Directory dir) => {
      for (final f in dir.listSync(recursive: true).whereType<File>())
        f.path.substring(dir.path.length + 1): f.readAsStringSync(),
    };

void main() {
  group('sha256', () {
    test('FIPS 180-4 vectors', () {
      expect(hex(sha256(utf8.encode(''))),
          'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
      expect(hex(sha256(utf8.encode('abc'))),
          'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
      expect(
          hex(sha256(utf8.encode(
              'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq'))),
          '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1');
      expect(hex(sha256(List<int>.filled(1000000, 0x61))),
          'cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0');
      // Every padding boundary around one and two blocks.
      for (final n in [55, 56, 63, 64, 65, 119, 120]) {
        expect(sha256(List<int>.filled(n, 1)), hasLength(32));
      }
    });

    test('reproduces the digests the example lock pins', () {
      final lock = Lockfile.parse(
          File('${exampleDir.path}/trust-tasks.lock').readAsStringSync());
      expect(lock.tasks, hasLength(4));
      for (final t in lock.tasks) {
        final bytes =
            File('${exampleDir.path}/${t.bundlePath}').readAsBytesSync();
        expect(sha256Digest(bytes), t.digest, reason: t.bundlePath);
      }
    });
  });

  group('lockfile', () {
    final ok = '# Written by `trust-tasks`. Do not edit by hand.\n'
        'version = 1\nregistry = "https://trusttasks.org"\n\n'
        '[[task]]\nslug = "acl/grant"\nversion = "0.2"\n'
        'type_uri = "https://trusttasks.org/spec/acl/grant/0.2"\n'
        'digest = "sha256:${'a' * 64}"\n';

    test('reads the CLI shape', () {
      final lock = Lockfile.parse(ok);
      expect(lock.registry, 'https://trusttasks.org');
      expect(lock.tasks.single.bundlePath,
          'trust-tasks/acl/grant/0.2.bundle.json');
    });

    test('refuses an unknown version, key, table or duplicate', () {
      expect(
          () => Lockfile.parse(ok.replaceFirst('version = 1', 'version = 2')),
          throwsA(isA<LockError>().having((e) => e.message, 'message',
              contains('lockfile version 2 is not supported'))));
      expect(
          () => Lockfile.parse('$ok extra = "x"\n'), throwsA(isA<LockError>()));
      expect(() => Lockfile.parse('$ok\n[other]\n'), throwsA(isA<LockError>()));
      expect(() => Lockfile.parse(ok + ok.substring(ok.indexOf('\n[[task]]'))),
          throwsA(isA<LockError>()));
      expect(() => Lockfile.parse(ok.replaceFirst('"0.2"', '0.2')),
          throwsA(isA<LockError>()));
      expect(() => Lockfile.parse(ok.replaceFirst('sha256:', 'sha1:')),
          throwsA(isA<LockError>()));
    });

    test('reads third-party sources', () {
      final sourced = ok.replaceFirst('\n[[task]]\n',
              '\n[sources.acme]\nurl = "https://tasks.acme.example"\n\n[[task]]\n') +
          ok
              .substring(ok.indexOf('\n[[task]]'))
              .replaceFirst('[[task]]\n', '[[task]]\nsource = "acme"\n');
      final lock = Lockfile.parse(sourced);
      expect(lock.sources, {'acme': 'https://tasks.acme.example'});
      expect(lock.tasks.map((t) => t.bundlePath), [
        'trust-tasks/acl/grant/0.2.bundle.json',
        'trust-tasks/@acme/acl/grant/0.2.bundle.json',
      ]);

      // A source must be declared, named properly, and have one location.
      expect(() => Lockfile.parse(sourced.replaceFirst('[sources.acme]', '')),
          throwsA(isA<LockError>()));
      expect(
          () => Lockfile.parse(
              sourced.replaceFirst('url = "https://tasks.acme.example"', '')),
          throwsA(isA<LockError>()));
      expect(
          () => Lockfile.parse(sourced.replaceFirst(
              'url = "https://tasks.acme.example"',
              'url = "https://x.example"\npath = "../x"')),
          throwsA(isA<LockError>()));
      expect(() => Lockfile.parse(sourced.replaceAll('acme', 'Acme')),
          throwsA(isA<LockError>()));
      expect(
          () => Lockfile.parse(sourced.replaceFirst(
              'url = "https://tasks.acme.example"', 'token = "x"')),
          throwsA(isA<LockError>()));
    });
  });

  group('defsOrder', () {
    // Independent definitions, so dependency order cannot decide; `Item` and
    // `item` both want the class name `Item`, so the first declared gets it.
    Map<String, dynamic> bundle(Object? defsOrder) {
      Map<String, dynamic> schema() => {
            r'$defs': {
              'Alpha': {
                'type': 'string',
                'enum': ['a'],
              },
              'Item': {
                'type': 'object',
                'properties': {
                  'upper': {'type': 'string'},
                },
              },
              'Response': {
                'type': 'object',
                'properties': {
                  'ok': {'type': 'boolean'},
                },
              },
              'Zeta': {
                'type': 'string',
                'enum': ['z'],
              },
              'item': {
                'type': 'object',
                'properties': {
                  'lower': {'type': 'string'},
                },
              },
            },
            'type': 'object',
            'properties': {
              'alpha': {r'$ref': r'#/$defs/Alpha'},
              'zeta': {r'$ref': r'#/$defs/Zeta'},
            },
          };
      final b = vaultGet()
        ..['schema'] = schema()
        ..['responseSchema'] = schema()
        ..['propertyOrder'] = <String, dynamic>{}
        ..remove('defsOrder');
      if (defsOrder != null) b['defsOrder'] = defsOrder;
      return b;
    }

    List<String> declared(Map<String, dynamic> b) => RegExp(
          r'^(?:class|typedef|extension type const) (\w+)',
          multiLine: true,
        )
            .allMatches(
                generateLibrary(Bundle.parse(jsonEncode(b)), source: 'test'))
            .map((m) => m[1]!)
            .toList();

    test('declares the root \$defs in the order it lists', () {
      final order = ['Zeta', 'item', 'Response', 'Item', 'Alpha'];
      final b = bundle({'/schema': order, '/responseSchema': order});
      expect(declared(b),
          ['Zeta', 'Item', 'Response', 'Item2', 'Alpha', 'Payload']);
      // `item` was reserved first, so it is the class with `lower`.
      final lib = generateLibrary(Bundle.parse(jsonEncode(b)), source: 'test');
      expect(lib, contains('class Item {\n  const Item({\n    this.lower,'));
    });

    test('a definition it does not name follows, in the bundle\'s order', () {
      final b = bundle({
        '/schema': ['Zeta'],
      });
      expect(declared(b),
          ['Zeta', 'Alpha', 'Item', 'Response', 'Item2', 'Payload']);
    });

    test('a bundle older than it keeps the bundle\'s key order', () {
      final legacy = declared(bundle(null));
      expect(legacy, ['Alpha', 'Item', 'Response', 'Zeta', 'Item2', 'Payload']);
      expect(declared(bundle(<String, dynamic>{})), legacy);
      // The example's vendored bundles predate defsOrder.
      for (final f in Directory('${exampleDir.path}/trust-tasks')
          .listSync(recursive: true)
          .whereType<File>()) {
        expect(f.readAsStringSync(), isNot(contains('"defsOrder"')),
            reason: f.path);
      }
    });

    test('refuses one of the wrong shape', () {
      for (final bad in <Object>[
        ['Zeta'],
        {
          '/schema': [1],
        },
        {'/schema': 'Zeta'},
      ]) {
        expect(
          () => Bundle.parse(jsonEncode(bundle(bad))),
          throwsA(isA<BundleError>()
              .having((e) => e.message, 'message', contains('defsOrder'))),
          reason: jsonEncode(bad),
        );
      }
    });
  });

  group('generate', () {
    test('writes exactly the locked tasks, deterministically', () async {
      final dir = workspace();
      final out = Directory('${dir.path}/out');
      final result = await generate(
          lockPath: '${dir.path}/trust-tasks.lock', outDir: out.path);
      expect(result.generated, [
        'acl/grant/v0_2/payload.dart',
        'example/payments/settle/v1_0/payload.dart',
        'keys/list/v0_2/payload.dart',
        'vault/get/v0_3/payload.dart',
      ]);
      final first = snapshot(out);
      expect(first.keys.toSet(), {
        ...result.generated,
        'extends_index.dart',
        markerFileName,
      });
      final grant = first['acl/grant/v0_2/payload.dart']!;
      expect(grant, contains("import 'package:trust_tasks/trust_tasks.dart';"));
      expect(grant, contains('extension type const'));

      await generate(
          lockPath: '${dir.path}/trust-tasks.lock', outDir: out.path);
      expect(snapshot(out), first);
    });

    test('a third-party task: its own namespace and its own Type URI',
        () async {
      final dir = workspace();
      // A second, URN-form task from another source (SPEC §6.1).
      pin(
          dir,
          settle()
            ..['slug'] = 'ledger/post'
            ..['typeUri'] = 'urn:example:acme:spec:ledger:post:1.0',
          source: 'acme-pay');
      final out = Directory('${dir.path}/out');
      final result = await generate(
          lockPath: '${dir.path}/trust-tasks.lock', outDir: out.path);
      expect(
          result.generated, contains('acme_pay/ledger/post/v1_0/payload.dart'));

      final files = snapshot(out);
      final settleLib = files['example/payments/settle/v1_0/payload.dart']!;
      expect(settleLib,
          contains("'https://tasks.example.com/spec/payments/settle/1.0'"));
      expect(settleLib, contains('#response'));
      expect(settleLib, isNot(contains('trusttasks.org/spec/payments')));
      expect(settleLib,
          contains('trust-tasks/@example/payments/settle/1.0.bundle.json'));
      expect(files['acme_pay/ledger/post/v1_0/payload.dart']!,
          contains("'urn:example:acme:spec:ledger:post:1.0'"));
    });

    test('refuses a source named for an official task\'s first segment',
        () async {
      final dir = workspace();
      pin(
          dir,
          settle()
            ..['slug'] = 'payments/refund'
            ..['typeUri'] =
                'https://tasks.example.com/spec/payments/refund/1.0',
          source: 'vault');
      expect(
          generate(
              lockPath: '${dir.path}/trust-tasks.lock',
              outDir: '${dir.path}/out'),
          refusal('source `vault`'));
    });

    test('refuses a Type URI under the wrong authority', () async {
      final sourcedRegistry = workspace();
      pin(
          sourcedRegistry,
          settle()
            ..['slug'] = 'payments/refund'
            ..['typeUri'] = 'https://trusttasks.org/spec/payments/refund/1.0',
          source: 'example');
      expect(
          generate(
              lockPath: '${sourcedRegistry.path}/trust-tasks.lock',
              outDir: '${sourcedRegistry.path}/out'),
          refusal('is a registry Type URI'));

      final officialForeign = workspace();
      pin(
          officialForeign,
          settle()
            ..['slug'] = 'payments/refund'
            ..['typeUri'] =
                'https://tasks.example.com/spec/payments/refund/1.0');
      expect(
          generate(
              lockPath: '${officialForeign.path}/trust-tasks.lock',
              outDir: '${officialForeign.path}/out'),
          refusal('an official task\'s Type URI must be'));
    });

    test('refuses a framework specification from a source', () async {
      final dir = workspace();
      pin(
          dir,
          settle()
            ..['slug'] = 'trust-task-ok'
            ..['typeUri'] = 'https://tasks.example.com/spec/trust-task-ok/1.0',
          source: 'example');
      expect(
          generate(
              lockPath: '${dir.path}/trust-tasks.lock',
              outDir: '${dir.path}/out'),
          refusal('cannot come from a third-party source'));
    });

    test('refuses a bundle that does not match its digest', () async {
      final dir = workspace();
      final bundle = File('${dir.path}/trust-tasks/vault/get/0.3.bundle.json');
      bundle.writeAsStringSync('${bundle.readAsStringSync()} ');
      expect(
        generate(
            lockPath: '${dir.path}/trust-tasks.lock',
            outDir: '${dir.path}/out'),
        throwsA(isA<GenerateError>().having(
            (e) => e.message,
            'message',
            contains(
                'vendored bundle does not match trust-tasks.lock — run `trust-tasks check`'))),
      );
    });

    test('refuses an unknown bundleVersion', () async {
      final dir = workspace();
      pin(
          dir,
          vaultGet()
            ..['bundleVersion'] = 2
            ..['slug'] = 'vault/get-next'
            ..['typeUri'] = 'https://trusttasks.org/spec/vault/get-next/0.3');
      expect(
        generate(
            lockPath: '${dir.path}/trust-tasks.lock',
            outDir: '${dir.path}/out'),
        throwsA(isA<GenerateError>()
            .having((e) => e.message, 'message', contains('bundleVersion 2'))),
      );
    });

    test('skips withdrawn and framework bundles with a note', () async {
      final dir = workspace();
      pin(
          dir,
          vaultGet()
            ..['version'] = '0.1'
            ..['typeUri'] = 'https://trusttasks.org/spec/vault/get/0.1'
            ..['status'] = 'withdrawn');
      pin(
          dir,
          vaultGet()
            ..['slug'] = 'trust-task-ok'
            ..['version'] = '0.1'
            ..['typeUri'] = 'https://trusttasks.org/spec/trust-task-ok/0.1');
      final result = await generate(
          lockPath: '${dir.path}/trust-tasks.lock', outDir: '${dir.path}/out');
      expect(result.generated, hasLength(4));
      expect(result.notes, [
        contains('vault/get/0.1: withdrawn'),
        contains('trust-task-ok/0.1: a framework specification'),
      ]);
    });

    test('owns its output directory', () async {
      final dir = workspace();
      final out = Directory('${dir.path}/out')..createSync();
      final mine = File('${out.path}/main.dart')
        ..writeAsStringSync('void main() {}');

      // Non-empty and unmarked: refused, and left alone.
      await expectLater(
        generate(lockPath: '${dir.path}/trust-tasks.lock', outDir: out.path),
        throwsA(isA<GenerateError>()
            .having((e) => e.message, 'message', contains(markerFileName))),
      );
      expect(mine.existsSync(), isTrue);

      // Marked: written whole, stale files removed.
      mine.deleteSync();
      await generate(
          lockPath: '${dir.path}/trust-tasks.lock', outDir: out.path);
      final stale = File('${out.path}/old/v0_1/payload.dart')
        ..parent.createSync(recursive: true)
        ..writeAsStringSync('// stale');
      await generate(
          lockPath: '${dir.path}/trust-tasks.lock', outDir: out.path);
      expect(stale.existsSync(), isFalse);
      expect(Directory('${out.path}/old').existsSync(), isFalse);
    });
  });
}
