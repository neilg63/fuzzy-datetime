use chrono::{NaiveDate, NaiveDateTime, ParseError};
use simple_string_patterns::{CharGroupMatch, CharType, SimplContainsType};
use to_segments::ToSegments;

mod date_order;
mod guess;
mod validators;
mod converters;
mod detect;
mod from_fuzzy_iso_string;

pub use date_order::{DateOrder, DateOptions};
pub use detect::{detect_date_format_from_list, detect_date_format_from_generic_list};
pub use from_fuzzy_iso_string::*;
use guess::surmise_date_order_and_splitter;
use validators::segment_is_subseconds;
use converters::{fuzzy_to_formatted_time_parts, to_formatted_date_string};

/// If the second argument is None, the function will attempt to guess the date order
/// Otherwise, it will use the provided date order and splitter
pub fn fuzzy_to_datetime(dt: &str, date_opts: Option<DateOptions>, time_separator: Option<char>) -> Result<NaiveDateTime, ParseError> {
  let formatted_str = fuzzy_to_datetime_string(dt, date_opts, time_separator).unwrap_or_default();
  NaiveDateTime::parse_from_str(&formatted_str, "%Y-%m-%dT%H:%M:%S%.3fZ")
}

/// convert a date-time-like string to a valid ISO 8601-compatible date-time string
/// for direct output or further processing via chrono
/// Assume all input dates conforms to the ISO 8601 order, even if incomplete. All guessing is short-circuited
/// This is compatible with original function in julian_day_converter
pub fn iso_fuzzy_string_to_datetime(dt: &str) -> Result<NaiveDateTime, ParseError> {
  fuzzy_to_datetime(dt, Some(DateOptions::default()), Some(':'))
}

/// If the second argument is None, the function will attempt to guess the date order
/// Otherwise, it will use the provided date order and splitter
pub fn fuzzy_to_date(dt: &str, date_opts: Option<DateOptions>) -> Result<NaiveDate, ParseError> {
  let date_str = fuzzy_to_date_string(dt, date_opts).unwrap_or_default();
  NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
}

/// Convert a ISO YMD date-like string to a NaiveDate
/// It assumes Y-M-D order and a hyphen as the splitter, but can accommodate missing month or day components
pub fn iso_fuzzy_to_date(dt: &str) -> Result<NaiveDate, ParseError> {
  fuzzy_to_date(dt, Some(DateOptions::default()))
}

/// convert a date-time-like string to a valid ISO 8601-compatible date string
/// for direct output or further processing via chrono
/// If date_opts is None, the function will attempt to guess the date order with bias towards YMD and DMY in case of ambiguity
/// For best performance, provide the date order and splitter
pub fn fuzzy_to_date_string(dt: &str, date_opts: Option<DateOptions>) -> Option<String> {
  if let Some((date_str, _t_str, _ms_tz)) = fuzzy_to_date_string_with_time(dt, date_opts) {
    if !date_str.is_empty() {
      return Some(date_str)
    }
  }
  None
}

/// convert a date-like assuming the source string follows the Y-M-D pattern
pub fn iso_fuzzy_to_date_string(dt: &str) -> Option<String> {
	fuzzy_to_date_string(dt, Some(DateOptions::default()))
}

/// convert a date-time-like assuming the source string follows the Y-M-D H:m:s pattern
pub fn iso_fuzzy_to_datetime_string(dt: &str) -> Option<String> {
	fuzzy_to_datetime_string_opts(dt, 'T', Some(DateOptions::default()), Some(':'), true)
}


/// convert a date-time-like string to a valid ISO 8601-compatible string
pub fn fuzzy_to_date_string_with_time(dt: &str, date_opts: Option<DateOptions>) -> Option<(String, String, String)> {
	
  let (dt_opt, mtz_opt) = dt.to_start_end(".");
  let has_mtz = if let Some(mtz) = mtz_opt {
    segment_is_subseconds(mtz)
  } else {
    false
  };
  let milli_tz = if has_mtz {
    mtz_opt.unwrap_or_default()
  } else {
    ""
  }.to_string();
  let dt_base = if has_mtz {
    dt_opt.unwrap_or_default()
  } else {
    dt
  }.to_string();
	let clean_dt = dt_base.replace("T", " ").trim().to_string();
	let mut dt_parts = clean_dt.split_whitespace();
	let date_part = dt_parts.next().unwrap_or("0000-01-01");
  let date_options = if let Some(dt_opts) = date_opts {
    dt_opts
  } else {
    surmise_date_order_and_splitter(date_part)
  };
	let time_part = dt_parts.next().unwrap_or("00:00:00");
	if date_part.contains_type(CharType::Alpha) {
			return None;
	}

	to_formatted_date_string(date_part, date_options.order(), date_options.splitter()).map(|formatted_date| (formatted_date, time_part.to_string(), milli_tz))
}


