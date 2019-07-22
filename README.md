# Throttled Bitcoin RPC Client
This crate started as a fork of JeanDudey's Bitcoin RPC client, but has since been almost entirely rewritten.

This crate implements an Bitcoin RPC client in Rust, it does not intend to be a complete implementation of all the bitcoin rpc methods so if you need some method you can create a pull request for it.



## AltCoins
Works with LTC and DOGE using relevant compiler flags.

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
throttled_bitcoin_rpc = "0.1.0"
```

And this to your crate root:
```rust
extern crate throttled_bitcoin_rpc;
```

### Example: Connecting to bitcoin rpc server
```rust
extern crate throttled_bitcoin_rpc;

use throttled_bitcoin_rpc::BitcoinRpcClient;

fn main() {
    let client = BitcoinRpcClient::new(
        "example.org:8331",
        Some("bitcoin".to_owned()),
        Some("local321".to_owned()),
        3, // max number of concurrent requests
        10, // max number of requests per second
        1000, // max size of batched requests
    );

    let block_count = match client.getblockcount().unwrap();

    println!("Block count: {}", block_count);
    
    let txs = vec![
        "02617e68e8c3e3fa8763502c0e701bf5af1e8f57835b9bef1ee333b0fcf2527",
        "969947e164bbfca77cb09eab343b192cb5605bfa0483b4d2a3ec626e55ad70bc",
        "743f6202f89acc41adce4496244f152833ffb9f1f7a6d6e6fc94d85580ac9461",
    ];
    
    use throttled_bitcoin_rpc::BatchRequest;
    let mut batcher = client::batcher::<String>();
    
    for txid in txs {
        batcher.getrawtransaction(txid.to_owned(), false).unwrap();
    }
    
    println!("Raw TxData: {:?}", batcher.send().unwrap());
}
```

### Integration Testing
So we want to use docker so that way the 'dev' setup isn't magic and is explicit and reproducable. Then we take the (BCH) and do some integration testing on them.

We utilzie the following command in order to get dynamic testing done..
`docker-compose -f bch-docker/compose-bch-integration.yml down; docker-compose -f bch-docker/compose-bch-integration.yml up`