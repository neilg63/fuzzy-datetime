use chrono::{format::ParseErrorKind, Date, NaiveDate, NaiveDateTime, ParseError};
use simple_string_patterns::{CharGroupMatch, CharType, SimplContainsType, ToSegments};
use std::{error::Error, f32::consts::E};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOrder {
  YMD,
  DMY,
  MDY,
}

impl DateOrder {
  /// render date format as indices for year, month and day
  pub fn to_ymd_indices(&self) -> (usize, usize, usize) {
    match self {
      DateOrder::YMD => (0, 1, 2),
      DateOrder::DMY => (2, 1, 0),
      DateOrder::MDY => (2, 0, 1),
    }
  }
}

/// Probable date-time format when comparing many sample date strings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOrderGuess {
  NonDate,
  YearFirst,
  DayFirst,
  MonthFirst,
  DayOrMonthFirst,
}

impl DateOrderGuess {
  
  // default to to one of the known date orders
  // YMD takes precedence over DMY unless the guessed order is DayFirst or DayOrMonthFirst
  pub fn to_order(&self) -> DateOrder{
    match self {
      Self::YearFirst | Self::NonDate => DateOrder::YMD,
      Self::MonthFirst => DateOrder::MDY,
      _ => DateOrder::DMY,
    }
  }
}

/// Options for parsing the date component of strings
pub struct DateOptions(pub DateOrder, pub char);

impl DateOptions {
  pub fn order(&self) -> DateOrder {
    self.0
  }

  pub fn splitter(&self) -> char {
    self.1
  }
}

impl Default for DateOptions {
  fn default() -> Self {
    DateOptions(DateOrder::YMD, '-')
  }
}

/// instantiate options with three common orders + split character
/// e.g. DateOptions::dmy('.')
impl DateOptions {
  pub fn ymd(splitter: char) -> Self {
    DateOptions(DateOrder::YMD, splitter)
  }

  pub fn dmy(splitter: char) -> Self {
    DateOptions(DateOrder::DMY, splitter)
  }

  pub fn mdy(splitter: char) -> Self {
    DateOptions(DateOrder::MDY, splitter)
  }
}


/// If the second argument is None, the function will attempt to guess the date order
/// Otherwise, it will use the provided date order and splitter
pub fn fuzzy_to_datetime(dt: &str, date_opts: Option<DateOptions>, time_separator: Option<char>) -> Result<NaiveDateTime, ParseError> {
  if let Some(formatted_str) = fuzzy_to_datetime_string(dt, date_opts, time_separator) {
    NaiveDateTime::parse_from_str(&formatted_str, "%Y-%m-%dT%H:%M:%S%.3fZ")
  } else {
    // trigger ParseErrorKind::InvalidFormat from an empty string
    NaiveDateTime::parse_from_str("", "%Y")
  }
}

/// convert a date-time-like string to a valid ISO 8601-compatible date-time string
/// for direct output or further processing via chrono
/// Assume all input dates conforms to the ISO 8601 order, even if incompmlete. All guessing is short-circuited
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
/// for driect output or further processing via chrono
/// If date_opts is None, the function will attempt to guess the date order
/// For best performance, provide the date order and splitter
pub fn fuzzy_to_date_string(dt: &str, date_opts: Option<DateOptions>) -> Option<String> {
  if let Some((date_str, _t_str, _ms_tz)) = fuzzy_to_date_string_with_time(dt, date_opts) {
    if !date_str.is_empty() {
      Some(date_str)
    } else {
      None
    }
  } else {
    None
  }
}

/// convert a date-time-like string to a valid ISO 8601-compatible string
pub fn fuzzy_to_date_string_with_time(dt: &str, date_opts: Option<DateOptions>) -> Option<(String, String, String)> {
	
  let (dt_str, mtz) = dt.to_start_end(".");
  let has_mtz = segment_is_subseconds(&mtz);
  let milli_tz = if has_mtz {
    mtz
  } else {
    "".to_string()
  };
  let dt_base = if has_mtz {
    dt_str
  } else {
    dt.to_string()
  };
	let clean_dt = dt_base.replace("T", " ").trim().to_string();
	let mut dt_parts = clean_dt.split_whitespace();
	let date_part = dt_parts.next().unwrap_or("0000-01-01");
  let date_options = if let Some(dt_opts) = date_opts {
    dt_opts
  } else {
    detect_date_order_and_splitter(date_part)
  };
	let time_part = dt_parts.next().unwrap_or("00:00:00");
	if date_part.contains_type(CharType::Alpha) {
			return None;
	}

	if let Some(formatted_date) = to_formatted_date_string(date_part, date_options.order(), date_options.splitter()) {
    Some((formatted_date, time_part.to_string(), milli_tz))
  } else {
    None
  }
}

