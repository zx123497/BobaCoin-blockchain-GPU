mod models;
use crate::models::{network::Network, node::Node};
use node::{
    node_message_client::NodeMessageClient, node_message_server::NodeMessageServer,
    JoinNetworkRequest, NodeInfo, UpdateBlockchainRequest,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request};
pub mod node {
    tonic::include_proto!("node");
}

const SUPER_NODE_PORT: i32 = 50051;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<bool>(1);
    let port: i32;
    // TODO: use a argument parser
    if std::env::args().len() == 1 {
        // if no port is provided, then start a super node
        port = 50051;
    } else if std::env::args().len() > 2 {
        panic!("Usage: {} [PORT]", std::env::args().nth(0).unwrap());
    } else {
        let args = std::env::args().collect::<Vec<String>>();
        port = args[1].parse().unwrap();
    }

    let node = Arc::new(Node::new(port, rx));

    let network = Network {
        node: node.clone(),
        tx: tx,
    };

    let node_info = NodeInfo {
        id: node.id.to_string(),
        ip: node.ip.clone(),
        port: node.port,
    };

    // if the node is not the master node, then should introduce itself to every node in the network
    if port != SUPER_NODE_PORT {
        // connect to the super node
        let mut client =
            NodeMessageClient::connect(format!("http://[::1]:{}", SUPER_NODE_PORT)).await?;
        let res = client
            .join_network(Request::new(JoinNetworkRequest {
                node: Some(node_info.clone()),
            }))
            .await?;

        // broadcast the new node to the rest of the network
        for node in res.into_inner().nodes {
            if node.port == port {
                continue;
            }
            let mut client =
                NodeMessageClient::connect(format!("http://{}:{}", node.ip, node.port)).await?;
            client
                .join_network(Request::new(JoinNetworkRequest {
                    node: Some(node_info.clone()),
                }))
                .await?;
        }
    }

    let addr = format!("[::1]:{}", port).parse().unwrap();

    // start a thread to handle incoming transactions, if any transaction is received, compute the hash and add it to the blockchain
    tokio::spawn(handle_transactions(node.clone(), port));

    println!("Node server listening on {}", addr);
    Server::builder()
        .add_service(NodeMessageServer::new(network))
        .serve(addr)
        .await?;

    Ok(())
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
