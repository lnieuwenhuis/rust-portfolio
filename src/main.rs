#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lars_portfolio::run().await
}
