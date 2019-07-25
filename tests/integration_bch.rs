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
fn get_static_infos_count() {
    let client = reqwest::Client::new();
    let user = std::env::var("USER").expect("env: USER");
    let password = std::env::var("PASS").expect("env: PASS");
    let rpc_client = BitcoinRpcClient::new(
        URL.clone().into(),
        Some(user.clone()),
        Some(password.clone()),
        1,
        10,
        1000,
    );

    let account_tx = rpc_client.getblockcount();
    assert!(
        account_tx.is_ok(),
        "Getting back an error {:?} from the server given the input",
        account_tx
    );

    let raw_response = client
        .post(&URL.clone())
        .basic_auth(user.clone(), Some(password.clone()))
        .json(&json!({
        "method": "getblockhash",
        "params": [
            0
        ]
        }))
        .send()
        .expect("Valid client response")
        .json::<Value>();
    let account_tx = rpc_client.getblockhash(0);
    assert!(
        account_tx.is_ok(),
        "Getting back an error {:?} from the server given the input , raw was {:?}",
        account_tx,
        raw_response
    );
}

#[test]
#[cfg(all(feature = "bch", feature = "integration"))]
fn running_through_transaction() {
    let user = std::env::var("USER").expect("env: USER");
    let password = std::env::var("PASS").expect("env: PASS");
    let rpc_client = BitcoinRpcClient::new(
        URL.clone().into(),
        Some(user.clone()),
        Some(password.clone()),
        1,
        10,
        1000,
    );
    let address = rpc_client
        .getnewaddress(None, None)
        .expect("Getting an address");
    let account_balance_before = rpc_client
        .getreceivedbyaddress(&address, 0)
        .expect("Getting balance before");

    rpc_client.generate(101, None).expect("Generating 101");
    let tx = rpc_client
        .sendtoaddress(&address, 25.0, None, None, None)
        .expect("Sending 25 to our address");
    let after_generations = rpc_client.generate(10, None).expect("Generating 1");
    let first_generated = &after_generations[0];

    let account_balance = rpc_client
        .getreceivedbyaddress(&address, 0)
        .expect("Getting balance");

    assert!(
        (25.0 - (account_balance - account_balance_before)).abs() < 0.01,
        "Current balance delta for our account, {} - {} = {}",
        account_balance,
        account_balance_before,
        (account_balance - account_balance_before)
    );

    rpc_client
        .getrawtransaction(&tx, 0)
        .expect("Getting the raw transaction");

    rpc_client
        .getblock(first_generated, true)
        .expect("Getting the block for the generation after tx");
}
