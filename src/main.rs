mod auth;
mod cli;
mod commands;
mod compiler;
mod dependency;
mod lockfile;
mod manifest;
mod registry;
mod resolver;
mod terminal;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, version } => commands::add::execute(name, version).await?,

        Commands::Authenticate => commands::authenticate::execute().await?,

        Commands::Build { release } => commands::build::execute(release)?,

        Commands::Install => commands::install::execute()?,

        Commands::Clean => commands::clean::execute()?,

        Commands::Init { name } => commands::init::execute(name)?,

        Commands::Make => commands::make::execute()?,

        Commands::Pack => commands::pack::execute()?,

        Commands::Publish => commands::publish::execute().await?,

        Commands::Remove { name } => commands::remove::execute(name)?,

        Commands::SetType { package_type } => commands::set_type::execute(package_type)?,
    }

    Ok(())
}
