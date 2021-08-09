extern crate chrono;

mod error;
mod ledger;
mod session;

use crate::error::Error;
use crate::ledger::Ledger;
use crate::session::{DATETIME_FMT, DATE_FMT, TIME_FMT};
use chrono::{Duration, Local, NaiveDate};
use main_error::MainError;
use std::cmp::Ordering;
use structopt::StructOpt;

fn main() -> Result<(), MainError> {
    let args = Args::from_args();

    let mut ledger = Ledger::load()?;

    match args.command {
        Command::Start => ledger.start()?,
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
            let date = day.unwrap_or_else(|| Local::now().naive_local().date());

            let mut total_duration = Duration::seconds(0);
            let mut overall_session = None;

            for session in ledger.history.iter().rev() {
                match session.start().date().cmp(&date) {
                    Ordering::Less => continue,
                    Ordering::Greater => break,
                    _ => {}
                }

                overall_session = Some(match overall_session {
                    None => (session.start().time(), session.end().time()),
                    Some((_, end)) => (session.start().time(), end),
                });

                total_duration = total_duration + session.duration();
            }

            match overall_session {
                Some((start, end)) => println!(
                    "Logged {:02}h{:02}m from {} to {}",
                    total_duration.num_hours(),
                    total_duration.num_minutes() % 60,
                    start,
                    end
                ),
                None => println!("No sessions were logged on {}", date.format(DATE_FMT)),
            }
        }
    }

    ledger.persist()?;

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
        /// The day in the following format: 30-Aug-1991
        #[structopt(short = "d", long = "day", parse(try_from_str = parse_date))]
        day: Option<NaiveDate>,
    },
}

fn parse_date(from: &str) -> Result<NaiveDate, Error> {
    NaiveDate::parse_from_str(from, DATE_FMT).map_err(Error::from)
}
