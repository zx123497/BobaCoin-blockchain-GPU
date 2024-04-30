use blockchain::start;

#[tokio::test]
async fn test_join_network() {
    let mut tasks = Vec::new();

    tasks.push(tokio::spawn(async move { start(50000, None) }));
    tasks.push(tokio::spawn(async move { start(50001, Some(50000)) }));
    tasks.push(tokio::spawn(async move { start(50002, Some(50000)) }));

    // do some tests here

    for task in tasks {
        task.abort();
    }
}