/// convert a date-time-like string to a valid ISO 8601-compatible string
pub fn fuzzy_to_datetime_string(dt: &str, date_opts: Option<DateOptions>, time_separator: Option<char>) -> Option<String> {
	fuzzy_to_datetime_string_opts(dt, 'T', date_opts, time_separator, true)
}

/// convert a date-time-like string to a valid ISO 8601-compatible string
/// dt: the date-time string
/// separator: the separator between the date and time parts
/// add_z: whether to add 'Z' timezone indicator
pub fn fuzzy_to_datetime_string_opts(dt: &str, separator: char, date_opts: Option<DateOptions>, time_separator: Option<char>, add_z: bool) -> Option<String> {
  if let Some((formatted_date, time_part, ms_tz)) = fuzzy_to_date_string_with_time(dt, date_opts) {
    // exclude the the whole date-time string if the time part is non-empty without digits
    if !time_part.is_empty() && !time_part.has_digits() {
      return None;
    }
    // fuzzy_to_formatted_time_parts failing here means a genuine, non-empty time chunk
    // was present but couldn't be parsed (e.g. "11.39" with a separator time_separator
    // doesn't recognise) -- time_part is *always* pre-filled to "00:00:00" upstream when
    // no time was present at all, so that always parses fine and never reaches this. The
    // previous `.unwrap_or_default()` here silently discarded a real parse failure and
    // produced a malformed, dangling result like "2026-07-19T" (date, separator, nothing)
    // instead of correctly failing the whole (date+time) parse.
    let (formatted_time, tz_suffix) = fuzzy_to_formatted_time_parts(&time_part, &ms_tz, time_separator, add_z)?;
    let formatted_str = format!("{}{}{}{}", formatted_date, separator, formatted_time, tz_suffix);
    if !formatted_str.is_empty() {
      return Some(formatted_str);
    }
	}
  None
}

// Check if a string is likely to be a date string with an optional time component
pub fn is_datetime_like(text: &str) -> bool {
  fuzzy_to_datetime_string(text, None, None).is_some()
}

#[cfg(test)]
mod tests {
    use guess::surmise_date_order;

  use super::*;

  #[test]
  fn test_fuzzy_dates() {
      let sample_1 = "2001-apple";
      assert!(fuzzy_to_datetime(sample_1, None, None).is_err());
      assert_eq!(fuzzy_to_datetime_string(sample_1, None, None), None);

      let sample_2 = "1876-08-29 17:15";
      assert!(fuzzy_to_datetime(sample_2, None, None).is_ok());

      // correct sample datetime
      let sample_3 = "2023-8-29 19:34:39";
      assert_eq!(
          fuzzy_to_datetime_string(sample_3, None, None),
          Some("2023-08-29T19:34:39.000Z".to_string())
      );

      // Correct date-only string
      let sample_4 = "2023-9-10";
      assert_eq!(
          fuzzy_to_date_string(sample_4, None),
          Some("2023-09-10".to_string())
      );
      // time-only strings should not be valid
      let sample_5 = "10:10:10";
      assert_eq!(
          fuzzy_to_datetime_string(sample_5, None, None),
          None
      );

      // datetime with extra milliseconds and timezone
      let sample_3 = "2023-08-29T19:34:39.678Z";
      assert_eq!(
          fuzzy_to_datetime_string(sample_3, None, None),
          Some(sample_3.to_string())
      );
  }

