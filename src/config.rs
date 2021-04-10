use crate::error::{Error, WithContext};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub static COLUMN_WIDTH: u16 = 50;
static DEFAULT_TAGS: &[&str] = &[
    "Composer",
    "Work",
    "Conductor",
    "Ensemble",
    "Performer",
    "Location",
    "Label",
];

pub struct Config {
    pub mpd_host: String,
    pub default_subcommand: Option<String>,
    pub tags: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mpd_host: "127.0.0.1".to_string(),
            default_subcommand: None,
            tags: DEFAULT_TAGS.iter().map(ToString::to_string).collect(),
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
        if let Some(mpd_host) = key_val("mpd_host", &line) {
            conf.mpd_host = mpd_host.to_string();
        } else if let Some(default_subcommand) = key_val("default_subcommand", &line) {
            conf.default_subcommand = Some(default_subcommand.to_string());
        } else if let Some(tags) = key_val("tags", &line) {
            conf.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
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

fn key_val<'a>(key: &'static str, line: &'a String) -> Option<&'a str> {
    let with_equals = key.to_string() + "=";
    if !line.starts_with(&*with_equals) {
        return None;
    }
    Some(line.split_at(with_equals.len()).1)
}
