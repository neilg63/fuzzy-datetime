
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