use time::error::IndeterminateOffset;
use tokio::task::JoinSet;
use tokio::process::{Command, Child};
use tokio_util::sync::CancellationToken;
use tokio::select;
use tokio::io::Result as tokio_result;
use tokio::time::{sleep, interval};
use std::env;
use std::time::Duration;
use std::process::ExitStatus;
use nix::sys::signal::{kill, Signal};
use nix::errno::Errno;
use nix::unistd::Pid;
use anyhow::{Context, anyhow};
use time::{OffsetDateTime, Duration as TimeDur, Weekday, UtcOffset};
use crate::init::config;
use crate::init::error::{WingmateInitError, CronConfigError};


const CRON_TRIGGER_WAIT_SECS: u64 = 20;
const ENV_UTC_OFFSET: &'static str = "WINGMATE_TIME_OFFSET";

pub fn start_services(ts: &mut JoinSet<Result<(), WingmateInitError>>, cfg: &config::Config, cancel: CancellationToken)
    -> Result<(), WingmateInitError> {

    for svc_ in cfg.get_service_iter() {
        let mut shell: String = String::new();
        if let config::Command::ShellPrefixed(_) = svc_ {
            shell = cfg.get_shell().ok_or::<WingmateInitError>(WingmateInitError::NoShellAvailable)?;
        }
        let svc = svc_.clone();
        let cancel = cancel.clone();
        ts.spawn(async move {
            'autorestart: loop {
                let mut child: Child;
                let svc = svc.clone();
                match svc {
                    config::Command::Direct(c) => {
                        let exp_str = c.clone();
                        child = Command::new(c).spawn().map_err(|e| {
                            WingmateInitError::SpawnError { source: e, message: exp_str }
                        })?;
                    },
                    config::Command::ShellPrefixed(s) => {
                        let shell = shell.clone();
                        let exp_str = s.clone();
                        let exp_shell = shell.clone();
                        child = Command::new(shell).arg(s).spawn().map_err(|e| {
                            WingmateInitError::SpawnError { source: e, message: format!("{} {}", exp_shell, exp_str) }
                        })?;
                    } 
                }

                select! {
                    _ = cancel.cancelled() => {
                        if let Some(id) = child.id() {
                            match kill(Pid::from_raw(id as i32), Some(Signal::SIGTERM)) {
                                Ok(_) => {
                                    select! {
                                        _ = sleep(Duration::from_secs(config::MAX_TERM_WAIT_TIME_SECS)) => {
                                            child.kill().await.expect("failed to kill process");
                                        },
                                        result = child.wait() => {
                                            if let Err(e) = result_match(result) {
                                                return Err(WingmateInitError::ChildExit { source: e });
                                            }
                                            break 'autorestart;
                                        }
                                    }
                                },
                                Err(e) => {
                                    if e != Errno::ESRCH {
                                        return Err(WingmateInitError::ChildNotFound);
                                    } else {
                                        break 'autorestart;
                                    }
                                }
                            }
                        } else {
                            break 'autorestart;
                        }
                    },
                    result = child.wait() => {
                        if let Err(e) = result_match(result) {
                            return Err(WingmateInitError::ChildExit { source: e });
                        }
                    },
                }
            }
            dbg!("starter: task completed");
            Ok(())
        });

    }
    dbg!("starter: spawning completed");

    Ok(())
}

fn result_match(result: tokio_result<ExitStatus>) -> Result<(), anyhow::Error> {
    if let Err(e) = result {
        if let Some(eos) = e.raw_os_error() {
            if eos != nix::Error::ECHILD as i32 {
                return Err(e).context("unexpected child exit status");
            }
        } else {
            return Err(e).context("unexpected child error");
        }
    }

    Ok(())
}

