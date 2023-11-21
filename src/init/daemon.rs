mod sighandler;
mod waiter;
mod starter;

use std::error;
use tokio::task::JoinSet;
use tokio::sync::watch;

pub async fn start() -> Result<(), Box<dyn error::Error>> {
    let (tx, mut _rx) = watch::channel::<i32>(1);

    let mut set: JoinSet<Result<(), Box<dyn error::Error + Send + Sync>>> = JoinSet::new();
    set.spawn(async move {
        sighandler::sighandler(tx).await
    });

    //TODO: start the process starter
    starter::start_process(&mut set);

    //TODO: spawn_blocking for waiter
    set.spawn_blocking(move || {
        waiter::wait_all();
        Ok(())
    });

    //TODO: we can't just return error when we got an error from a task
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