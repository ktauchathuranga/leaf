mod config;
mod installer;
mod package;
mod package_manager;
mod utils;

use crate::package_manager::PackageManager;
use crate::utils::print_error;
use clap::{Arg, Command};
use std::process;

#[tokio::main]
async fn main() {
    let matches = Command::new("leaf")
        .version("1.0.0")
        .author("ktauchathuranga")
        .about("🍃 A simple, sudo-free package manager for Linux")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("install").about("Install a package").arg(
                Arg::new("package")
                    .help("Package name to install")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove an installed package")
                .arg(
                    Arg::new("package")
                        .help("Package name to remove")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("list").about("List installed packages"))
        .subcommand(
            Command::new("search")
                .about("Search available packages")
                .arg(Arg::new("term").help("Search term").required(true).index(1)),
        )
        .subcommand(Command::new("update").about("Update package definitions"))
        .subcommand(
            Command::new("nuke")
                .about("Remove all packages and Leaf itself (DESTRUCTIVE)")
                .arg(
                    Arg::new("confirmed")
                        .long("confirmed")
                        .help("Confirm the nuclear option")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("self-update").about("Update the leaf package manager itself"))
        .get_matches();

    let mut pm = match PackageManager::new().await {
        Ok(pm) => pm,
        Err(e) => {
            print_error(&format!("Failed to initialize package manager: {}", e));
            process::exit(1);
        }
    };

    let result = match matches.subcommand() {
        Some(("install", sub_matches)) => {
            let package = sub_matches.get_one::<String>("package").unwrap();
            pm.install_package(package).await
        }
        Some(("remove", sub_matches)) => {
            let package = sub_matches.get_one::<String>("package").unwrap();
            pm.remove_package(package).await
        }
        Some(("list", _)) => pm.list_packages().await,
        Some(("search", sub_matches)) => {
            let term = sub_matches.get_one::<String>("term").unwrap();
            pm.search_packages(term).await
        }
        Some(("update", _)) => pm.update_packages().await,
        Some(("nuke", sub_matches)) => {
            let confirmed = sub_matches.get_flag("confirmed");
            pm.nuke_everything(confirmed).await
        }
        Some(("self-update", _)) => pm.self_update().await,
        _ => {
            print_error("Unknown command");
            Ok(())
        }
    };

    if let Err(e) = result {
        print_error(&format!("Error: {}", e));
        process::exit(1);
    }
}
