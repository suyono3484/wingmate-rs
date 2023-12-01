mod sighandler;
mod waiter;
mod starter;
mod constants;

use tokio::{select, pin};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use std::sync::{Arc, Mutex};
use std::time::{Duration,Instant};
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use crate::init::config;
use crate::init::error as wmerr;
use crate::init::error::WingmateInitError;

pub async fn start(cfg: config::Config) -> Result<(), WingmateInitError> {
    let sync_flag = Arc::new(Mutex::new(false));
    let sig_sync_flag = sync_flag.clone();

    let sighandler_cancel = CancellationToken::new();
    let waiter_cancel_sighandler = sighandler_cancel.clone();
    let signal_pump_stop = sighandler_cancel.clone();

    let cancel = CancellationToken::new();
    let starter_service_cancel = cancel.clone();
    let starter_cron_cancel = cancel.clone();
    let signal_pump_start = cancel.clone();

    let mut set: JoinSet<Result<(), wmerr::WingmateInitError>> = JoinSet::new();
    set.spawn(async move {
        signal_pump(signal_pump_start, signal_pump_stop).await
    });

    set.spawn(async move {
        sighandler::sighandler(sig_sync_flag, cancel, sighandler_cancel).await
    });

    starter::start_services(&mut set, &cfg, starter_service_cancel)?;
    starter::start_cron(&mut set, &cfg, starter_cron_cancel)?;

    //TODO: spawn_blocking for waiter
    set.spawn_blocking(move || {
        waiter::wait_all(sync_flag, waiter_cancel_sighandler);
        Ok(())
    });

    while let Some(res) = set.join_next().await {
        match res {
            Ok(v) => {
                if let Err(ev) = v {
                    dbg!(&ev);
                    match ev {
                        WingmateInitError::SpawnError { source, message } => {
                            eprintln!("{}", WingmateInitError::SpawnError { source, message });
                        },
                        _ => {
                            return Err(ev);
                        }
                    }
                }
            },
            Err(e) => {
                dbg!(&e);
                return Err(WingmateInitError::Join { source: e });
            },
        }
    }

    Ok(())
}

async fn signal_pump(start: CancellationToken, stop: CancellationToken) -> Result<(), WingmateInitError> {
    const TERM_MODE: u8 = 0;
    const KILL_MODE: u8 = 1;
    const ALL_CHILDREN_PID: i32 = -1;

    start.cancelled().await;

    let stop_time = Instant::now();
    let mut wait_time_millis: u64 = 100;
    let mut mode = TERM_MODE;

    'signal: loop {
        let stop = stop.clone();
        let s = tokio::time::sleep(Duration::from_millis(wait_time_millis));
        pin!(s);

        select! {
            () = &mut s => {
                if mode == TERM_MODE {
                    if let Err(e) = kill(Pid::from_raw(ALL_CHILDREN_PID), Signal::SIGTERM) {
                        eprintln!("daemon: sending TERM signal got {}", e);
                    }
                } else {
                    if let Err(e) = kill(Pid::from_raw(ALL_CHILDREN_PID), Signal::SIGKILL) {
                        eprintln!("daemon: sending KILL signal got {}", e);
                    }
                }

                let time_peek = Instant::now();
                if time_peek.saturating_duration_since(stop_time).as_secs() >= config::MAX_TERM_WAIT_TIME_SECS && mode == TERM_MODE {
                    wait_time_millis = 10;
                    mode = KILL_MODE;
                }
            }
            _ = stop.cancelled() => {
                break 'signal;
            }
        }
    }

    Ok(())
}