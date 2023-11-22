mod daemon;
mod config;
pub(crate) mod error;

use std::error as std_err;

pub async fn start() -> Result<(), Box<dyn std_err::Error>> {
    let _config = config::Config::find(vec![String::from("/etc/wingmate")])?;
    dbg!(_config);
    daemon::start().await
}