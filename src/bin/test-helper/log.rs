use std::env;
use std::fs;
use std::io;
use std::io::Write;
use time::OffsetDateTime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect(); 

    if args.len() == 3 {
        let file = fs::OpenOptions::new().create(true).append(true).open(&args[1])?;
        let mut buf = io::BufWriter::new(file);
        let local_time = OffsetDateTime::now_local()?;
        buf.write_all(format!("{} {}\n", local_time, &args[2]).as_bytes())?;
    } else {
        return Err(anyhow::anyhow!("invalid argument").into());
    }
    Ok(())
}