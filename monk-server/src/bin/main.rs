use std::env;

use monk_server::matrix::login_and_sync;
use tracing::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (homeserver_url, username, password) =
        match (env::args().nth(1), env::args().nth(2), env::args().nth(3)) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                eprintln!(
                    "Usage: {} <homeserver_url> <username> <password>",
                    env::args().next().unwrap()
                );
                std::process::exit(1)
            }
        };

    info!("Logging in and syncing");

    login_and_sync(homeserver_url, username, password)
        .await
        .map_err(Into::into)
}
