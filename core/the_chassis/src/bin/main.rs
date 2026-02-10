use anyhow::Result;
use the_chassis;

#[tokio::main]
async fn main() -> Result<()> {
    the_chassis::run().await
}
