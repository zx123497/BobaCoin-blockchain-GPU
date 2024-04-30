use crate::models::node::Node;
use crate::node::node_message_client::NodeMessageClient;
use crate::node::{
    node_message_server::NodeMessage, JoinNetworkRequest, JoinNetworkResponse,
    UpdateBlockchainRequest, UpdateBlockchainResponse,
};
use crate::node::{UpdateTransactionRequest, UpdateTransactionResponse};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};
pub struct Network {
    pub node: Arc<Node>,
    pub tx: Sender<bool>,
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

        if request_bc.len() > current_bc.blockchain.len() {
            if current_bc.check_blockchain_validity().await {
                // delete transactions that are already in the blockchain
                current_bc
                    .transactions
                    .retain(|x| !request_bc.iter().any(|y| y.transactions.contains(x)));

                current_bc.blockchain = request_bc;
                println!(
                    "[INFO] Update blockchain from other peer: {:?}",
                    current_bc.blockchain.len()
                );
                match self.tx.send(true).await {
                    Ok(_) => Ok(Response::new(UpdateBlockchainResponse { success: true })),
                    Err(_) => Ok(Response::new(UpdateBlockchainResponse { success: false })),
                }
            } else {
                Ok(Response::new(UpdateBlockchainResponse { success: false }))
            }
        } else {
            Ok(Response::new(UpdateBlockchainResponse { success: false }))
        }
    }

    /// Receive the transactions from another node and add them to the current node's transaction pool
    async fn update_transaction(
        &self,
        request: Request<UpdateTransactionRequest>,
    ) -> Result<Response<UpdateTransactionResponse>, Status> {
        let mut blockchain = self.node.blockchain.lock().await;
        for transaction in request.into_inner().transactions {
            if blockchain.transactions.contains(&transaction) {
                continue;
            }
            if transaction.check_transaction_validity() {
                blockchain.transactions.push(transaction);
            }
        }
        println!(
            "[INFO] Update transaction from other peer: {:?}",
            blockchain.transactions.len()
        );
        Ok(Response::new(UpdateTransactionResponse { success: true }))
    }

    /// Receive the transactions from client and broadcast them to the network
    async fn update_client_transaction(
        &self,
        request: Request<UpdateTransactionRequest>,
    ) -> Result<Response<UpdateTransactionResponse>, Status> {
        let mut blockchain = self.node.blockchain.lock().await;
        let req_transaction = request.into_inner().transactions;
        println!(
            "[INFO] New transaction recieved from clien {:?}",
            req_transaction.len()
        );
        for transaction in &req_transaction {
            if transaction.check_transaction_validity() {
                blockchain.transactions.push(transaction.clone());
            }
        }

        // broadcast the new transaction to the rest of the network
        for node in self.node.peers.lock().await.iter() {
            if node.port == self.node.port {
                continue;
            }
            let mut client = NodeMessageClient::connect(format!("http://[::1]:{}", node.port))
                .await
                .unwrap();
            client
                .update_transaction(Request::new(UpdateTransactionRequest {
                    transactions: req_transaction.clone(),
                }))
                .await
                .unwrap();
        }
        Ok(Response::new(UpdateTransactionResponse { success: true }))
    }
}
