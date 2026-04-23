mod chrome;
mod err;
mod srv;
mod tools;

use rmcp::ServiceExt;
use srv::Srv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter("info")
        .init();

    let (b, _jh) = chrome::init(9222).await?;
    let s = Srv::new(b)
        .serve(rmcp::transport::io::stdio())
        .await?;
    s.waiting().await?;
    Ok(())
}
