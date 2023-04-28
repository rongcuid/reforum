use color_eyre::*;
use reforum::startup::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;
    Ok(())
}
