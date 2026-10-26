// `trust-tasks.lock` — read, never written, by this generator.
//
// The lockfile is TOML written only by the `trust-tasks` CLI, in one fixed
// shape (trust-tasks-cli/src/lock.rs renders it by hand):
//
//   # Written by `trust-tasks`. Do not edit by hand.
//   version = 1
//   registry = "https://trusttasks.org"
//
//   [sources.acme]                       # zero or more third-party sources
//   url = "https://tasks.acme.example"   # or path = "…"; exactly one
//
//   [[task]]
//   source = "acme"                      # optional; absent = the registry
//   slug = "vault/upsert"
//   version = "0.4"
//   type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//   digest = "sha256:<hex>"
//
// This is a strict parser of exactly that shape, not of TOML: anything else —
// another table, an unknown key, a value of another type — is refused rather
// than guessed at, so a lock this parser accepts means what the CLI meant.
// Taking a TOML dependency for a handful of keys would break the package's
// no-dependencies stance (see pubspec.yaml). A source's `url`/`path` is read
// and checked but never used: generation needs no network.

/// The lockfile shape this generator reads.
const int lockVersion = 1;

/// A lockfile this generator cannot use, with the reason.
class LockError implements Exception {
  LockError(this.message);
  final String message;
  @override
  String toString() => message;
}

/// One pinned specification version.
class LockedTask {
  const LockedTask({
    required this.slug,
    required this.version,
    required this.typeUri,
    required this.digest,
    this.source,
  });

  /// The `[sources.<name>]` this task comes from, or null for the registry.
  final String? source;

  /// e.g. `vault/upsert`.
  final String slug;

  /// `MAJOR.MINOR`, e.g. `0.4`.
  final String version;

  /// The request Type URI.
  final String typeUri;

  /// `sha256:<lowercase hex>` of the vendored bundle's bytes.
  final String digest;

  /// `<slug>/<M.N>`, prefixed `@<source>/` for a third-party task.
  String get label =>
      source == null ? '$slug/$version' : '@$source/$slug/$version';

  /// The vendored bundle's path relative to the lock's directory:
  /// `trust-tasks/<slug>/<M.N>.bundle.json`, or
  /// `trust-tasks/@<source>/<slug>/<M.N>.bundle.json` for a sourced task (`@`
  /// cannot begin a slug, so the two never collide).
  String get bundlePath => 'trust-tasks/$label.bundle.json';
}

/// A parsed `trust-tasks.lock`.
class Lockfile {
  const Lockfile({
    required this.registry,
    required this.tasks,
    this.sources = const {},
  });

  /// The registry the versions were resolved against, when recorded.
  final String? registry;

  /// The third-party sources, by name, to their `url` or `path`.
  final Map<String, String> sources;

  /// One entry per pinned version, in file order.
  final List<LockedTask> tasks;

  static final _key = RegExp(r'^([A-Za-z0-9_-]+)\s*=\s*(.*)$');
  static final _slug = RegExp(r'^[a-z0-9][a-z0-9-]*(/[a-z0-9][a-z0-9-]*)*$');
  static final _version = RegExp(r'^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$');
  static final _digest = RegExp(r'^sha256:[0-9a-f]{64}$');
  static final _sourceName = RegExp(r'^[a-z][a-z0-9]*(-[a-z0-9]+)*$');
  static final _sourceTable = RegExp(r'^\[sources\.([^\]\s]*)\]$');

