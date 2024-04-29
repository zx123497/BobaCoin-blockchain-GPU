use crate::models::node::Node;
use crate::node::{
    node_message_server::NodeMessage, JoinNetworkRequest, JoinNetworkResponse,
    UpdateBlockchainRequest, UpdateBlockchainResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct Network {
    pub node: Arc<Node>,
}

/// Implement the NodeMessage trait for the BlockchainNetwork struct
/// This will allow the BlockchainNetwork struct to be used as a gRPC service
#[tonic::async_trait]
impl NodeMessage for Network {
    async fn join_network(
        &self,
        request: Request<JoinNetworkRequest>,
    ) -> Result<Response<JoinNetworkResponse>, Status> {
        let mut peers = self.node.peers.lock().await;
        let req_node = request.into_inner().node.unwrap();

        peers.push(req_node.clone());

        let reply = JoinNetworkResponse {
            nodes: (*peers).clone(),
        };
        println!("New node joined the network: {:?}", req_node.port);
        Ok(Response::new(reply))
    }

    /// Receive the blockchain from another node, if the received blockchain is longer than the current blockchain, then replace the current blockchain with the received blockchain
    async fn update_blockchain(
        &self,
        request: Request<UpdateBlockchainRequest>,
    ) -> Result<Response<UpdateBlockchainResponse>, Status> {
        let request_bc = request.into_inner().chain;
        let mut current_bc = self.node.blockchain.lock().await;

        if request_bc.len() >= current_bc.blockchain.len() {
            current_bc.blockchain = request_bc;
            Ok(Response::new(UpdateBlockchainResponse { success: true }))
        } else {
            Ok(Response::new(UpdateBlockchainResponse { success: false }))
        }
    }
}
