use std::{env, thread, time};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let myi: u64;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        myi = args[1].parse().unwrap();
        thread::sleep(time::Duration::from_secs(myi));
    } else {
        return Err(anyhow::anyhow!("invalid arguments").into());
    }

    Ok(())
}
