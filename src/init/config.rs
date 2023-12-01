use std::fs;
use std::env;
use std::path::PathBuf;
use std::io::{BufReader, BufRead};
use crate::init::error as wingmate_error;
use anyhow::anyhow;
use nix::unistd::{access, AccessFlags};
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::Context;

pub const MAX_TERM_WAIT_TIME_SECS: u64 = 5;

const CRON_REGEX_STR: &'static str = r"^\s*(?P<minute>\S+)\s+(?P<hour>\S+)\s+(?P<dom>\S+)\s+(?P<month>\S+)\s+(?P<dow>\S+)\s+(?P<command>\S.*\S)\s*$";
const MINUTE: &'static str = "minute";
const HOUR: &'static str = "hour";
const DAY_OF_MONTH_ABBRV: &'static str = "dom";
const DAY_OF_MONTH: &'static str = "day of month";
const MONTH: &'static str = "month";
const DAY_OF_WEEK_ABBRV: &'static str = "dow";
const DAY_OF_WEEK: &'static str = "day of week";
const COMMAND: &'static str = "command";
const WINGMATE_SHELL_ENV: &'static str = "WINGMATE_SHELL";


#[derive(Debug)]
pub enum Command {
    ShellPrefixed(String),
    Direct(String)
}

#[derive(Debug)]
pub enum CronTimeFieldSpec {
    Any,
    Exact(u8),
    MultiOccurrence(Vec<u8>)
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
    pub cron: Vec<Crontab>,
    shell_path: Option<String>,
}

