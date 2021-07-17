use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::io::prelude::*;

/// Represents path to scan for database
/// and path where files shall be created
#[derive(Debug, Deserialize, Serialize)]
pub struct Storage {
    pub db_path: String,
    pub file_path: String,
}

/// Represents the url to use for acces to json-rpc
/// The latency in ms is set for each individual url
/// Calculate latency by 1000/rate_limit_per_second
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpc {
    pub url_1: String,
    pub url_2: String,
    pub latency_1: u32,
    pub latency_2: u32,
}

/// Represents a BSCScan or EtherScan api key depending on the config file
#[derive(Debug, Deserialize, Serialize)]
pub struct Scan {
    pub key: String,
}

/// Represents a MythX api key
#[derive(Debug, Deserialize, Serialize)]
pub struct MythX {
    pub key: String,
}

/// This represents the settings as set in the config file
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub storage: Storage,
    pub jsonrpc: JsonRpc,
    pub scan: Scan,
    pub mythx: MythX,
}

impl Settings {
    /// Returns the settings, Settings::new reads the settings file in the config dir
    /// and initializes it as a struct that can be read from. If there is a local config
    /// file in the same dir as the executable it takes precedence over the config file
    /// in the config dir.
    ///
    /// # Arguments
    ///
    /// * `chain` - A string slice that holds the current chain, will decide which config file
    ///             to use.

    pub fn new(chain: &str) -> Result<Self, ConfigError> {
        let mut s = Config::default();
        match dirs::config_dir() {
            Some(mut v) => {
                v.push("merter");
                if chain == "eth" {
                    v.push(".ethconf");
                }
                if chain == "bsc" {
                    v.push(".bscconf")
                }
                v.set_extension("toml");
                let path_str = v.into_os_string().into_string().unwrap();
                s.merge(File::with_name(&path_str).required(false))?;
            }
            None => {}
        }
        match std::env::current_exe() {
            Ok(mut exe_path) => {
                exe_path.pop();
                if chain == "eth" {
                    exe_path.push(".ethconf");
                }
                if chain == "bsc" {
                    exe_path.push(".bscconf");
                }
                exe_path.set_extension("toml");
                let path_str = exe_path.into_os_string().into_string().unwrap();
                s.merge(File::with_name("config/local").required(false))?;
            }
            Err(e) => println!("failed to get current exe path: {}", e),
        }

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

pub fn create(
    chain: &str,
    jsonrpc_str: &str,
    jsonrpc_str_2: &str,
    latency_1: &u32,
    latency_2: &u32,
    scan_api: &str,
    mythx_api: &str,
    db_path: &str,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings_struct = Settings {
        storage: Storage {
            db_path: db_path.to_string(),
            file_path: file_path.to_string(),
        },
        jsonrpc: JsonRpc {
            url_1: jsonrpc_str.to_string(),
            url_2: jsonrpc_str_2.to_string(),
            latency_1: *latency_1,
            latency_2: *latency_2,
        },
        scan: Scan {
            key: scan_api.to_string(),
        },
        mythx: MythX {
            key: mythx_api.to_string(),
        },
    };

    let toml = toml::to_string(&settings_struct).unwrap();
    let mut path_str = String::new();

    match dirs::config_dir() {
        Some(mut v) => {
            v.push("merter");
            let v_copy = v.clone();
            let dir = v_copy.into_os_string().into_string().unwrap();
            std::fs::create_dir_all(dir).unwrap();
            if chain == "eth" {
                v.push(".ethconf");
            }
            if chain == "bsc" {
                v.push(".bscconf");
            }
            v.set_extension("toml");
            path_str = v.into_os_string().into_string().unwrap();
        }
        None => match std::env::current_exe() {
            Ok(mut exe_path) => {
                exe_path.pop();
                if chain == "eth" {
                    exe_path.push(".ethconf");
                }
                if chain == "bsc" {
                    exe_path.push(".bscconf");
                }
                exe_path.set_extension("toml");
                path_str = exe_path.into_os_string().into_string().unwrap();
            }
            Err(e) => println!("failed to get current exe path: {}", e),
        },
    }

    let path_str_clone = path_str.clone();
    let mut file = std::fs::File::create(path_str)?;
    file.write_all(toml.as_bytes())?;
    file.sync_all()?;

    println!("{} written!", &path_str_clone);

    Ok(())
}
