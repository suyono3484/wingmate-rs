use std::fmt;
use std::error;

#[derive(Debug, Clone)]
pub struct InvalidConfigSearchPathError;

impl fmt::Display for InvalidConfigSearchPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid config search path")
    }
}

impl error::Error for InvalidConfigSearchPathError {}


#[derive(Debug,Clone)]
pub struct NoServiceOrCronFoundError;

impl fmt::Display for NoServiceOrCronFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no service or cron found")
    }
}

impl error::Error for NoServiceOrCronFoundError {}

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