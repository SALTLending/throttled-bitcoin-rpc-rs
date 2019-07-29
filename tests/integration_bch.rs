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
        .expect("Getting the raw transaction 0");

    let raw_response = reqwest::Client::new()
        .post(&URL.clone())
        .basic_auth(user.clone(), Some(password.clone()))
        .json(&json!({
        "method": "getrawtransaction",
        "params": [
            tx,
            1
        ]
        }))
        .send()
        .expect("Valid client response")
        .json::<Value>();
    println!("Raw Response {:#?}", raw_response);

    rpc_client
        .getrawtransaction(&tx, 1)
        .expect("Getting the raw transaction 1");

    rpc_client
        .getblock(first_generated, true)
        .expect("Getting the block for the generation after tx");
    rpc_client
        .getblock(first_generated, false)
        .expect("Getting the block for the generation after tx");
}

// #[test]
// fn custom_type() {
//     use serde_json::json;

//     let before = json!({"hex":"0100000001704a3855c614c7a7f03d4bde04e3e2c033c886a73852e5e7fac016cd0536428f0000000049483045022100a3966b8d9c33bee457ad5bbba0d8f8acc5bd9d30ef1b93175da6046d96e54ef8022045c8fcd5d83835dee28633b8853d06a9ecf4b847ddd9153797b4335950a2a06941feffffff0200ea0295000000001976a914d20e2943a785ceb711ce4736829019e7a9f80f0288ac00f90295000000001976a914bc1b5c792621392266edcc26801cee96137b4e9c88ac65000000","txid":"5cb7b6b8f68263db6a23f87cd9677cc07856c40e66800668f0730a5f7d939eaf","size":192,"version":1,"locktime":101,"vin":[{"txid":"8f423605cd16c0fae7e55238a786c833c0e2e304de4b3df0a7c714c655384a70","vout":0,"scriptSig":{"asm":"3045022100a3966b8d9c33bee457ad5bbba0d8f8acc5bd9d30ef1b93175da6046d96e54ef8022045c8fcd5d83835dee28633b8853d06a9ecf4b847ddd9153797b4335950a2a069[ALL|FORKID]","hex":"483045022100a3966b8d9c33bee457ad5bbba0d8f8acc5bd9d30ef1b93175da6046d96e54ef8022045c8fcd5d83835dee28633b8853d06a9ecf4b847ddd9153797b4335950a2a06941"},"sequence":4294967294}],"vout":[{"value":24.99996160,"n":0,"scriptPubKey":{"asm":"OP_DUP OP_HASH160 d20e2943a785ceb711ce4736829019e7a9f80f02 OP_EQUALVERIFY OP_CHECKSIG","hex":"76a914d20e2943a785ceb711ce4736829019e7a9f80f0288ac","reqSigs":1,"type":"pubkeyhash","addresses":["bchreg:qrfqu22r57zuadc3eerndq5sr8n6n7q0qgn5750yev"]}},{"value":25.00000000,"n":1,"scriptPubKey":{"asm":"OP_DUP OP_HASH160 bc1b5c792621392266edcc26801cee96137b4e9c OP_EQUALVERIFY OP_CHECKSIG","hex":"76a914bc1b5c792621392266edcc26801cee96137b4e9c88ac","reqSigs":1,"type":"pubkeyhash","addresses":["bchreg:qz7pkhreycsnjgnxahxzdqqua6tpx76wns2ghhm3mh"]}}],"blockhash":"38853fb6a878c3c039c77accdd3df5adee015cd1498aecb404cf59d041a45c82","confirmations":101,"time":1564413711,"blocktime":1564413711});
//     let _after: throttled_bitcoin_rpc::Transaction = serde_json::from_value(before).unwrap();
// }
