use std::error;
use tokio::signal::unix::{signal, SignalKind};
use tokio::select;

#[allow(dead_code)]
pub async fn sighandler() -> Result<(), Box<dyn error::Error + Send + Sync>> {
    let mut sigint = signal(SignalKind::interrupt())?; 
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigchld = signal(SignalKind::child())?;
    select! {
        _ = sigint.recv() => {
            println!("got SIGINT");
        },
        _ = sigterm.recv() => {
            println!("got SIGTERM");
        },
        _ = sigchld.recv() => {
            // return Err(())
        },
    }

    Ok(())
}