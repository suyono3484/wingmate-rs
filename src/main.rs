use std::fmt;
use std::error::Error;
use tokio::signal::unix::{signal, SignalKind};
use tokio::select;
use tokio::task::{self, JoinHandle};

#[derive(Debug,Clone)]
struct InvalidState;

impl fmt::Display for InvalidState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot recover the state")
    }
}

impl Error for InvalidState {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let sighandler: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> = task::spawn(async {
        let mut sigint = signal(SignalKind::interrupt())?; 
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigchld = signal(SignalKind::child())?;
        select! {
            _ = sigint.recv() => {
                println!("got SIGINT");
            },
            _ = sigterm.recv() => {
                println!("got SIGTERM");
                return Err(InvalidState.into());
            },
            _ = sigchld.recv() => {
                // return Err(())
            },
        }

        Ok(())
    });

    if let Err(v) = sighandler.await? {
        return Err(v as Box<dyn Error>)
    }

    Ok(())
}