impl Config {
    pub fn find(search_path: Vec<String>) -> Result<Config, wingmate_error::WingmateInitError> {
        if search_path.is_empty() {
            return Err(wingmate_error::WingmateInitError::InvalidConfigSearchPath.into());
        }

        let mut svc_commands: Vec<Command> = Vec::new();
        let mut cron : Vec<Crontab> = Vec::new();
        'search: for p in search_path {
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
                                    svc_commands.push(Command::Direct(String::from(ep.to_string_lossy())));
                                } else {
                                    // call with shell
                                    svc_commands.push(Command::ShellPrefixed(String::from(ep.to_string_lossy())));
                                }
                            }
                        }
                    }

                    cron = Self::read_crontab(&mut buf).map_err(|e| { wingmate_error::WingmateInitError::Cron { source: e }})?;

                    //TODO: need to include cron in the condition
                    if !svc_commands.is_empty() || !cron.is_empty() {
                        break 'search;
                    }
                } else {
                    // reserve for future use; when we have a centralized config file
                }
            }
        }

        if svc_commands.is_empty() && cron.is_empty() {
            return Err(wingmate_error::WingmateInitError::NoServiceOrCron.into());
        }

        let mut config = Config { 
            services: svc_commands,
            cron,
            shell_path: None,
        };
        config.find_shell().map_err(|e| { wingmate_error::WingmateInitError::FindShell { source: e } })?;

        Ok(config)
    }

    fn read_crontab(path: &mut PathBuf) -> Result<Vec<Crontab>, wingmate_error::CronParseError> {
        lazy_static! {
            static ref CRON_REGEX: Regex = Regex::new(CRON_REGEX_STR).unwrap();
        }

        let cron_path = path.join("crontab");
        let mut ret_vec: Vec<Crontab> = Vec::new();

        if let Ok(f) = fs::File::open(cron_path.as_path()) {
            for line in BufReader::new(f).lines() {
                if let Ok(l) = line {
                    let cap = CRON_REGEX.captures(&l).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::InvalidSyntax(String::from(&l))
                    )?;
                    
                    let mut match_str = cap.name(MINUTE).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::FieldMatch { cron_line: String::from(&l), field_name: String::from(MINUTE) }
                    )?;
                    let minute = Self::to_cron_time_field_spec(&match_str, 60u8).map_err(|e| { 
                        wingmate_error::CronParseError::Parse {
                            source: e,
                            cron_line: String::from(&l),
                            matched: String::from(match_str.as_str()),
                            field_name: String::from(MINUTE)
                        }
                    })?;
    
                    match_str = cap.name(HOUR).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::FieldMatch { cron_line: String::from(&l), field_name: String::from(HOUR) }
                    )?;
                    let hour = Self::to_cron_time_field_spec(&match_str, 24u8).map_err(|e| { 
                        wingmate_error::CronParseError::Parse {
                            source: e,
                            cron_line: String::from(&l),
                            matched: String::from(match_str.as_str()),
                            field_name: String::from(HOUR)
                        }
                    })?;
    
                    match_str = cap.name(DAY_OF_MONTH_ABBRV).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::FieldMatch { cron_line: String::from(&l), field_name: String::from(DAY_OF_MONTH) }
                    )?;
                    let dom = Self::to_cron_time_field_spec(&match_str, 31u8).map_err(|e| {
                        wingmate_error::CronParseError::Parse {
                            source: e,
                            cron_line: String::from(&l),
                            matched: String::from(match_str.as_str()),
                            field_name: String::from(DAY_OF_MONTH)
                        }
                    })?;
    
                    match_str = cap.name(MONTH).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::FieldMatch { cron_line: String::from(&l), field_name: String::from(MONTH) }
                    )?;
                    let month = Self::to_cron_time_field_spec(&match_str, 12u8).map_err(|e| {
                        wingmate_error::CronParseError::Parse {
                            source: e,
                            cron_line: String::from(&l),
                            matched: String::from(match_str.as_str()),
                            field_name: String::from(MONTH)
                        }
                    })?;
    
                    match_str = cap.name(DAY_OF_WEEK_ABBRV).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::FieldMatch { cron_line: String::from(&l), field_name: String::from(DAY_OF_WEEK) }
                    )?;
                    let dow = Self::to_cron_time_field_spec(&match_str, 7u8).map_err(|e| {
                        wingmate_error::CronParseError::Parse {
                            source: e,
                            cron_line: String::from(&l),
                            matched: String::from(match_str.as_str()),
                            field_name: String::from(DAY_OF_WEEK)
                        }
                    })?;
    
                    match_str = cap.name(COMMAND).ok_or::<wingmate_error::CronParseError>(
                        wingmate_error::CronParseError::FieldMatch { cron_line: String::from(&l), field_name: String::from(COMMAND) }
                    )?;
    
                    ret_vec.push(Crontab {
                        minute,
                        hour,
                        day_of_month: dom,
                        month,
                        day_of_week: dow,
                        command: String::from(match_str.as_str())
                    })
                }
            }
        }

        Ok(ret_vec)
    }

    fn to_cron_time_field_spec(match_str: &regex::Match, max: u8) -> Result<CronTimeFieldSpec, anyhow::Error> {
        let field = match_str.as_str();

        if field == "*" {
            return Ok(CronTimeFieldSpec::Any);
        } else if field.starts_with("*/") {
            let every = field[2..].parse::<u8>().context("parsing on field matching \"every\" pattern")?;
            if every >= max {
                return Err(anyhow!("invalid value {}", every));
            }
            let mut next_value = every;
            let mut multi: Vec<u8> = Vec::new();
            while next_value < max {
                multi.push(next_value);
                next_value += every;
            }
            return Ok(CronTimeFieldSpec::MultiOccurrence(multi));
        } else if field.contains(",") {
            let multi: Vec<&str> = field.split(",").collect();
            let mut multi_occurrence: Vec<u8> = Vec::new();

            for m in multi {
                let ur = m.parse::<u8>().context("parsing on field matching \"multi occurrence\" pattern")?;
                if ur >= max {
                    return Err(anyhow!("invalid value {}", field));
                }
                multi_occurrence.push(ur);
            }

            return Ok(CronTimeFieldSpec::MultiOccurrence(multi_occurrence));
        } else {
            let n = field.parse::<u8>().context("parsing on field matching \"exact\" pattern")?;
            if n >= max {
                return Err(anyhow!("invalid value {}", n));
            }
            return Ok(CronTimeFieldSpec::Exact(n));
        }
    }

    fn find_shell(&mut self) -> Result<(), wingmate_error::FindShellError> {

        let  shell: String;
        match env::var(WINGMATE_SHELL_ENV) {
            Ok(sh) => {
                shell = sh;
            },
            Err(e) => {
                match e {
                    env::VarError::NotPresent => {
                        shell = String::from("sh");
                    },
                    env::VarError::NotUnicode(_) => {
                        return Err(e).context(format!("reading {} env var", WINGMATE_SHELL_ENV))
                            .map_err(|e| { wingmate_error::FindShellError::Other { source: e } })
                    }
                }
            }
        }

        let env_path  = env::var("PATH").context("getting PATH env variable")
            .map_err(|e| { wingmate_error::FindShellError::Other { source: e } })?;
        let vec_path: Vec<&str> = env_path.split(':').collect();

        for p in vec_path {
            let mut search_path = PathBuf::new();
            search_path.push(p);

            let shell_path = search_path.join(&shell);
            if let Ok(_) = fs::metadata(shell_path.as_path()) {
                self.shell_path = Some(String::from(shell_path.to_string_lossy()));
                return Ok(());
            }
        }

        Err(wingmate_error::FindShellError::ShellNotFound)
    }

    pub fn get_service_iter(&self) -> std::slice::Iter<Command> {
        self.services.iter()
    }

    pub fn get_cron_iter(&self) -> std::slice::Iter<Crontab> {
        self.cron.iter()
    }

    pub fn get_shell(&self) -> Option<String> {
        if let Some(shell) = &self.shell_path {
            return Some(shell.clone());
        }
        None
    }
}

