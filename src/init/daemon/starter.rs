use tokio::task::JoinSet;
use tokio::process::Command;
use std::error;

pub fn start_process(ts: &mut JoinSet<Result<(), Box<dyn error::Error + Send + Sync>>>) {
    for _j in 0..5 {
        ts.spawn(async move {
            for _i in 0..5 {
                let mut child = Command::new("sleep").arg("1")
                    .spawn()
                    .expect("failed to spawn");

                match child.wait().await {
                    Ok(status) => {
                        println!("starter: sleep exited: {}", status);
                    },
                    Err(e) => {
                        if let Some(eos) = e.raw_os_error() {
                            if eos != nix::Error::ECHILD as i32 {
                                return Err(e.into());
                            }
                        } else {
                            return Err(e.into());
                        }
                    },
                }
                // let status = child.wait().await?;
                // println!("starter: sleep exited: {}", status);
            }
            println!("starter: task completed");
            Ok(())
        });

    }
    println!("starter: spawning completed");
}