extern crate chrono;

mod error;
mod ledger;
mod session;

use crate::error::Error;
use crate::ledger::Ledger;
use crate::session::{DATETIME_FMT, DATE_FMT, TIME_FMT};
use chrono::{Duration, Local, NaiveDate};
use main_error::MainError;
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
            let mut duration = Duration::seconds(0);

            let sessions = ledger
                .history
                .iter()
                .filter(|s| s.start().date() == date || s.end().date() == date)
                .collect::<Vec<_>>();

            for (i, session) in sessions.iter().enumerate() {
                let session_duration = session.duration();

                sessions_logs.push(format!(
                    "{} {} -- {:02}h{:02}m --> {}",
                    if i == sessions.len() - 1 && (date != now.date() || ledger.current.is_none()) {
                        "└"
                    } else {
                        "├"
                    },
                    session.start().format(TIME_FMT),
                    session_duration.num_hours(),
                    session_duration.num_minutes() % 60,
                    session.end().format(TIME_FMT)
                ));

                duration = duration + session.duration();
            }

            if date == now.date() {
                if let Some(current) = ledger.current {
                    let current_duration = now.signed_duration_since(current);
                    duration = duration + current_duration;
                    sessions_logs.push(format!(
                        "└ {} -- {:02}h{:02}m --> STILL RUNNING",
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
                sessions_logs.iter().for_each(|l| println!("{}", l));
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
        /// The day in the following format: 30-Aug-1991
        #[structopt(short = "d", long = "day", parse(try_from_str = parse_date))]
        day: Option<NaiveDate>,
    },
}

fn parse_date(from: &str) -> Result<NaiveDate, Error> {
    NaiveDate::parse_from_str(from, DATE_FMT).map_err(Error::from)
}
