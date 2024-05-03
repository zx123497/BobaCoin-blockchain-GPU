mod common;
use blockchain::node::node_message_client::NodeMessageClient;
use blockchain::node::GenerateTransactionRequest;
use blockchain::node::GetBlockchainRequest;
use blockchain::node::UpdateTransactionRequest;
use blockchain::start;
use std::time::Duration;
use tonic::Request;
use uuid::Uuid;

#[tokio::test]
async fn test_longer_chain_20_percents() {
    let mut tasks = Vec::new();

    // start the first two nodes
    let nodes = vec![50000, 50001, 50002];
    tasks.push(tokio::spawn(start(nodes[0], None)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[1], Some(nodes[0]))));
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Wait for the nodes to start

    // Create a client and create public key and private key
    let client = common::Client::new();
    let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", nodes[0]))
        .await
        .expect("Failed to connect to node");
    let tx1 = grpc_client
        .generate_transaction(Request::new(GenerateTransactionRequest {
            id: Uuid::new_v4().to_string(),
            sender: client.public_key.clone(),
            private_key: client.private_key.clone(),
            receiver: "receiver1".to_string(),
            amount: 100,
            fee: 1,
        }))
        .await
        .unwrap()
        .into_inner()
        .transaction
        .unwrap();

    let tx2 = grpc_client
        .generate_transaction(Request::new(GenerateTransactionRequest {
            id: Uuid::new_v4().to_string(),
            sender: client.public_key.clone(),
            private_key: client.private_key.clone(),
            receiver: "receiver2".to_string(),
            amount: 200,
            fee: 1,
        }))
        .await
        .unwrap()
        .into_inner()
        .transaction
        .unwrap();

    let tx3 = grpc_client
        .generate_transaction(Request::new(GenerateTransactionRequest {
            id: Uuid::new_v4().to_string(),
            sender: client.public_key.clone(),
            private_key: client.private_key.clone(),
            receiver: "receiver3".to_string(),
            amount: 50,
            fee: 2,
        }))
        .await
        .unwrap()
        .into_inner()
        .transaction
        .unwrap();

    // send a transaction to the nodes, so that they can start mining
    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![tx1.clone()],
        }))
        .await
        .unwrap();

    // start third node after the first two nodes have start mining
    tasks.push(tokio::spawn(start(nodes[2], Some(nodes[1]))));
    tokio::time::sleep(Duration::from_millis(200)).await;

    // send two more transactions to the nodes, the third node should start mining block with tx2 and tx3
    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![tx2.clone(), tx3.clone()],
        }))
        .await
        .unwrap();

    // wait for the transaction to be sent to the blockchain
    tokio::time::sleep(Duration::from_secs(5)).await;

    // eventually the third node should have a chain with 2 blocks
    for node in &nodes {
        let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", node))
            .await
            .expect("Failed to connect to node");
        let res = grpc_client
            .get_blockchain(Request::new(GetBlockchainRequest {}))
            .await
            .unwrap();
        let blockchain = res.into_inner().chain;
        assert_eq!(blockchain.len(), 2);
        assert_eq!(blockchain[0].transactions[0], tx1);
        assert_eq!(blockchain[1].transactions[0], tx2);
        assert_eq!(blockchain[1].transactions[1], tx3);
    }
    for task in tasks {
        task.abort();
    }
}
