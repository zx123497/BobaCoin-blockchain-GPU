mod blockchain_network;
use crate::blockchain_network::BlockchainNetwork;
use node::node_message_client::NodeMessageClient;
use node::node_message_server::NodeMessageServer;
use node::{JoinNetworkRequest, NodeInfo};
use tonic::{transport::Server, Request};
pub mod node {
    tonic::include_proto!("node");
}

pub struct Block {
    pub index: i32,
    pub timestamp: i32,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: i32;
    if std::env::args().len() == 1 {
        // if no port is provided, then start a super node
        port = 50051;
    } else if std::env::args().len() > 2 {
        panic!("Usage: {} [PORT]", std::env::args().nth(0).unwrap());
    } else {
        let args = std::env::args().collect::<Vec<String>>();
        port = args[1].parse().unwrap();
    }

    let network = BlockchainNetwork::default();
    let ip = "127.1.1.1".to_string();

    // get the ip address of the current node
    let node_info = NodeInfo {
        id: calculate_id(&ip, port),
        ip: ip,
        port: port,
    };

    // if the node is not the master node, then should introduce itself to every node in the network
    if port != 50051 {
        // connect to the super node
        let mut client = NodeMessageClient::connect("http://[::1]:50051").await?;
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
                NodeMessageClient::connect(format!("http://[::1]:{}", node.port)).await?;
            client
                .join_network(Request::new(JoinNetworkRequest {
                    node: Some(node_info.clone()),
                }))
                .await?;
        }
    }

    let addr = format!("[::1]:{}", port).parse().unwrap();
    println!("Start the server at: {:?}", addr);
    Server::builder()
        .add_service(NodeMessageServer::new(network))
        .serve(addr)
        .await?;

    Ok(())
}

pub fn calculate_id(ip: &String, port: i32) -> i32 {
    let mut id = 0;
    for c in ip.chars() {
        id += c as i32;
    }
    id += port;
    id
}
