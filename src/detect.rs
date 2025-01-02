use crate::{guess::{guess_date_order, guess_date_splitter, DateOrderGuess}, DateOptions, DateOrder};

/// This assumes all date strings are in the same format
/// and deduces through elimination
pub fn detect_date_format_from_list(date_list: &[&str]) -> DateOptions {
    detect_date_format_from_generic_list(date_list, |&x| Some(x.to_string()))
  }
  
  /// This assumes all objects in the list have a date string
  /// and deduces through elimination
  pub fn detect_date_format_from_generic_list<T, F>(date_list: &[T], extract_date: F) -> DateOptions 
  where 
      F: Fn(&T) -> Option<String>,
  {
    let mut order = DateOrder::YMD;
    let mut splitter = '-';
  
    for row in date_list {
      if let Some(dt_str) = extract_date(row) {
        if dt_str.trim().is_empty() {
          continue; // Skip empty string
        }
        let split_char = guess_date_splitter(&dt_str);
        let guess = guess_date_order(&dt_str, split_char);
        match guess {
            DateOrderGuess::YearFirst => {
                order = DateOrder::YMD;
                splitter = split_char;
                return DateOptions(order, splitter);
            },
            DateOrderGuess::DayFirst => {
                order = DateOrder::DMY;
                splitter = split_char;
                return DateOptions(order, splitter);
            },
            DateOrderGuess::MonthFirst => {
                order = DateOrder::MDY;
                splitter = split_char;
                return DateOptions(order, splitter);
            },
            _ => continue, // NonDate or ambiguous format, keep looking
        }
      }
    }
    // If we didn't find a conclusive format, we might want to handle this case better
    DateOptions(order, splitter)
  }