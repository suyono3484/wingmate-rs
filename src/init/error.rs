use thiserror::Error;

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
    },

    #[error("parsing cron")]
    Cron {
        #[source]
        source: CronParseError,        
    },

    #[error("looking for shell")]
    FindShell {
        #[source]
        source: FindShellError,
    },

    #[error("child exited")]
    ChildExit {
        #[source]
        source: anyhow::Error,
    },

    #[error("cannot find the child process")]
    ChildNotFound,

    #[error("failed to setup signal handler")]
    Signal {
        #[source]
        source: std::io::Error,
    },

    #[error("no shell available")]
    NoShellAvailable,

    #[error("problem when join task")]
    Join {
        #[source]
        source: tokio::task::JoinError,
    },

    #[error("cron config")]
    CronConfig {
        #[source]
        source: CronConfigError,
    },

    #[error("from nix")]
    FromNix {
        #[source]
        source: nix::Error,
    },

    #[error("tripped over")]
    Other {
        #[source]
        source: anyhow::Error,
    }
}

#[derive(Error,Debug)]
pub enum CronConfigError {
    #[error("setting day of week and day of month at the same time will lead to unexpected behavior")]
    ClashingConfig,

    #[error("when setting time for higher order, the smallest (minute) muste be set")]
    MissingMinute,

    #[error("something went wrong")]
    Other {
        #[source]
        source: anyhow::Error,
    }
}

#[derive(Error,Debug)]
pub enum CronParseError {
    #[error("invalid cron syntax: {}", .0)]
    InvalidSyntax(String),
    
    #[error("cannot capture {} in \"{}\"", field_name, cron_line)]
    FieldMatch {
        cron_line: String,
        field_name: String,
    },

    #[error("failed to parse {} \"{}\" in \"{}\"", field_name, matched, cron_line)]
    Parse {
        #[source]
        source: anyhow::Error,
        cron_line: String,
        matched: String,
        field_name: String,
    }
}

#[derive(Error,Debug)]
pub enum FindShellError {
    #[error("shell not found")]
    ShellNotFound,

    #[error("when finding shell")]
    Other {
        #[source]
        source: anyhow::Error
    }
}
