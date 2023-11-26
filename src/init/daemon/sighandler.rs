use tokio::signal::unix::{signal, SignalKind};
use tokio::select;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use crate::init::error::WingmateInitError;

pub async fn sighandler(flag: Arc<Mutex<bool>>, cancel: CancellationToken, exit: CancellationToken) -> Result<(), WingmateInitError> {
    let mut sigint = signal(SignalKind::interrupt()).map_err(|e| { WingmateInitError::Signal { source: e } })?; 
    let mut sigterm = signal(SignalKind::terminate()).map_err(|e| { WingmateInitError::Signal { source: e } })?;
    let mut sigchld = signal(SignalKind::child()).map_err(|e| { WingmateInitError::Signal { source: e } })?;

    'signal: loop {
        select! {
            _ = sigint.recv() => {
                println!("got SIGINT");
                initiate_stop(flag.clone(), cancel.clone());
            },
            _ = sigterm.recv() => {
                println!("got SIGTERM");
                initiate_stop(flag.clone(), cancel.clone());
            },
            _ = sigchld.recv() => {
                // do nothing intentionally
            },
            _ = exit.cancelled() => {
                break 'signal;
            }
        }
    }

    Ok(())
}

fn initiate_stop(flag: Arc<Mutex<bool>>, cancel: CancellationToken) {
    {
        let mut fl = flag.lock().unwrap();
        *fl = true;
    }
    cancel.cancel();
}