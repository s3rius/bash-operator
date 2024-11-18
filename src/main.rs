use clap::Parser;
use cli_args::Cli;
use error::BOResult;

pub mod cli_args;
pub mod error;
pub mod operator;
pub mod utils;

#[tokio::main]
async fn main() -> BOResult<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    operator::run_operator(args).await?;
    Ok(())
}
