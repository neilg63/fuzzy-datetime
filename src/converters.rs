use simple_string_patterns::CharGroupMatch;
use crate::{guess::guess_time_splitter, DateOrder};

/// convert the state component of a date-time string to a valid ISO-compatible string
pub(crate) fn to_formatted_date_string(date_srr: &str,date_order: DateOrder, splitter: char) -> Option<String> {
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


pub(crate) fn fuzzy_to_formatted_time_parts(time_part: &str, ms_tz: &str, time_separator: Option<char>, add_z: bool) -> Option<(String, String)> {
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
  Some((formatted_time, tz_suffix))
}