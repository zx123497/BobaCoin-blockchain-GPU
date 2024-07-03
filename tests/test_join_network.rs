use blockchain::node::node_message_client::NodeMessageClient;
use blockchain::node::GetPeerListRequest;
use blockchain::start;
use std::time::Duration;
use tonic::Request;

#[tokio::test]
async fn test_join_network_10_percents() {
    let mut tasks = Vec::new();

    let nodes = vec![50000, 50001, 50002];
    tasks.push(tokio::spawn(start(nodes[0], None)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[1], Some(nodes[0]))));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tasks.push(tokio::spawn(start(nodes[2], Some(nodes[1]))));
    tokio::time::sleep(Duration::from_millis(200)).await;
    // Wait for the nodes to start

    for node in &nodes {
        let mut client = NodeMessageClient::connect(format!("http://127.0.0.1:{}", node))
            .await
            .expect("Failed to connect to node");
        let res = client
            .get_peer_list(Request::new(GetPeerListRequest {}))
            .await
            .unwrap();
        let peers = res.into_inner().nodes;
        assert_eq!(peers.len(), 3);
        assert!(peers.iter().any(|peer| peer.port == nodes[0] as u32));
        assert!(peers.iter().any(|peer| peer.port == nodes[1] as u32));
        assert!(peers.iter().any(|peer| peer.port == nodes[2] as u32));
    }
    for task in tasks {
        task.abort();
    }
}
