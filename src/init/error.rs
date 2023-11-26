use thiserror::Error;

use std::fmt;
use std::error;


#[derive(Error,Debug)]
pub enum WingmateInitError {
    #[error("invalid config search path")]
    InvalidConfigSearchPath,

    #[error("no service or cron found")]
    NoServiceOrCron,
    
    #[error("failed to spawn: {}", message)]
    SpawnError {
        #[source]
        source: std::io::Error,
        message: String,
    }
}

#[derive(Debug,Clone)]
pub struct CronSyntaxError(pub String);

impl fmt::Display for CronSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cron syntax error at: {}", self.0)
    }
}

impl error::Error for CronSyntaxError {}

#[derive(Debug,Clone)]
pub struct ShellNotFoundError(pub String);

impl fmt::Display for ShellNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "shell not found: {}", self.0)
    }
}

impl error::Error for ShellNotFoundError {}

#[derive(Debug,Clone)]
pub struct NoShellAvailableError;

impl fmt::Display for NoShellAvailableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no shell available")
    }
}

impl error::Error for NoShellAvailableError {}
