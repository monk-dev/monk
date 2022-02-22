use monk::types::config::MonkConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = MonkConfig::default();
    let _monk = monk_server::run(config).await?;

    Ok(())
}
