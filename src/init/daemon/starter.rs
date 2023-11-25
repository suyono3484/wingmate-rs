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
use crate::init::error::NoShellAvailableError;


pub fn start_services(ts: &mut JoinSet<Result<(), Box<dyn error::Error + Send + Sync>>>, cfg: &config::Config, cancel: CancellationToken)
    -> Result<(), Box<dyn error::Error>> {

    for svc_ in cfg.get_service_iter() {
        let shell: String = cfg.get_shell().ok_or::<Box<dyn error::Error>>(NoShellAvailableError.into())?;
        let svc = svc_.clone();
        // if let config::Command::ShellPrefixed(_) = svc {
        //     shell = cfg.get_shell().ok_or::<Box<dyn error::Error>>(NoShellAvailableError.into())?;
        // }
        let cancel = cancel.clone();
        ts.spawn(async move {
            'autorestart: loop {
                let mut child: Child;
                let shell = shell.clone();
                let svc = svc.clone();
                match svc {
                    config::Command::Direct(c) => {
                        child = Command::new(c).spawn().expect("change me");
                    },
                    config::Command::ShellPrefixed(s) => {
                        child = Command::new(shell).arg(s).spawn().expect("change me");
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
            println!("starter: task completed");
            Ok(())
        });

    }
    println!("starter: spawning completed");

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

    //TODO: remove me! this is for debug + tracing purpose
    println!("starter: sleep exited");

    Ok(())
}