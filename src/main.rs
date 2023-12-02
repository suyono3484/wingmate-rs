mod init;

use std::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    if let Err(e) = init::start().await {
        eprintln!("{}", e);
        return Err(e.into());
    }

    Ok(())
}