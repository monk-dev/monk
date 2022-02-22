use tracing::metadata::LevelFilter;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(LevelFilter::INFO.into())
                .add_directive("sqlx::query=warn".parse()?)
                .add_directive("html5ever::serialize=off".parse()?)
                .add_directive("tantivy=warn".parse()?),
        )
        .finish()
        .init();

    let path = std::env::args().nth(1).unwrap();
    let text = std::fs::read_to_string(path).unwrap();

    let summary = monk_summary::summarize(&text);
    println!("{summary}");

    Ok(())
}