  /// Parse a lockfile's text, refusing anything outside the CLI's shape.
  static Lockfile parse(String text) {
    int? version;
    String? registry;
    final tasks = <LockedTask>[];
    final sources = <String, Map<String, String>>{};
    Map<String, String>? current; // the open [[task]] or [sources.x] table
    var inTask = false;
    var currentLine = 0;

    void finishTask() {
      final t = current;
      if (t == null || !inTask) return;
      for (final k in const ['slug', 'version', 'type_uri', 'digest']) {
        if (!t.containsKey(k)) {
          throw LockError('the [[task]] at line $currentLine has no `$k`');
        }
      }
      final slug = t['slug']!, v = t['version']!, digest = t['digest']!;
      if (!_slug.hasMatch(slug)) {
        throw LockError('`$slug` (line $currentLine) is not a task slug');
      }
      if (!_version.hasMatch(v)) {
        throw LockError('$slug: `$v` is not a MAJOR.MINOR version');
      }
      if (!_digest.hasMatch(digest)) {
        throw LockError('$slug $v: `$digest` is not a sha256:<hex> digest');
      }
      final task = LockedTask(
        source: t['source'],
        slug: slug,
        version: v,
        typeUri: t['type_uri']!,
        digest: digest,
      );
      if (tasks.any((e) => e.label == task.label)) {
        throw LockError('${task.label} is locked more than once');
      }
      tasks.add(task);
    }

    void openTable(Map<String, String> table, bool isTask, int lineNo) {
      finishTask();
      current = table;
      inTask = isTask;
      currentLine = lineNo;
    }

    final lines = text.split('\n');
    for (var i = 0; i < lines.length; i++) {
      final lineNo = i + 1;
      var line = lines[i];
      if (line.endsWith('\r')) line = line.substring(0, line.length - 1);
      line = line.trim();
      if (line.isEmpty || line.startsWith('#')) continue;

      if (line == '[[task]]') {
        openTable(<String, String>{}, true, lineNo);
        continue;
      }
      final sourceTable = _sourceTable.firstMatch(line);
      if (sourceTable != null) {
        final name = sourceTable.group(1)!;
        if (!_sourceName.hasMatch(name)) {
          throw LockError('line $lineNo: `$name` is not a source name');
        }
        if (sources.containsKey(name)) {
          throw LockError('line $lineNo: [sources.$name] twice');
        }
        final table = <String, String>{};
        sources[name] = table;
        openTable(table, false, lineNo);
        continue;
      }
      if (line.startsWith('[')) {
        throw LockError('line $lineNo: unexpected table `$line`');
      }

      final m = _key.firstMatch(line);
      if (m == null) throw LockError('line $lineNo: not `key = value`');
      final key = m.group(1)!;
      final raw = m.group(2)!;

      final table = current;
      if (table == null) {
        if (key == 'version') {
          if (version != null) throw LockError('line $lineNo: `version` twice');
          final n = int.tryParse(raw.trim());
          if (n == null) {
            throw LockError('line $lineNo: `version` must be an integer');
          }
          version = n;
        } else if (key == 'registry') {
          if (registry != null) {
            throw LockError('line $lineNo: `registry` twice');
          }
          registry = _basicString(raw, lineNo);
        } else {
          throw LockError('line $lineNo: unknown key `$key`');
        }
        continue;
      }
      final allowed = inTask
          ? const {'source', 'slug', 'version', 'type_uri', 'digest'}
          : const {'url', 'path'};
      if (!allowed.contains(key)) {
        throw LockError('line $lineNo: unknown '
            '${inTask ? '[[task]]' : 'source'} key `$key`');
      }
      if (table.containsKey(key)) {
        throw LockError('line $lineNo: `$key` twice in one table');
      }
      table[key] = _basicString(raw, lineNo);
    }
    finishTask();

    if (version == null) throw LockError('no `version`');
    if (version != lockVersion) {
      throw LockError(
        'lockfile version $version is not supported '
        '(this generator reads $lockVersion) — upgrade trust_tasks',
      );
    }
    final locations = <String, String>{};
    for (final MapEntry(key: name, value: table) in sources.entries) {
      if (table.length != 1) {
        throw LockError(
            '[sources.$name] must have exactly one of `url` or `path`');
      }
      locations[name] = table.values.single;
    }
    for (final t in tasks) {
      if (t.source != null && !locations.containsKey(t.source)) {
        throw LockError('${t.slug}/${t.version} names source `${t.source}`, '
            'but there is no [sources.${t.source}] table');
      }
    }
    return Lockfile(registry: registry, tasks: tasks, sources: locations);
  }

  /// A TOML basic string (`"…"`), the only string form the CLI writes, with
  /// nothing but an optional comment after it.
  static String _basicString(String raw, int lineNo) {
    final s = raw.trim();
    if (!s.startsWith('"')) {
      throw LockError('line $lineNo: expected a "quoted" string');
    }
    final out = StringBuffer();
    var i = 1;
    while (true) {
      if (i >= s.length) throw LockError('line $lineNo: unterminated string');
      final ch = s[i];
      if (ch == '"') break;
      if (ch == r'\') {
        if (i + 1 >= s.length) {
          throw LockError('line $lineNo: unterminated string');
        }
        final esc = s[i + 1];
        i += 2;
        switch (esc) {
          case '"':
            out.write('"');
          case r'\':
            out.write(r'\');
          case 'b':
            out.write('\b');
          case 't':
            out.write('\t');
          case 'n':
            out.write('\n');
          case 'f':
            out.write('\f');
          case 'r':
            out.write('\r');
          case 'u' || 'U':
            final len = esc == 'u' ? 4 : 8;
            final hex = i + len <= s.length ? s.substring(i, i + len) : '';
            final cp =
                hex.length == len && RegExp(r'^[0-9A-Fa-f]+$').hasMatch(hex)
                    ? int.parse(hex, radix: 16)
                    : null;
            if (cp == null || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff)) {
              throw LockError('line $lineNo: bad \\$esc escape');
            }
            out.writeCharCode(cp);
            i += len;
          default:
            throw LockError('line $lineNo: unknown escape \\$esc');
        }
        continue;
      }
      if (ch.codeUnitAt(0) < 0x20 && ch != '\t') {
        throw LockError('line $lineNo: control character in a string');
      }
      out.write(ch);
      i++;
    }
    final rest = s.substring(i + 1).trim();
    if (rest.isNotEmpty && !rest.startsWith('#')) {
      throw LockError('line $lineNo: unexpected `$rest` after the string');
    }
    return out.toString();
  }
}
