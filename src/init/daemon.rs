mod sighandler;
mod waiter;

use std::error;
use tokio::task::{self, JoinHandle};
use tokio::sync::watch;

pub async fn start() -> Result<(), Box<dyn error::Error>> {
    let (tx, mut _rx) = watch::channel::<i32>(1);

    let sig_handler_fn: JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>> = task::spawn(async move {
        sighandler::sighandler(tx).await
    });

    if let Err(v) = sig_handler_fn.await? {
        return Err(v as Box<dyn error::Error>)
    }

    Ok(())
}