impl Clone for Command {
    fn clone(&self) -> Self {
        match self {
            Command::Direct(d) => Command::Direct(String::from(d)),
            Command::ShellPrefixed(s) => Command::ShellPrefixed(String::from(s))
        }
    }
}

impl Clone for Crontab {
    fn clone(&self) -> Self {
        Self { 
            minute: self.minute.clone(),
            hour: self.hour.clone(),
            day_of_month: self.day_of_month.clone(),
            month: self.month.clone(),
            day_of_week: self.day_of_week.clone(),
            command: self.command.clone()
        }
    }
}

impl CronTimeFieldSpec {
    pub fn is_match(&self, current: u8) -> bool {
        match self {
            Self::Any => { return true; },
            Self::Exact(x) => { return *x == current; },
            Self::MultiOccurrence(v) => {
                for i in v {
                    if *i == current {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Clone for CronTimeFieldSpec {
    fn clone(&self) -> Self {
        match self {
            Self::Any => Self::Any,
            Self::Exact(x) => Self::Exact(*x),
            Self::MultiOccurrence(x) => {
                Self::MultiOccurrence(x.clone())
            },
        }
    }
}

struct CronFieldCmpHelper<'a>(u8, u8, Option<&'a Vec<u8>>);
impl PartialEq for CronTimeFieldSpec {
    fn eq(&self, other: &Self) -> bool {
        let lhs: CronFieldCmpHelper;
        let rhs: CronFieldCmpHelper;
        match self {
            CronTimeFieldSpec::Any => { lhs = CronFieldCmpHelper(0, 0, None); }
            CronTimeFieldSpec::Exact(x) => { lhs = CronFieldCmpHelper(1, *x, None); }
            CronTimeFieldSpec::MultiOccurrence(v) => { lhs = CronFieldCmpHelper(1, 0, Some(v)); }
        }

        match other {
            CronTimeFieldSpec::Any => { rhs = CronFieldCmpHelper(0, 0, None); }
            CronTimeFieldSpec::Exact(x) => { rhs = CronFieldCmpHelper(1, *x, None); }
            CronTimeFieldSpec::MultiOccurrence(v) => { rhs = CronFieldCmpHelper(2, 0, Some(v)); }
        }

        if lhs.0 == rhs.0 {
            if lhs.0 == 3u8 {
                if let Some(lv) = lhs.2 {
                    if let Some(rv) = rhs.2 {
                        if lv.len() != rv.len() {
                            return false;
                        }

                        let mut l_iter = lv.iter();
                        let mut r_iter = rv.iter();
                        'item: loop {
                            if let Some(liv) = l_iter.next() {
                                if let Some(riv) = r_iter.next() {
                                    if *liv != *riv {
                                        return false;
                                    }
                                } else {
                                    break 'item;
                                }
                            } else {
                                break 'item;
                            }
                        }
                        return true;
                    }
                }
            } else {
                return lhs.1 == rhs.1;
            }
        }

        false
    }
}