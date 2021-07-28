use super::jsonrpc;
use super::settings;
use super::timers;

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;

use futures::stream::{FuturesUnordered, StreamExt};

#[derive(Default, Debug)]
struct Entry {
    address: String,
    balance: f32,
}

pub async fn run_csv(chain: &str, min_balance: f32, limit: usize, csv_file: &str) {
    //Load settings
    let setting = settings::Settings::new(chain).unwrap_or_else(|err| {
        println!(
            "Couldn't load settings file.
            \nTry running merter --config --{} \n{}",
            err, chain
        );
        std::process::exit(1);
    });

    //Check if passed parameter is a csv file
    if !is_csv(csv_file) {
        println!("{} is not a csv file, exiting", csv_file);
        std::process::exit(1);
    }

    //Create vector of addresses above minimum treshold
    let mut addr_vec = csv_to_vec(csv_file, min_balance).unwrap_or_else(|err| {
        println!("Error while reading csv file: \n{}", err);
        std::process::exit(1);
    });
    addr_vec.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap());

    //Start latency timer, for rate limitting of the jsonrpc api
    std::thread::spawn(|| timers::count_down_rpc());

    //Spawn tasks that check if the addresses are contracts and execute them concurrently
    let mut tasks = FuturesUnordered::new();
    let mut contracts: HashMap<String, bool> = HashMap::new();

    for entry in addr_vec {
        let url = setting.jsonrpc.url_1.clone();
        let address = entry.address.clone();
        let latency = setting.jsonrpc.latency_1.clone();

        let container = jsonrpc::IsContractResponse {
            address: address,
            is_contract: false,
            count: 0,
        };

        timers::push_time_rpc(latency);
        let sleep_time = std::time::Duration::from_millis(timers::get_sleep_time_rpc() as u64);
        tokio::time::sleep(sleep_time).await;

        tasks.push(tokio::spawn(async move {
            jsonrpc::is_contract(container, url).await
        }));
    }

    while let Some(finished_task) = tasks.next().await {
        if finished_task.is_err() {
            println!(
                "JoinError while scanning for contract: \n{}",
                finished_task.err().unwrap()
            );
            continue;
        }

        match finished_task.unwrap() {
            Err(e) => {
                if e.container.count > 5 {
                    println!(
                        "Can't see if {} is a contract, skipping. \nError: {}",
                        e.container.address, e.error
                    );
                } else {
                    println!(
                        "Error while checking {}, retrying. \nError: {}",
                        e.container.address, e.error
                    );

                    let url = setting.jsonrpc.url_2.clone();
                    let latency = setting.jsonrpc.latency_2.clone();

                    if e.container.count % 2 == 0 {
                        let url = setting.jsonrpc.url_1.clone();
                        let latency = setting.jsonrpc.latency_1.clone();
                    }

                    timers::push_time_rpc(latency);
                    let sleep_time =
                        std::time::Duration::from_millis(timers::get_sleep_time_rpc() as u64);
                    tokio::time::sleep(sleep_time).await;

                    tasks.push(tokio::spawn(async move {
                        jsonrpc::is_contract(e.container, url).await
                    }));
                }
            }
            Ok(v) => {
                if v.is_contract {
                    contracts.insert(v.address, false);
                }
            }
        }
    }

    for (key, value) in contracts {
        println!("{}, {}", key, value);
    }
    /*
    for (ix, entry) in addr_vec.iter().enumerate() {
        if ix > limit && limit != 0 {
            break;
        }
    }
    */
}

fn await_is_contract_thread() {}

fn csv_to_vec(csv_file: &str, min_balance: f32) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut addr_vec: Vec<Entry> = Vec::new();

    let mut rdr = csv::Reader::from_path(csv_file)?;

    for result in rdr.records() {
        let record = result?;

        let address = &record[0];
        let balance: f32 = record[1].parse()?;

        let mut entry = Entry::default();
        entry.balance = balance;
        entry.address = address.to_string();

        if balance > min_balance {
            addr_vec.push(entry);
        }
    }
    Ok(addr_vec)
}

fn is_csv(filename: &str) -> bool {
    match Path::new(filename).extension().and_then(OsStr::to_str) {
        Some(extension) => extension.eq("csv"),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_csv() {
        assert_eq!(is_csv("abc.csv"), true);
    }

    #[test]
    fn test_is_csv_long_path() {
        assert_eq!(is_csv("/opt/c/c/d/abc.csv"), true);
    }

    #[test]
    fn test_is_csv_win_path() {
        assert_eq!(is_csv("C:DERP\\derp\\escape\\dir.csv"), true);
    }

    #[test]
    fn test_is_csv_wrong_filetype() {
        assert_eq!(is_csv("abc.gz"), false);
    }
}
