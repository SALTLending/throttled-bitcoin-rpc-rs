#[macro_use]
extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

#[macro_use]
mod macros;

use std::collections::HashMap;

pub type SerializedData = String;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub hash: String,
    pub confirmations: serde_json::Number,
    pub size: serde_json::Number,
    pub height: serde_json::Number,
    pub version: serde_json::Number,
    pub merkleroot: String,
    pub tx: Vec<String>,
    pub time: serde_json::Number,
    pub nonce: serde_json::Number,
    pub bits: String,
    pub difficulty: serde_json::Number,
    pub chainwork: String,
    pub previousblockhash: Option<String>,
    pub nextblockhash: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FullBlock {
    pub hash: String,
    pub confirmations: serde_json::Number,
    pub strippedsize: serde_json::Number,
    pub size: serde_json::Number,
    pub weight: serde_json::Number,
    pub height: serde_json::Number,
    pub version: serde_json::Number,
    pub version_hex: String,
    pub merkleroot: String,
    pub tx: Vec<Transaction>,
    pub time: serde_json::Number,
    pub mediantime: serde_json::Number,
    pub nonce: serde_json::Number,
    pub bits: String,
    pub difficulty: serde_json::Number,
    pub chainwork: String,
    pub previousblockhash: Option<String>,
    pub nextblockhash: Option<String>,
}

#[cfg(not(any(feature = "doge", feature = "dash")))]
#[derive(Deserialize, Clone, Debug)]
pub struct Transaction {
    pub txid: String,
    pub hash: String,
    pub version: serde_json::Number,
    pub size: serde_json::Number,
    pub vsize: serde_json::Number,
    pub locktime: serde_json::Number,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: Option<String>,
    pub confirmations: Option<serde_json::Number>,
    pub time: Option<serde_json::Number>,
    pub blocktime: Option<serde_json::Number>,
}

#[cfg(feature = "dash")]
#[derive(Deserialize, Clone, Debug)]
pub struct Transaction {
    pub txid: String,
    pub version: serde_json::Number,
    pub locktime: serde_json::Number,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: Option<String>,
    pub confirmations: Option<serde_json::Number>,
    pub time: Option<serde_json::Number>,
    pub blocktime: Option<serde_json::Number>,
    pub height: Option<serde_json::Number>,
    #[serde(default)]
    pub instantlock: bool,
    pub size: serde_json::Number,
}

