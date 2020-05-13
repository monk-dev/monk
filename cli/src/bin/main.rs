use cli::{error::Error, Args};
use structopt::StructOpt;

use daemon::server::{request::Request, response::Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let args = Args::from_args();

    let request = Request::Add {
        name: "Operating Systems".into(),
        url: "https://memes.are.dreams".parse().unwrap(),
    };

    let response: Response = reqwest::Client::new()
        .get("http://127.0.0.1:8888/add")
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    println!("[resp] {:?}", response);

    // cli::run(args)
    Ok(())
}
