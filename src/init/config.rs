use std::fs;
use std::env;
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
    pub cron: Vec<Crontab>,
    shell_path: Option<String>,
}

impl Config {
    pub fn find(search_path: Vec<String>) -> Result<Config, Box<dyn std_error::Error>> {
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

                    cron = Self::read_crontab(&mut buf)?;

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
        config.find_shell()?;

        Ok(config)
    }

    fn read_crontab(path: &mut PathBuf) -> Result<Vec<Crontab>, Box<dyn std_error::Error>> {
        lazy_static! {
            static ref CRON_REGEX: Regex = Regex::new(
                r"^\s*(?P<minute>\S+)\s+(?P<hour>\S+)\s+(?P<dom>\S+)\s+(?P<month>\S+)\s+(?P<dow>\S+)\s+(?P<command>\S.*\S)\s*$"
            ).unwrap();
        }

        let cron_path = path.join("crontab");
        let mut ret_vec: Vec<Crontab> = Vec::new();

        if let Ok(f) = fs::File::open(cron_path.as_path()) {
            for line in BufReader::new(f).lines() {
                if let Ok(l) = line {
                    let cap = CRON_REGEX.captures(&l).ok_or::<Box<dyn std_error::Error>>(wingmate_error::CronSyntaxError(String::from(&l)).into())?;
                    
                    let mut match_str = cap.name("minute").ok_or::<Box<dyn std_error::Error>>(
                        wingmate_error::CronSyntaxError(format!("cannot capture minute in \"{}\"", &l)).into()
                    )?;
                    let minute = Self::to_cron_time_field_spec(&match_str).map_err(|e| { 
                        Box::new(wingmate_error::CronSyntaxError(format!("failed to parse minute \"{}\" in \"{}\": {}", &match_str.as_str(), &l, e)))
                    })?;
    
                    match_str = cap.name("hour").ok_or::<Box<dyn std_error::Error>>(
                        wingmate_error::CronSyntaxError(format!("cannot capture hour in \"{}\"", &l)).into()
                    )?;
                    let hour = Self::to_cron_time_field_spec(&match_str).map_err(|e| { 
                        Box::new(wingmate_error::CronSyntaxError(format!("failed to parse hour \"{}\" in \"{}\": {}", &match_str.as_str(), &l, e)))
                    })?;
    
                    match_str = cap.name("dom").ok_or::<Box<dyn std_error::Error>>(
                        wingmate_error::CronSyntaxError(format!("cannot capture day of month in \"{}\"", &l)).into()
                    )?;
                    let dom = Self::to_cron_time_field_spec(&match_str).map_err(|e| {
                        Box::new(wingmate_error::CronSyntaxError(format!("failed to parse day of month \"{}\" in \"{}\": {}", &match_str.as_str(), &l, e)))
                    })?;
    
                    match_str = cap.name("month").ok_or::<Box<dyn std_error::Error>>(
                        wingmate_error::CronSyntaxError(format!("cannot capture month in \"{}\"", &l)).into()
                    )?;
                    let month = Self::to_cron_time_field_spec(&match_str).map_err(|e| {
                        Box::new(wingmate_error::CronSyntaxError(format!("failed to parse month \"{}\" in \"{}\": {}", &match_str.as_str(), &l, e)))
                    })?;
    
                    match_str = cap.name("dow").ok_or::<Box<dyn std_error::Error>>(
                        wingmate_error::CronSyntaxError(format!("cannot capture day of week in \"{}\"", &l)).into()
                    )?;
                    let dow = Self::to_cron_time_field_spec(&match_str).map_err(|e| {
                        Box::new(wingmate_error::CronSyntaxError(format!("failed to parse day of week \"{}\" in \"{}\": {}", &match_str.as_str(), &l, e)))
                    })?;
    
                    match_str = cap.name("command").ok_or::<Box<dyn std_error::Error>>(
                        wingmate_error::CronSyntaxError(format!("cannot capture command in \"{}\"", &l)).into()
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

    fn find_shell(&mut self) -> Result<(), Box<dyn std_error::Error>> {

        let  shell: String;
        match env::var("WINGMATE_SHELL") {
            Ok(sh) => {
                shell = sh;
            },
            Err(e) => {
                match e {
                    env::VarError::NotPresent => {
                        shell = String::from("sh");
                    },
                    env::VarError::NotUnicode(_) => {
                        return Err(e.into());
                    }
                }
            }
        }

        let env_path  = env::var("PATH")?;
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

        Err(wingmate_error::ShellNotFoundError(shell).into())
    }

    pub fn get_service_iter(&self) -> std::slice::Iter<Command> {
        self.services.iter()
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
