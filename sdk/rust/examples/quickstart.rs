use muninn::MuninnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MuninnClient::new(
        "http://localhost:8476",
        std::env::var("MUNINN_TOKEN").unwrap_or_default().as_str(),
    );

    // Write an engram
    let resp = client
        .write(
            "my-vault",
            "rust-quickstart",
            "Rust SDK is working!",
            vec!["sdk".into(), "rust".into()],
        )
        .await?;
    println!("wrote engram: {}", resp.id);

    // Read it back
    let engram = client.read(&resp.id, "my-vault").await?;
    println!("read back: {} — {}", engram.concept, engram.content);

    // Activate with a query
    let activation = client
        .activate(&muninn::ActivateRequest::new(
            "my-vault",
            vec!["rust sdk".into()],
        ))
        .await?;
    println!("found {} activations", activation.total_found);
    for item in &activation.activations {
        println!("  [{:.2}] {}", item.score, item.concept);
    }

    Ok(())
}
