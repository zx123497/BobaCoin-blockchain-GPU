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
async fn test_fork_30_percents() {
    let mut tasks = Vec::new();

    // start two nodes without connecting them, so they will have different blockchains
    let nodes = vec![50000, 50001, 50002, 50003];
    tasks.push(tokio::spawn(start(nodes[0], None)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[1], None)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[2], None)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[3], Some(nodes[2]))));
    tokio::time::sleep(Duration::from_millis(200)).await;

    println!("{:?}", nodes);

    // Create a client and create public key and private key
    let client = common::Client::new();
    let mut grpc_client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", nodes[0]))
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

    let tx1 = res.await.unwrap().into_inner().transaction.unwrap();
    let res = grpc_client.generate_transaction(Request::new(GenerateTransactionRequest {
        id: Uuid::new_v4().to_string(),
        sender: client.public_key.clone(),
        private_key: client.private_key.clone(),
        receiver: "receiver2".to_string(),
        amount: 200,
        fee: 2,
    }));

    let tx2 = res.await.unwrap().into_inner().transaction.unwrap();
    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![tx1.clone()],
        }))
        .await
        .unwrap();

    let mut grpc_client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", nodes[1]))
        .await
        .expect("Failed to connect to node");

    // wait for the transaction to be sent to the blockchain
    tokio::time::sleep(Duration::from_secs(1)).await;

    let bc1 = grpc_client
        .get_blockchain(Request::new(GetBlockchainRequest {}))
        .await
        .unwrap()
        .into_inner()
        .chain;

    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![tx2.clone()],
        }))
        .await
        .unwrap();

    // wait for the transaction to be sent to the blockchain
    tokio::time::sleep(Duration::from_secs(1)).await;

    let bc2 = grpc_client
        .get_blockchain(Request::new(GetBlockchainRequest {}))
        .await
        .unwrap()
        .into_inner()
        .chain;

    // start another two nodes, and update different chains to them, so they will have forked blockchains
    let mut grpc_client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", nodes[2]))
        .await
        .expect("Failed to connect to node");

    grpc_client
        .update_blockchain(Request::new(blockchain::node::UpdateBlockchainRequest {
            blocks: bc1.clone(),
        }))
        .await
        .unwrap();

    let mut grpc_client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", nodes[3]))
        .await
        .expect("Failed to connect to node");

    grpc_client
        .update_blockchain(Request::new(blockchain::node::UpdateBlockchainRequest {
            blocks: bc2.clone(),
        }))
        .await
        .unwrap();

    // submit a transaction to the network, expect the blockchain fork to be resolved
    let res = grpc_client.generate_transaction(Request::new(GenerateTransactionRequest {
        id: Uuid::new_v4().to_string(),
        sender: client.public_key.clone(),
        private_key: client.private_key.clone(),
        receiver: "receiver3".to_string(),
        amount: 50,
        fee: 2,
    }));

    let tx3 = res.await.unwrap().into_inner().transaction.unwrap();
    grpc_client
        .update_client_transaction(Request::new(UpdateTransactionRequest {
            transactions: vec![tx3.clone()],
        }))
        .await
        .unwrap();

    // wait for the transaction to be sent to the blockchain
    tokio::time::sleep(Duration::from_secs(1)).await;

    // eventually the two nodes should have the same blockchain
    let bc3 = grpc_client
        .get_blockchain(Request::new(GetBlockchainRequest {}))
        .await
        .unwrap()
        .into_inner()
        .chain;

    let bc4 = NodeMessageClient::connect(format!("http://127.0.0.1:{}", nodes[2]))
        .await
        .expect("Failed to connect to node")
        .get_blockchain(Request::new(GetBlockchainRequest {}))
        .await
        .unwrap()
        .into_inner()
        .chain;

    assert_eq!(bc3, bc4);
}
