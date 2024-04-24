use node::node_message_client::NodeMessageClient;
use node::node_message_server::{NodeMessage, NodeMessageServer};
use node::{JoinNetworkRequest, JoinNetworkResponse, NodeInfo};
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
pub mod node {
    tonic::include_proto!("node");
}

#[derive(Debug, Default)]
pub struct BlockchainNetwork {
    pub id: i32,
    pub nodes: Arc<Mutex<Vec<NodeInfo>>>,
}

#[tonic::async_trait]
impl NodeMessage for BlockchainNetwork {
    async fn join_network(
        &self,
        request: Request<JoinNetworkRequest>,
    ) -> Result<Response<JoinNetworkResponse>, Status> {
        // add the new node to the list of nodes

        print!("Received request: {:?}", request);
        let new_node_id = self.nodes.lock().unwrap().len() as i32;
        // mutex lock and push the new node to list
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(NodeInfo {
            id: new_node_id,
            ip: request.get_ref().ip.clone(),
            port: request.get_ref().port,
        });

        let reply = JoinNetworkResponse {
            nodes: vec![NodeInfo {
                id: new_node_id,
                ip: "1.1.1.1".to_string(),
                port: 50051,
            }],
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let network = BlockchainNetwork::default();

    // get the ip address of the current node

    let req = JoinNetworkRequest {
        ip: "127.0.0.1".to_string(),
        port: 50051,
    };

    // connect to the network
    let mut client = NodeMessageClient::connect("http://[::1]:50051").await?;
    let res = client.join_network(Request::new(req)).await?;

    for node in res.into_inner().nodes {
        let mut client =
            NodeMessageClient::connect(format!("http://{}:{}", node.ip, node.port)).await?;
        client
            .join_network(Request::new(JoinNetworkRequest {
                ip: "127.0.0.1".to_string(),
                port: 50051,
            }))
            .await?;
    }

    Server::builder()
        .add_service(NodeMessageServer::new(network))
        .serve(addr)
        .await?;

    Ok(())
}
