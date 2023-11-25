mod sighandler;
mod waiter;
mod starter;
mod constants;

use std::error;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use std::sync::{Arc, Mutex};
use crate::init::config;

pub async fn start(cfg: config::Config) -> Result<(), Box<dyn error::Error>> {
    let sync_flag = Arc::new(Mutex::new(false));
    let sig_sync_flag = sync_flag.clone();

    let sighandler_cancel = CancellationToken::new();
    let waiter_cancel_sighandler = sighandler_cancel.clone();

    let cancel = CancellationToken::new();
    let starter_cancel = cancel.clone();

    let mut set: JoinSet<Result<(), Box<dyn error::Error + Send + Sync>>> = JoinSet::new();
    set.spawn(async move {
        sighandler::sighandler(sig_sync_flag, cancel, sighandler_cancel).await
    });

    //TODO: start the process starter
    starter::start_services(&mut set, &cfg, starter_cancel)?;

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
                    return Err(ev as Box<dyn error::Error>);
                }
            },
            Err(e) => {
                dbg!(&e);
                return Err(e.into());
            },
        }
    }

    Ok(())
}