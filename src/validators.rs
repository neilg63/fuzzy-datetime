use simple_string_patterns::CharGroupMatch;

/// check if athe captured last segment represents milliseconds, microseconds or nanoseconds with an optional character at at the end
pub(crate) fn segment_is_subseconds(segment: &str) -> bool {
    let s_len = segment.len();
    if s_len >= 3 {
      if s_len > 3 {
        let last = &segment[s_len - 1..];
        let head = &segment[..s_len - 1];
        head.is_digits_only() && last.has_alphanumeric()
      } else {
        segment.is_digits_only()
      }
    } else {
      false
    }
  }