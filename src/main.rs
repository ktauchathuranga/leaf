mod package_manager;
mod config;
mod package;
mod installer;
mod utils;

use clap::{Parser, Subcommand};
use anyhow::Result;
use package_manager::PackageManager;

#[derive(Parser)]
#[command(name = "leaf")]
#[command(about = "🍃 Leaf Package Manager - A simple, sudo-free package manager")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package
    Install {
        /// Package name to install
        package: String,
    },
    /// Remove an installed package
    Remove {
        /// Package name to remove
        package: String,
    },
    /// List installed packages
    List,
    /// Search for packages
    Search {
        /// Search term
        term: String,
    },
    /// Update package definitions
    Update,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut pm = PackageManager::new().await?;

    match cli.command {
        Commands::Install { package } => {
            pm.install_package(&package).await?;
        }
        Commands::Remove { package } => {
            pm.remove_package(&package).await?;
        }
        Commands::List => {
            pm.list_packages().await?;
        }
        Commands::Search { term } => {
            pm.search_packages(&term).await?;
        }
        Commands::Update => {
            pm.update_packages().await?;
        }
    }

    Ok(())
}