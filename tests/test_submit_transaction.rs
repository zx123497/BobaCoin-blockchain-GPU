mod common;
use blockchain::node::node_message_client::NodeMessageClient;
use blockchain::node::GenerateTransactionRequest;
use blockchain::node::GetTransactionListRequest;
use blockchain::node::UpdateTransactionRequest;
use blockchain::start;
use std::time::Duration;
use tonic::Request;
use uuid::Uuid;

#[tokio::test]
async fn test_submit_transaction_10_percents() {
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
    let mut invalid_transaction = transaction.clone();
    invalid_transaction.amount = 1000;

    // send a valid transaction and an invalid transaction
    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![transaction.clone(), invalid_transaction.clone()],
        }))
        .await
        .unwrap();

    // wait for the transaction to be sent to the blockchain
    tokio::time::sleep(Duration::from_millis(200)).await;

    // check that only the valid transaction is in every node
    for node in &nodes {
        let mut grpc_client = NodeMessageClient::connect(format!("http://[::1]:{}", node))
            .await
            .expect("Failed to connect to node");
        let res = grpc_client
            .get_transaction_list(Request::new(GetTransactionListRequest {}))
            .await
            .unwrap();
        let transactions = res.into_inner().transactions;
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0], transaction);
    }
    for task in tasks {
        task.abort();
    }
}
