use std::vec;
use chrono::{Datelike, Utc};
use simple_string_patterns::{CharGroupMatch, StripCharacters};
use to_segments::ToSegments;
use crate::{guess::guess_time_splitter, DateOrder};

/// How many years into the future a 2-digit year is still expanded to the current
/// century before rolling back to the previous one -- e.g. with today in 2026, "46"
/// still means 2046, but "47" means 1947. Slides forward year by year rather than
/// pinning to a fixed 50/50 split, matching the common Excel/strptime-style convention.
const PIVOT_YEARS_AHEAD: u16 = 20;

/// Expands a bare 2-digit year (0-99) into a full 4-digit year using a sliding pivot
/// (see `PIVOT_YEARS_AHEAD`). Values already >= 100 (a real 3-or-more-digit year, e.g.
/// a historical/astronomical date) are returned unchanged -- this only ever applies to
/// genuinely ambiguous 2-digit shorthand, common in spreadsheet/CSV date cells (e.g.
/// "21-06-23"), not to short-but-real historical years.
fn expand_two_digit_year(yr: u16) -> u16 {
  if yr >= 100 {
    return yr;
  }
  let current_year = Utc::now().year() as u16;
  let this_century_start = (current_year / 100) * 100;
  let pivot_year = current_year + PIVOT_YEARS_AHEAD;
  let candidate = this_century_start + yr;
  if candidate <= pivot_year {
    candidate
  } else {
    candidate - 100
  }
}

/// convert the state component of a date-time string to a valid ISO-compatible string
pub(crate) fn to_formatted_date_string(date_srr: &str,date_order: DateOrder, splitter: Option<char>) -> Option<String> {
    let parts: Vec<String> = if let Some(split_char) = splitter {
      date_srr.to_parts(&split_char.to_string())
    } else {
      digits_to_date_parts(date_srr, date_order)
    };
    let (yr_idx, month_idx, day_idx) = date_order.to_ymd_indices();
    let mut date_parts: Vec<u16> = parts.into_iter()
      .filter(|n| n.is_digits_only())
      .map(|dp| dp.parse::<u16>().unwrap_or(0))
      .collect();
    let num_parts = date_parts.len();
    while date_parts.len() < 3 {
      date_parts.push(0);
    }
    // ':' is only ever a last-resort *guessed* splitter (see guess_date_splitter) for a
    // string with no real date separator at all -- most commonly a bare time string like
    // "10:10:10" with nothing to distinguish it from a date. Century expansion must not
    // apply there, or a plain time gets misread as a valid (if nonsensical) date.
    //
    // It's also restricted to a genuine 3-component D-M-Y date: a bare 2-part value like
    // "12.5" or "12.30" is far more likely to be a plain decimal number (a price, a
    // measurement) than an abbreviated 2-digit-year date -- "." in particular is a
    // decimal point as often as it's a date separator. Without this, a short "year" like
    // "12" would get expanded into a plausible-looking 2012 and the whole value silently
    // misread as a date. A *year-and-month-only* partial date (this crate's original
    // use case, e.g. "1678-6" for June 1678) is unaffected either way, since it already
    // carries a real 4-digit year needing no expansion at all.
    let yr_raw = date_parts[yr_idx];
    let yr = if splitter == Some(':') || num_parts < 3 { yr_raw } else { expand_two_digit_year(yr_raw) };
    if yr < 1000 {
      return None;
    }
    let mut month = date_parts[month_idx];
    // default 0 for a missing month will be set to 1
    if month < 1 {
      month = 1
    }
    if month > 12 {
      return None;
    }
    // default 0 for a missing day will be set to 1
    let mut day = date_parts[day_idx];
    if day < 1 {
      day = 1
    }
    if day > 31 {
      return None;
    }
    Some(format!("{:04}-{:02}-{:02}", yr, month, day))
  }

/// extract the time and millseconds components of a date-time string
pub(crate) fn fuzzy_to_formatted_time_parts(time_part: &str, ms_tz: &str, time_separator: Option<char>, add_z: bool) -> Option<(String, String)> {
  let t_split_opt = if let Some(t_splitter) = time_separator {
    Some(t_splitter)
  } else {
    guess_time_splitter(time_part)
  };
  let t_parts: Vec<&str> = if let Some(t_split) = t_split_opt {
    time_part.split(t_split).collect()
  } else {
    vec![time_part[0..2].as_ref(), time_part[2..4].as_ref(), time_part[4..6].as_ref()]
  };
  if let Some(&first) = t_parts.first() {
    if !first.is_digits_only() {
      return None;
    }
  }
  let mut time_parts: Vec<u8> = t_parts.into_iter()
  .filter(|&n| n.is_digits_only())
  .map(|tp| tp.parse::<u8>().unwrap_or(0))
  .collect();

  while time_parts.len() < 3 {
      time_parts.push(0);
  }
  let hrs = time_parts[0];
  if hrs > 23 {
      return None;
  }
  let mins = time_parts[1];
  if mins > 59 {
      return None;
  }
  let secs = time_parts[2];
  if secs > 59 {
      return None;
  }
  let formatted_time = format!("{:02}:{:02}:{:02}", hrs, mins, secs);
  let tz_suffix = if add_z {
      let max_len = if ms_tz.len() > 3 {
      3
      } else {
      ms_tz.len()
      };
      let ms = ms_tz[0..max_len].parse::<u16>().unwrap_or(0);
      format!(".{:03}Z", ms)
  } else {
      "".to_string()
  };
  Some((formatted_time, tz_suffix))
}


