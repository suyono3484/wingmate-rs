
use std::error;
use wingmate_rs::init;
use tokio::task::{self, JoinHandle};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let sig_handler_fn: JoinHandle<Result<(), Box<dyn error::Error + Send + Sync>>> = task::spawn(async {
        init::sighandler::sighandler().await
    });

    if let Err(v) = sig_handler_fn.await? {
        return Err(v as Box<dyn error::Error>)
    }

    Ok(())
}