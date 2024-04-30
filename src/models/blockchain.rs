use crate::node::{Block, Transaction};
use tokio::sync::mpsc::Receiver;
use tonic::Status;
pub struct Blockchain {
    pub transactions: Vec<Transaction>,
    pub blockchain: Vec<Block>,
    pub difficulty: i32,
    pub rx: Receiver<bool>,
}

/// Implement the BlockchainNetwork struct
/// This struct will hold the list of nodes and the blockchain
impl Blockchain {
    pub fn new(rx: Receiver<bool>) -> Blockchain {
        Blockchain {
            transactions: Vec::new(),
            blockchain: Vec::new(),
            difficulty: 4,
            rx: rx,
        }
    }
    /// Check if the block is valid
    pub async fn check_blockchain_validity(&self) -> bool {
        let mut current_timestamp = 0;
        for (i, block) in self.blockchain.iter().enumerate() {
            if i != block.id as usize {
                return false;
            }
            if block.timestamp <= current_timestamp {
                return false;
            }
            if !block.check_block_validity() {
                return false;
            }
            current_timestamp = block.timestamp;
        }
        true
    }

    /// Add the block to the blockchain
    pub async fn add_block_to_blockchain(&mut self, block: Block) {
        self.blockchain.push(block);
    }

    pub async fn mine_new_block(
        &mut self,
        transactions: Vec<Transaction>,
    ) -> Result<Block, Status> {
        let last_block = self.blockchain.last().unwrap();
        let mut nonce = 0;

        let mut new_block = Block {
            id: last_block.id + 1,
            timestamp: 0, // TODO
            prev_hash: last_block.hash.clone(),
            hash: "".to_string(),
            nonce: 0,
            difficulty: self.difficulty,
            transactions: transactions.clone(),
        };

        let mut current_hash = new_block.compute_hash(nonce);

        while !self.check_hash_validity(&current_hash, self.difficulty) {
            // if received a signal to stop mining, then return an error
            if let Ok(_) = self.rx.try_recv() {
                return Err(Status::cancelled("Mining stopped"));
            }

            nonce += 1;
            current_hash = new_block.compute_hash(nonce);
        }

        new_block.nonce = nonce;
        new_block.hash = current_hash.clone();

        // send the new block to the rest of the network
        Ok(new_block)
    }

    /// check if the hash has the required number of leading zeros
    fn check_hash_validity(&self, hash: &String, difficulty: i32) -> bool {
        let mut count = 0;
        for c in hash.chars() {
            if c == '0' {
                count += 1;
            } else {
                break;
            }
        }
        count < difficulty
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
            return false;
        }
        true
    }
}

/// Compute the SHA256 hash of the input data
pub fn sha_hash(data: &str) -> String {
    let hash = sha256::digest(data.as_bytes());
    hash.to_string()
}
