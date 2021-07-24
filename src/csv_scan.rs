use super::settings;

use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Default, Debug)]
struct Entry {
    address: String,
    balance: f32,
}

pub async fn run_csv(chain: &str, min_balance: &f32, limit: &u32, csv_file: &str) {
    //Load settings
    let settings = settings::Settings::new(chain).unwrap_or_else(|err| {
        println!(
            "Couldn't load settings file.
            \nTry running merter --config --{} \n{}",
            err, chain
        );
        std::process::exit(1);
    });
    //Check if passed parameter is a csv file
    if is_csv(csv_file) {
        //Create vector of addresses above minimum treshold
        let mut addr_vec = csv_to_vec(csv_file, min_balance).unwrap_or_else(|err| {
            println!("Error while reading csv file: \n{}", err);
            std::process::exit(1);
        });

        addr_vec.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap());

        for entry in addr_vec {
            println!("{}", entry.address);
        }
    } else {
        println!("{} is not a csv file, exiting", csv_file);
        std::process::exit(1);
    }
}

fn csv_to_vec(csv_file: &str, min_balance: &f32) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut addr_vec: Vec<Entry> = Vec::new();

    let mut rdr = csv::Reader::from_path(csv_file)?;

    for result in rdr.records() {
        let record = result?;

        let address = &record[0];
        let balance: f32 = record[1].parse()?;

        let mut entry = Entry::default();
        entry.balance = balance;
        entry.address = address.to_string();

        if balance > min_balance.to_owned() {
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
