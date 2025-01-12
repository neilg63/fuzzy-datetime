use chrono::NaiveDateTime;

use crate::iso_fuzzy_string_to_datetime;

/// This trait may be implemented by any Date or DateTime object
/// An implementation for chrono::NaiveDateTime is provided below
/// 
/// It Will be removed in 0.4.0. The functionality has moved to the fuzzy-datetime crate
/// with more advanced date-time parsing and correction options
pub trait FromFuzzyISOString {
  
  ///
  /// Convert from any ISO-8601-like string (yyyy-mm-dd HH:MM:SS) to a DateTime object
  /// Valid formats
  /// Full date-time: e.g. 2023-11-15T17:53:26
  /// with optional millisecends (ignored): e.g. 2023-11-15T17:53:26.383Z
  /// with space rather than T: 2023-11-15 17:53:26
  /// without seconds: 2023-11-15T17:53 (rounded to the start of the minute)
  /// without minutes: 2023-11-15T17 (rounded to the top of the hour)
  /// without time: 2023-11-15 (rounded to the start of the day)
  /// without the month day: 2023-11 (rounded to the start of the month)
  /// Year only: 2023 (rounded to the year start)
  ///
  fn from_fuzzy_iso_string(dt_str: &str) -> Option<Self>  where Self: Sized;

}

/// Implement the FromFuzzyISOString trait for NaiveDateTime
/// Use the fuzzy-datetime crate for more robust date-time parsing and correction
impl FromFuzzyISOString for NaiveDateTime {
  /// construct a DateTime object from an exact or approximate ISO-8601-compatible string
  fn from_fuzzy_iso_string(dt_str: &str) -> Option<Self> {
    if let Ok(dt) = iso_fuzzy_string_to_datetime(dt_str) {
      Some(dt)
    } else {
      None
    }
  }
}