#![allow(missing_docs, unused_variables, trivial_casts)]

#[allow(unused_extern_crates)]
#[macro_use]
extern crate swagger;
#[allow(unused_extern_crates)]
extern crate futures;
#[macro_use]
extern crate swagger_client;
#[allow(unused_extern_crates)]
extern crate uuid;
extern crate clap;
extern crate tokio_core;
extern crate crypto;
extern crate hex;
extern crate blake2b;
extern crate rand;

extern crate curl;
use curl::easy::Easy;

extern crate rust_base58;
use rust_base58::{ToBase58, FromBase58};

extern crate rust_sodium;

extern crate serde_json;
use serde_json::{Value, Error};

use rand::prelude::*;

use swagger::{ContextBuilder, EmptyContext, XSpanIdString, Has, Push, AuthData};

#[allow(unused_imports)]
use futures::{Future, future, Stream, stream};
use tokio_core::reactor;

pub mod transaction;

use transaction::KeyPair;

use swagger_client::{ ApiNoContext, ContextWrapperExt,Api, ApiError,
                      CallContractResponse, CompileContractResponse,
                      EncodeCalldataResponse,
                      GetAccountBalanceResponse,
                      GetAccountsBalancesResponse,
                      GetBlockByHashResponse,
                      GetBlockByHeightResponse,
                      GetBlockGenesisResponse, GetBlockLatestResponse,
                      GetBlockPendingResponse,
                      GetCommitmentHashResponse,
                      GetContractCallFromTxResponse,
                      GetHeaderByHashResponse,
                      GetHeaderByHeightResponse, GetInfoResponse,
                      GetNameResponse, GetPeerKeyResponse,
                      GetTopResponse, GetTxResponse, GetTxsResponse,
                      GetVersionResponse, PostBlockResponse,
                      PostChannelCloseMutualResponse,
                      PostChannelCloseSoloResponse,
                      PostChannelCreateResponse,
                      PostChannelDepositResponse,
                      PostChannelSettleResponse,
                      PostChannelSlashResponse,
                      PostChannelWithdrawalResponse,
                      PostContractCallResponse,
                      PostContractCallComputeResponse,
                      PostContractCreateResponse,
                      PostNameClaimResponse, PostNamePreclaimResponse,
                      PostNameRevokeResponse,
                      PostNameTransferResponse,
                      PostNameUpdateResponse,
                      PostOracleExtendResponse,
                      PostOracleQueryResponse,
                      PostOracleRegisterResponse,
                      PostOracleResponseResponse, PostSpendResponse,
                      PostTxResponse,
                      GetActiveRegisteredOraclesResponse,
                      GetBlockNumberResponse,
                      GetBlockTxsCountByHashResponse,
                      GetBlockTxsCountByHeightResponse,
                      GetGenesisBlockTxsCountResponse,
                      GetLatestBlockTxsCountResponse,
                      GetOracleQuestionsResponse, GetPeersResponse,
                      GetPendingBlockTxsCountResponse,
                      GetPubKeyResponse,
                      GetTransactionFromBlockHashResponse,
                      GetTransactionFromBlockHeightResponse,
                      GetTransactionFromBlockLatestResponse,
                      GetTxsListFromBlockRangeByHashResponse,
                      GetTxsListFromBlockRangeByHeightResponse,
                      PostNameClaimTxResponse,
                      PostNamePreclaimTxResponse,
                      PostNameRevokeTxResponse,
                      PostNameTransferTxResponse,
                      PostNameUpdateTxResponse,
                      PostOracleExtendTxResponse,
                      PostOracleQueryTxResponse,
                      PostOracleRegisterTxResponse,
                      PostOracleResponseTxResponse,
                      PostSpendTxResponse };

pub struct Epoch {
    base_uri: String,
    client: swagger_client::client::Client,
    context: swagger_client::Context,
}