  #[test]
  fn test_dot_separated_dates_are_recognised_under_guessing() {
      // Regression: segment_is_subseconds misread a bare 4-digit year (e.g. "2026" from
      // "19.07.2026") as milliseconds-plus-timezone-suffix, since it only checked that
      // the trailing character was *alphanumeric* -- true for a digit too -- rather than
      // genuinely non-numeric (like the "Z" in "678Z"). to_start_end(".") then silently
      // chopped the year off as a fake millisecond/timezone segment, leaving only
      // "19.07" for order-guessing, which has no year at all and always failed.
      for (value, expected) in [
          ("19.07.2026", "2026-07-19"), // DMY (day 19 rules out MDY)
          ("07.19.2026", "2026-07-19"), // MDY (day 19 rules out DMY)
          ("2026.07.19", "2026-07-19"), // YMD
      ] {
          assert_eq!(
              fuzzy_to_date_string(value, None),
              Some(expected.to_string()),
              "{:?} should resolve to {:?}",
              value,
              expected
          );
      }
  }

  #[test]
  fn test_a_genuinely_unparseable_time_component_fails_the_whole_datetime_rather_than_producing_a_dangling_result() {
      // Regression: fuzzy_to_datetime_string_opts used `.unwrap_or_default()` on a
      // failed time-part parse, silently discarding the failure and producing a
      // malformed, dangling result like "2026-07-19T" (date, separator, nothing) instead
      // of correctly failing the whole date+time parse. "11.39" as a time chunk (dot
      // separator, not colon) can't be parsed as a time at all here.
      assert_eq!(fuzzy_to_datetime_string_opts("2026-07-19 11.39", 'T', None, Some(':'), true), None);
      // the date-only equivalent is unaffected -- no time component is even attempted
      assert_eq!(fuzzy_to_date_string("2026-07-19", None), Some("2026-07-19".to_string()));
      // a genuine, valid time component still works fine
      assert_eq!(
          fuzzy_to_datetime_string_opts("2026-07-19 11:39:05", 'T', None, Some(':'), true),
          Some("2026-07-19T11:39:05.000Z".to_string())
      );
  }

  #[test]
  fn test_is_datetime_like() {
      assert!(is_datetime_like("2023-10-10T10:10:10"));
      assert!(is_datetime_like("2023-10-10 10:10:10"));
      assert!(is_datetime_like("2023-10-10"));
      assert!(!is_datetime_like("10:10:10"));
      assert!(!is_datetime_like("invalid-date"));
      assert!(!is_datetime_like("2023-10-10Tinvalid"));
  }

  #[test]
  fn test_surmise_date_order() {
    let sample_date_1 = "1876-08-29";      
    assert_eq!(surmise_date_order(sample_date_1, Some('-')), DateOrder::YMD);

    let sample_date_2 = "28/02/1998";
    assert_eq!(surmise_date_order(sample_date_2, Some('/')), DateOrder::DMY);

    let sample_date_3 = "02/28/1998";
    assert_eq!(surmise_date_order(sample_date_3, Some('/')), DateOrder::MDY);

    // Ambiguous year-last dates will default to DMY (sorry Americans)
    // However, this can be overridden by specifying the date order
    // order parsing a set of dates to see if any have numbers greater than 12 in the second position
    // and no numbers over 12 in the first position
    let sample_date_4 = "08/07/1998";
    assert_eq!(surmise_date_order(sample_date_4, Some('/')), DateOrder::DMY);
  }

  #[test]
  fn test_surmise_date_order_and_splitter() {
    let sample_date_1 = "1876-08-29";
    let date_opts_1 = surmise_date_order_and_splitter(sample_date_1);
    assert_eq!(date_opts_1.order(), DateOrder::YMD);
    assert_eq!(date_opts_1.splitter(), Some('-'));

    let sample_date_2 = "28/02/1998";
    let date_opts_2 = surmise_date_order_and_splitter(sample_date_2);
    assert_eq!(date_opts_2.order(), DateOrder::DMY);
    assert_eq!(date_opts_2.splitter(), Some('/'));
  }

  #[test]
  fn test_surmise_date_order_and_splitter_plain() {

    let sample_date_1 = "28021998";
    let date_opts_1 = surmise_date_order_and_splitter(sample_date_1);
    assert_eq!(date_opts_1.order(), DateOrder::DMY);
    assert_eq!(date_opts_1.splitter(), None);

    let sample_date_2 = "02281998";
    let date_opts_2 = surmise_date_order_and_splitter(sample_date_2);
    assert_eq!(date_opts_2.order(), DateOrder::MDY);

    let sample_date_3 = "18761208";
    let date_opts_3 = surmise_date_order_and_splitter(sample_date_3);
    assert_eq!(date_opts_3.order(), DateOrder::YMD);

  }

