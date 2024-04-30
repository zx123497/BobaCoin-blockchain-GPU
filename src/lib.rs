mod models;
pub mod node {
    tonic::include_proto!("node");
}

use crate::models::{network::Network, node::Node};
use node::{
    node_message_client::NodeMessageClient, node_message_server::NodeMessageServer,
    JoinNetworkRequest, NodeInfo, UpdateBlockchainRequest,
};
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinSet};
use tonic::{transport::Server, Request};

pub async fn start(port: u16, peer_port: Option<u16>) {
    let (tx, rx) = mpsc::channel::<bool>(1);
    let node = Arc::new(Node::new(port, rx));
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
        // connect to the super node
        let mut client = NodeMessageClient::connect(format!("http://[::1]:{}", peer))
            .await
            .expect("Failed to connect to peer node");
        let res = client
            .join_network(Request::new(JoinNetworkRequest {
                node: Some(node_info.clone()),
            }))
            .await
            .expect("Failed to join network on peer node");

        // broadcast the new node to the rest of the network
        let mut broadcast = JoinSet::new();
        res.into_inner().nodes.into_iter().for_each(|node| {
            if node.port == port as i32 {
                return;
            }
            let node_info = node_info.clone();
            broadcast.spawn(async move {
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
    }

    // start a thread to handle incoming transactions, if any transaction is received, compute the hash and add it to the blockchain
    let addr = format!("[::1]:{}", port).parse().unwrap();

    tokio::spawn(handle_transactions(node, port as i32));

    println!("Node server listening on {}", addr);
    Server::builder()
        .add_service(NodeMessageServer::new(network))
        .serve(addr)
        .await
        .unwrap();
}

pub async fn handle_transactions(node: Arc<Node>, port: i32) {
    let mut blockchain = node.blockchain.lock().await;
    let transactions = blockchain.transactions.clone();

    while transactions.len() > 0 {
        match blockchain.mine_new_block(transactions.clone()).await {
            Ok(block) => {
                // broadcast the new block to the rest of the network
                blockchain.add_block_to_blockchain(block).await;
                for node in node.peers.lock().await.iter() {
                    if node.port == port {
                        continue;
                    }
                    let mut client =
                        NodeMessageClient::connect(format!("http://[::1]:{}", node.port))
                            .await
                            .unwrap();
                    client
                        .update_blockchain(Request::new(UpdateBlockchainRequest {
                            chain: blockchain.blockchain.clone(),
                        }))
                        .await
                        .unwrap();
                }
                blockchain.transactions.clear();
            }
            Err(error) => {
                println!("[Error] Failed to mine new block: {:?}", error);
            }
        }
    }
}
