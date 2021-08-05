extern crate chrono;

mod error;
mod ledger;
mod session;

use crate::session::{DATETIME_FMT, TIME_FMT};
use chrono::Local;
use ledger::Ledger;
use main_error::MainError;
use std::str::FromStr;
use structopt::StructOpt;

fn main() -> Result<(), MainError> {
    let args = Args::from_args();

    let mut ledger = Ledger::load()?;

    match args.command {
        Command::In => {
            if let Err(e) = ledger.start() {
                println!("{}", e)
            }
        }
        Command::Out => {
            let duration = ledger.stop()?;
            println!(
                "Session ended ({:02}h{:02}m)",
                duration.num_hours(),
                duration.num_minutes() % 60,
            );
        }
        Command::Log => {
            ledger.history.iter().rev().for_each(|s| println!("{}", s));
            if let Some(curr) = ledger.current {
                println!("{}", curr.format(DATETIME_FMT))
            }
        }
        Command::Status => match ledger.current {
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
    }

    ledger.persist()?;

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// in | out | status | log
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Starts a new session
    In,
    /// Ends the current session
    Out,
    /// Status of current session
    Status,
    /// Show historical log
    Log,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            "log" => Ok(Self::Log),
            "status" => Ok(Self::Status),
            _ => Err(format!("{} is not a valid command", s)),
        }
    }
}
