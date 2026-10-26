// Drives dart_apitool's analyzer and differ for scripts/check-dart-api.mjs.
//
// Usage: dart --packages=<dart_apitool's package_config.json> \
//          scripts/lib/dart_api_diff.dart <old package dir> <new package dir> <out.json>
//
// Run against the globally activated dart_apitool (its package config lives in
// $PUB_CACHE/global_packages/dart_apitool/.dart_tool/package_config.json), so
// the version is the one the workflow pins, not a second copy resolved here.
//
// Why not the `dart-apitool diff` CLI, which does the same two steps:
//
//  * The CLI treats only files DIRECTLY in lib/ as entry points. The analyzer
//    takes `doConsiderNonSrcAsEntryPoints`, which also makes every library
//    under lib/ outside lib/src one — and trust_tasks' generated libraries
//    live in lib/specs/**, unreachable from its barrel by design. The CLI does
//    not expose the switch.
//  * The CLI's reports name a change ("Interface \"Payload\" removed") but not
//    the library it is in, and ~1,100 generated libraries each declare a
//    `Payload`. ApiChange carries the declaration, and so its relativePath.
//
// The output is a JSON array of
//   {breaking, code, description, library, context: [names…]}.

import 'dart:convert';
import 'dart:io';

import 'package:dart_apitool/api_tool.dart';

Future<void> main(List<String> args) async {
  if (args.length != 3) {
    stderr.writeln('usage: dart_api_diff.dart <old> <new> <out.json>');
    exit(64);
  }
  Future<PackageApi> analyze(String dir) => PackageApiAnalyzer(
        packagePath: dir,
        // Android/iOS platform constraints do not apply to a pure-Dart package.
        doAnalyzePlatformConstraints: false,
        doConsiderNonSrcAsEntryPoints: true,
      ).analyze();

  final oldApi = await analyze(args[0]);
  final newApi = await analyze(args[1]);
  final result = PackageApiDiffer(
    options: const PackageApiDifferOptions(
      // An SDK floor change is a release question, not an API diff.
      doCheckSdkVersion: false,
    ),
  ).diff(oldApi: oldApi, newApi: newApi);

  final changes = [
    for (final c in result.apiChanges)
      {
        'breaking': c.isBreaking,
        'code': c.changeCode.code,
        'description': c.changeDescription,
        'library': (c.contextTrace.isNotEmpty
                ? c.contextTrace.first
                : c.affectedDeclaration)
            ?.relativePath,
        'context': [for (final d in c.contextTrace) d.name],
      },
  ];
  File(args[2]).writeAsStringSync(jsonEncode(changes));
}
