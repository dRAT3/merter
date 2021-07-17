#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Params {
    String(String),
    Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthRequest {
    jsonrpc: String,
    method: String,
    params: [Params; 2],
    id: i32,
}

#[derive(Debug, Deserialize)]
pub struct EthTransactionObj {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
pub struct EthTransactions {
    transactions: Vec<EthTransactionObj>,
}

#[derive(Debug, Deserialize)]
pub struct EthBlockTxResponse {
    pub result: EthTransactions,
}

#[derive(Debug, Deserialize)]
pub struct EthBlockCtResponse {
    result: String,
}

type Db = Arc<Mutex<HashMap<String, bool>>>;

async fn get_block_addresses() -> Result<EthBlockTxResponse, reqwest::Error> {
    let new_eth_request = EthRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_getBlockByNumber".to_string(),
        params: [Params::String("latest".to_string()), Params::Boolean(true)],
        id: 1,
    };

    let new_eth_response: EthBlockTxResponse = reqwest::Client::new()
        .post(JSONRPCAPI)
        .json(&new_eth_request)
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", new_eth_response);
    //println!("{}", new_eth_response.result.transactions[0].from);

    Ok(new_eth_response)
}

async fn check_if_contract(address: String, db: Db) -> Result<(), reqwest::Error> {
    let new_eth_request = EthRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_getCode".to_string(),
        params: [
            Params::String(address.to_string()),
            Params::String("latest".to_string()),
        ],
        id: 1,
    };
    println!("eth request set-up");

    let new_eth_response: EthBlockCtResponse = reqwest::Client::new()
        .post(JSONRPCAPI)
        .json(&new_eth_request)
        .send()
        .await?
        .json()
        .await?;

    println!("eth request sent");

    println!("{}", new_eth_response.result);

    if new_eth_response.result.chars().count() > 2 {
        let mut db = db.lock().unwrap();
        db.insert(address.to_string(), false);
        Ok(())
    } else {
        Ok(())
    }
}

async fn get_contract_data() {}
