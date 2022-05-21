use crate::error::Error;
use crate::format::DATE_FMT;
use chrono::{Datelike, IsoWeek, NaiveDate};

pub fn parse_date(from: &str) -> Result<NaiveDate, Error> {
    NaiveDate::parse_from_str(from, DATE_FMT).map_err(Error::from)
}

pub fn parse_week(from: &str) -> Result<IsoWeek, Error> {
    let mut split = from.trim().split('-');

    let week = split
        .next()
        .ok_or_else(|| Error::from(format!("Could not parse '{}' as a week", from)))?
        .parse()?;

    if week == 0 || week > 52 {
        return Err(Error::from("Week number must be between 1 and 52"));
    }

    let year = split
        .next()
        .ok_or_else(|| Error::from(format!("Could not parse '{}' as a week", from)))?
        .parse()?;

    Ok(NaiveDate::from_isoywd(year, week, chrono::Weekday::Mon).iso_week())
}
