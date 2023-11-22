use std::fs;
use std::path::PathBuf;
use std::error as std_error;
use crate::init::error as wingmate_error;
use nix::unistd::{access, AccessFlags};

#[derive(Debug)]
pub enum Command {
    ShellPrefixed(String),
    Direct(String)
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

                    let cron = buf.join("cron");
                    if let Ok(cron_iter) = fs::read_dir(cron.as_path()) {
                        for entry in cron_iter {
                            if let Ok(_dirent) = entry {
                                // read the cron file
                            }
                        }
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
}