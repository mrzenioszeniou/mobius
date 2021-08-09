use crate::error::Error;
use crate::session::Session;
use crate::session::DATETIME_FMT;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDateTime;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct Ledger {
    pub history: BTreeSet<Session>,
    pub current: Option<NaiveDateTime>,
    path: PathBuf,
}

impl Ledger {
    pub fn load() -> Result<Self, Error> {
        let mut path = home::home_dir().ok_or(Error::HomeDirectoryNotFound)?;

        path.push(".mobius");

        assert!(std::fs::create_dir_all(&path).is_ok());

        path.push("ledger");

        let mut current = None;
        let mut history = BTreeSet::new();

        if path.as_path().is_file() {
            let mut file = OpenOptions::new().read(true).open(&path)?;

            let mut content = String::new();

            file.read_to_string(&mut content)?;

            for line in content.trim().split('\n').filter(|s| !s.is_empty()) {
                let session = match Session::from_str(line) {
                    Ok(s) => s,
                    Err(_) => {
                        current = Some(NaiveDateTime::parse_from_str(line, DATETIME_FMT)?);
                        continue;
                    }
                };

                history.insert(session);
            }
        }

        Ok(Self {
            history,
            current,
            path,
        })
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if let Some(start) = self.current {
            return Err(Error::SessionRunning(start));
        }

        self.current = Some(Local::now().naive_local());
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Duration, Error> {
        match self.current.take() {
            None => Err(Error::NoSessionRunning),
            Some(start) => {
                self.history.insert(Session::with_start(start));
                Ok(Local::now().naive_local() - start)
            }
        }
    }

    pub fn persist(&self) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        for session in self.history.iter() {
            // println!("{}", session);

            let line = format!(
                "{}-{}\n",
                session.start().format(DATETIME_FMT),
                session.end().format(DATETIME_FMT)
            );

            file.write_all(line.as_bytes())?;
        }

        if let Some(current) = self.current {
            file.write_all(format!("{}\n", current.format(DATETIME_FMT)).as_bytes())?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Ledger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HISTORY:")?;
        for session in self.history.iter() {
            writeln!(f, "  {}", session)?;
        }

        match self.current {
            Some(start) => write!(f, "SESSION ACTIVE SINCE {}", start),
            None => write!(f, "NO ACTIVE SESSION"),
        }
    }
}
