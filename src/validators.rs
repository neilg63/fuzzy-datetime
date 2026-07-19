use simple_string_patterns::CharGroupMatch;

/// check if athe captured last segment represents milliseconds, microseconds or nanoseconds with an optional character at at the end
pub(crate) fn segment_is_subseconds(segment: &str) -> bool {
    let s_len = segment.len();
    if s_len >= 3 {
      if s_len > 3 {
        let last = &segment[s_len - 1..];
        let head = &segment[..s_len - 1];
        // The trailing character must be a genuine non-digit timezone-ish indicator (e.g.
        // "678Z") for this to be milliseconds-plus-suffix -- `last.has_alphanumeric()`
        // used to accept *any* alphanumeric character here, and a digit is alphanumeric
        // too, so an all-digit tail with 4+ characters (e.g. a bare 4-digit year "2026"
        // sitting after the last '.' in a dot-separated date like "19.07.2026") was
        // wrongly misread as "milliseconds + suffix" and silently swallowed.
        head.is_digits_only() && !last.is_digits_only()
      } else {
        segment.is_digits_only()
      }
    } else {
      false
    }
  }