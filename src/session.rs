use crate::format::{DATETIME_FMT, TIME_FMT};
use chrono::{Duration, Local, NaiveDateTime};
use std::str::FromStr;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Session {
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Session {
    pub fn with_start(start: NaiveDateTime) -> Self {
        let end = Local::now().naive_local();

        assert!(end > start);

        Self { start, end }
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }

    pub fn start(&self) -> &NaiveDateTime {
        &self.start
    }

    pub fn end(&self) -> &NaiveDateTime {
        &self.end
    }
}

impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.start.format(DATETIME_FMT);
        let end = self.end.format(TIME_FMT);
        let duration = self.duration();
        let hours = duration.num_hours();
        let mins = duration.num_minutes() % 60;
        write!(f, "{} -- {:02}h{:02}m --> {}", start, hours, mins, end)
    }
}

impl FromStr for Session {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('-');

        let start_str = split
            .next()
            .ok_or(format!("Could not parse '{}' as a session", s))?;
        let start =
            NaiveDateTime::parse_from_str(start_str, DATETIME_FMT).map_err(|e| e.to_string())?;

        let end_str = split
            .next()
            .ok_or(format!("Could not parse '{}' as a session", s))?;
        let end =
            NaiveDateTime::parse_from_str(end_str, DATETIME_FMT).map_err(|e| e.to_string())?;

        assert!(split.next().is_none());

        Ok(Self { start, end })
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn ord() {
        let stamps = vec![
            Session {
                start: NaiveDateTime::parse_from_str("1991/08/30 01:43:12", DATETIME_FMT).unwrap(),
                end: NaiveDateTime::parse_from_str("1991/08/30 02:43:12", DATETIME_FMT).unwrap(),
            },
            Session {
                start: NaiveDateTime::parse_from_str("1991/08/30 13:43:12", DATETIME_FMT).unwrap(),
                end: NaiveDateTime::parse_from_str("1991/08/30 13:57:12", DATETIME_FMT).unwrap(),
            },
            Session {
                start: NaiveDateTime::parse_from_str("1991/08/29 13:43:13", DATETIME_FMT).unwrap(),
                end: NaiveDateTime::parse_from_str("1991/08/29 13:43:11", DATETIME_FMT).unwrap(),
            },
        ];

        let ordered: BTreeSet<&Session> = stamps.iter().collect();

        let mut iterator = ordered.into_iter();

        assert_eq!(iterator.next(), Some(&stamps[2]));
        assert_eq!(iterator.next(), Some(&stamps[0]));
        assert_eq!(iterator.next(), Some(&stamps[1]));
    }
}