impl Epoch {
    fn new(base_url: String) -> Epoch {
        let mut core = reactor::Core::new().unwrap();
        let mut client;
        if base_url.starts_with("https://") {
            client = swagger_client::client::Client::try_new_https(&base_url, "test/ca.pem").expect("Failed to connect");
        } else {
            client = swagger_client::client::Client::try_new_http(&base_url).expect("Failed to connect");
        }
        let context = swagger_client::Context::new_with_span_id(self::uuid::Uuid::new_v4().to_string());

        Epoch { client: client, context: context, base_uri: base_url } }

    fn top(&self) -> i64 {

        let future_val: std::boxed::Box<futures::Future<Error=swagger_client::ApiError, Item=swagger_client::GetTopResponse>> =
            self.client.get_top(&self.context);
        let topresult: swagger_client::GetTopResponse =
            future_val.wait().unwrap();
        match topresult {
            swagger_client::GetTopResponse::SuccessfulOperation(op) =>
                op.height.unwrap_or(0),
        }
    }

    fn get_block_at_height(&self, height: i64) ->
        Option<serde_json::Value> {
/*            
            let val =
                self.client.get_block_by_height(height as i32, Some(String::from("json")),
                                                &self.context).wait();
            let result: swagger_client::GetBlockByHeightResponse;
            match val {
                Ok(res) => result = res,
                Err(_) => return None,
            }
            match result {
                swagger_client::GetBlockByHeightResponse::TheBlockBeingFound(block) =>
                    return Some(block),
                swagger_client::GetBlockByHeightResponse::BlockNotFound(_) =>
                    return None,
            }
        }
             */
            let uri = self.base_uri.clone() + "/v2/block/height/" + &height.to_string();
            let mut data = Vec::new();
            let mut handle = Easy::new();
            handle.url(&uri).unwrap();
            {
                let mut transfer = handle.transfer();
                transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                }).unwrap();
                transfer.perform().unwrap();
            }
            let value: Value = serde_json::from_str(std::str::from_utf8(&data).unwrap()).unwrap();
            Some(value)
        }
}
    
fn main() {
    let epoch = Epoch::new(String::from("https://sdk-testnet.aepps.com"));
    println!("Top: {:?}", epoch.top());
    for x in 1360 .. epoch.top() {
        let block;
        match epoch.get_block_at_height(x) {
            Some(res) => block = res,
            None => {
                println!("Failed at block {}", x);
                continue
            }
        }
        println!("Tx: {:?}", block["height"]);
    }
        
}

#[cfg(test)]
mod tests {
    use transaction::KeyPair;
    #[test]
    fn test_read_sign_verify() {
        // Read a key pair from a file (these were generated by the JS
        // SDK so this also tests ineroperability. Sign and check
        // verification works
        let key_pair = KeyPair::read_from_files(&String::from("test/keys/testkey.pub"),
                                                &String::from("test/keys/testkey"),
                                                &String::from(""));
        let msg = b"this is a test thing";
        let mut bytes = key_pair.sign(msg).unwrap();
        println!("Sig: {:?}", KeyPair::bytes_to_hex(bytes));
        key_pair.verify(&bytes, msg).unwrap();
    }
    #[test]
    #[should_panic(expected = "Verification failed")]
    fn test_generate_sign_verify() {
        // generate 2 key pairs. Generate with one, verify with the
        // other. Should blow up!
        let key_pair = KeyPair::generate().unwrap();
        let new_key_pair = KeyPair::generate().unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        key_pair.verify(&bytes, msg).unwrap();
    }

    #[test]
    fn test_write_sign_verify() {
        // generate a key pair, write it to a file. Read from the file
        // into a new variable, sign with one and check that
        // verification with the other works
        let new_key_pair = KeyPair::generate().unwrap();
        new_key_pair.write_to_files(&String::from("test/keys/new.pub"),
                                    &String::from("test/keys/new")).unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        let loaded_key_pair = KeyPair::read_from_files(&String::from("test/keys/new.pub"),
                                                       &String::from("test/keys/new"),
                                                       &String::from(""));
        loaded_key_pair.verify(&bytes, msg).unwrap();
    }
    
}

