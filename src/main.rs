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

extern crate rust_base58;
use rust_base58::{ToBase58, FromBase58};

extern crate rust_sodium;

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
                      GetAccountTransactionsResponse,
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

use clap::{App,Arg};

pub struct Epoch {
    client: swagger_client::client::Client,
    context: swagger_client::Context,
}

impl Epoch {
    fn new(base_url: String) -> Epoch {
        let mut core = reactor::Core::new().unwrap();
        let client = swagger_client::client::Client::try_new_http(&base_url).expect("Failed to connect");
        let context = swagger_client::Context::new_with_span_id(self::uuid::Uuid::new_v4().to_string());
        Epoch { client: client, context: context }
    }

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
}
    


fn main() {
//    let epoch = Epoch::new(String::from("http:localhost:3013"));
    //    println!("Top: {:?}", epoch.top());

}

#[cfg(test)]
mod tests {
    use transaction::KeyPair;
    #[test]
    fn test_read_sign_verify() {
        let key_pair = KeyPair::read_from_files(&String::from("test/keys/testkey.pub"),
                                                &String::from("test/keys/testkey"),
                                                &String::from(""));
        let msg = b"this is a test thing";
        let mut bytes = key_pair.sign(msg).unwrap();
        println!("Sig: {:?}", KeyPair::bytes_to_hex(bytes));
        key_pair.verify(&bytes, msg).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_generate_sign_verify() {        
        let key_pair = KeyPair::generate().unwrap();
        let new_key_pair = KeyPair::generate().unwrap();
        let msg  =b"this is a test thing";
        let bytes = new_key_pair.sign(msg).unwrap();
        key_pair.verify(&bytes, msg).unwrap();
    }

    #[test]
    fn test_write_sign_verify() {
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

