use serde_json::json;
use serde_json::value::Value;
use throttled_bitcoin_rpc::BitcoinRpcClient;

#[macro_use]
extern crate lazy_static;

const FALL_BACK_URL: &'static str = "rpc.blockchain.info";

lazy_static! {
    static ref URL: String = std::env::var("NODE").unwrap_or_else(|_| {
        println!("Falling back for the url to {:?}", FALL_BACK_URL);
        FALL_BACK_URL.into()
    });
}

#[test]
#[cfg(all(feature = "bch", feature = "integration"))]
fn get_block_count() {
    let client = reqwest::Client::new();
    let user = std::env::var("BTC_USER").expect("env: BTC_USER");
    let password = std::env::var("BTC_PASS").expect("env: BTC_PASS");
    let rpc_client = BitcoinRpcClient::new(
        URL.clone().into(),
        Some(user.clone()),
        Some(password.clone()),
        1,
        10,
        1000,
    );

    let raw_response = client
        .post(&URL.clone())
        .basic_auth(user.clone(), Some(password.clone()))
        .json(&json!({
        "method": "getblockcount",
        "params": [
        ]
        }))
        .send()
        .expect("Valid client response")
        .json::<Value>();
    let account_tx = rpc_client.getblockcount();
    assert!(
        account_tx.is_ok(),
        "Getting back an error {:?} from the server given the input , raw was {:?}",
        account_tx,
        raw_response
    );
}
