#![allow(dead_code)]
#![allow(unused_variables)]

extern crate config;
extern crate dirs;
extern crate reqwest;
extern crate text_io;
extern crate toml;

#[macro_use]
extern crate serde;

mod jsonrpc;
mod settings;

use awaitgroup::WaitGroup;
use clap::{App, AppSettings, Arg, ArgGroup, ErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


type Db = Arc<Mutex<HashMap<String, bool>>>;

static JSONRPCAPI: &'static str = "";
static BSCSCANAPI: &'static str = "";

/// Grabs the arguments from terminal and execute the correct branch. Currently there exist
/// three branches. More might be implemented in the future.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = App::new("Merter")
        .version("0.0.1")
        .author("dRAT3 <dRAT3<ath>protonmail.ch>")
        .about(
            " 
                           ._ _  _ ._.-+- _ ._.
                           [ | )(/,[   | (/,[  
                    
-----------------------------------------------------------------------------
Merter automates finding valuable contracts on ethereum or binance smart chain.
When it finds a valuable contract it grabs the code from etherscan/bscscan and
uses the MythX API, to scan for vulnerabilities. It stores the contracts on disk
and keeps the vulnerability data in a database that can be easily searched 
through. Together with a schema of the entry-points and flow of the contract.

                    Exploitation has to be done manually.

-----------------------------------------------------------------------------
              Happy Bounty Hunting || Wargaming || B31n6 3v1l!                   

-----------------------------------------------------------------------------",
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("csv")
                .short("c")
                .long("csv")
                .takes_value(true)
                .value_name("CSV")
                .help(
                    "Csv mode, the csv files need to be manually down-
loaded from etherscan||bscscan
/exportData?type=tokenholders&contract=[addr]
or use the same formatting",
                ),
        )
        .arg(
            Arg::with_name("find")
                .short("f")
                .long("find")
                .takes_value(false)
                .help(
                    "Find mode, loops over the latest block, grabs all trans-
actions, filters out contract addresses, checks value,
downloads and scans them",
                ),
        )
        .arg(Arg::with_name("config").long("config").help(
            "Set API keys, database, working directory and Json-RPC
node. If you use your own node make shure it's behind a
ngingx proxy that keeps the connections alive. The requests
to the endpoint are done concurrently so it overloads the 
node. If you use an API make shure to calculate latency
according to rate limit.",
        ))
        .group(
            ArgGroup::with_name("req_flags")
                .args(&["csv", "find", "config"])
                .required(true),
        )
        .arg(
            Arg::with_name("balance")
                .short("b")
                .long("balance")
                .takes_value(true)
                .help(
                    "Specify the minimum balance of the contract.
In --csv mode the balance is denominated in the 
current token.
In --find mode the balance is denominated in eth
or bnb.",
                ),
        )
        .arg(
            Arg::with_name("limit")
                .short("l")
                .long("limit")
                .takes_value(true)
                .help("Sets maximum amount of contracts to scan [mythx]"),
        )
        .arg(
            Arg::with_name("ethereum")
                .long("eth")
                .takes_value(false)
                .help("Selects ethereum config file"),
        )
        .arg(
            Arg::with_name("binance")
                .long("bsc")
                .takes_value(false)
                .help("Selects binance smart chain config file"),
        )
        .group(
            ArgGroup::with_name("chain_flags")
                .args(&["ethereum", "binance"])
                .required(true),
        )
        .get_matches();

    //Setting the chain
    let mut chain: String = String::new();
    if res.is_present("ethereum") {
        chain = "eth".to_string();
    }
    if res.is_present("binance") {
        chain = "bsc".to_string();
    }

    //Setting the minimum balance
    let mut min_balance: f32 = 0.0;
    match res.value_of("balance").unwrap_or("0").parse::<f32>() {
        Ok(v) => {
            min_balance = v;
        }
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }

    //Setting the max limit of scans
    let mut limit: u32 = 0;
    match res.value_of("limit").unwrap_or("0").parse::<u32>() {
        Ok(v) => {
            limit = v;
        }
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }

    //Choosing the correct branch to execute.
    if res.is_present("config") {
        run_setup(&chain);
    }

    if res.is_present("csv") {
        println!("Running in csv mode");
        run_csv(&chain, &min_balance, &limit).await?;
    }

    if res.is_present("find") {
        println!("Running in find mode");
    }

    Ok(())
}

fn run_setup(chain: &str) {
    match dirs::config_dir() {
        Some(mut v) => {
            v.push("merter");
            let v_copy = v.clone();
            
            if chain == "eth" {
                v.push(".ethconf");
            }
            if chain == "bsc" {
                v.push(".bscconf");
            }
            v.set_extension("toml");

            let path_str = v.into_os_string().into_string().unwrap();

            if std::path::Path::new(&path_str).exists() {
                println!(
                    "Are you shure you want to overwrite settings file: .{}conf.toml (y/n)",
                    chain
                );
                let answ: String = text_io::read!("{}\n");
                if answ.eq_ignore_ascii_case("y") || answ.eq_ignore_ascii_case("yes") {
                } else {
                std::process::exit(1);
                }
            }
        }
        None => {
            ;
        }

    
    println!("Enter JSON-RPC 1 api url:");
    let jsonrpc_str: String = text_io::read!("{}\n");

    println!("Enter JSON-RPC 2 api url (optional):");
    let jsonrpc_str_2: String = text_io::read!("{}\n");

    println!("Enter JSON-RPC 1's latency in ms");
    let latency_1: u32 = text_io::read!("{}\n");

    println!("Enter JSON-RPC 2's latency in ms");
    let latency_2: u32 = text_io::read!("{}\n");

    let mut scan_api: String = String::new();
    if chain == "eth" {
        println!("Enter EtherScan API key:");
        scan_api = text_io::read!("{}\n");
    }

    if chain == "bsc" {
        println!("Enter BscScan API key:");
        scan_api = text_io::read!("{}\n");
    }

    println!("Enter MythX API key:");
    let mythx_api: String = text_io::read!("{}\n");

    println!("Enter db location (if it doesn't exist it will create a new db):");
    let db_path: String = text_io::read!("{}\n");

    println!("Enter folder where downloaded contracts will be stored:");
    let file_path: String = text_io::read!("{}\n");

    let ret = settings::create(
        chain,
        &jsonrpc_str,
        &jsonrpc_str_2,
        &latency_1,
        &latency_2,
        &scan_api,
        &mythx_api,
        &db_path,
        &file_path,
    );
}

async fn run_csv(
    chain: &str,
    min_balance: &f32,
    limit: &u32,
) -> Result<(), Box<dyn std::error::Error>> {
    match settings::Settings::new(chain) {
        Ok(setting) => {
            println!("settings test {:?}", setting.storage.db_path);
        }
        Err(e) => {
            println!(
                "Error while loading settings {:?} 
                \nTry running merter --config --{}",
                e, chain
            );
            std::process::exit(1);
        }
    }
    Ok(())
}

async fn run_find(
    chain: &str,
    min_balance: &f32,
    limit: &u32,
) -> Result<(), Box<dyn std::error::Error>> {
    /*
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let mut wg = WaitGroup::new();

    let block_res = get_block_addresses().await;

    match block_res {
        Ok(v) => {
            println!("Block downloaded, grabbing contracts");
            println!("Txs: {}", v.result.transactions.len());

            for obj in v.result.transactions {
                let db1 = db.clone();
                let db2 = db.clone();

                let to = obj.to.clone();
                let from = obj.from.clone();

                let worker = wg.worker();
                let worker1 = wg.worker();

                let to_handle = tokio::spawn(async move {
                    let resp = check_if_contract(to, db1).await;
                    println!("{:?}", resp);
                    worker.done();
                });

                let from_handle = tokio::spawn(async move {
                    let resp = check_if_contract(from, db2).await;
                    println!("{:?}", resp);
                    worker1.done();
                });
            }

            wg.wait().await;

            let db3 = db.clone();
            for (key, value) in db3.lock().unwrap().iter() {
                println!("{}: {}", key, value);
            }
        }

        Err(e) => {
            println!("error parsing header: {:?}", e);
        }
    }
    */
    Ok(())
}
