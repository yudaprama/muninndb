use muninn::MuninnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MuninnClient::new(
        "http://localhost:8476",
        std::env::var("MUNINN_TOKEN").unwrap_or_default().as_str(),
    );
    let vault = "lifecycle-demo";

    // Write
    let w = client
        .write(vault, "auth-strategy", "Use JWT tokens for API auth", vec!["auth".into()])
        .await?;
    println!("created: {}", w.id);

    // Evolve — the strategy changed
    let e = client
        .evolve(&w.id, vault, "Switched to API keys for simplicity", "team decision")
        .await?;
    println!("evolved: {}", e.id);

    // Read the latest version
    let engram = client.read(&e.id, vault).await?;
    println!("current content: {}", engram.content);

    // Consolidate two engrams
    let w2 = client
        .write(vault, "auth-detail", "API keys are stored in env vars", vec!["auth".into()])
        .await?;
    let merged = client
        .consolidate(vault, vec![w.id.clone(), w2.id.clone()], "Auth uses API keys stored in env vars; JWT deprecated")
        .await?;
    println!("consolidated: {} (archived: {:?})", merged.id, merged.archived);

    // Decide
    let d = client
        .decide(
            vault,
            "Use Redis for session cache",
            "Redis is fast and well-understood",
            Some(vec!["Memcached".into(), "SQLite".into()]),
            None,
        )
        .await?;
    println!("decision recorded: {}", d.id);

    // Restore a deleted engram
    client.forget(&w.id, vault).await?;
    let restored = client.restore(&w.id, vault).await?;
    println!(
        "restored: {} — {} (state: {})",
        restored.id, restored.restored, restored.state
    );

    // Check contradictions
    let c = client.contradictions(vault).await?;
    println!("contradictions found: {}", c.contradictions.len());

    // Get a guide
    let g = client.guide(vault).await?;
    println!("vault guide:\n{}", g);

    Ok(())
}
