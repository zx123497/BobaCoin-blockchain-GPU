use crate::node::{Block, Transaction};
use openssl::{hash::MessageDigest, pkey::PKey, rsa::Rsa, sign::Verifier};
use std::time::SystemTime;
use tokio::sync::mpsc::Receiver;
use tonic::Status;
pub struct Blockchain {
    pub transactions: Vec<Transaction>,
    pub chain: Vec<Block>,
    pub difficulty: i32,
}

/// Implement the BlockchainNetwork struct
/// This struct will hold the list of nodes and the blockchain
impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            transactions: Vec::new(),
            chain: Vec::new(),
            difficulty: 4,
        }
    }
    /// Check if the block is valid
    pub async fn check_blockchain_validity(&self) -> bool {
        let mut current_timestamp = 0;
        let mut prev_hash = "".to_string();
        for (i, block) in self.chain.iter().enumerate() {
            if i != block.id as usize {
                return false;
            }

            if block.prev_hash != prev_hash {
                return false;
            }

            if block.timestamp <= current_timestamp {
                return false;
            }
            if !block.check_block_validity() {
                return false;
            }
            current_timestamp = block.timestamp;
            prev_hash = block.hash.clone();
        }
        true
    }
}

impl Block {
    pub fn compute_hash(&self, nonce: i32) -> String {
        let mut tx_hashes = "".to_string();
        for (i, transaction) in self.transactions.iter().enumerate() {
            if i < self.transactions.len() - 1 {
                tx_hashes.push_str(&transaction.compute_hash());
                tx_hashes.push_str("|");
            } else {
                tx_hashes.push_str(&transaction.compute_hash());
            }
        }
        let data = format!("{}|{}|{}", self.id, self.prev_hash, tx_hashes);
        let hex_hash = sha_hash(&data);
        let input = hex_hash + nonce.to_string().as_str();
        sha_hash(&input)
    }

    /// Check if the block is valid
    pub fn check_block_validity(&self) -> bool {
        if self.id < 0 {
            return false;
        }

        if self.hash != self.compute_hash(self.nonce) {
            return false;
        }

        // check if timestamp is valid
        let mut current_timestamp = 0;
        for transaction in self.transactions.iter() {
            if current_timestamp >= transaction.timestamp {
                return false;
            }
            if !transaction.check_transaction_validity() {
                return false;
            }
            current_timestamp = transaction.timestamp;
        }
        true
    }
}

impl Transaction {
    pub fn compute_hash(&self) -> String {
        let data = format!(
            "{}|{}|{}|{}|{}",
            self.timestamp, self.sender, self.receiver, self.amount, self.fee
        );
        sha_hash(&data)
    }

    /// Check if the transaction is valid
    pub fn check_transaction_validity(&self) -> bool {
        if self.amount < 0 {
            return false;
        }

        if self.fee < 0 {
            return false;
        }

        if self.sender == self.receiver {
            return false;
        }

        if self.hash != self.compute_hash() {
            println!(
                "Hash: {}, computed hash: {}",
                self.hash,
                self.compute_hash()
            );
            return false;
        }

        match self.verify_signature() {
            Ok(valid) => valid,
            Err(_) => false,
        }
    }

    fn verify_signature(&self) -> Result<bool, openssl::error::ErrorStack> {
        let public_key = hex::decode(&self.sender).unwrap();
        let pub_key = Rsa::public_key_from_pem(public_key.as_slice())?;
        let keypair = PKey::from_rsa(pub_key)?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &keypair)?;
        verifier.update(self.hash.as_bytes())?;
        let signature = hex::decode(&self.signature).unwrap();
        println!("Passed ? {:?}", verifier.verify(&signature));
        verifier.verify(&signature)
    }
}

/// Compute the SHA256 hash of the input data
pub fn sha_hash(data: &str) -> String {
    let hash = sha256::digest(data.as_bytes());
    hash.to_string()
}

/// check if the hash has the required number of leading zeros
fn check_hash_validity(hash: &String, difficulty: i32) -> bool {
    let mut count = 0;
    for c in hash.chars() {
        if c == '0' {
            count += 1;
        } else {
            break;
        }
    }
    count >= difficulty
}

pub async fn mine_new_block(
    last_block: &Block,
    transactions: Vec<Transaction>,
    difficulty: i32,
    rx: &mut Receiver<bool>,
) -> Result<Block, Status> {
    println!("[INFO] Mining new block");
    let mut nonce = 0;

    let mut new_block = Block {
        id: last_block.id + 1,
        timestamp: 0,
        prev_hash: last_block.hash.clone(),
        hash: "".to_string(),
        nonce: 0,
        difficulty: difficulty,
        transactions: transactions.clone(),
    };

    let mut current_hash = new_block.compute_hash(nonce);

    while !check_hash_validity(&current_hash, difficulty) {
        // if received a signal to stop mining, then return an error
        if let Ok(_) = rx.try_recv() {
            println!("[INFO] Mining interrupted");
            return Err(Status::cancelled("Mining stopped"));
        }

        nonce += 1;
        current_hash = new_block.compute_hash(nonce);
        if nonce % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }

    new_block.nonce = nonce;
    new_block.hash = current_hash.clone();
    new_block.timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    println!(
        "[INFO] New block mined: id = {:?}, hash = {:?}",
        new_block.id, new_block.hash
    );

    // send the new block to the rest of the network
    Ok(new_block)
}
