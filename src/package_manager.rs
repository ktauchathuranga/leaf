use crate::config::Config;
use crate::installer::Installer;
use crate::package::Package;
use crate::utils::{print_error, print_info, print_success, print_warning};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use tokio::fs;

pub struct PackageManager {
    config: Config,
    packages: HashMap<String, Package>,
    installed: HashMap<String, Package>,
}

impl PackageManager {
    pub async fn new() -> Result<Self> {
        let config = Config::load_or_create().await?;

        // Ensure directories exist
        fs::create_dir_all(&config.packages_dir).await?;
        fs::create_dir_all(&config.cache_dir).await?;
        fs::create_dir_all(&config.bin_dir).await?;

        let mut pm = PackageManager {
            config,
            packages: HashMap::new(),
            installed: HashMap::new(),
        };

        pm.load_packages().await?;
        pm.load_installed().await?;

        Ok(pm)
    }

    async fn load_packages(&mut self) -> Result<()> {
        let packages_file = self.config.install_dir.join("packages.json");

        if !packages_file.exists() {
            // Use Box::pin to handle async recursion
            Box::pin(self.update_packages()).await?;
        }

        if packages_file.exists() {
            let content = fs::read_to_string(&packages_file).await?;

            // Debug: Print first 100 characters to see what we got
            if content.trim().is_empty() {
                print_error("Downloaded packages.json is empty");
                return Ok(());
            }

            // Check if content looks like HTML (common issue with GitHub URLs)
            if content.trim_start().starts_with("<!DOCTYPE html>")
                || content.trim_start().starts_with("<html")
            {
                print_error("Downloaded packages.json appears to be HTML instead of JSON");
                print_error("This usually means the download URL is incorrect");
                fs::remove_file(&packages_file).await.ok(); // Remove the bad file
                return Ok(());
            }

            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    if let Some(obj) = json.as_object() {
                        for (name, value) in obj {
                            if let Ok(package) = serde_json::from_value::<Package>(value.clone()) {
                                self.packages.insert(name.clone(), package);
                            }
                        }
                    }
                }
                Err(e) => {
                    print_error(&format!("Failed to parse packages.json: {}", e));
                    print_error("Content preview:");
                    let preview = content.chars().take(200).collect::<String>();
                    print_error(&format!("'{}'", preview));
                    return Err(anyhow::anyhow!("Invalid packages.json format"));
                }
            }
        }

        Ok(())
    }

    async fn load_installed(&mut self) -> Result<()> {
        if !self.config.packages_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&self.config.packages_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let package_name = entry.file_name().to_string_lossy().to_string();
                let metadata_file = entry.path().join("leaf-package.json");

                if metadata_file.exists() {
                    let content = fs::read_to_string(&metadata_file).await?;
                    if let Ok(package) = serde_json::from_str::<Package>(&content) {
                        self.installed.insert(package_name, package);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn install_package(&mut self, name: &str) -> Result<()> {
        if self.installed.contains_key(name) {
            print_warning(&format!("Package '{}' is already installed", name));
            return Ok(());
        }

        let package = self
            .packages
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found", name))?
            .clone();

        print_info(&format!("Installing {}...", name));

        let installer = Installer::new();
        installer
            .install_package(name, &package, &self.config)
            .await?;

        // Create symlinks for executables
        let package_dir = self.config.packages_dir.join(name);
        for executable_info in package.get_executables() {
            let exe_path = package_dir.join(&executable_info.path);

            // Fix borrowing issue by creating owned strings
            let default_name = executable_info
                .path
                .split('/')
                .last()
                .unwrap_or("executable")
                .to_string();
            let symlink_name = executable_info.name.as_ref().unwrap_or(&default_name);
            let symlink_path = self.config.bin_dir.join(symlink_name);

            if exe_path.exists() {
                if symlink_path.exists() {
                    fs::remove_file(&symlink_path).await?;
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    symlink(&exe_path, &symlink_path)?;
                }

                #[cfg(not(unix))]
                {
                    fs::copy(&exe_path, &symlink_path).await?;
                }
            }
        }

        // Save package metadata
        let metadata_file = package_dir.join("leaf-package.json");
        let metadata = serde_json::to_string_pretty(&package)?;
        fs::write(&metadata_file, metadata).await?;

        self.installed.insert(name.to_string(), package);

        print_success(&format!("Successfully installed {}", name));
        Ok(())
    }

    pub async fn remove_package(&mut self, name: &str) -> Result<()> {
        if !self.installed.contains_key(name) {
            print_warning(&format!("Package '{}' is not installed", name));
            return Ok(());
        }

        print_info(&format!("Removing {}...", name));

        let package_dir = self.config.packages_dir.join(name);

        // Remove symlinks
        if let Some(package) = self.installed.get(name) {
            for executable_info in package.get_executables() {
                // Fix borrowing issue by creating owned strings
                let default_name = executable_info
                    .path
                    .split('/')
                    .last()
                    .unwrap_or("executable")
                    .to_string();
                let symlink_name = executable_info.name.as_ref().unwrap_or(&default_name);
                let symlink_path = self.config.bin_dir.join(symlink_name);

                if symlink_path.exists() {
                    fs::remove_file(&symlink_path).await?;
                }
            }
        }

        // Remove package directory
        if package_dir.exists() {
            fs::remove_dir_all(&package_dir).await?;
        }

        self.installed.remove(name);

        print_success(&format!("Successfully removed {}", name));
        Ok(())
    }

    pub async fn list_packages(&self) -> Result<()> {
        if self.installed.is_empty() {
            print_info("No packages installed");
            return Ok(());
        }

        println!("Installed packages:");
        for (name, package) in &self.installed {
            println!("  {} - {} ({})", name, package.description, package.version);
        }

        Ok(())
    }

    pub async fn search_packages(&self, term: &str) -> Result<()> {
        let mut found = Vec::new();
        let term_lower = term.to_lowercase();

        for (name, package) in &self.packages {
            let matches_name = name.to_lowercase().contains(&term_lower);
            let matches_desc = package.description.to_lowercase().contains(&term_lower);
            let matches_tags = package.tags.as_ref().map_or(false, |tags| {
                tags.iter()
                    .any(|tag| tag.to_lowercase().contains(&term_lower))
            });

            if matches_name || matches_desc || matches_tags {
                found.push((name, package));
            }
        }

        if found.is_empty() {
            print_info(&format!("No packages found matching '{}'", term));
            return Ok(());
        }

        println!("Found {} package(s):", found.len());
        for (name, package) in found {
            let installed = if self.installed.contains_key(name) {
                " [INSTALLED]"
            } else {
                ""
            };
            println!(
                "  {}{} - {} ({})",
                name, installed, package.description, package.version
            );
            if let Some(tags) = &package.tags {
                if !tags.is_empty() {
                    println!("    Tags: {}", tags.join(", "));
                }
            }
        }

        Ok(())
    }

    pub async fn update_packages(&mut self) -> Result<()> {
        print_info("Updating package definitions...");

        let packages_url =
            "https://raw.githubusercontent.com/ktauchathuranga/leaf/main/packages.json";
        let packages_file = self.config.install_dir.join("packages.json");

        let client = reqwest::Client::builder()
            .user_agent("leaf-package-manager/1.0.0")
            .build()?;

        match client.get(packages_url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let content = response.text().await?;

                    match serde_json::from_str::<Value>(&content) {
                        Ok(_) => {
                            fs::write(&packages_file, &content).await?;

                            // Clear and reload packages from the new file
                            self.packages.clear();
                            let json: Value = serde_json::from_str(&content)?;
                            if let Some(obj) = json.as_object() {
                                for (name, value) in obj {
                                    if let Ok(package) =
                                        serde_json::from_value::<Package>(value.clone())
                                    {
                                        self.packages.insert(name.clone(), package);
                                    }
                                }
                            }

                            print_success("Package definitions updated successfully");
                            Ok(())
                        }
                        Err(e) => Err(anyhow::anyhow!(
                            "Downloaded content is not valid JSON: {}",
                            e
                        )),
                    }
                } else {
                    let error_body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Could not read error body".to_string());
                    Err(anyhow::anyhow!(
                        "Failed to download packages.json: HTTP {} - {}",
                        status,
                        error_body
                    ))
                }
            }
            Err(e) => Err(anyhow::anyhow!(
                "Network error downloading packages.json: {}",
                e
            )),
        }
    }

    pub async fn nuke_everything(&self, confirmed: bool) -> Result<()> {
        if !confirmed {
            print_error("This will completely remove all packages and Leaf itself!");
            print_error("This action cannot be undone.");
            print_error("");
            print_error("If you're sure, run: leaf nuke --confirmed");
            return Ok(());
        }

        print_warning("NUCLEAR OPTION ACTIVATED!");
        print_warning("Removing all packages and Leaf itself...");

        // Remove all installed packages first
        print_info("Removing all installed packages...");

        // Remove all symlinks in bin directory
        if self.config.bin_dir.exists() {
            let mut entries = fs::read_dir(&self.config.bin_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_symlink() {
                    // Check if it's a leaf-managed symlink
                    if let Ok(target) = fs::read_link(&path).await {
                        if target.to_string_lossy().contains(".local/leaf/packages") {
                            fs::remove_file(&path).await?;
                            print_info(&format!("Removed symlink: {}", path.display()));
                        }
                    }
                }
            }
        }

        // Remove the entire leaf directory
        if self.config.install_dir.exists() {
            fs::remove_dir_all(&self.config.install_dir).await?;
            print_info(&format!(
                "Removed leaf directory: {}",
                self.config.install_dir.display()
            ));
        }

        print_success("🍃 Leaf and all packages have been nuked!");
        print_info("To complete the uninstallation, please remove the executable:");
        print_info(&format!(
            "  rm {}",
            self.config.bin_dir.join("leaf").display()
        ));

        Ok(())
    }

    pub async fn self_update(&self) -> Result<()> {
        print_info("Checking for new version of Leaf...");

        let install_script_url =
            "https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh";

        print_info("Running the installation/update script...");

        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("curl -sSL {} | bash", install_script_url))
            .status()
            .with_context(|| "Failed to start the update process.")?;

        if status.success() {
            // The script already prints success messages. We can add one more.
            print_success("Update process completed.");
            print_info("Please restart your terminal if you encounter any issues.");
        } else {
            return Err(anyhow::anyhow!(
                "The update script failed to run. Check the output above for details."
            ));
        }

        Ok(())
    }
}