pub fn start_cron(ts: &mut JoinSet<Result<(), WingmateInitError>>, cfg: &config::Config, cancel: CancellationToken)
    -> Result<(), WingmateInitError> {

    dbg!("cron: starting");
    for c_ in cfg.get_cron_iter() {
        let cron = c_.clone();
        let in_loop_cancel = cancel.clone();
        dbg!("cron: item", c_);

        ts.spawn(async move {
            if cron.day_of_month != config::CronTimeFieldSpec::Any
                && cron.day_of_week != config::CronTimeFieldSpec::Any {
                    return Err(WingmateInitError::CronConfig { source: CronConfigError::ClashingConfig });
            }

            dbg!("cron: async task spawned");

            let cron = cron.clone();
            let mut cron_interval = interval(Duration::from_secs(CRON_TRIGGER_WAIT_SECS));
            let mut cron_procs: JoinSet<Result<(), WingmateInitError>> = JoinSet::new();
            let mut last_running: Option<OffsetDateTime> = None;
            'continuous: loop {
                let cron = cron.clone();
                let cron_proc_cancel = in_loop_cancel.clone();
                dbg!("cron: single: in loop", &cron.command);
                
                let mut flag = true;

                let tr: Result<OffsetDateTime, IndeterminateOffset>;
                if let Ok(offset) = env::var(ENV_UTC_OFFSET) {
                    if let Ok(i_off) =  offset.parse::<i8>() {
                        let utc_time = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(i_off, 0, 0).unwrap());
                        tr = Ok(utc_time);
                    } else {
                        tr = OffsetDateTime::now_local();
                    }
                } else {
                    tr = OffsetDateTime::now_local();
                }
                // let tr = OffsetDateTime::now_local();
                if let Ok(local_time) = tr {
                    dbg!("cron: current local time", &local_time);
                    if let Some(last) = last_running {
                        dbg!("cron: last runing instance", &last);
                        if local_time - last < TimeDur::minutes(1) {
                            flag = false;
                        } else {
                            flag = flag && cron.minute.is_match(local_time.minute()) &&
                                cron.hour.is_match(local_time.hour()) &&
                                cron.day_of_month.is_match(local_time.day()) &&
                                cron.day_of_week.is_match(weekday_map(local_time.weekday()));
                        }
                    } else {
                        flag = flag && cron.minute.is_match(local_time.minute()) &&
                            cron.hour.is_match(local_time.hour()) &&
                            cron.day_of_month.is_match(local_time.day()) &&
                            cron.day_of_week.is_match(weekday_map(local_time.weekday()));
                    }

                    if flag {
                        dbg!("cron: timing: hit: {}", &cron.command);
                        last_running = Some(local_time);
                        cron_procs.spawn(async move {
                            run_cron_command(cron.command.clone(), cron_proc_cancel).await
                        });
                    }    
                } else {
                    dbg!("cron: unexpected error");
                    if let Err(e) = tr {
                        dbg!(e);
                    }
                }

                if cron_procs.is_empty() {
                    select! {
                        _ = in_loop_cancel.cancelled() => {
                            break 'continuous;
                        },
                        _ = cron_interval.tick() => {},
                    }    
                } else {
                    'task: while !cron_procs.is_empty() {
                        select! {
                            opt_res = cron_procs.join_next() => {
                                if let Some(res) = opt_res {
                                    if let Err(e) = res {
                                        eprintln!("running cron got problem {:?}", e);
                                    }                                        
                                }
                            },
                            _ = in_loop_cancel.cancelled() => {
                                while let Some(res) = cron_procs.join_next().await {
                                    if let Err(e) = res {
                                        eprintln!("running cron got problem {:?}", e);
                                    }                                        
                                }
                                break 'continuous;
                            },
                            _ = cron_interval.tick() => {
                                break 'task;
                            },
                        }
                    }
                }
            }
            Ok(())
        });
    }
    Ok(())
}

fn weekday_map(wd: Weekday) -> u8 {
    match wd {
        Weekday::Sunday => 0,
        Weekday::Monday => 1,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 3,
        Weekday::Thursday => 4,
        Weekday::Friday => 5,
        Weekday::Saturday => 6
    }
}

async fn run_cron_command(command: String, cancel: CancellationToken) -> Result<(), WingmateInitError> {
    let mut args: Vec<&str> = Vec::new();
    for part in command.split(' ') {
        if part.len() > 0 {
            args.push(part);
        }
    }

    dbg!("cron: in running command");
    if args.is_empty() {
        return Err(WingmateInitError::Other { source: anyhow!("parsed as empty: {}", command) });
    }

    let cmd = args.swap_remove(0);
    let mut child: Child;
    if args.is_empty() {
        child = Command::new(cmd).spawn().map_err(|e| {
            WingmateInitError::SpawnError { source: e, message: command }
        })?;
    } else {
        child = Command::new(cmd).args(args.as_slice()).spawn().map_err(|e| {
            WingmateInitError::SpawnError { source: e, message: command }
        })?;
    }

    select! {
        _ = cancel.cancelled() => {
            if let Some(id) = child.id() {
                match kill(Pid::from_raw(id as i32), Some(Signal::SIGTERM)) {
                    Ok(_) => {
                        if let Err(e) = result_match(child.wait().await) {
                            return Err(WingmateInitError::ChildExit { source: e });
                        }            
                    },
                    Err(e) => {
                        match e {
                            Errno::ESRCH => {
                                return Err(WingmateInitError::ChildNotFound);
                            },
                            _ => {
                                return Err(WingmateInitError::FromNix { source: e });
                            }
                        }
                    }        
                }
            }
        },
        result = child.wait() => {
            if let Err(e) = result_match(result) {
                return Err(WingmateInitError::ChildExit { source: e });
            }
        }
    }

    Ok(())
}