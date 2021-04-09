use crate::error::{Error, WithContext};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub static COLUMN_WIDTH: u16 = 50;

pub struct Config {
    pub mpd_host: String,
    pub default_subcommand: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mpd_host: "127.0.0.1".to_string(),
            default_subcommand: None,
        }
    }
}

pub fn get_config() -> Result<Config, Error> {
    let home = env::var("HOME").expect("$HOME was not set!");
    let home_config_path: PathBuf = [&*home, ".config", "davis", "davis.conf"].iter().collect();
    let etc_config_path: PathBuf = ["/", "etc", "davis", "davis.conf"].iter().collect();

    let mut conf = Config::default();

    let conf_file = match File::open(home_config_path).or(File::open(etc_config_path)) {
        Ok(f) => f,
        Err(_) => return Ok(conf),
    };

    for line in BufReader::new(conf_file).lines() {
        let line = line.context("reading config file")?;
        if line.starts_with("mpd_host=") {
            conf.mpd_host = line.chars().skip_while(|c| *c != '=').skip(1).collect();
        } else if line.starts_with("default_subcommand=") {
            conf.default_subcommand =
                Some(line.chars().skip_while(|c| *c != '=').skip(1).collect());
        }
    }

    if let Some(var) = mpd_host_env_var() {
        conf.mpd_host = var;
    }

    Ok(conf)
}

fn mpd_host_env_var() -> Option<String> {
    std::env::var("MPD_HOST").ok()
}
