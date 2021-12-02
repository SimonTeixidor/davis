use crate::error::{Error, WithContext};
use configparser::ini::Ini;
use std::collections::HashMap;
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

pub struct Config {
    pub hosts: Vec<Host>,
    pub tags: Vec<Tag>,
}

impl Config {
    pub fn default_mpd_host(&self) -> String {
        if self.hosts.is_empty() {
            log::trace!("Found no host in config file, defaulting to 127.0.0.1.");
            "127.0.0.1".to_string()
        } else if let Some(host) = self.hosts.iter().find(|h| &*h.label == "default") {
            log::trace!("Using default host from config: {}", host.host);
            host.host.clone()
        } else {
            log::trace!(
                "No default host configured, using random host from config: {}",
                self.hosts[0].host
            );
            self.hosts[0].host.clone()
        }
    }
}

pub struct Tag {
    pub tag: String,
    pub label: Option<String>,
}

pub struct Host {
    pub host: String,
    pub label: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hosts: Vec::new(),
            tags: DEFAULT_TAGS
                .iter()
                .map(|t| Tag {
                    tag: (*t).to_string(),
                    label: None,
                })
                .collect(),
        }
    }
}

pub fn get() -> Config {
    let home = env::var("HOME").expect("$HOME was not set!");
    let home_config_path: PathBuf = [&*home, ".config", "davis", "davis.conf"].iter().collect();
    let etc_config_path: PathBuf = ["/", "etc", "davis", "davis.conf"].iter().collect();

    let mut conf = Config::default();

    match File::open(&home_config_path)
        .or_else(|_| File::open(&etc_config_path))
        .context("opening config file")
        .and_then(|mut f| {
            log::trace!("Read config from {:?}", f);
            let mut buf = String::new();
            f.read_to_string(&mut buf).context("reading config file")?;
            parse_config(&Ini::new_cs().read(buf).map_err(Error::Config)?)
        }) {
        Ok(f) => {
            conf = f;
        }
        Err(e) if etc_config_path.exists() || home_config_path.exists() => {
            log::warn!(
                "Failed to read config file, will use default instead: {}",
                e
            );
        }
        _ => log::trace!("No config file found, using default."),
    }

    conf
}

fn parse_config(map: &HashMap<String, HashMap<String, Option<String>>>) -> Result<Config, Error> {
    let hosts = map.get("hosts").map_or_else(|| Ok(vec![]), parse_hosts)?;

    let tags = map
        .get("tags")
        .and_then(parse_tags)
        .unwrap_or_else(|| Config::default().tags);

    Ok(Config { hosts, tags })
}

fn parse_hosts(map: &HashMap<String, Option<String>>) -> Result<Vec<Host>, Error> {
    map.iter()
        .map(|(label, host)| {
            Ok(Host {
                host: host.clone().ok_or_else(|| {
                    Error::Config(format!("Missing hostname for host {}.", label))
                })?,
                label: label.clone(),
            })
        })
        .collect::<Result<Vec<Host>, Error>>()
}

fn parse_tags(map: &HashMap<String, Option<String>>) -> Option<Vec<Tag>> {
    map.get("enabled")
        .and_then(Option::as_ref)
        .map(|e| e.split(','))
        .map(|ts| {
            ts.map(|t| Tag {
                tag: t.into(),
                label: map.get(t).and_then(Clone::clone),
            })
            .collect()
        })
}

pub fn mpd_host_env_var() -> Option<String> {
    std::env::var("MPD_HOST").ok()
}
