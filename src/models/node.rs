use crate::models::blockchain::Blockchain;
use crate::node::NodeInfo;
use tokio::sync::Mutex;
use uuid::Uuid;
pub struct Node {
    pub peers: Mutex<Vec<NodeInfo>>,
    pub blockchain: Mutex<Blockchain>,
    pub ip: String,
    pub port: i32,
    pub id: Uuid,
}

impl Node {
    pub fn new(port: i32) -> Node {
        Node {
            peers: Mutex::new(Vec::new()),
            blockchain: Mutex::new(Blockchain::default()),
            ip: "127.0.0.1".to_string(),
            port: port,
            id: Uuid::new_v4(),
        }
    }
}
