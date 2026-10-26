// `dart run trust_tasks:generate` — generate the Dart libraries for exactly the
// Trust Tasks an application pins.
//
//   dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
//
// Reads `trust-tasks.lock` (written by the `trust-tasks` CLI) and the bundles
// vendored beside it under `trust-tasks/<slug>/<M.N>.bundle.json`, checks each
// bundle's SHA-256 against the lock, and writes one library per pinned version
// to `<out>/<slug>/v<M>_<N>/payload.dart` (segments snake_cased), plus
// `<out>/extends_index.dart`. The output directory is owned by the generator:
// rewritten whole, and refused if it is non-empty without the
// `.trust-tasks-generated` marker. Needs no network, no Node and no Rust.
//
// Import a generated library with a prefix, as you would a prebuilt one:
//
//   import 'package:trust_tasks/trust_tasks.dart';
//   import 'trust_tasks/acl/grant/v0_2/payload.dart' as acl_grant;

import 'dart:io';

import 'package:trust_tasks/src/generator/generate.dart';

const _usage = '''
Usage: dart run trust_tasks:generate --out <dir> [--lock <path>] [--language-version <M.N>]

  --lock <path>               trust-tasks.lock (default: trust-tasks.lock)
  --out <dir>                 the directory to generate into (required); owned by
                              the generator and rewritten whole
  --language-version <M.N>    the language version to format at
                              (default: $defaultLanguageVersion, this package's SDK floor)
''';

Future<void> main(List<String> args) async {
  var lock = 'trust-tasks.lock';
  String? out;
  var languageVersion = defaultLanguageVersion;

  for (var i = 0; i < args.length; i++) {
    final arg = args[i];
    String value() {
      if (i + 1 >= args.length) _fail('$arg needs a value\n\n$_usage');
      return args[++i];
    }

    switch (arg) {
      case '--lock':
        lock = value();
      case '--out':
        out = value();
      case '--language-version':
        languageVersion = value();
      case '-h' || '--help':
        stdout.write(_usage);
        return;
      default:
        if (arg.startsWith('--lock=')) {
          lock = arg.substring('--lock='.length);
        } else if (arg.startsWith('--out=')) {
          out = arg.substring('--out='.length);
        } else if (arg.startsWith('--language-version=')) {
          languageVersion = arg.substring('--language-version='.length);
        } else {
          _fail('unknown argument `$arg`\n\n$_usage');
        }
    }
  }
  if (out == null) _fail('--out is required\n\n$_usage');

  try {
    final result = await generate(
      lockPath: lock,
      outDir: out,
      languageVersion: languageVersion,
    );
    for (final note in result.notes) {
      stderr.writeln('note: $note');
    }
    stdout.writeln(
        'generated ${result.generated.length} Trust Task libraries into $out');
  } on GenerateError catch (e) {
    _fail(e.message);
  }
}

Never _fail(String message) {
  stderr.writeln('error: $message');
  exit(1);
}
