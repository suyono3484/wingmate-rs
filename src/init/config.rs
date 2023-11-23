use std::fs;
use std::path::PathBuf;
use std::io::{BufReader, BufRead};
use std::error as std_error;
use crate::init::error as wingmate_error;
use nix::unistd::{access, AccessFlags};
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub enum Command {
    ShellPrefixed(String),
    Direct(String)
}

#[derive(Debug)]
pub enum CronTimeFieldSpec {
    Any,
    Exact(u8),
    MultiOccurrence(Vec<u8>),
    Every(u8)
}

#[derive(Debug)]
pub struct Crontab {
    pub minute: CronTimeFieldSpec,
    pub hour: CronTimeFieldSpec,
    pub day_of_month: CronTimeFieldSpec,
    pub month: CronTimeFieldSpec,
    pub day_of_week: CronTimeFieldSpec,
    pub command: String,
}

#[derive(Debug)]
pub struct Config {
    pub services: Vec<Command>,
}

impl Config {
    pub fn find(search_path: Vec<String>) -> Result<Config, Box<dyn std_error::Error>> {
        if search_path.is_empty() {
            return Err(wingmate_error::InvalidConfigSearchPathError.into());
        }

        let mut svc_commands: Vec<Command> = Vec::new();
        for p in search_path {
            let mut buf = PathBuf::new();
            buf.push(p);
            if let Ok(m) = fs::metadata(buf.as_path()) {
                if m.is_dir() {
                    let svc = buf.join("services");
                    if let Ok(svc_iter) = fs::read_dir(svc.as_path()) {
                        for entry in svc_iter {
                            if let Ok(dirent) = entry {
                                let ep = dirent.path();
                                if let Ok(_) = access(ep.as_path(), AccessFlags::X_OK) {
                                    // execute directly
                                    svc_commands.push(Command::Direct(String::from(ep.as_path().to_string_lossy())));
                                } else {
                                    // call with shell
                                    svc_commands.push(Command::ShellPrefixed(String::from(ep.as_path().to_string_lossy())));
                                }
                            }
                        }
                    }

                    if let Ok(_crontab) = Self::read_crontab(&mut buf) {
                        //TODO: fix me! empty branch
                    }
                } else {
                    // reserve for future use; when we have a centralized config file
                }
            }
        }

        if svc_commands.is_empty() {
            return Err(wingmate_error::NoServiceOrCronFoundError.into());
        }

        let config = Config { services: svc_commands };
        Ok(config)
    }

    fn read_crontab(path: &mut PathBuf) -> Result<Vec<Crontab>, Box<dyn std_error::Error>> {
        lazy_static! {
            static ref CRON_REGEX: Regex = Regex::new(
                r"^\s*(?P<minute>\S+)\s+(?P<hour>\S+)\s+(?P<dom>\S+)\s+(?P<month>\S+)\s+(?P<dow>\S+)\s+(?P<command>\S.*\S)\s*$"
            ).unwrap();
        }

        let cron_path = path.join("crontab");
    
        {
            let f = fs::File::open(cron_path.as_path())?;
            for line in BufReader::new(f).lines() {
                if let Ok(l) = line {
                    let cap = CRON_REGEX.captures(&l).ok_or::<Box<dyn std_error::Error>>(wingmate_error::CronSyntaxError(String::from(&l)).into())?;
                    
                    let mut match_str = cap.name("minute").ok_or::<Box<dyn std_error::Error>>(wingmate_error::CronSyntaxError(String::from("cannot capture minute")).into())?;
                    let _minute = Self::to_cron_time_field_spec(&match_str)?;

                    match_str = cap.name("hour").ok_or::<Box<dyn std_error::Error>>(wingmate_error::CronSyntaxError(String::from("cannot capture hour")).into())?;
                    let _hour = Self::to_cron_time_field_spec(&match_str)?;
                }
            }
        }
        
    
        Err(wingmate_error::NoServiceOrCronFoundError.into())
    }

    fn to_cron_time_field_spec(match_str: &regex::Match) -> Result<CronTimeFieldSpec, Box<dyn std_error::Error>> {
        let field = match_str.as_str();

        if field == "*" {
            return Ok(CronTimeFieldSpec::Any);
        } else if field.starts_with("*/") {
            let every = field[2..].parse::<u8>()?;
            return Ok(CronTimeFieldSpec::Every(every));
        } else if field.contains(",") {
            let multi: Vec<&str> = field.split(",").collect();
            let mut multi_occurrence: Vec<u8> = Vec::new();

            for m in multi {
                let ur = m.parse::<u8>()?;
                multi_occurrence.push(ur);
            }

            return Ok(CronTimeFieldSpec::MultiOccurrence(multi_occurrence));
        } else {
            let n = field.parse::<u8>()?;
            return Ok(CronTimeFieldSpec::Exact(n));
        }
    }
}