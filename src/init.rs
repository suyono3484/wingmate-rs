mod daemon;
mod config;
pub(crate) mod error;

use std::env;
use anyhow::Context;

const WINGMATE_CONFIG_PATH: &'static str = "WINGMATE_CONFIG_PATH";

pub async fn start() -> Result<(), error::WingmateInitError> {
    let mut vec_search: Vec<String> = Vec::new();

    match env::var(WINGMATE_CONFIG_PATH) {
        Ok(paths) => {
            for p in paths.split(':') {
                vec_search.push(String::from(p));
            }
        },
        Err(e) => {
            if let env::VarError::NotUnicode(_) = e {
                return Err(e).context(format!("reading {} env var", WINGMATE_CONFIG_PATH))
                    .map_err(|e| {error::WingmateInitError::Other { source: e }} );
            } else {
                vec_search.push(String::from("/etc/wingmate"));
            }
        }
    }

    let config = config::Config::find(vec_search)?;
    // dbg!(&config);
    daemon::start(config).await
}