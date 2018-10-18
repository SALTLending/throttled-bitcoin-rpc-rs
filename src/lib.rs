#[macro_use] extern crate failure;
extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

#[macro_use] mod macros;

use std::collections::HashMap;

pub type SerializedData = String;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub hash: String,
    pub confirmations: i64,
    pub size: i64,
    pub height: i64,
    pub version: i64,
    pub merkleroot: String,
    pub tx: Vec<String>,
    pub time: i64,
    pub nonce: i64,
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
    pub confirmations: i64,
    pub strippedsize: i64,
    pub size: i64,
    pub weight: i64,
    pub height: i64,
    pub version: i64,
    pub version_hex: String,
    pub merkleroot: String,
    pub tx: Vec<Transaction>,
    pub time: i64,
    pub mediantime: i64,
    pub nonce: i64,
    pub bits: String,
    pub difficulty: serde_json::Number,
    pub chainwork: String,
    pub previousblockhash: Option<String>,
    pub nextblockhash: Option<String>,
}

#[cfg(not(feature = "doge"))]
#[derive(Deserialize, Clone, Debug)]
pub struct Transaction {
    pub txid: String,
    pub hash: String,
    pub version: i64,
    pub size: i64,
    pub vsize: i64,
    pub locktime: i64,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: Option<String>,
    pub confirmations: Option<i64>,
    pub time: Option<i64>,
    pub blocktime: Option<i64>,
}

#[cfg(feature = "doge")]
#[derive(Deserialize, Clone, Debug)]
pub struct Transaction {
    pub txid: String,
    pub version: i64,
    pub locktime: i64,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: Option<String>,
    pub confirmations: Option<i64>,
    pub time: Option<i64>,
    pub blocktime: Option<i64>,
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
    pub vout: i64,
    pub script_sig: ScriptSig,
    pub txinwitness: Option<Vec<String>>,
    pub sequence: i64
}

#[derive(Deserialize, Clone, Debug)]
pub struct VinCoinbase {
    pub coinbase: String,
    pub sequence: i64
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    pub value: serde_json::Number,
    pub n: i64,
    pub script_pub_key: ScriptPubKey,
}

#[cfg(all(not(feature = "ltc"), not(feature = "doge")))]
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GetBlockReply {
    Zero(SerializedData),
    One(Block),
    Two(FullBlock)
}

#[cfg(any(feature = "ltc", feature = "doge"))]
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GetBlockReply {
    False(SerializedData),
    True(Block),
}

#[derive(Deserialize, Clone, Debug)]
pub struct Enforce {
    pub status: bool,
    pub found: i64,
    pub required: i64,
    pub window: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Reject {
    pub status: bool,
    pub found: i64,
    pub required: i64,
    pub window: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Softfork {
    pub id: String,
    pub version: i64,
    pub enforce: Enforce,
    pub reject: Reject,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockChainInfo {
    pub chain: String,
    pub blocks: i64,
    pub headers: i64,
    pub bestblockhash: String,
    pub difficulty: serde_json::Number,
    pub mediantime: i64,
    pub verificationprogress: serde_json::Number,
    pub chainwork: String,
    pub pruned: bool,
    pub softforks: Vec<Softfork>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Tip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MemPoolInfo {
    pub size: i64,
    pub bytes: i64,
    pub usage: i64,
    pub maxmempool: i64,
    pub mempoolminfee: serde_json::Number,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScriptPubKey {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<i64>,
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
    pub confirmations: i64,
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
    pub height: i64,
    pub bestblock: String,
    pub transactions: i64,
    pub txouts: i64,
    pub bytes_serialized: i64,
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
    pub size: i64,
    pub fee: serde_json::Number,
    pub modifiedfee: serde_json::Number,
    pub time: i64,
    pub height: i64,
    pub descendantcount: i64,
    pub descendantsize: i64,
    pub descendantfees: i64,
    pub ancestorcount: i64,
    pub ancestorsize: i64,
    pub ancestorfees: i64,
    pub wtxid: String,
    pub depends: Vec<String>
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
    pub vout: i32,
    #[serde(rename="Sequence")]
    pub sequence: Option<i32>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct TxOutput {
    pub txid: String,
    pub vout: i32,
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
    pub fn createrawtransaction(&mut self, inputs: Vec<TxInput>, outputs: HashMap<String, f64>, locktime: Option<i32>) -> Result<String>;
    pub fn dumpprivkey(&mut self, address: String) -> Result<String>;
    pub fn generate(&mut self, number: i32, iterations: Option<i32>) -> Result<Vec<String>>;
    #[cfg(all(not(feature = "ltc"), not(feature = "doge")))] pub fn getblock(&mut self, header_hash: String, verbosity: i32) -> Result<GetBlockReply>;
    #[cfg(any(feature = "ltc", feature = "doge"))] pub fn getblock(&mut self, header_hash: String, verbosity: bool) -> Result<GetBlockReply>;
    pub fn getblockchaininfo(&mut self) -> Result<BlockChainInfo>;
    pub fn getblockcount(&mut self) -> Result<i64>;
    pub fn getblockhash(&mut self, block_height: i64) -> Result<String>;
    pub fn getnewaddress(&mut self, account: Option<String>, address_type: Option<String>) -> Result<String>;
    pub fn getrawmempool(&mut self, format: bool) -> Result<RawMemPool>;
    #[cfg(all(not(feature = "ltc"), not(feature = "doge")))] pub fn getrawtransaction(&mut self, txid: String, verbose: bool) -> Result<GetRawTransactionReply>;
    #[cfg(any(feature = "ltc", feature = "doge"))] pub fn getrawtransaction(&mut self, txid: String, verbose: i32) -> Result<GetRawTransactionReply>;
    pub fn gettxout(&mut self, txid: String, vout: i64, unconfirmed: bool) -> Result<GetTxOutReply>;
    pub fn sendrawtransaction(&mut self, transaction: String, allow_high_fee: Option<bool>) -> Result<String>;
    pub fn sendtoaddress(&mut self, address: String, amount: f64, comment: Option<String>, comment_to: Option<String>, include_fee: Option<bool>) -> Result<String>;
    pub fn signrawtransaction(&mut self, transaction: String, outputs: Option<Vec<TxOutput>>, privkeys: Option<Vec<String>>, sig_hash_type: Option<String>) -> Result<SignedTx>;
});

