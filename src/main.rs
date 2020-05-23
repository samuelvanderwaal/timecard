use warp::Filter;
use anyhow::Result;

mod db;

#[tokio::main]
async fn main() -> Result<()> {
    run().await;

    Ok(())
}

async fn run() {
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
