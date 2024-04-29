use crate::node::{Block, Transaction};
use tonic::Status;

pub struct Blockchain {
    pub transactions: Vec<Transaction>,
    pub blockchain: Vec<Block>,
    pub difficulty: i32,
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain {
            transactions: Vec::new(),
            blockchain: Vec::new(),
            difficulty: 4,
        }
    }
}

/// Implement the BlockchainNetwork struct
/// This struct will hold the list of nodes and the blockchain
impl Blockchain {
    /// Check if the block is valid
    pub async fn check_block_validity(&self, block: &Block) -> bool {
        // TODO: implement the check_block_validity function
        let last_block = self.blockchain.last().unwrap();
        if last_block.id + 1 != block.id {
            return false;
        }
        if last_block.hash != block.prev_hash {
            return false;
        }
        true
    }

    /// Add the block to the blockchain
    pub async fn add_block_to_blockchain(&mut self, block: Block) {
        self.blockchain.push(block);
    }

    pub async fn mine_new_block(&self, transactions: Vec<Transaction>) -> Result<Block, Status> {
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
}

impl Transaction {
    pub fn compute_hash(&self) -> String {
        let data = format!(
            "{}|{}|{}|{}|{}",
            self.timestamp, self.sender, self.receiver, self.amount, self.fee
        );
        sha_hash(&data)
    }
}

pub fn sha_hash(data: &str) -> String {
    let hash = sha256::digest(data.as_bytes());
    hash.to_string()
}