/// convert teh state component of a date-time string to a valid ISO-compatible string
fn to_formatted_date_string(date_srr: &str,date_order: DateOrder, splitter: char) -> Option<String> {
  let parts: Vec<&str> = date_srr.split(splitter).collect();
  let (yr_idx, month_idx, day_idx) = date_order.to_ymd_indices();
  let mut date_parts: Vec<u16> = parts.into_iter()
    .filter(|&n| n.is_digits_only())
    .map(|dp| dp.parse::<u16>().unwrap_or(0))
    .collect();
  while date_parts.len() < 3 {
    date_parts.push(0);
  }
  let yr = date_parts[yr_idx];
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
    let t_splitter = time_separator.unwrap_or(guess_time_splitter(&time_part));
		let t_parts: Vec<&str> = time_part.split(t_splitter).collect();
    if let Some(&first) = t_parts.get(0) {
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
    let formatted_str = format!("{}{}{}{}", formatted_date, separator, formatted_time, tz_suffix);
    if !formatted_str.is_empty() {
      Some(formatted_str)
    } else {
      None
    }
	} else {
		None
	}
}

pub fn is_datetime_like(text: &str) -> bool {
    fuzzy_to_datetime_string(text, None, None).is_some()
}

fn guess_date_splitter(date_str: &str) -> char {
  guess_unit_splitter(date_str, &['.', '·', '-', '/']).unwrap_or('-')
}

fn guess_time_splitter(time_str: &str) -> char {  
  // If no valid separator found, default to '-'
  guess_unit_splitter(time_str, &[':', '.']).unwrap_or(':')
}

fn guess_unit_splitter(unit_str: &str, separators: &[char]) -> Option<char> {
  let trimmed = unit_str.trim();
  let num_chars = trimmed.chars().count();
  let mut index = 0;
  for c in trimmed.chars() {
    if index > 0 && index < num_chars - 1 && separators.contains(&c) {
      return Some(c);
    }
    index += 1;
  }
  None
}

/// Detect the date order and splitter from a date string
pub fn detect_date_order_and_splitter(date_str: &str) -> DateOptions {
  let splitter = guess_date_splitter(date_str);
  DateOptions(detect_date_order(date_str, splitter), splitter)
}

pub fn detect_date_order(date_str: &str, splitter: char) -> DateOrder {
  guess_date_order(date_str, splitter).to_order()
}

/// Guess the date order from a date string
/// assuming YMD, DMY or MDY as the likely order
/// but catering for ambiguous cases or invalid dates
/// Date strings with fewer than 3 parts must include the year
pub fn guess_date_order(date_str: &str, splitter: char) -> DateOrderGuess {
  let str_parts = date_str.split(splitter).collect::<Vec<&str>>();
  let date_parts: Vec<&str> = str_parts.into_iter().filter(|n| n.is_digits_only()).collect();
  let num_parts = date_parts.len();
  let first_len = if num_parts > 0 {
    date_parts[0].len()
  } else {
    0
  };

  // It's not a date, if the first element's length is less than 4 and there are fewer than 3 parts 
  // or otherwise if the first element has no digits
  if (first_len < 1 && num_parts > 2) || (first_len < 4 && num_parts < 3) {
    return DateOrderGuess::NonDate;
  }
  // If the length of the first segment is 4, it's likely a year
  if num_parts < 2 || first_len == 4 {
    return DateOrderGuess::YearFirst;
  } else {
    let first_num = date_parts[0].parse::<u16>().unwrap_or(0);
    if num_parts==2 {
      if first_num < 13 {
        return DateOrderGuess::DayFirst;
      } else {
        return DateOrderGuess::YearFirst;
      }
    } else {
      let second_num = date_parts[1].parse::<u16>().unwrap_or(0);
      let third_num = date_parts[2].parse::<u16>().unwrap_or(0);
      if first_num > 31 {
        return DateOrderGuess::YearFirst;
      } else if first_num < 13 {
        if second_num > 12 && third_num > 31 {
          return DateOrderGuess::MonthFirst;
        } else {
          return DateOrderGuess::DayOrMonthFirst;
        }
      } else if first_num > 12 && third_num > 31 {
        return DateOrderGuess::DayFirst;
      } else {
        return DateOrderGuess::YearFirst;
      }
    }
  }
}