#[cfg(feature = "doge")]
#[derive(Deserialize, Clone, Debug)]
pub struct Transaction {
    pub txid: String,
    pub version: serde_json::Number,
    pub locktime: serde_json::Number,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: Option<String>,
    pub confirmations: Option<serde_json::Number>,
    pub time: Option<serde_json::Number>,
    pub blocktime: Option<serde_json::Number>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Vin {
    Coinbase(VinCoinbase),
    Tx(VinTx),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VinTx {
    pub txid: String,
    pub vout: serde_json::Number,
    pub script_sig: ScriptSig,
    pub txinwitness: Option<Vec<String>>,
    pub sequence: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VinCoinbase {
    pub coinbase: String,
    pub sequence: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    pub value: serde_json::Number,
    pub n: serde_json::Number,
    pub script_pub_key: ScriptPubKey,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Enforce {
    pub status: bool,
    pub found: serde_json::Number,
    pub required: serde_json::Number,
    pub window: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Reject {
    pub status: bool,
    pub found: serde_json::Number,
    pub required: serde_json::Number,
    pub window: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Softfork {
    pub id: String,
    pub version: serde_json::Number,
    pub enforce: Enforce,
    pub reject: Reject,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockChainInfo {
    pub chain: String,
    pub blocks: serde_json::Number,
    pub headers: serde_json::Number,
    pub bestblockhash: String,
    pub difficulty: serde_json::Number,
    pub mediantime: serde_json::Number,
    pub verificationprogress: serde_json::Number,
    pub chainwork: String,
    pub pruned: bool,
    pub softforks: Vec<Softfork>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Tip {
    pub height: serde_json::Number,
    pub hash: String,
    pub branchlen: serde_json::Number,
    pub status: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MemPoolInfo {
    pub size: serde_json::Number,
    pub bytes: serde_json::Number,
    pub usage: serde_json::Number,
    pub maxmempool: serde_json::Number,
    pub mempoolminfee: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<serde_json::Number>,
    #[serde(rename = "type")]
    pub script_type: String,
    pub addresses: Option<Vec<String>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxOut {
    pub bestblock: String,
    pub confirmations: serde_json::Number,
    pub value: serde_json::Number,
    pub script_pub_key: ScriptPubKey,
    pub coinbase: bool,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GetTxOutReply {
    Null(()),
    TxOut(TxOut),
}

#[derive(Deserialize, Clone, Debug)]
pub struct TxOutSetInfo {
    pub height: serde_json::Number,
    pub bestblock: String,
    pub transactions: serde_json::Number,
    pub txouts: serde_json::Number,
    pub bytes_serialized: serde_json::Number,
    pub hash_serialized: String,
    pub total_amount: serde_json::Number,
}

#[cfg(all(not(feature = "ltc"), not(feature = "doge")))]
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GetRawTransactionReply {
    False(SerializedData),
    True(Transaction),
}

#[cfg(any(feature = "ltc", feature = "doge"))]
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GetRawTransactionReply {
    Zero(SerializedData),
    One(Transaction),
}

#[derive(Deserialize, Clone, Debug)]
pub struct MemPoolTx {
    pub size: serde_json::Number,
    pub fee: serde_json::Number,
    pub modifiedfee: serde_json::Number,
    pub time: serde_json::Number,
    pub height: serde_json::Number,
    pub descendantcount: serde_json::Number,
    pub descendantsize: serde_json::Number,
    pub descendantfees: serde_json::Number,
    pub ancestorcount: serde_json::Number,
    pub ancestorsize: serde_json::Number,
    pub ancestorfees: serde_json::Number,
    pub wtxid: String,
    pub depends: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RawMemPool {
    True(HashMap<String, MemPoolTx>),
    False(Vec<String>),
}

#[derive(Serialize, Clone, Debug)]
pub struct TxInput {
    pub txid: String,
    pub vout: serde_json::Number,
    #[serde(rename = "Sequence")]
    pub sequence: Option<serde_json::Number>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxOutput {
    pub txid: String,
    pub vout: serde_json::Number,
    pub script_pub_key: String,
    pub redeem_script: Option<String>,
    pub amount: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SignedTx {
    pub hex: String,
    pub complete: bool,
}

jsonrpc_client!(pub struct BitcoinRpcClient {
    single:
        pub fn createrawtransaction(&self, inputs: Vec<TxInput>, outputs: HashMap<String, f64>, locktime: Option<isize>) -> Result<String>;
        pub fn dumpprivkey(&self, address: String) -> Result<String>;
        pub fn generate(&self, number: isize, iterations: Option<isize>) -> Result<Vec<String>>;
        pub fn getbalance(&self) -> Result<f64>;
        pub fn getreceivedbyaddress(&self, address: &str, confirmations: isize) -> Result<f64>;
        pub fn getblockchaininfo(&self) -> Result<BlockChainInfo>;
        pub fn getblockcount(&self) -> Result<isize>;
        pub fn getblockhash(&self, block_height: isize) -> Result<String>;
        pub fn getnewaddress(&self, account: Option<String>, address_type: Option<String>) -> Result<String>;
        pub fn getrawmempool(&self, format: bool) -> Result<RawMemPool>;
        pub fn sendrawtransaction(&self, transaction: String, allow_high_fee: Option<bool>) -> Result<String>;
        pub fn sendtoaddress(&self, address: String, amount: f64, comment: Option<String>, comment_to: Option<String>, include_fee: Option<bool>) -> Result<String>;
        pub fn signrawtransaction(&self, transaction: String, outputs: Option<Vec<TxOutput>>, privkeys: Option<Vec<String>>, sig_hash_type: Option<String>) -> Result<SignedTx>;
        pub fn gettxout(&self, txid: String, vout: isize, unconfirmed: bool) -> Result<Option<TxOut>>;
    enum:
        #[cfg(all(not(feature = "ltc"), not(feature = "bch"), not(feature = "doge")))] pub fn getblock(&self, header_hash: String, verbosity: isize) -> Result<Zero(SerializedData)|One(Block)|Two(FullBlock)>;
        #[cfg(any(feature = "ltc", feature = "bch", feature = "doge"))] pub fn getblock(&self, header_hash: String, verbosity: bool) -> Result<False(SerializedData)|True(Block)>;
        #[cfg(all(not(feature = "ltc"), not(feature = "bch"), not(feature = "doge")))] pub fn getrawtransaction(&self, txid: String, verbose: bool) -> Result<False(SerializedData)|True(Transaction)>;
        #[cfg(any(feature = "ltc", feature = "bch", feature = "doge"))] pub fn getrawtransaction(&self, txid: String, verbose: isize) -> Result<Zero(SerializedData)|One(Transaction)>;
});
