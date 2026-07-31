use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cakeman",
    version,
    about = "A C/C++ package manager and build system"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a dependency
    Add {
        /// Dependency name
        name: String,

        /// Dependency version (optional)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Authenticate to GitHub
    Authenticate,

    /// Build the project
    Build {
        #[arg(short, long)]
        release: bool,
    },

    /// Remove build artifacts
    Clean,

    /// Create a new project
    Init { name: Option<String> },

    /// Make/build package
    Make,

    /// Package project
    Pack,

    /// Publish package
    Publish,

    /// Remove dependency
    Remove { name: String },

    /// Change project type
    SetType { package_type: String },
}
