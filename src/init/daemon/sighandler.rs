use std::error;
use tokio::signal::unix::{signal, SignalKind};
use tokio::select;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

pub async fn sighandler(flag: Arc<Mutex<bool>>, cancel: CancellationToken, exit: CancellationToken) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let mut sigint = signal(SignalKind::interrupt())?; 
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigchld = signal(SignalKind::child())?;

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