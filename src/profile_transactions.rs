use bitcoin::hash_types::Txid;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::env;

use hex as hex2;

use super::graph::Graph;

lazy_static! {
    static ref RPC_CLIENT: Client = {
        dotenv::dotenv().ok();
        let rpc_url: String = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set");
        let rpc_user: String = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set");
        let rpc_password: String =
            env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set");
        Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)).unwrap()
    };
}

pub fn build_transaction_graph(start_height: u64, end_height: u64, client: &Client) -> Graph<Txid> {
    // Every Transaction has a set of Inputs and outputs
    // Each Input refers to an output of some earlier transaction
    // We say a Transaction A funds Transaction B if an ouput of A is an input of B
    // Build a graph where vertex represents Txid and an edge (t1, t2) is in the graph
    // if the transaction t1 funds transaction t2
    let mut tx_graph: Graph<Txid> = Graph::new();

    for block_height in start_height..=end_height {
        if block_height % 10 == 0 {
            println!("{}", block_height);
        }
        let block_hash = (client)
            .get_block_hash(block_height)
            .expect("error getting block hash");
        let block = (client)
            .get_block(&block_hash)
            .expect("error getting block");
        let txs = block.txdata;

        for tx in txs {
            let inputs = &tx.input;
            for input in inputs {
                let input_tx_id = input.previous_output.txid;
                let tx_id = tx.compute_txid();
                tx_graph.insert_edge(input_tx_id, tx_id);
            }
        }
    }
    tx_graph
}

#[cfg(test)]
mod tests {
    use core::num;

    use bitcoin::{block, consensus::{serde::hex, Decodable}, Address, AddressType, Amount, OutPoint, Transaction};
    use bitcoincore_rpc::jsonrpc::client;
    use bitcoind;

    use super::*;

    fn get_address(client: &Client) -> Address {
        client
            .get_new_address(
                None,
                Some(bitcoincore_rpc::bitcoincore_rpc_json::AddressType::Bech32),
            )
            .unwrap()
            .assume_checked()
    }

    fn generate_blocks(client: &Client, address: Address, number_of_blocks: u64) -> () {
        client
            .generate_to_address(number_of_blocks, &address)
            .unwrap();
    }

    #[test]
    fn test_coinbase() {
        let bitcoind = bitcoind::BitcoinD::from_downloaded().unwrap();
        let client = &bitcoind.client;
        
        let address = get_address(client);

        let number_of_blocks = 100;

        generate_blocks(client, address, number_of_blocks);

        let graph = build_transaction_graph(1, number_of_blocks, client);
        assert_eq!(100, bitcoind.client.get_blockchain_info().unwrap().blocks);
        assert_eq!(1, graph.number_of_vertices());

        let coin_base_tx_id = OutPoint::null().txid;
        assert_eq!(
            number_of_blocks,
            graph.neighbors(&coin_base_tx_id).len() as u64
        );
        assert_eq!(coin_base_tx_id, *graph.vertices()[0]);
    }

    #[test]

    fn test_transactions() {
        let bitcoind = bitcoind::BitcoinD::from_downloaded().unwrap();
        let client = &bitcoind.client;

        let address = get_address(client);

        let number_of_blocks = 102;
        let number_of_transactions = 10;
        generate_blocks(client, address, number_of_blocks);

        let address2 = get_address(client);

        let mut v: Vec<Txid> = Vec::with_capacity(number_of_transactions);

        for i in 1..number_of_transactions + 1 {
            let txid = client
                .send_to_address(
                    &address2,
                    Amount::from_btc(i as f64).unwrap(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            v.push(txid);
        }
        client.generate_to_address(1, &get_address(client)).unwrap();
        let graph = build_transaction_graph(0 , number_of_blocks + 1, client);

        assert_eq!(11, graph.number_of_vertices());

        for txid in v {
            assert!(graph.contains_vertex(&txid))
        }
    }

    #[test]
    fn test_path_exists() {

        let bitcoind = bitcoind::BitcoinD::from_downloaded().unwrap();
        let client = &bitcoind.client;

        let address = get_address(client);

        let number_of_blocks = 102;
        generate_blocks(client, address, number_of_blocks);

        let txid = client.send_to_address(&get_address(client), Amount::from_sat(100000), None, None, None, None, None, None).unwrap();
        client.generate_to_address(1, &get_address(client)).unwrap();
        let graph = build_transaction_graph(0, number_of_blocks + 1, client);  

        let block_hash = client.get_block_hash(number_of_blocks + 1).unwrap();
        let block = client.get_block(&block_hash).unwrap();
        let tx = &block.txdata[1];


        assert!(graph.path_exists_between(&tx.input[0].previous_output.txid, &txid))
    }
}
