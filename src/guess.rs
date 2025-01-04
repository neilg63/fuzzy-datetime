use simple_string_patterns::{CharGroupMatch, StripCharacters, ToSegments};
use crate::{converters::digits_to_date_parts, date_order::{DateOptions, DateOrder}};


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

/// Detect the date order and splitter from a date string
pub fn surmise_date_order_and_splitter(date_str: &str) -> DateOptions {
    let splitter = guess_date_splitter(date_str);
    DateOptions(surmise_date_order(date_str, splitter), splitter)
  }
  
  pub fn surmise_date_order(date_str: &str, splitter: Option<char>) -> DateOrder {
    guess_date_order(date_str, splitter).to_order()
  }
  
  /// Guess the date order from a date string
  /// assuming YMD, DMY or MDY as the likely order
  /// but catering for ambiguous cases or invalid dates
  /// Date strings with fewer than 3 parts must include the year
  pub fn guess_date_order(date_str: &str, splitter: Option< char>) -> DateOrderGuess {
    let str_parts = if let Some(split_char) = splitter {
      date_str.to_parts(&split_char.to_string())
    } else {
      let parts = digits_to_date_parts(date_str, DateOrder::DMY);
      if parts.len() < 3 {
        return DateOrderGuess::NonDate;
      }
      let first = parts[0].parse::<u16>().unwrap_or(0);
      let second = parts[1].parse::<u16>().unwrap_or(0);
      let third = parts[2].parse::<u16>().unwrap_or(0);
      let first_4 = vec![parts[0].as_str(), parts[1].as_str()].join("").parse::<u16>().unwrap_or(0);
      if first < 12 && first > 0 && second > 0 && third >= 1800 {
        return if second > 12 {
          DateOrderGuess::MonthFirst
        } else {
          DateOrderGuess::DayOrMonthFirst
        };
      } else if second < 12 && second > 0 && first > 0 && first < 32 && third > 1800 {
        return DateOrderGuess::DayFirst;
      } else {
        return DateOrderGuess::YearFirst;
      }
    };
    let date_parts: Vec<String> = str_parts.into_iter().filter(|n| n.is_digits_only()).collect();
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

  pub(crate) fn guess_date_splitter(date_str: &str) -> Option<char> {
    if let Some(splitter) = guess_unit_splitter(date_str, &['-', '/', '.']) {
      Some(splitter)
    } else {
      if date_str.contains("T") {
        Some('T')
      } else {
        if date_str.strip_non_digits().len() >= 8 {
          None
        } else {
          Some(':')
        }
      }
    }
  }
  
  pub(crate) fn guess_time_splitter(time_str: &str) -> Option<char> {  
    // If no valid separator found, default to '-'
    if let Some(splitter) = guess_unit_splitter(time_str, &[':', '.']) {
      Some(splitter)
    } else {
      if time_str.strip_non_digits().len() >= 4 {
        None
      } else {
        Some(':')
      }
    }
  }
  
  pub(crate) fn guess_unit_splitter(unit_str: &str, separators: &[char]) -> Option<char> {
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