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
    /// Reads the settings file in the config dir and initializes it as a public struct
    /// If there is a local config file in the executable's working directory it takes
    /// precedence over the config file in the config dir.
    ///
    /// # Arguments
    ///
    /// * `chain` - A string slice that holds the current chain, will decide which config file
    ///             to use.
    ///
    pub fn new(chain: &str) -> Result<Self, ConfigError> {
        let mut s = Config::default();

        // Merge settings from config dir
        match return_config_path(chain) {
            Ok(config_path) => {
                s.merge(File::with_name(&config_path).required(false))?;
            }
            Err(e) => {
                println!(
                    "Can't find config directory, trying to read from current working directory"
                );
            }
        }

        // Merge settings from executable's working dir
        match return_local_path(chain) {
            Ok(local_path) => {
                s.merge(File::with_name(&local_path).required(false))?;
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        // #TODO: Check if values are loaded/valid

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

/// Returns the location of the config file in the dirs::config_dir, for the selected chain.
///
pub fn return_config_path(chain: &str) -> Result<std::string::String, Box<dyn std::error::Error>> {
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

            Ok(path_str)
        }
        None => Err("Config dir not found".into()),
    }
}

/// Returns the location of the config file in the current working directory of the executable.
pub fn return_local_path(chain: &str) -> Result<std::string::String, Box<dyn std::error::Error>> {
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

            Ok(path_str)
        }
        Err(e) => Err(e.into()),
    }
}

/// Creates a config file .ethconf or .bscconf in the dir dirs::config_dir (see crate dirs).
/// If it is on a system without a config_dir it will create a config file in the working
/// directory of the executable.
///
/// #Arguments - All the arguments from the Settings struct.
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
    // Create struct and convert to toml string.
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

    // Create config directory
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("merter");
            let dir_str = dir.into_os_string().into_string().unwrap();
            std::fs::create_dir_all(dir_str)?;
        }
        None => {}
    }

    // Get path to config file either in config dir or in executable's
    // working dir. If the latter doesn't work exit(1)
    let config_path = return_config_path(chain)
        .or_else(|err| {
            println!("Error: {}, falling back to working directory", err);
            return_local_path(chain)
        })
        .unwrap_or_else(|err| {
            println!("{}", err);
            std::process::exit(1);
        });

    // Write config file
    let mut file = std::fs::File::create(&config_path)?;
    file.write_all(toml.as_bytes())?;
    file.sync_all()?;

    println!("{} written!", &config_path);

    Ok(())
}
