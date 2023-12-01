use std::error::Error;
use std::process::Command;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let x: u64 = args[1].parse()?;
        for _i in 0..x {
            // println!("spawning {}", _i);
            Command::new("/usr/local/bin/dummy").arg("10").spawn()?;
        }
    }
    Ok(())
}
