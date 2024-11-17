use clap::Parser;
use cli_args::Cli;

pub mod cli_args;
pub mod operator;
pub mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    match args.subcommand {
        cli_args::Cmds::Operator(operator_args) => {
            operator::run_operator(operator_args).await?;
        }
        cli_args::Cmds::Utils { subcommand } => match subcommand {
            cli_args::UtilsSub::AddFinalizer {
                finalizer_name,
                path_to_mainfest,
            } => {
                utils::update_finalizer(&finalizer_name, &path_to_mainfest, true).await?;
            }
            cli_args::UtilsSub::RemoveFinalizer {
                finalizer_name,
                path_to_mainfest,
            } => {
                utils::update_finalizer(&finalizer_name, &path_to_mainfest, false).await?;
            }
        },
    };
    Ok(())
}
