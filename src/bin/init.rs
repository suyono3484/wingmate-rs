
use std::error;
use wingmate_rs::init;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    if let Err(e) = init::start().await {
        eprintln!("{}", e);
        return Err(e);
    }

    Ok(())
}