use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// Represents database url and path where
/// files shall be created
#[derive(Debug, Deserialize, Serialize)]
pub struct Storage {
    pub db_url: String,
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

/// Represents the settings as in the config file
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
                s.merge(File::from(config_path).required(false))?;
            }
            Err(e) => {
                println!("Error: {}, falling back to working directory", e);
            }
        }

        // Merge settings from executable's working dir
        match return_local_path(chain) {
            Ok(local_path) => {
                s.merge(File::from(local_path).required(false))?;
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        // Check if values can be read, and contain data.
        let jsonrpc_str = s.get_str("jsonrpc.url_1")?;
        let jsonrpc_str_2 = s.get_str("jsonrpc.url_2")?;
        let latency_1 = s.get_int("jsonrpc.latency_1")?;
        let latency_2 = s.get_int("jsonrpc.latency_2")?;
        let scan_api = s.get_str("scan.key")?;
        let mythx_api = s.get_str("mythx.key")?;
        let db_path = s.get_str("storage.db_url")?;
        let file_path = s.get_str("storage.file_path")?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

/// Returns the location of the config file in the dirs::config_dir, for the selected chain.
pub fn return_config_path(chain: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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
            Ok(v)
        }
        None => Err("Config dir not found".into()),
    }
}

/// Returns the location of the config file in the current working directory of the executable.
pub fn return_local_path(chain: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

            Ok(exe_path)
        }
        Err(e) => Err(e.into()),
    }
}

/// Takes the Settings values as string slices and returns
/// them as a String with toml formatting
pub fn create_conf_toml(
    db_url: &str,
    file_path: &str,
    jsonrpc_url: &str,
    jsonrpc_url_2: &str,
    latency_1: &u32,
    latency_2: &u32,
    scan_api: &str,
    mythx_api: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create struct and convert to toml string.
    let settings_struct = Settings {
        storage: Storage {
            db_url: db_url.to_string(),
            file_path: file_path.to_string(),
        },
        jsonrpc: JsonRpc {
            url_1: jsonrpc_url.to_string(),
            url_2: jsonrpc_url_2.to_string(),
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
    let toml = toml::to_string(&settings_struct)?;

    Ok(toml)
}

pub fn create_conf_file(
    config_path: &Path,
    toml: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory of config file
    let mut config_dir = PathBuf::from(config_path);
    config_dir.pop();
    std::fs::create_dir_all(config_dir)?;

    // Write config file
    let mut file = std::fs::File::create(config_path)?;
    file.write_all(toml.as_bytes())?;
    file.sync_all()?;

    println!("{} written!", config_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_toml_formatting_and_correct_values() {
        let toml = create_conf_toml(
            "dbtest",
            "filetest",
            "jsontest1",
            "jsontest2",
            &40,
            &41,
            "scantest",
            "mythtest",
        )
        .unwrap();

        let test_values = vec![
            "[storage]",
            "db_url = \"dbtest\"",
            "file_path = \"filetest\"",
            "",
            "[jsonrpc]",
            "url_1 = \"jsontest1\"",
            "url_2 = \"jsontest2\"",
            "latency_1 = 40",
            "latency_2 = 41",
            "",
            "[scan]",
            "key = \"scantest\"",
            "",
            "[mythx]",
            "key = \"mythtest\"",
        ];

        let mut test_values_iter = test_values.iter();
        let mut toml_iter = toml.lines();
        assert!(
            test_values_iter.all(|&x| Some(x) == toml_iter.next()),
            "One of the lines in the toml string doesn't match the expected output"
        );
    }
}
