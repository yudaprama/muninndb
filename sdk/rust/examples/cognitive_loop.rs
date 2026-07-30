use futures::StreamExt;
use muninn::MuninnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MuninnClient::new(
        "http://localhost:8476",
        std::env::var("MUNINN_TOKEN").unwrap_or_default().as_str(),
    );
    let vault = "cognitive-demo";

    // 1. Write several related engrams
    let id1 = client
        .write(vault, "architecture", "MuninnDB uses a Pebble LSM key-value store", vec!["design".into()])
        .await?;
    let id2 = client
        .write(vault, "recall", "Recall uses predictive activation scoring", vec!["cognitive".into()])
        .await?;
    let id3 = client
        .write(vault, "decay", "Memory fades with Ebbinghaus temporal decay", vec!["cognitive".into()])
        .await?;
    println!("wrote 3 engrams: {}, {}, {}", id1.id, id2.id, id3.id);

    // 2. Link them
    client
        .link(&muninn::LinkRequest {
            vault: vault.to_string(),
            source_id: id1.id.clone(),
            target_id: id2.id.clone(),
            rel_type: 1,
            weight: 0.8,
        })
        .await?;
    client
        .link(&muninn::LinkRequest {
            vault: vault.to_string(),
            source_id: id2.id.clone(),
            target_id: id3.id.clone(),
            rel_type: 1,
            weight: 0.6,
        })
        .await?;
    println!("linked engrams");

    // 3. Activate and traverse
    let activation = client
        .activate(
            &muninn::ActivateRequest::new(vault, vec!["memory system".into()])
                .max_results(5)
                .include_why(true),
        )
        .await?;
    println!("activations: {}", activation.total_found);

    let graph = client
        .traverse(vault, &id1.id, 2, 10, None, None)
        .await?;
    println!(
        "traversal: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    // 4. Subscribe to live events
    println!("subscribing to vault events...");
    let mut events = client.subscribe(vault).await?;
    while let Some(event) = events.next().await {
        match event {
            Ok(e) => println!("event [{}]: {}", e.event_type, &e.data[..e.data.len().min(120)]),
            Err(e) => {
                eprintln!("sse error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
