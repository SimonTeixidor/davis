use crate::error::{Error, WithContext};
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

static DEFAULT_TAGS: &[&str] = &[
    "Composer",
    "Work",
    "Conductor",
    "Ensemble",
    "Performer",
    "Location",
    "Label",
];

#[serde(default)]
#[derive(Deserialize)]
pub struct Config {
    pub hosts: Vec<Host>,
    pub tags: Vec<Tag>,
    pub width: usize,
    pub grouped_queue: bool,
    no_default_tags: bool,
}

impl Config {
    pub fn default_mpd_host(&self) -> String {
        if self.hosts.is_empty() {
            return "127.0.0.1".to_string();
        } else {
            return self.hosts[0].host.clone();
        }
    }
}

#[derive(Deserialize)]
pub struct Tag {
    pub tag: String,
    pub label: Option<String>,
}

#[derive(Deserialize)]
pub struct Host {
    pub host: String,
    pub label: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hosts: Vec::new(),
            tags: DEFAULT_TAGS
                .iter()
                .map(|t| Tag {
                    tag: t.to_string(),
                    label: None,
                })
                .collect(),
            width: 50,
            grouped_queue: false,
            no_default_tags: false,
        }
    }
}

pub fn get_config() -> Result<Config, Error> {
    let home = env::var("HOME").expect("$HOME was not set!");
    let home_config_path: PathBuf = [&*home, ".config", "davis", "davis.conf"].iter().collect();
    let etc_config_path: PathBuf = ["/", "etc", "davis", "davis.conf"].iter().collect();

    let mut conf = Config::default();

    match File::open(&home_config_path)
        .or(File::open(&etc_config_path))
        .context("opening config file")
        .and_then(|mut f| {
            let mut buf = String::new();
            f.read_to_string(&mut buf).context("reading config file")?;
            Ok(toml::from_str(&*buf)?)
        }) {
        Ok(f) => {
            conf = f;
            if !conf.no_default_tags {
                conf.tags = Config::default()
                    .tags
                    .into_iter()
                    .chain(conf.tags.into_iter())
                    .collect();
            }
        }
        Err(e) if etc_config_path.exists() || home_config_path.exists() => {
            println!(
                "Failed to read config file, will use default instead: {}",
                e
            );
        }
        _ => {}
    }

    Ok(conf)
}

pub fn mpd_host_env_var() -> Option<String> {
    std::env::var("MPD_HOST").ok()
}
