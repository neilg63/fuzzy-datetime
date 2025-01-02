[![mirror](https://img.shields.io/badge/mirror-github-blue)](https://github.com/neilg63/fuzzy-datetime)
[![crates.io](https://img.shields.io/crates/v/fuzzy-datetime.svg)](https://crates.io/crates/fuzzy-datetime)
[![docs.rs](https://docs.rs/fuzzy-datetime/badge.svg)](https://docs.rs/fuzzy-datetime)

# Fuzzy DateTime

This library crate provides a set of functions to detect, normalise and complete date and date-time strings for bulk conversion to ISO-8601-compatible date or date-time strings. It can fill in missing time, day and month component if the year and other longer units are detected. 

In all cases, dates must come first and may be separated from the optional time component by a space or letter T.

## Core functions

#### `fuzzy_to_datetime(dt: &str, date_opts: Option<DateOptions>, time_separator: Option<char>) -> Result<NaiveDateTime, ParseError>`

This is most versatile function. If the last two arguments are set to None and None, the function will guess format, which in the case of m/d/Y and d/m/Y may be ambiguous. When processing large arrays of date-like strings, you may use the format detection functions first.

```rust

let date_opts = DateOptions::dmy('/');

if let Ok(sample_chrono_datetime) = fuzzy_to_datetime("09/11/2019 17:30:45", date_options, Some(':')) {
    let unix_time = date.timestamp();
    println!("Unix timestamp: {}", unix_time);
}
```


#### `fuzzy_to_datetime_string(dt: &str, date_opts: Option<DateOptions>, time_separator: Option<char>) -> Option<String>`

If you only need a normalised ISO-8601-compatible string for direct output or saving to a database, this function bypasses the chrono crate altogether:

```rust

let date_opts = DateOptions::dmy('/');

if let Some(sample_chrono_datetime_string) = fuzzy_to_datetime_string("09/11/2019 17:30:45", date_options, Some(':')) {
    println!("{}", sample_chrono_datetime_string); // should be 2019-11-09T17:30:45.000Z
}
```

#### `iso_fuzzy_string_to_datetime(dt: &str) -> Result<NaiveDateTime, ParsedError>`

This assumes a *Y-m-d* date order and is fully compatible with the original function used with the [julian day- converter](https://crates.io/crates/julian_day_converter) crate.

```rust

let date_opts = DateOptions::dmy('/');

if let Ok(sample_chrono_datetime) = iso_fuzzy_string_to_datetime("2019-11-09 17") {
    println!("{}", sample_chrono_datetime,to_rfc3339()); // should be 2019-11-09T17:00:45.000Z
}
```

#### `detect_date_format_from_generic_list(date_list: &[&str]) -> DateOptions` and `detect_date_format_from_generic_list<T, F>(date_list: &[T], extract_date: F) -> DateOption`

These functions serve to identify the correct format by analysing as many sample dates as necessary to surmise the date format so you can apply the correct date order and separator options before converting large data sets.

```rust
struct SpecialDay {
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
```
