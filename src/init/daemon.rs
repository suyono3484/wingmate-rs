mod sighandler;
mod waiter;
mod starter;
mod constants;

use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use std::sync::{Arc, Mutex};
use crate::init::config;
use crate::init::error as wmerr;
use crate::init::error::WingmateInitError;

pub async fn start(cfg: config::Config) -> Result<(), WingmateInitError> {
    let sync_flag = Arc::new(Mutex::new(false));
    let sig_sync_flag = sync_flag.clone();

    let sighandler_cancel = CancellationToken::new();
    let waiter_cancel_sighandler = sighandler_cancel.clone();

    let cancel = CancellationToken::new();
    let starter_service_cancel = cancel.clone();
    let starter_cron_cancel = cancel.clone();

    let mut set: JoinSet<Result<(), wmerr::WingmateInitError>> = JoinSet::new();
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
                    // return Err(ev as Box<dyn error::Error>);
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