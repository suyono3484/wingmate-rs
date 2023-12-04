#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

use std::error::Error;
use std::fs::OpenOptions;
use std::process::Command;
use std::env;
use rand::Rng;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut rng = rand::thread_rng();

    let log_path = env::var("LOG_PATH")?;
    let file = OpenOptions::new().append(true).create(true).open(log_path)?;
    WriteLogger::init(LevelFilter::Debug, Config::default(), file)?;

    if args.len() > 1 {
        let x: u64 = args[1].parse()?;
        for _i in 0..x {
            let sleep_time = rng.gen_range(10..20);
            info!("starting wmtest-helper-dummy {}", &sleep_time);
            let child = Command::new("/usr/local/bin/wmtest-helper-dummy").arg(format!("{}", sleep_time)).spawn();
            if let Err(e) = child {
                error!("error spawning child: {e}");
            }
        }

        let pause_time = rng.gen_range(5..10);
        info!("going to sleep for {}", &pause_time);
        std::thread::sleep(std::time::Duration::from_secs(pause_time));
    } else {
        return Err(anyhow::anyhow!("invalid arguments").into());
    }

    Ok(())
}