/// check if athe captured last segment represents milliseconds, microseconds or nanoseconds with an optional character at at the end
fn segment_is_subseconds(segment: &str) -> bool {
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


/// This assumes all date strings are in the same format
/// and deduces through elimination
pub fn detect_date_format_from_list(date_list: &[&str]) -> DateOptions {
  let mut order = DateOrder::YMD;
  let mut splitter = '-';
  for dt_str in date_list {
    let split_char = guess_date_splitter(dt_str);
    let guess = guess_date_order(dt_str, split_char);
    match guess {
      DateOrderGuess::YearFirst => {
        order = DateOrder::YMD;
        splitter = split_char;
        break;
      },
      DateOrderGuess::DayFirst => {
        order = DateOrder::DMY;
        splitter = split_char;
        break;
      },
      DateOrderGuess::MonthFirst => {
        order = DateOrder::MDY;
        splitter = split_char;
        break;
      },
      _ => continue,
    }
  }
  DateOptions(order, splitter)
}

#[cfg(test)]
mod tests {
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
    fn test_is_datetime_like() {
        assert!(is_datetime_like("2023-10-10T10:10:10"));
        assert!(is_datetime_like("2023-10-10 10:10:10"));
        assert!(is_datetime_like("2023-10-10"));
        assert!(!is_datetime_like("10:10:10"));
        assert!(!is_datetime_like("invalid-date"));
        assert!(!is_datetime_like("2023-10-10Tinvalid"));
    }

    #[test]
    fn test_detect_date_order() {
      let sample_date_1 = "1876-08-29";      
      assert_eq!(detect_date_order(sample_date_1, '-'), DateOrder::YMD);

      let sample_date_2 = "28/02/1998";
      assert_eq!(detect_date_order(sample_date_2, '/'), DateOrder::DMY);

      let sample_date_3 = "02/28/1998";
      assert_eq!(detect_date_order(sample_date_3, '/'), DateOrder::MDY);

      // Ambiguous year-last dates will default to DMY (sorry Americans)
      // However, this can be overridden by specifying the date order
      // order parsing a set of dates to see if any have numbers greater than 12 in the second position
      // and no numbers over 12 in the first position
      let sample_date_4 = "08/07/1998";
      assert_eq!(detect_date_order(sample_date_4, '/'), DateOrder::DMY);
    }

    #[test]
    fn test_detect_date_order_and_splitter() {
      let sample_date_1 = "1876-08-29";
      let date_opts_1 = detect_date_order_and_splitter(sample_date_1);
      assert_eq!(date_opts_1.order(), DateOrder::YMD);
      assert_eq!(date_opts_1.splitter(), '-');

      let sample_date_2 = "28/02/1998";
      let date_opts_2 = detect_date_order_and_splitter(sample_date_2);
      assert_eq!(date_opts_2.order(), DateOrder::DMY);
      assert_eq!(date_opts_2.splitter(), '/');
    }

    #[test]
    fn test_millisecond_splitter() {
      
        let sample_1 = "2023-08-29T19.34.39.678Z";
        let (dt_base, milli_tz) = sample_1.to_start_end(".");
        assert_eq!(dt_base, "2023-08-29T19.34.39");
        assert_eq!(milli_tz, "678Z");

        assert_eq!(segment_is_subseconds("678Z"), true);
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
      assert_eq!(date_opts_usa.splitter(), '/');

      // Many other countries use DMY with slashes
      let sample_dates_dmy = vec![
        "08/07/1998",
        "10/09/2021",
        "15/12/2022",
        "09/11/1999",
      ];

      let date_opts_dmy = detect_date_format_from_list(&sample_dates_dmy);
      assert_eq!(date_opts_dmy.order(), DateOrder::DMY);
      assert_eq!(date_opts_dmy.splitter(), '/');


      // Dates in Germany and many other European countries are DMY with full stops
      let sample_dates_de = vec![
        "8.7.1998",
        "10.9.2021",
        "15.12.2022",
        "9.11.1999",
      ];
      let date_opts_de = detect_date_format_from_list(&sample_dates_de);
      assert_eq!(date_opts_de.order(), DateOrder::DMY);
      assert_eq!(date_opts_de.splitter(), '.');

      // French dates are also DMY, but often with hyphens
      let sample_dates_fr = vec![
        "08-07-1998",
        "10-09-2021",
        "15-12-2022",
        "09-11-1999",
      ];
      let date_opts_fr = detect_date_format_from_list(&sample_dates_fr);
      assert_eq!(date_opts_fr.order(), DateOrder::DMY);
      assert_eq!(date_opts_fr.splitter(), '-');

      let sample_dates_iso = vec![
        "1998-07-08",
        "2021-09-10",
        "2022-12-15",
        "1999-11-09",
      ];
      let date_opts_iso = detect_date_format_from_list(&sample_dates_iso);
      assert_eq!(date_opts_iso.order(), DateOrder::YMD);
      assert_eq!(date_opts_iso.splitter(), '-');
    }

    #[test]
    fn test_fuzz_to_date_string() {
      // correct date
      let sample_str_1 = fuzzy_to_date_string("1993-8-29", Some(DateOptions::default()));
      assert_eq!(sample_str_1, Some("1993-08-29".to_string()));

      let sample_str_2 = fuzzy_to_date_string("1993-8", Some(DateOptions::default()));
      assert_eq!(sample_str_2, Some("1993-08-01".to_string()));

      // correct date
      let sample_str_3 = fuzzy_to_date_string("29/08/1993", Some(DateOptions::dmy('/')));
      assert_eq!(sample_str_3, Some("1993-08-29".to_string()));
    }
}