// ==================
// |   TEST SUITE   |
// ==================
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::fs;

    /// This test reads the `packages.json` file from the project root and sends an HTTP HEAD
    /// request to each package's URL to confirm it is reachable and not a broken link.
    #[tokio::test]
    async fn test_package_urls_are_valid() {
        // Read the packages.json file from the project root
        let content = fs::read_to_string("packages.json")
            .await
            .expect("Failed to read packages.json. Make sure it's in the project root.");

        // Parse the JSON content
        let packages: HashMap<String, Package> = serde_json::from_str(&content)
            .expect("Failed to parse packages.json. Check for syntax errors.");

        // Create a reqwest client with a User-Agent to avoid being blocked
        let client = reqwest::Client::builder()
            .user_agent("leaf-package-manager-test-suite/1.0")
            .build()
            .unwrap();

        let mut failed_urls = Vec::new();

        // Iterate through all packages and test their URLs
        for (name, package) in packages {
            let url = &package.url;
            println!("- Testing URL for package '{}': {}", name, url);

            // Send a HEAD request, which is lightweight and ideal for checking links
            let response = client.head(url).send().await;

            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        println!("  ✓ Success ({})", res.status());
                    } else {
                        println!("  ✗ Failure ({})", res.status());
                        failed_urls.push(format!("'{}': {} (Status: {})", name, url, res.status()));
                    }
                }
                Err(e) => {
                    println!("  ✗ Network Error: {}", e);
                    failed_urls.push(format!("'{}': {} (Error: {})", name, url, e));
                }
            }
        }

        // Assert that there were no failed URLs
        assert!(
            failed_urls.is_empty(),
            "One or more package URLs are invalid:\n- {}\n",
            failed_urls.join("\n- ")
        );
    }
}
