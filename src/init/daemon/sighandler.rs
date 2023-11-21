use std::error;
use tokio::signal::unix::{signal, SignalKind};
use tokio::select;
use tokio::sync::watch::Sender;

pub async fn sighandler(s: Sender<i32>) -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let mut sigint = signal(SignalKind::interrupt())?; 
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigchld = signal(SignalKind::child())?;

    'signal: loop {
        select! {
            _ = sigint.recv() => {
                println!("got SIGINT");
                drop(s);
                break 'signal;
            },
            _ = sigterm.recv() => {
                println!("got SIGTERM");
                drop(s);
                break 'signal;
            },
            _ = sigchld.recv() => {
                // do nothing intentionally
                // return Err(())
            },
        }
        }

    Ok(())
}