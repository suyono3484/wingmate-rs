use tokio::task::JoinSet;
use tokio::process::{Command, Child};
use tokio_util::sync::CancellationToken;
use tokio::select;
use tokio::io::Result as tokio_result;
use tokio::time::sleep;
use std::time::Duration;
use std::process::ExitStatus;
use std::error;
use nix::sys::signal::{kill, Signal};
use nix::errno::Errno;
use nix::unistd::Pid;
use crate::init::config;
use crate::init::error::{NoShellAvailableError, WingmateInitError};


pub fn start_services(ts: &mut JoinSet<Result<(), Box<dyn error::Error + Send + Sync>>>, cfg: &config::Config, cancel: CancellationToken)
    -> Result<(), Box<dyn error::Error>> {

    for svc_ in cfg.get_service_iter() {
        let mut shell: String = String::new();
        if let config::Command::ShellPrefixed(_) = svc_ {
            shell = cfg.get_shell().ok_or::<Box<dyn error::Error>>(NoShellAvailableError.into())?;
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
                            Box::new(WingmateInitError::SpawnError { source: e, message: exp_str })
                        })?;
                    },
                    config::Command::ShellPrefixed(s) => {
                        let shell = shell.clone();
                        let exp_str = s.clone();
                        let exp_shell = shell.clone();
                        child = Command::new(shell).arg(s).spawn().map_err(|e| {
                            Box::new(WingmateInitError::SpawnError { source: e, message: format!("{} {}", exp_shell, exp_str) })
                        })?;
                    } 
                }

                select! {
                    _ = cancel.cancelled() => {
                        if let Some(id) = child.id() {
                            match kill(Pid::from_raw(id as i32), Some(Signal::SIGTERM)) {
                                Ok(_) => {
                                    select! {
                                        _ = sleep(Duration::from_secs(5)) => {
                                            child.kill().await.expect("failed to kill process");
                                        },
                                        result = child.wait() => {
                                            if let Err(e) = result_match(result) {
                                                return Err(e);
                                            }
                                            break 'autorestart;
                                        }
                                    }
                                },
                                Err(e) => {
                                    if e != Errno::ESRCH {
                                        return Err(e.into());
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
                            return Err(e);
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

fn result_match(result: tokio_result<ExitStatus>) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    if let Err(e) = result {
        if let Some(eos) = e.raw_os_error() {
            if eos != nix::Error::ECHILD as i32 {
                return Err(e.into());
            }
        } else {
            return Err(e.into());
        }
    }

    dbg!("starter: sleep exited");

    Ok(())
}