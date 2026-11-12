/// Parse an RFC 3339 `date-time` (SPEC §4.2), or return null.
///
/// Stricter than [DateTime.tryParse], which reads a timestamp with no zone
/// designator as *local* time and accepts forms RFC 3339 does not (a bare date,
/// a space for the `T`, no separators at all). The Rust and Go runtimes refuse
/// both, and an instant that depends on the consumer's time zone is not one
/// instant at all.
///
/// Every field must also be in range: month 1-12, a day the month has (leap
/// years included), hour 0-23, minute and second 0-59 (no leap second), and an
/// offset of at most 23:59. [DateTime.tryParse] rolls an out-of-range field
/// over (`2026-13-45T25:61:61Z` reads as February 2027) and takes any offset,
/// so it is only consulted once the fields are known to name a real instant.
/// The Rust, Go and TypeScript runtimes apply the same ranges, and
/// `fixtures/consume/timestamps.json` holds the shared cases.
DateTime? parseRfc3339(String value) {
  final m = _rfc3339.firstMatch(value);
  if (m == null) return null;
  final [year, month, day, hour, minute, second] = [
    for (var i = 1; i <= 6; i++) int.parse(m.group(i)!),
  ];
  if (month < 1 ||
      month > 12 ||
      day < 1 ||
      day > _daysInMonth(year, month) ||
      hour > 23 ||
      minute > 59 ||
      second > 59) {
    return null;
  }
  if (m.group(8) != null &&
      (int.parse(m.group(9)!) > 23 || int.parse(m.group(10)!) > 59)) {
    return null;
  }
  return DateTime.tryParse(value.toUpperCase());
}

final _rfc3339 = RegExp(
  r'^(\d{4})-(\d{2})-(\d{2})[Tt](\d{2}):(\d{2}):(\d{2})(\.\d+)?'
  r'(?:[Zz]|([+-])(\d{2}):(\d{2}))$',
);

/// Days in [month] (1-12) of the proleptic Gregorian [year].
int _daysInMonth(int year, int month) {
  if (month == 2) {
    final leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    return leap ? 29 : 28;
  }
  return const {4, 6, 9, 11}.contains(month) ? 30 : 31;
}
