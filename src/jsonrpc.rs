#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Params {
    String(String),
    Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize)]
struct EthRequest {
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

///Represents a response from is_contract() contains is_contract true if it is a contract
pub struct IsContractResponse {
    pub address: String,
    pub is_contract: bool,
    pub count: u8,
}

pub struct IsContractErr {
    pub container: IsContractResponse,
    pub error: reqwest::Error,
}

pub async fn is_contract(
    container: IsContractResponse,
    json_rpc_api: String,
) -> Result<IsContractResponse, IsContractErr> {
    let mut container = container;
    container.count += 1;

    let res = match is_contract_reqwest(&container.address, json_rpc_api).await {
        Ok(res) => res,
        Err(err) => {
            return Err(IsContractErr {
                container: container,
                error: err,
            })
        }
    };

    if res.result.chars().count() > 2 {
        container.is_contract = true;
    }

    Ok(container)
}

pub async fn is_contract_reqwest(
    address: &str,
    json_rpc_api: String,
) -> Result<EthBlockCtResponse, reqwest::Error> {
    let new_eth_request = EthRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_getCode".to_string(),
        params: [
            Params::String(address.to_string()),
            Params::String("latest".to_string()),
        ],
        id: 1,
    };

    let new_eth_response: EthBlockCtResponse = reqwest::Client::new()
        .post(&json_rpc_api)
        .json(&new_eth_request)
        .send()
        .await?
        .json()
        .await?;

    Ok(new_eth_response)
}

async fn get_latest_block(json_rpc_api: &str) -> Result<EthBlockTxResponse, reqwest::Error> {
    let new_eth_request = EthRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_getBlockByNumber".to_string(),
        params: [Params::String("latest".to_string()), Params::Boolean(true)],
        id: 1,
    };

    let new_eth_response: EthBlockTxResponse = reqwest::Client::new()
        .post(json_rpc_api)
        .json(&new_eth_request)
        .send()
        .await?
        .json()
        .await?;

    println!("{:#?}", new_eth_response);
    //println!("{}", new_eth_response.result.transactions[0].from);

    Ok(new_eth_response)
}
