#![allow(dead_code)]
#![allow(unused_variables)]

extern crate config;
extern crate dirs;
extern crate reqwest;
extern crate text_io;
extern crate toml;

#[macro_use]
extern crate serde;

mod csv_scan;
mod jsonrpc;
mod settings;

use awaitgroup::WaitGroup;
use clap::{App, AppSettings, Arg, ArgGroup};
use serde::{Deserialize, Serialize};

/// Grabs the arguments from terminal and execute the correct branch. Currently there exist
/// three branches (run_config(), run_csv() and run_find().
///
/// More might be implemented in the future.

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

    let mut chain: String = String::new();
    if res.is_present("ethereum") {
        chain = "eth".to_string();
    }
    if res.is_present("binance") {
        chain = "bsc".to_string();
    }

    let min_balance = res
        .value_of("balance")
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or_else(|_| {
            println!("Error: --balance option must be a number");
            std::process::exit(1);
        });

    let scan_limit = res
        .value_of("limit")
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or_else(|_| {
            println!("Error: --limit option must be a positive number");
            std::process::exit(1);
        });

    //Choosing branch to execute.
    if res.is_present("config") {
        settings::run_setup(&chain);
    }

    if res.is_present("csv") {
        let csv_file = res.value_of("csv").unwrap();
        println!("Running in csv mode");
        csv_scan::run_csv(&chain, &min_balance, &scan_limit, &csv_file).await;
    }

    if res.is_present("find") {
        println!("Running in find mode");
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
