pub mod collection;
pub mod executor;
pub mod variables;

pub async fn run() -> anyhow::Result<()> {
    println!("API Tester launching...");
    Ok(())
}
