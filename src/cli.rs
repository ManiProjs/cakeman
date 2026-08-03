use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cman",
    version = "v0.1.0",
    about = "A no-nonsense C/C++ package manager"
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

    /// Install a binary
    Install {
        name: String,
    },

    Uninstall {
        name: String,
    },

    /// Remove build artifacts
    Clean,

    /// Create a new project
    Init {
        name: Option<String>,
    },

    /// Make/build package
    Make,

    /// Package project
    Pack,

    /// Publish package
    Publish,

    /// Remove dependency
    Remove {
        name: String,
    },

    /// Change project type
    SetType {
        package_type: String,
    },
}