  #[test]
  fn test_millisecond_splitter() {
    
      let sample_1 = "2023-08-29T19.34.39.678Z";
      let (dt_base, milli_tz) = sample_1.to_start_end(".");
      assert_eq!(dt_base.unwrap(), "2023-08-29T19.34.39");
      assert_eq!(milli_tz.unwrap_or_default(), "678Z");

      assert!(segment_is_subseconds("678Z"));
  }

  #[test]
  fn test_detect_date_format_from_list() {
    
    // American dates are usually MDY with slashes
    let sample_dates_usa = vec![
      "07/08/1998",
      "09/10/2021",
      "12/15/2022",
      "11/09/1999",
    ];

    let date_opts_usa = detect_date_format_from_list(&sample_dates_usa);
    assert_eq!(date_opts_usa.order(), DateOrder::MDY);
    assert_eq!(date_opts_usa.splitter(), Some('/'));

    // Many other countries use DMY with slashes
    let sample_dates_dmy = vec![
      "08/07/1998",
      "10/09/2021",
      "15/12/2022",
      "09/11/1999",
    ];

    let date_opts_dmy = detect_date_format_from_list(&sample_dates_dmy);
    assert_eq!(date_opts_dmy.order(), DateOrder::DMY);
    assert_eq!(date_opts_dmy.splitter(), Some('/'));


    // Dates in Germany and many other European countries are DMY with full stops
    let sample_dates_de = vec![
      "8.7.1998",
      "10.9.2021",
      "15.12.2022",
      "9.11.1999",
    ];
    let date_opts_de = detect_date_format_from_list(&sample_dates_de);
    assert_eq!(date_opts_de.order(), DateOrder::DMY);
    assert_eq!(date_opts_de.splitter(), Some('.'));

    // French dates are also DMY, but often with hyphens
    let sample_dates_fr = vec![
      "08-07-1998",
      "10-09-2021",
      "15-12-2022",
      "09-11-1999",
    ];
    let date_opts_fr = detect_date_format_from_list(&sample_dates_fr);
    assert_eq!(date_opts_fr.order(), DateOrder::DMY);
    assert_eq!(date_opts_fr.splitter(), Some('-'));

    let sample_dates_iso = vec![
      "1998-07-08",
      "2021-09-10",
      "2022-12-15",
      "1999-11-09",
    ];
    let date_opts_iso = detect_date_format_from_list(&sample_dates_iso);
    assert_eq!(date_opts_iso.order(), DateOrder::YMD);
    assert_eq!(date_opts_iso.splitter(), Some('-'));


    struct SpecialDay {
      #[allow(dead_code)]
      name: String,
      date: String,
    }

    let rows: Vec<SpecialDay> = vec![
      SpecialDay {
        name: "Independence Day".to_string(),
        date: "07/04/1776".to_string(),
      },
      SpecialDay {
        name: "Christmas Day".to_string(),
        date: "12/25/2021".to_string(),
      },
      SpecialDay {
        name: "New Year's Day".to_string(),
        date: "01/01/2022".to_string(),
      },
    ];

    let date_opts_special = detect_date_format_from_generic_list(&rows, |x| Some(x.date.clone()));
    assert_eq!(date_opts_special.order(), DateOrder::MDY);
  }

  #[test]
  fn test_fuzzy_to_date_string() {
    // correct date
    let sample_str_1 = fuzzy_to_date_string("1993-8-29", Some(DateOptions::default()));
    assert_eq!(sample_str_1, Some("1993-08-29".to_string()));

    let sample_str_2 = fuzzy_to_date_string("1993-8", Some(DateOptions::default()));
    assert_eq!(sample_str_2, Some("1993-08-01".to_string()));

    // correct date
    let sample_str_3 = fuzzy_to_date_string("29/08/1993", Some(DateOptions::dmy('/')));
    assert_eq!(sample_str_3, Some("1993-08-29".to_string()));
  }
  #[test]
  fn test_fuzzy_datetime_to_naive_datetime() {
    let datetime_str = "1876-9-25 15:45"; // incomplete without zero-padding
    let dt = NaiveDateTime::from_fuzzy_iso_string(datetime_str).unwrap();
    assert_eq!(dt.to_string(), "1876-09-25 15:45:00".to_owned());
  }
}
