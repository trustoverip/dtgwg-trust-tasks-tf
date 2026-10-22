/// Parse an RFC 3339 `date-time` (SPEC §4.2), or return null.
///
/// Stricter than [DateTime.tryParse], which reads a timestamp with no zone
/// designator as *local* time and accepts forms RFC 3339 does not (a bare date,
/// a space for the `T`, no separators at all). The Rust and Go runtimes refuse
/// both, and an instant that depends on the consumer's time zone is not one
/// instant at all.
DateTime? parseRfc3339(String value) =>
    _rfc3339.hasMatch(value) ? DateTime.tryParse(value.toUpperCase()) : null;

final _rfc3339 = RegExp(
  r'^\d{4}-\d{2}-\d{2}[Tt]\d{2}:\d{2}:\d{2}(\.\d+)?([Zz]|[+-]\d{2}:\d{2})$',
);
