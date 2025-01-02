use simple_string_patterns::CharGroupMatch;
use crate::date_order::{DateOptions, DateOrder};


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
  
  pub fn surmise_date_order(date_str: &str, splitter: char) -> DateOrder {
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

  pub(crate) fn guess_date_splitter(date_str: &str) -> char {
    guess_unit_splitter(date_str, &['.', '·', '-', '/']).unwrap_or('-')
  }
  
  pub(crate) fn guess_time_splitter(time_str: &str) -> char {  
    // If no valid separator found, default to '-'
    guess_unit_splitter(time_str, &[':', '.']).unwrap_or(':')
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