// `trust-tasks.lock` — read, never written, by this generator.
//
// The lockfile is TOML written only by the `trust-tasks` CLI, in one fixed
// shape (trust-tasks-cli/src/lock.rs renders it by hand):
//
//   # Written by `trust-tasks`. Do not edit by hand.
//   version = 1
//   registry = "https://trusttasks.org"
//
//   [[task]]
//   slug = "vault/upsert"
//   version = "0.4"
//   type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//   digest = "sha256:<hex>"
//
// This is a strict parser of exactly that shape, not of TOML: anything else —
// another table, an unknown key, a value of another type — is refused rather
// than guessed at, so a lock this parser accepts means what the CLI meant.
// Taking a TOML dependency for five keys would break the package's
// no-dependencies stance (see pubspec.yaml).

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
  });

  /// e.g. `vault/upsert`.
  final String slug;

  /// `MAJOR.MINOR`, e.g. `0.4`.
  final String version;

  /// The request Type URI.
  final String typeUri;

  /// `sha256:<lowercase hex>` of the vendored bundle's bytes.
  final String digest;

  /// The vendored bundle's path relative to the lock's directory:
  /// `trust-tasks/<slug>/<M.N>.bundle.json`.
  String get bundlePath => 'trust-tasks/$slug/$version.bundle.json';
}

/// A parsed `trust-tasks.lock`.
class Lockfile {
  const Lockfile({required this.registry, required this.tasks});

  /// The registry the versions were resolved against, when recorded.
  final String? registry;

  /// One entry per pinned version, in file order.
  final List<LockedTask> tasks;

  static final _key = RegExp(r'^([A-Za-z0-9_-]+)\s*=\s*(.*)$');
  static final _slug = RegExp(r'^[a-z0-9][a-z0-9-]*(/[a-z0-9][a-z0-9-]*)*$');
  static final _version = RegExp(r'^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$');
  static final _digest = RegExp(r'^sha256:[0-9a-f]{64}$');

  /// Parse a lockfile's text, refusing anything outside the CLI's shape.
  static Lockfile parse(String text) {
    int? version;
    String? registry;
    final tasks = <LockedTask>[];
    Map<String, String>? current;
    var currentLine = 0;

    void finishTask() {
      final t = current;
      if (t == null) return;
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
      if (tasks.any((e) => e.slug == slug && e.version == v)) {
        throw LockError('$slug/$v is locked more than once');
      }
      tasks.add(LockedTask(
        slug: slug,
        version: v,
        typeUri: t['type_uri']!,
        digest: digest,
      ));
      current = null;
    }

    final lines = text.split('\n');
    for (var i = 0; i < lines.length; i++) {
      final lineNo = i + 1;
      var line = lines[i];
      if (line.endsWith('\r')) line = line.substring(0, line.length - 1);
      line = line.trim();
      if (line.isEmpty || line.startsWith('#')) continue;

      if (line == '[[task]]') {
        finishTask();
        current = <String, String>{};
        currentLine = lineNo;
        continue;
      }
      if (line.startsWith('[')) {
        throw LockError('line $lineNo: unexpected table `$line`');
      }

      final m = _key.firstMatch(line);
      if (m == null) throw LockError('line $lineNo: not `key = value`');
      final key = m.group(1)!;
      final raw = m.group(2)!;

      final task = current;
      if (task == null) {
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
      } else {
        if (!const {'slug', 'version', 'type_uri', 'digest'}.contains(key)) {
          throw LockError('line $lineNo: unknown [[task]] key `$key`');
        }
        if (task.containsKey(key)) {
          throw LockError('line $lineNo: `$key` twice in one [[task]]');
        }
        task[key] = _basicString(raw, lineNo);
      }
    }
    finishTask();

    if (version == null) throw LockError('no `version`');
    if (version != lockVersion) {
      throw LockError(
        'lockfile version $version is not supported '
        '(this generator reads $lockVersion) — upgrade trust_tasks',
      );
    }
    return Lockfile(registry: registry, tasks: tasks);
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
