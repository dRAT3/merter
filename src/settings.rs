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

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }

    /// Creates an empty settings struct
    fn default() -> Self {
        Settings {
            storage: Storage {
                db_url: "".to_string(),
                file_path: "".to_string(),
            },
            jsonrpc: JsonRpc {
                url_1: "".to_string(),
                url_2: "".to_string(),
                latency_1: 0,
                latency_2: 0,
            },
            scan: Scan {
                key: "".to_string(),
            },
            mythx: MythX {
                key: "".to_string(),
            },
        }
    }
}

///Gets the config path, asks for the settings and write to config file. If the config dir
///doesn't exist it will create the conf file in the working dir of the executable.
pub fn run_setup(chain: &str) {
    let config_path = return_config_path(chain)
        .or_else(|err| {
            println!(
                "Error: Couldn't find config directory, falling back to working directory \n{}",
                err
            );
            return_local_path(chain)
        })
        .unwrap_or_else(|err| {
            println!("Error: Couldn't find working directory, exiting \n{}", err);
            std::process::exit(1);
        });

    let setup_struct = ask_for_settings(chain, &config_path);

    let toml = toml::to_string(&setup_struct).unwrap_or_else(|err| {
        println!("Error: Couldn't parse settings as toml. \n {}", err);
        std::process::exit(1);
    });

    create_conf_file(&config_path, toml).unwrap_or_else(|err| {
        println!("Error: Couldn't write settings file. \n {}", err);
        std::process::exit(1);
    });

    println!("{} written!", config_path.display());
}

/// Asks the user for all the configurations in settings::Setting.
fn ask_for_settings(chain: &str, config_path: &Path) -> Settings {
    println!(
        "Are you shure you want to overwrite settings file: {} (y/n)",
        config_path.display()
    );
    let answ: String = text_io::read!("{}\n");

    if answ.eq_ignore_ascii_case("y") || answ.eq_ignore_ascii_case("yes") {
    } else {
        std::process::exit(1);
    }

    let mut setup_struct = Settings::default();

    while !valid_url(&setup_struct.jsonrpc.url_1) {
        println!("Enter JSON-RPC 1 api url:");
        setup_struct.jsonrpc.url_1 = text_io::read!("{}\n");
        println!("url invalid, use http(s)://");
    }

    while !valid_url(&setup_struct.jsonrpc.url_2) {
        println!("Enter JSON-RPC 2 api url (optional press s to skip):");
        setup_struct.jsonrpc.url_2 = text_io::read!("{}\n");
        if setup_struct.jsonrpc.url_2.eq("s") {
            break;
        }
        println!("url invalid, use http(s)://");
    }

    println!("Enter JSON-RPC 1's latency in ms");
    setup_struct.jsonrpc.latency_1 = text_io::read!("{}\n");

    if setup_struct.jsonrpc.url_2.chars().count() > 2 {
        println!("Enter JSON-RPC 2's latency in ms");
        setup_struct.jsonrpc.latency_2 = text_io::read!("{}\n");
    }

    if chain == "eth" {
        println!("Enter EtherScan API key:");
    }
    if chain == "bsc" {
        println!("Enter BscScan API key:");
    }
    setup_struct.scan.key = text_io::read!("{}\n");

    println!("Enter MythX API key:");
    setup_struct.mythx.key = text_io::read!("{}\n");

    while !valid_url(&setup_struct.jsonrpc.url_1) {
        println!("Enter db url for :");
        setup_struct.storage.db_url = text_io::read!("{}\n");
    }

    while !valid_path(&setup_struct.storage.file_path) {
        println!("Enter folder where downloaded contracts will be stored:");
        setup_struct.storage.file_path = text_io::read!("{}\n");
        println!("Path invalid, enter a path with write acces");
    }
    setup_struct
}

/// Returns the location of the config file in the dirs::config_dir, for the selected chain.
fn return_config_path(chain: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
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
fn return_local_path(chain: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

fn create_conf_file(config_path: &Path, toml: String) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory of config file
    let mut config_dir = PathBuf::from(config_path);
    config_dir.pop();
    std::fs::create_dir_all(config_dir)?;

    // Write config file
    let mut file = std::fs::File::create(config_path)?;
    file.write_all(toml.as_bytes())?;
    file.sync_all()?;

    Ok(())
}

fn valid_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn valid_path(path: &str) -> bool {
    let path_p = Path::new(path);

    match std::fs::metadata(path_p) {
        Ok(md) => md.is_dir() && !md.permissions().readonly(),
        Err(e) => false,
    }
}
