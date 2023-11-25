mod daemon;
mod config;
pub(crate) mod error;

use std::env;
use std::error as std_err;

pub async fn start() -> Result<(), Box<dyn std_err::Error>> {
    let mut vec_search: Vec<String> = Vec::new();

    match env::var("WINGMATE_CONFIG_PATH") {
        Ok(paths) => {
            for p in paths.split(':') {
                vec_search.push(String::from(p));
            }
        },
        Err(e) => {
            if let env::VarError::NotUnicode(_) = e {
                return Err(e.into());
            } else {
                vec_search.push(String::from("/etc/wingmate"));
            }
        }
    }

    let config = config::Config::find(vec_search)?;
    dbg!(&config);
    daemon::start(config).await
}