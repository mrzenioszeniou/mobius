mod error;
mod format;
mod ledger;
mod parse;
mod session;

use crate::error::Error;
use crate::format::{format_duration, DATETIME_FMT, DATE_FMT, TIME_FMT};
use crate::ledger::Ledger;
use crate::parse::{parse_date, parse_week};
use chrono::{Datelike, Duration, IsoWeek, Local, NaiveDate, Weekday};
use main_error::MainError;
use std::collections::BTreeMap;
use std::fmt::Display;
use structopt::StructOpt;

fn main() -> Result<(), MainError> {
    let args = Args::from_args();

    let mut ledger = Ledger::load()?;

    match args.command {
        Command::Start => {
            ledger.start()?;
            ledger.persist()?;
        }
        Command::Stop => {
            let duration = ledger.stop()?;
            ledger.persist()?;
            println!(
                "Session ended ({:02}h{:02}m)",
                duration.num_hours(),
                duration.num_minutes() % 60,
            );
        }
        Command::Show => match ledger.current {
            Some(start) => {
                let duration = Local::now().naive_local() - start;

                println!(
                    "Session running since {} ({:02}h{:02}m)",
                    start.format(TIME_FMT),
                    duration.num_hours(),
                    duration.num_minutes() % 60
                );
            }
            None => println!("No session running"),
        },
        Command::Log => {
            ledger.history.iter().for_each(|s| println!("{}", s));
            if let Some(curr) = ledger.current {
                println!("{}", curr.format(DATETIME_FMT))
            }
        }
        Command::Day { day } => {
            let now = Local::now().naive_local();

            let date = day.unwrap_or_else(|| now.date());

            let mut sessions_logs = vec![];
            let mut duration = Duration::zero();

            for session in ledger
                .history
                .iter()
                .filter(|s| s.start().date() == date || s.end().date() == date)
            {
                let session_duration = session.duration();

                sessions_logs.push(format!(
                    "{} -- {:02}h{:02}m --> {}",
                    session.start().format(TIME_FMT),
                    session_duration.num_hours(),
                    session_duration.num_minutes() % 60,
                    session.end().format(TIME_FMT)
                ));

                duration = duration + session_duration;
            }

            if date == now.date() {
                if let Some(current) = ledger.current {
                    let current_duration = now.signed_duration_since(current);
                    duration = duration + current_duration;
                    sessions_logs.push(format!(
                        "{} -- {:02}h{:02}m --> STILL RUNNING",
                        current.format(TIME_FMT),
                        current_duration.num_hours(),
                        current_duration.num_minutes() % 60,
                    ))
                }
            }

            if duration.is_zero() {
                println!("No sessions were logged on {}", date.format(DATE_FMT));
            } else {
                println!(
                    "Logged {:02}h{:02}m on {}",
                    duration.num_hours(),
                    duration.num_minutes() % 60,
                    date.format(DATE_FMT)
                );
                print_logs(sessions_logs.iter());
            }
        }
        Command::Week { week } => {
            let now = Local::now().naive_local();

            let week = week.unwrap_or_else(|| now.iso_week());

            let mut duration = Duration::zero();
            let mut days = BTreeMap::new();

            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Mon),
                Duration::zero(),
            );
            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Tue),
                Duration::zero(),
            );
            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Wed),
                Duration::zero(),
            );
            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Thu),
                Duration::zero(),
            );
            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Fri),
                Duration::zero(),
            );
            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Sat),
                Duration::zero(),
            );
            days.insert(
                NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Sun),
                Duration::zero(),
            );

            for session in ledger
                .history
                .iter()
                .filter(|s| s.start().iso_week() == week)
            {
                let session_duration = session.duration();

                let day = session.start().date();

                match days.get_mut(&day) {
                    Some(d) => {
                        *d = *d + session_duration;
                    }
                    None => return Err(Error::from("INTERNAL ERROR: Day not found").into()),
                }

                duration = duration + session_duration;
            }

            if week == now.iso_week() {
                if let Some(current) = ledger.current {
                    match days.get_mut(&now.date()) {
                        Some(d) => {
                            let current_duration = now.signed_duration_since(current);
                            duration = duration + current_duration;
                            *d = *d + current_duration;
                        }
                        None => return Err(Error::from("INTERNAL ERROR: Day not found").into()),
                    }
                }
            }

            if duration.is_zero() {
                println!(
                    "No sessions were logged on week {}-{}",
                    week.week(),
                    week.year()
                );
            } else {
                println!(
                    "Logged {:02}h{:02}m on week {}-{}",
                    duration.num_hours(),
                    duration.num_minutes() % 60,
                    week.week(),
                    week.year(),
                );

                let mut logs = vec![];

                for (day, day_duration) in days.into_iter() {
                    logs.push(format!(
                        "{}: {}",
                        day.format("%a %e-%b"),
                        format_duration(&day_duration)
                    ));
                }

                print_logs(logs.iter());
            }
        }
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Start a new session
    Start,
    /// Ends the current session
    Stop,
    /// Show status of current session
    Show,
    /// Show historical log
    Log,
    /// Show a day's summary
    Day {
        /// The day in the following format: 30-Aug-2021
        #[structopt(short = "d", long = "day", parse(try_from_str = parse_date))]
        day: Option<NaiveDate>,
    },
    /// Show a week's summary
    Week {
        /// The week and year in the following format: 35-2021
        #[structopt(short = "w", long = "week", parse(try_from_str = parse_week))]
        week: Option<IsoWeek>,
    },
}

fn print_logs<I, T>(mut logs: I)
where
    I: Iterator<Item = T>,
    T: Display,
{
    while let Some(log) = logs.next() {
        if logs.size_hint().0 > 0 {
            println!("├ {}", log);
        } else {
            println!("└ {}", log);
        }
    }
}
