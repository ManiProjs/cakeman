mod cli;
mod commands;
mod lockfile;
mod manifest;
mod registry;
mod terminal;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, version } => commands::add::execute(name, version)?,

        Commands::Authenticate => commands::authenticate::execute()?,

        Commands::Build { release } => commands::build::execute(release)?,

        Commands::Clean => commands::clean::execute()?,

        Commands::Init { name } => commands::init::execute(name)?,

        Commands::Make => commands::make::execute()?,

        Commands::Pack => commands::pack::execute()?,

        Commands::Publish => commands::publish::execute()?,

        Commands::Remove { name } => commands::remove::execute(name)?,

        Commands::SetType { package_type } => commands::set_type::execute(package_type)?,
    }

    Ok(())
}
