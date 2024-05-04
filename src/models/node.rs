//! Node model
//! Node model is used to represent a node in the network
//! The node has a list of peers, a blockchain, an ip, a port, and an id
//! The node is used to store the state of the node in the network
use crate::models::blockchain::Blockchain;
use crate::node::NodeInfo;
use tokio::sync::Mutex;
use uuid::Uuid;
pub struct Node {
    pub peers: Mutex<Vec<NodeInfo>>,
    pub blockchain: Mutex<Blockchain>,
    pub ip: String,
    pub port: u32,
    pub id: Uuid,
}

impl Node {
    pub fn new(port: u32) -> Node {
        Node {
            peers: Mutex::new(Vec::new()),
            blockchain: Mutex::new(Blockchain::new()),
            ip: "[::1]".to_string(),
            port: port,
            id: Uuid::new_v4(),
        }
    }
}
