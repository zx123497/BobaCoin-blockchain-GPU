//! This module contains the implementation of the Network struct
//! This is for all the gRPC service implementations
use crate::models::node::Node;

use crate::node::{
    node_message_client::NodeMessageClient, node_message_server::NodeMessage, GetBlockchainRequest,
    GetBlockchainResponse, JoinNetworkRequest, JoinNetworkResponse, UpdateBlockchainRequest,
    UpdateBlockchainResponse, UpdateTransactionRequest, UpdateTransactionResponse,
};
use crate::node::{
    GenerateTransactionRequest, GenerateTransactionResponse, GetPeerListRequest,
    GetPeerListResponse, GetTransactionListRequest, GetTransactionListResponse, Transaction,
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
        let current_bc = self.node.blockchain.lock().await;

        let reply = JoinNetworkResponse {
            nodes: (*peers).clone(),
            chain: current_bc.chain.clone(),
            transactions: current_bc.transactions.clone(),
        };
        println!("[INFO] New node joined the network: {:?}", req_node.port);
        Ok(Response::new(reply))
    }

    /// Receive the blockchain from another node, if the received blockchain is longer than the current blockchain, then replace the current blockchain with the received blockchain
    async fn update_blockchain(
        &self,
        request: Request<UpdateBlockchainRequest>,
    ) -> Result<Response<UpdateBlockchainResponse>, Status> {
        let req = request.into_inner();
        let blocks = req.blocks;
        let current_bc = self.node.blockchain.lock().await;
        let mut chain = current_bc.chain.clone();
        drop(current_bc);

        if blocks.is_empty() {
            println!("[Warning] Received blocks is empty");
            return Ok(Response::new(UpdateBlockchainResponse {
                success: false,
                chain_length: chain.len() as u32,
            }));
        }
        if blocks[0].id as usize > chain.len() {
            // should return false to get updated blockchain
            println!("[Warning] Need previous blocks to update blockchain, current blockchain length: {}",
                     chain.len());
            return Ok(Response::new(UpdateBlockchainResponse {
                success: false,
                chain_length: chain.len() as u32,
            }));
        } else {
            // check if the received blockchain is longer than the current blockchain
            if (blocks.last().unwrap().id as usize) < chain.len() {
                println!("[Warning] Received blockchain is shorter than current blockchain",);
                return Err(Status::invalid_argument(
                    "Received blockchain is shorter than current blockchain",
                ));
            }

            // truncate the current blockchain and add the received blocks
            chain.truncate(blocks[0].id as usize);
            chain.extend(blocks.clone());

            let mut prev_hash = String::new();
            if blocks[0].id != 0 {
                prev_hash = chain[blocks[0].id as usize - 1].hash.clone();
            }
            let mut encluded_transactions = Vec::<Transaction>::new();
            // check if the received blockchain is valid
            for block in blocks {
                if block.prev_hash != prev_hash {
                    println!("[Warning] Previous hash does not match in block ");
                    return Ok(Response::new(UpdateBlockchainResponse {
                        success: false,
                        chain_length: 0,
                    }));
                }

                prev_hash = block.hash.clone();

                if !block.check_block_validity() {
                    println!("[Warning] Invalid block in received blockchain");
                    return Err(Status::invalid_argument(
                        "Invalid block in received blockchain",
                    ));
                }

                encluded_transactions.extend(block.transactions.clone());
            }

            // if the received blockchain is valid, then update the current blockchain
            let mut current_bc = self.node.blockchain.lock().await;
            current_bc.chain = chain;

            // remove the transactions that are included in the new blockchain
            current_bc
                .transactions
                .retain(|tx| !encluded_transactions.contains(tx));

            println!("[INFO] Updated blockchain from other peer.");

            match self.tx.send(true).await {
                Ok(_) => Ok(Response::new(UpdateBlockchainResponse {
                    success: true,
                    chain_length: current_bc.chain.len() as u32,
                })),

                Err(_) => Ok(Response::new(UpdateBlockchainResponse {
                    success: false,
                    chain_length: current_bc.chain.len() as u32,
                })),
            }
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
            } else {
                println!("[INFO] Invalid transaction recieved from client");
            }
        }

        println!("[INFO] New transaction recieved from client");

        // broadcast the new transaction to the rest of the network
        for node in self.node.peers.lock().await.iter() {
            if node.port == self.node.port {
                continue;
            }
            let mut client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", node.port))
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

    /// Return the current blockchain to the client
    async fn get_blockchain(
        &self,
        _: Request<GetBlockchainRequest>,
    ) -> Result<Response<GetBlockchainResponse>, Status> {
        let blockchain = self.node.blockchain.lock().await;
        Ok(Response::new(GetBlockchainResponse {
            chain: blockchain.chain.clone(),
        }))
    }
    /// Generate a transaction and return it to the client
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
        let private_key_string = hex::decode(req.private_key).unwrap();
        let private_key = Rsa::private_key_from_pem(&private_key_string).unwrap();
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
    /// Return the list of peers to the client
    async fn get_peer_list(
        &self,
        _: Request<GetPeerListRequest>,
    ) -> Result<Response<GetPeerListResponse>, Status> {
        let peers = self.node.peers.lock().await;
        Ok(Response::new(GetPeerListResponse {
            nodes: (*peers).clone(),
        }))
    }
    /// Return the list of transactions to the client
    async fn get_transaction_list(
        &self,
        _: Request<GetTransactionListRequest>,
    ) -> Result<Response<GetTransactionListResponse>, Status> {
        let blockchain = self.node.blockchain.lock().await;
        Ok(Response::new(GetTransactionListResponse {
            transactions: blockchain.transactions.clone(),
        }))
    }
}
