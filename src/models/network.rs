use crate::models::node::Node;

use crate::node::{
    node_message_client::NodeMessageClient, node_message_server::NodeMessage, GetBlockchainRequest,
    GetBlockchainResponse, JoinNetworkRequest, JoinNetworkResponse, UpdateBlockchainRequest,
    UpdateBlockchainResponse, UpdateTransactionRequest, UpdateTransactionResponse,
};
use crate::node::{
    GenerateTransactionRequest, GenerateTransactionResponse, GetPeerListRequest,
    GetPeerListResponse, Transaction,
};
use openssl::{hash::MessageDigest, pkey::PKey, rsa::Rsa, sign::Signer};
use std::sync::Arc;
use std::time::SystemTime;
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
            chain: self.node.blockchain.lock().await.chain.clone(),
        };
        println!("[INFO] New node joined the network: {:?}", req_node.port);
        Ok(Response::new(reply))
    }

    /// Receive the blockchain from another node, if the received blockchain is longer than the current blockchain, then replace the current blockchain with the received blockchain
    async fn update_blockchain(
        &self,
        request: Request<UpdateBlockchainRequest>,
    ) -> Result<Response<UpdateBlockchainResponse>, Status> {
        let request_bc = request.into_inner().chain;
        let mut current_bc = self.node.blockchain.lock().await;

        if request_bc.len() > current_bc.chain.len() {
            if current_bc.check_blockchain_validity().await {
                // delete transactions that are already in the blockchain
                current_bc
                    .transactions
                    .retain(|x| !request_bc.iter().any(|y| y.transactions.contains(x)));

                current_bc.chain = request_bc;
                println!("[INFO] Update blockchain from other peer");
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
        println!("[INFO] Update transaction from other peer");
        Ok(Response::new(UpdateTransactionResponse { success: true }))
    }

    /// Receive the transactions from client and broadcast them to the network
    async fn update_client_transaction(
        &self,
        request: Request<UpdateTransactionRequest>,
    ) -> Result<Response<UpdateTransactionResponse>, Status> {
        let mut blockchain = self.node.blockchain.lock().await;
        let req_transaction = request.into_inner().transactions;

        for transaction in &req_transaction {
            if blockchain.transactions.contains(&transaction) {
                continue;
            }
            if transaction.check_transaction_validity() {
                blockchain.transactions.push(transaction.clone());
            }
        }

        println!("[INFO] New transaction recieved from client");

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

    async fn get_blockchain(
        &self,
        _: Request<GetBlockchainRequest>,
    ) -> Result<Response<GetBlockchainResponse>, Status> {
        let blockchain = self.node.blockchain.lock().await;
        Ok(Response::new(GetBlockchainResponse {
            chain: blockchain.chain.clone(),
        }))
    }

    async fn generate_transaction(
        &self,
        request: Request<GenerateTransactionRequest>,
    ) -> Result<Response<GenerateTransactionResponse>, Status> {
        let req = request.into_inner();
        let mut transaction = Transaction {
            id: req.id,
            sender: req.sender,
            receiver: req.receiver,
            amount: req.amount,
            fee: req.fee,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            hash: "".to_string(),
            signature: "".to_string(),
        };
        let private_key_string = hex::decode(transaction.sender.clone()).unwrap();
        let private_key = Rsa::private_key_from_pem(private_key_string.as_slice()).unwrap();
        let keypair = PKey::from_rsa(private_key).unwrap();
        transaction.hash = transaction.compute_hash();
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(transaction.hash.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();
        transaction.signature = hex::encode(signature);

        Ok(Response::new(GenerateTransactionResponse {
            transaction: Some(transaction),
        }))
    }

    async fn get_peer_list(
        &self,
        _: Request<GetPeerListRequest>,
    ) -> Result<Response<GetPeerListResponse>, Status> {
        let peers = self.node.peers.lock().await;
        Ok(Response::new(GetPeerListResponse {
            nodes: (*peers).clone(),
        }))
    }
}
