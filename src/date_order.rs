use std::ops::Range;


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

  pub fn fixed_offsets(&self, length: u8) -> (Range<usize>, Range<usize>, Range<usize>) {
    let short_date = length < 8;
    match self {
      DateOrder::YMD => {
        if short_date {
          (0..2, 2..4, 4..6)
        } else {
          (0..4, 4..6, 6..8)
        }
      },
      DateOrder::DMY => {
        if short_date {
          (4..6, 2..4, 0..2)
        } else {
          (4..8, 2..4, 0..2)
        }
      },
      DateOrder::MDY => {
        if short_date {
          (4..6, 0..2, 2..4)
        } else {
          (4..8, 0..2, 2..4)
        }
      }
    }
  }

}


/// Options for parsing the date component of strings
pub struct DateOptions(pub DateOrder, pub Option<char>);

impl DateOptions {
  pub fn order(&self) -> DateOrder {
    self.0
  }

  pub fn splitter(&self) -> Option<char> {
    self.1
  }
}

impl Default for DateOptions {
  fn default() -> Self {
    DateOptions(DateOrder::YMD, Some('-'))
  }
}

/// instantiate options with three common orders + split character
/// e.g. DateOptions::dmy('.')
impl DateOptions {
  pub fn ymd(splitter: char) -> Self {
    DateOptions(DateOrder::YMD, Some(splitter))
  }

  pub fn ymd_fixed() -> Self {
    DateOptions(DateOrder::YMD, None)
  }

  pub fn dmy(splitter: char) -> Self {
    DateOptions(DateOrder::DMY, Some(splitter))
  }

  pub fn dmy_fixed() -> Self {
    DateOptions(DateOrder::DMY, None)
  }

  pub fn mdy(splitter: char) -> Self {
    DateOptions(DateOrder::MDY, Some(splitter))
  }
  
  pub fn mdy_fixed() -> Self {
    DateOptions(DateOrder::MDY, None)
  }
}

