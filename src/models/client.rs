//! Client module
//! Contains the client struct and its implementation
//! The client struct is used to interact with the blockchain network
//! The client can send transactions to the network
use crate::node::UpdateTransactionRequest;
use crate::node::{node_message_client::NodeMessageClient, Transaction};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::sign::Signer;
use std::time::SystemTime;
use tonic::Request;
use uuid::Uuid;
pub struct Client {
    pub public_key: String,
    pub private_key: String,
    pub port: u16,
}

impl Client {
    pub fn new(port: u16) -> Client {
        let (public_key, private_key) = generate_keypair();
        Client {
            public_key: public_key,
            private_key: private_key,
            port: port,
        }
    }
    /// Start the client
    pub async fn start(self) {
        // transaction input stdin
        let mut input = String::new();
        loop {
            println!("Enter a command: ");
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            match input {
                "new" => {
                    println!("send to: ");
                    let mut receiver = String::new();
                    std::io::stdin().read_line(&mut receiver).unwrap();
                    let receiver = receiver.trim();
                    println!("amount: ");
                    let mut amount = String::new();
                    std::io::stdin().read_line(&mut amount).unwrap();
                    let amount = amount.trim().parse::<i32>().unwrap();
                    println!("fee: ");
                    let mut fee = String::new();
                    std::io::stdin().read_line(&mut fee).unwrap();
                    let fee = fee.trim().parse::<i32>().unwrap();
                    let transaction = self.generate_transaction(receiver.to_string(), amount, fee);

                    let mut grpc_client =
                        NodeMessageClient::connect(format!("http://127.0.0.1:{}", self.port))
                            .await
                            .expect("Failed to connect to node");

                    grpc_client
                        .update_client_transaction(Request::new(UpdateTransactionRequest {
                            transactions: vec![transaction.clone()],
                        }))
                        .await
                        .unwrap();
                    println!("[INFO] Transaction sent\n\n");
                }
                "exit" => {
                    break;
                }
                _ => {
                    println!("Invalid command");
                }
            }
        }
    }
    /// Generate a transaction
    fn generate_transaction(&self, receiver: String, amount: i32, fee: i32) -> Transaction {
        let mut transaction = Transaction {
            id: Uuid::new_v4().to_string(),
            sender: self.public_key.clone(),
            receiver: receiver,
            amount: amount,
            fee: fee,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            hash: "".to_string(),
            signature: "".to_string(),
        };
        let private_key_string = hex::decode(self.private_key.clone()).unwrap();
        let private_key = Rsa::private_key_from_pem(&private_key_string).unwrap();
        let keypair = PKey::from_rsa(private_key).unwrap();
        transaction.hash = transaction.compute_hash();
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
        signer.update(transaction.hash.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();
        transaction.signature = hex::encode(signature);
        transaction
    }
}
/// Generate a keypair
fn generate_keypair() -> (String, String) {
    let rsa = Rsa::generate(2048).unwrap();
    let public_key = hex::encode(rsa.public_key_to_pem().unwrap());
    let private_key = hex::encode(rsa.private_key_to_pem().unwrap());
    (public_key, private_key)
}
