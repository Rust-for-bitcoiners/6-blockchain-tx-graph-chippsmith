mod graph;
mod profile_transactions;

use std::str::FromStr;

use bitcoincore_rpc::RpcApi;
use graph::Graph;
use profile_transactions::build_transaction_graph;

//#[cfg(feature = "download")]
//use bitcoincore_rpc::RpcApi;

#[macro_use]
extern crate lazy_static;

fn main() {
    println!("hello ")
}
