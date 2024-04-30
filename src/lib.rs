mod models;
pub mod node {
    tonic::include_proto!("node");
}

use crate::models::{network::Network, node::Node};
use node::{
    node_message_client::NodeMessageClient, node_message_server::NodeMessageServer,
    JoinNetworkRequest, NodeInfo,
};
use std::sync::Arc;
use tokio::task::JoinSet;
use tonic::{transport::Server, Request};

pub async fn start(port: u16, peer_port: Option<u16>) {
    let node = Arc::new(Node::new(port));
    let network = Network { node: node.clone() };
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
    println!("Node server listening on {}", addr);
    Server::builder()
        .add_service(NodeMessageServer::new(network))
        .serve(addr)
        .await
        .unwrap();
}
