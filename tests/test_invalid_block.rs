mod common;
use blockchain::node::node_message_client::NodeMessageClient;
use blockchain::node::GenerateTransactionRequest;
use blockchain::node::GetBlockchainRequest;
use blockchain::node::UpdateBlockchainRequest;
use blockchain::node::UpdateTransactionRequest;
use blockchain::start;
use std::time::Duration;
use tonic::Request;
use uuid::Uuid;
#[tokio::test]
async fn test_invalid_block() {
    let mut tasks = Vec::new();

    let nodes = vec![50000, 50001, 50002];
    tasks.push(tokio::spawn(start(nodes[0], None)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[1], Some(nodes[0]))));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[2], Some(nodes[1]))));
    tokio::time::sleep(Duration::from_millis(200)).await;
    // Wait for the nodes to start

    // Create a client and create public key and private key
    let client = common::Client::new();
    let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", nodes[0]))
        .await
        .expect("Failed to connect to node");
    let res = grpc_client.generate_transaction(Request::new(GenerateTransactionRequest {
        id: Uuid::new_v4().to_string(),
        sender: client.public_key.clone(),
        private_key: client.private_key.clone(),
        receiver: "receiver".to_string(),
        amount: 100,
        fee: 1,
    }));

    let transaction = res.await.unwrap().into_inner().transaction.unwrap();
    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![transaction.clone()],
        }))
        .await
        .unwrap();

    // wait for the transaction to be sent to the blockchain
    tokio::time::sleep(Duration::from_secs(5)).await;

    // simulate node 0 being a bad node, and trying to spread a bad block to the network
    let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", nodes[0]))
        .await
        .expect("Failed to connect to node");

    let res = grpc_client
        .get_blockchain(Request::new(GetBlockchainRequest {}))
        .await
        .unwrap();
    let blockchain = res.into_inner().chain;

    // create a bad block
    let mut bad_block = blockchain[0].clone();
    bad_block.id = 1;
    bad_block.prev_hash = bad_block.hash.clone();
    bad_block.transactions[0].amount = 1000;

    let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", nodes[1]))
        .await
        .expect("Failed to connect to node");

    assert!(grpc_client
        .update_blockchain(Request::new(UpdateBlockchainRequest {
            blocks: vec![bad_block.clone()],
        }))
        .await
        .is_err());

    let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", nodes[2]))
        .await
        .expect("Failed to connect to node");
    assert!(grpc_client
        .update_blockchain(Request::new(UpdateBlockchainRequest {
            blocks: vec![bad_block.clone()],
        }))
        .await
        .is_err());

    // expect the other nodes to ignore the bad block
    for node in &nodes {
        let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", node))
            .await
            .expect("Failed to connect to node");
        let res = grpc_client
            .get_blockchain(Request::new(GetBlockchainRequest {}))
            .await
            .unwrap();
        let blockchain = res.into_inner().chain;
        assert_eq!(blockchain.len(), 1);
        assert_eq!(blockchain[0].transactions[0], transaction);
    }
    for task in tasks {
        task.abort();
    }
}
