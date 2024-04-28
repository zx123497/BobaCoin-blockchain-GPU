use crate::node::node_message_server::NodeMessage;
use crate::node::{
    Block, JoinNetworkRequest, JoinNetworkResponse, NodeInfo, SendNewBlockRequest,
    SendNewBlockResponse,
};

use std::sync::Mutex;
use tonic::{Request, Response, Status};
pub struct BlockchainNetwork {
    pub nodes: Mutex<Vec<NodeInfo>>,
    pub blockchain: Mutex<Vec<Block>>,
}

/// Implement the BlockchainNetwork struct
/// This struct will hold the list of nodes and the blockchain
impl BlockchainNetwork {
    /// Check if the block is valid
    fn check_block_validity(&self, block: &Block) -> bool {
        let blocks = self.blockchain.lock().unwrap();
        let last_block = blocks.last().unwrap();
        if last_block.id + 1 != block.id {
            return false;
        }
        if last_block.hash != block.prev_hash {
            return false;
        }
        true
    }
    /// Add the block to the blockchain
    fn add_block_to_blockchain(&self, block: Block) {
        let mut blocks = self.blockchain.lock().unwrap();
        blocks.push(block);
    }
}

impl Default for BlockchainNetwork {
    fn default() -> Self {
        BlockchainNetwork {
            nodes: Mutex::new(Vec::new()),
            blockchain: Mutex::new(Vec::new()),
        }
    }
}

/// Implement the NodeMessage trait for the BlockchainNetwork struct
/// This will allow the BlockchainNetwork struct to be used as a gRPC service
#[tonic::async_trait]
impl NodeMessage for BlockchainNetwork {
    async fn join_network(
        &self,
        request: Request<JoinNetworkRequest>,
    ) -> Result<Response<JoinNetworkResponse>, Status> {
        let mut nodes = self.nodes.lock().unwrap();
        let req_node = request.into_inner().node.unwrap();

        nodes.push(NodeInfo {
            id: req_node.id,
            ip: req_node.ip,
            port: req_node.port,
        });

        let reply = JoinNetworkResponse {
            nodes: nodes.clone(),
        };
        println!("New node joined the network: {:?}", req_node.port);
        Ok(Response::new(reply))
    }

    async fn send_new_block(
        &self,
        _request: Request<SendNewBlockRequest>,
    ) -> Result<Response<SendNewBlockResponse>, Status> {
        // TODO: Check if the block is valid
        // TODO: Add the block to the blockchain
        let reply = SendNewBlockResponse { success: true };
        Ok(Response::new(reply))
    }
}
