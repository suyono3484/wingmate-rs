
use std::error;
use wingmate_rs::init;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    init::daemon::start().await
}