pub fn digits_to_date_parts(date_str: &str, order: DateOrder) -> Vec<String> {
  let digits = date_str.strip_non_digits();
  let num_digits = digits.len() as u8;
  if num_digits > 5 && num_digits < 9 {
    let offsets = order.fixed_offsets(num_digits);
    vec![digits[offsets.0].to_string(), digits[offsets.1].to_string(), digits[offsets.2].to_string()]
  } else {
    vec![digits]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_expand_two_digit_year_stays_within_current_century_near_now() {
    // A 2-digit year matching "now" always expands to the current century, regardless
    // of when this test runs.
    let current_year = Utc::now().year() as u16;
    let this_century_start = (current_year / 100) * 100;
    let yy = current_year % 100;
    assert_eq!(expand_two_digit_year(yy), this_century_start + yy);
  }

  #[test]
  fn test_expand_two_digit_year_rolls_back_past_the_pivot() {
    // The 2-digit year immediately after the pivot must resolve at or before the pivot
    // itself, not further into the future -- i.e. it rolls back a century rather than
    // landing implausibly ahead. Checking the invariant directly (rather than
    // re-deriving the exact expected year) keeps this robust across century-boundary
    // edge cases in the "current year" used to compute the pivot.
    let current_year = Utc::now().year() as u16;
    let pivot_year = current_year + PIVOT_YEARS_AHEAD;
    let yy = (pivot_year + 1) % 100;
    let expanded = expand_two_digit_year(yy);
    assert!(expanded <= pivot_year, "expected {} to resolve at or before the pivot year {}, got {}", yy, pivot_year, expanded);
    assert_eq!(expanded % 100, yy, "expanded year should still end in the requested 2 digits");
  }

  #[test]
  fn test_expand_two_digit_year_leaves_longer_years_untouched() {
    // Already-full years (>= 100, e.g. a genuine historical/astronomical date) must not
    // be reinterpreted as 2-digit shorthand.
    assert_eq!(expand_two_digit_year(1678), 1678);
    assert_eq!(expand_two_digit_year(100), 100);
  }

  #[test]
  fn test_colon_splitter_is_not_expanded_guarding_against_time_only_strings() {
    // "10:10:10" guesses DMY with ':' as a last-resort splitter (see
    // guess::guess_date_splitter) since there's no real date separator at all -- this
    // must not be treated as a 2-digit-year date, or a bare time string like this would
    // get misread as a valid (if nonsensical) date.
    assert_eq!(to_formatted_date_string("10:10:10", DateOrder::DMY, Some(':')), None);
  }

  #[test]
  fn test_real_separators_do_get_two_digit_year_expansion() {
    // "-", "/" and "." are the separators that prevail for date components in
    // practice -- all three must expand a genuine 3-component 2-digit-year date
    // identically.
    let current_year = Utc::now().year() as u16;
    let this_century_start = (current_year / 100) * 100;
    let expected = Some(format!("{:04}-06-23", this_century_start + 21));
    for (value, splitter) in [("21-06-23", '-'), ("21/06/23", '/'), ("21.06.23", '.')] {
      assert_eq!(
        to_formatted_date_string(value, DateOrder::YMD, Some(splitter)),
        expected,
        "{:?} with splitter {:?} should expand the 2-digit year the same way",
        value,
        splitter
      );
    }
  }

  #[test]
  fn test_two_part_values_are_not_expanded_as_dates() {
    // Regression: a bare 2-component value like "12.30" or "12.5" is far more likely to
    // be a plain decimal (a price, a measurement) than an abbreviated 2-digit-year date
    // -- "." in particular is a decimal point as often as it's a date separator.
    // Century expansion must not turn a short "year" component into a plausible-looking
    // 20xx and silently misread the whole value as a date.
    for (value, splitter) in [("12.30", '.'), ("12.5", '.'), ("3.14", '.'), ("0.99", '.')] {
      assert_eq!(
        to_formatted_date_string(value, DateOrder::YMD, Some(splitter)),
        None,
        "{:?} should not be read as a date",
        value
      );
    }
  }

  #[test]
  fn test_year_and_month_only_partial_dates_are_unaffected() {
    // The crate's original use case (a full 4-digit year with just year+month known,
    // e.g. "1678-6" for June 1678) is a 2-component value too, but it's untouched by
    // the num_parts < 3 restriction above since a real 4-digit year never goes through
    // expand_two_digit_year in the first place (it's already >= 100).
    assert_eq!(
      to_formatted_date_string("1678-6", DateOrder::YMD, Some('-')),
      Some("1678-06-01".to_string())
    );
  }
}