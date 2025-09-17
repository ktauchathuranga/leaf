use crate::config::Config;
use crate::installer::Installer;
use crate::package::{InstallInfo, Package, PackageRegistry};
use crate::utils::*;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tokio::fs;

pub struct PackageManager {
    config: Config,
    installer: Installer,
}

impl PackageManager {
    pub async fn new() -> Result<Self> {
        let config = Config::load_or_create().await?;
        let installer = Installer::new();

        Ok(Self { config, installer })
    }

    async fn load_packages(&self) -> Result<PackageRegistry> {
        let packages_file = self.config.packages_file();
        if !packages_file.exists() {
            print_error("Package definitions not found. Run 'leaf update' first.");
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(packages_file).await?;
        let packages: PackageRegistry = serde_json::from_str(&content)?;
        Ok(packages)
    }

    pub async fn install_package(&mut self, name: &str) -> Result<()> {
        let packages = self.load_packages().await?;

        let package = match packages.get(name) {
            Some(pkg) => pkg,
            None => {
                print_error(&format!("Package '{}' not found", name));
                print_info(&format!("Run 'leaf search {}' to find similar packages", name));
                return Ok(());
            }
        };

        let pkg_dir = self.config.packages_dir.join(name);

        // Check if already installed
        if pkg_dir.exists() {
            print_warning(&format!("Package '{}' is already installed", name));
            return Ok(());
        }

        println!("🍃 Installing {}...", bold(name));

        // Create package directory
        fs::create_dir_all(&pkg_dir).await?;

        // Download package
        let url = &package.url;
        let filename = url.split('/').last().unwrap();
        let cache_file = self.config.cache_dir.join(filename);

        if !cache_file.exists() {
            print_info(&format!("Downloading {}...", name));
            self.installer.download_file(url, &cache_file).await?;
        } else {
            print_info("Using cached download...");
        }

        // Extract if needed
        if package.package_type.as_deref() == Some("archive") {
            print_info("Extracting archive...");
            self.installer.extract_archive(&cache_file, &pkg_dir).await?;
        } else {
            // Copy binary directly
            fs::copy(&cache_file, pkg_dir.join(filename)).await?;
        }

        // Create symlinks for executables
        let executables = package.get_executables();
        for exe_info in &executables {
            let link_name = exe_info.name.as_deref().unwrap_or(name);
            let source_path = pkg_dir.join(&exe_info.path);

            if !source_path.exists() {
                print_warning(&format!("Executable not found: {}", exe_info.path));
                continue;
            }

            let link_path = self.config.bin_dir.join(link_name);

            // Remove existing symlink
            if link_path.exists() {
                fs::remove_file(&link_path).await?;
            }

            // Create new symlink
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                symlink(source_path.canonicalize()?, &link_path)?;
            }

            // Make executable
            let metadata = fs::metadata(&source_path).await?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&source_path, permissions).await?;
        }

        // Save installation info
        let install_info = InstallInfo {
            name: name.to_string(),
            version: package.version.clone(),
            installed_files: self.get_installed_files(&pkg_dir).await?,
            executables: package.executables.clone(),
        };

        let install_info_json = serde_json::to_string_pretty(&install_info)?;
        fs::write(pkg_dir.join("install.json"), install_info_json).await?;

        print_success(&format!("Successfully installed {}", name));
        Ok(())
    }

    pub async fn remove_package(&mut self, name: &str) -> Result<()> {
        let pkg_dir = self.config.packages_dir.join(name);

        if !pkg_dir.exists() {
            print_error(&format!("Package '{}' is not installed", name));
            return Ok(());
        }

        println!("🗑️  Removing {}...", name);

        // Load install info
        let install_info_file = pkg_dir.join("install.json");
        if install_info_file.exists() {
            let content = fs::read_to_string(&install_info_file).await?;
            let install_info: InstallInfo = serde_json::from_str(&content)?;

            // Remove symlinks
            if let Some(executables) = &install_info.executables {
                let package = Package {
                    description: String::new(),
                    url: String::new(),
                    version: install_info.version.clone(),
                    package_type: None,
                    executables: Some(executables.clone()),
                    tags: None,
                };

                let executables = package.get_executables();
                for exe_info in executables {
                    let link_name = exe_info.name.as_deref().unwrap_or(name);
                    let link_path = self.config.bin_dir.join(link_name);

                    if link_path.exists() {
                        fs::remove_file(&link_path).await?;
                    }
                }
            }
        }

        // Remove package directory
        fs::remove_dir_all(&pkg_dir).await?;

        print_success(&format!("Successfully removed {}", name));
        Ok(())
    }

    pub async fn list_packages(&self) -> Result<()> {
        let mut installed = Vec::new();

        if !self.config.packages_dir.exists() {
            print_info("No packages installed");
            return Ok(());
        }

        let mut entries = fs::read_dir(&self.config.packages_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let pkg_name = entry.file_name().to_string_lossy().to_string();
                let install_info_file = entry.path().join("install.json");

                if install_info_file.exists() {
                    let content = fs::read_to_string(&install_info_file).await?;
                    let install_info: InstallInfo = serde_json::from_str(&content)?;
                    installed.push((install_info.name, install_info.version));
                } else {
                    installed.push((pkg_name, "unknown".to_string()));
                }
            }
        }

        if installed.is_empty() {
            print_info("No packages installed");
            return Ok(());
        }

        println!("{}", bold("Installed packages:"));
        installed.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, version) in installed {
            println!("  {} {} ({})", "•".green(), name, version);
        }

        Ok(())
    }

    pub async fn search_packages(&self, term: &str) -> Result<()> {
        let packages = self.load_packages().await?;

        if packages.is_empty() {
            return Ok(());
        }

        let mut matches = Vec::new();
        let term_lower = term.to_lowercase();

        for (name, pkg) in &packages {
            let name_match = name.to_lowercase().contains(&term_lower);
            let desc_match = pkg.description.to_lowercase().contains(&term_lower);
            let tag_match = pkg
                .tags
                .as_ref()
                .map(|tags| tags.iter().any(|tag| tag.to_lowercase().contains(&term_lower)))
                .unwrap_or(false);

            if name_match || desc_match || tag_match {
                matches.push((name.clone(), pkg.clone()));
            }
        }

        if matches.is_empty() {
            print_info(&format!("No packages found matching '{}'", term));
            return Ok(());
        }

        println!("{}", bold(&format!("Found {} package(s):", matches.len())));
        matches.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, pkg) in matches {
            println!(
                "  {} {} - {}",
                "•".green(),
                bold(&name),
                pkg.description
            );
        }

        Ok(())
    }

    pub async fn update_packages(&mut self) -> Result<()> {
        println!("🔄 Updating package list...");

        let packages_url = "https://raw.githubusercontent.com/ktauchathuranga/leaf/main/packages.json";
        let packages_file = self.config.packages_file();

        self.installer
            .download_file(packages_url, &packages_file)
            .await?;

        print_success("Package list updated successfully");
        Ok(())
    }

    async fn get_installed_files(&self, pkg_dir: &Path) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(pkg_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(relative_path) = entry.path().strip_prefix(pkg_dir).ok() {
                    files.push(relative_path.to_string_lossy().to_string());
                }
            }
        }

        Ok(files)
    }
}