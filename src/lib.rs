//! This file contains the main logic of the node server
//! The node server is responsible for handling incoming transactions, mining new blocks, and broadcasting the new blocks to the rest of the network
//! The node server is also responsible for introducing new nodes to the network
pub mod models;
pub mod node {
    tonic::include_proto!("node");
}
use crate::models::{blockchain::mine_new_block, network::Network, node::Node};
use node::node_message_client::NodeMessageClient;
use node::{
    node_message_server::NodeMessageServer, Block, JoinNetworkRequest, NodeInfo,
    UpdateBlockchainRequest,
};
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinSet};
use tonic::{transport::Server, Request};

/// Start the node server
pub async fn start(port: u16, peer_port: Option<u16>) {
    let port: u32 = port as u32;
    let (tx, rx) = mpsc::channel::<bool>(1);
    let node = Arc::new(Node::new(port));
    let network = Network {
        node: node.clone(),
        tx,
    };
    let node_info = NodeInfo {
        id: node.id.to_string(),
        ip: node.ip.clone(),
        port: node.port,
    };

    // if the node is not the master node, then should introduce itself to every node in the network
    if let Some(peer) = peer_port {
        // connect to the peer node
        let mut client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", peer))
            .await
            .expect("Failed to connect to peer node");
        let res = client
            .join_network(Request::new(JoinNetworkRequest {
                node: Some(node_info.clone()),
            }))
            .await
            .expect("Failed to join network on peer node");

        // get the peer list and the blockchain from the peer node
        let res = res.into_inner();
        let peer_list = res.nodes;
        node.peers.lock().await.extend(peer_list.clone());
        node.blockchain.lock().await.chain = res.chain;
        node.blockchain.lock().await.transactions = res.transactions;

        // broadcast the new node to the rest of the network
        let mut broadcast = JoinSet::new();

        peer_list.into_iter().for_each(|node| {
            if node.port == port || node.port == peer as u32 {
                return;
            }
            let node_info = node_info.clone();
            broadcast.spawn(async move {
                println!("[INFO] Broadcasting to node: {:?}", node.port);
                let mut client =
                    NodeMessageClient::connect(format!("http://{}:{}", node.ip, node.port))
                        .await
                        .unwrap();
                client
                    .join_network(Request::new(JoinNetworkRequest {
                        node: Some(node_info.clone()),
                    }))
                    .await
                    .unwrap();
            });
        });
        while let Some(_) = broadcast.join_next().await {}
    } else {
        // if the node is the master node, then add itself to the peer list
        node.peers.lock().await.push(node_info.clone());
    }

    // start a thread to handle incoming transactions, if any transaction is received, compute the hash and add it to the blockchain
    let addr = format!("127.0.0.1:{}", port).parse().unwrap();

    tokio::spawn(handle_transactions(node, port, rx));

    println!("[INFO] Node server listening on {}", addr);
    Server::builder()
        .add_service(NodeMessageServer::new(network))
        .serve(addr)
        .await
        .unwrap();
}
/// Handle incoming transactions
pub async fn handle_transactions(node: Arc<Node>, port: u32, mut rx: mpsc::Receiver<bool>) {
    loop {
        let blockchain = node.blockchain.lock().await;
        let difficulty = blockchain.difficulty;
        let last_block = blockchain
            .chain
            .last()
            .unwrap_or(&Block {
                id: -1,
                timestamp: 0,
                nonce: 0,
                prev_hash: "".to_string(),
                hash: "".to_string(),
                transactions: vec![],
                difficulty: difficulty,
            })
            .clone();

        let transactions = blockchain.transactions.clone();
        drop(blockchain);
        // if there are transactions in the transaction pool, then mine a new block
        if transactions.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        }
        match mine_new_block(&last_block, transactions.clone(), difficulty, &mut rx).await {
            Ok(block) => {
                // broadcast the new block to the rest of the network
                let mut blockchain = node.blockchain.lock().await;
                blockchain.chain.push(block.clone());

                for node in node.peers.lock().await.iter() {
                    if node.port == port {
                        continue;
                    }
                    let mut start_idx = blockchain.chain.len() - 1;
                    let mut client =
                        NodeMessageClient::connect(format!("http://127.0.0.1:{}", node.port))
                            .await
                            .unwrap();
                    loop {
                        match client
                            .update_blockchain(Request::new(UpdateBlockchainRequest {
                                blocks: blockchain.chain[start_idx..].to_vec(),
                            }))
                            .await
                        {
                            Ok(res) => {
                                let response = res.into_inner();
                                if response.success {
                                    break;
                                } else {
                                    start_idx = response.chain_length as usize;
                                }
                            }
                            Err(error) => {
                                eprintln!(
                                    "[Warning] Failed to update blockchain on node: {}, {:?}",
                                    node.port, error
                                );
                                break;
                            }
                        }
                    }
                }
                transactions.iter().for_each(|transaction| {
                    blockchain.transactions.retain(|x| x != transaction);
                });
            }
            Err(error) => {
                println!("[Warning] Failed to mine new block: {:?}", error);
            }
        }
    }
}
