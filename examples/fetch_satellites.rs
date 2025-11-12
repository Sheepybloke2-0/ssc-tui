/// Example: Fetch satellites from NASA SSC API
///
/// Run with: cargo run --example fetch_satellites
use ssc_tui::ssc::SscClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Fetching satellites from NASA SSC API...\n");

    let mut client = SscClient::new()?;

    let satellites = client.get_satellites().await?;

    println!("✓ Fetched {} satellites\n", satellites.len());

    // Display first 10 satellites
    println!("First 10 satellites:");
    println!("{:<15} {:<30} {:<10} {:<20}", "ID", "Name", "Resolution", "Status");
    println!("{}", "-".repeat(80));

    for sat in satellites.iter().take(10) {
        println!(
            "{:<15} {:<30} {:<10} {}",
            sat.id,
            sat.name,
            sat.resolution_str(),
            sat.description()
        );
    }

    println!("\n--- Quick Lookup Test ---\n");

    // Test quick lookup
    if let Some(iss) = client.get_satellite_info("iss").await? {
        println!("ISS Details:");
        println!("  ID: {}", iss.id);
        println!("  Name: {}", iss.name);
        println!("  Resolution: {}", iss.resolution_str());
        println!("  Start: {}", iss.start_time);
        println!("  End: {}", iss.end_time);
        if let Some(resource_id) = &iss.resource_id {
            println!("  SPASE ID: {}", resource_id);
        }
    }

    println!("\n--- Search Test ---\n");

    // Test search
    let results = client.search_satellites("geo").await?;
    println!("Satellites matching 'geo': {}", results.len());
    for sat in results.iter().take(5) {
        println!("  - {} ({})", sat.name, sat.id);
    }

    println!("\nCache stats: {} satellites cached", client.cached_count());

    Ok(())
}
