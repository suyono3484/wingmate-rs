use tokio::task::JoinSet;
use tokio::process::Command;
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


pub fn start_process(ts: &mut JoinSet<Result<(), Box<dyn error::Error + Send + Sync>>>, cancel: CancellationToken) {
    for _j in 0..5 {
        let cancel = cancel.clone();
        ts.spawn(async move {
            'autorestart: loop {
                let mut child = Command::new("sleep").arg("1")
                    .spawn()
                    .expect("failed to spawn");


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