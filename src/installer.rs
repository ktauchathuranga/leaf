use crate::config::Config;
use crate::package::{Package, PlatformDetails};
use crate::utils::{print_info, print_step, print_success};
use anyhow::{Result, anyhow};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use xz2::read::XzDecoder;

fn extract_archive_sync(archive_path: &Path, extract_to: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let filename = archive_path.file_name().unwrap().to_string_lossy();

    if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(extract_to)?;
    } else if filename.ends_with(".tar.xz") {
        let decoder = XzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(extract_to)?;
    } else {
        return Err(anyhow!("Unsupported archive format: {}", filename));
    }

    Ok(())
}

/// Parse filename from Content-Disposition header, handling both regular and RFC 5987 encoded formats
fn parse_content_disposition_filename(content_disposition: &str) -> Option<String> {
    // Handle RFC 5987 encoded filenames: filename*=UTF-8''example.zip
    if let Some(encoded_part) = content_disposition.split("filename*=").nth(1) {
        if let Some(filename_part) = encoded_part.split("''").nth(1) {
            // Simple URL decoding for basic cases (just remove %XX sequences)
            let decoded = filename_part.replace("%20", " ");
            return Some(decoded);
        }
    }

    // Handle regular filenames: filename="example.zip" or filename=example.zip
    if let Some(regular_part) = content_disposition.split("filename=").nth(1) {
        let filename = regular_part.split(';').next().unwrap_or(regular_part);
        return Some(filename.trim_matches('"').to_string());
    }

    None
}

/// Sanitize filename for the current platform
fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = filename.to_string();

    // Remove or replace invalid characters for Windows
    if cfg!(windows) {
        // Windows invalid characters: < > : " | ? * \ /
        sanitized = sanitized
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\\' | '/' => '_',
                _ => c,
            })
            .collect();
    }

    // Ensure we have a reasonable fallback filename
    if sanitized.is_empty() {
        sanitized = "download".to_string();
    }

    sanitized
}

pub struct Installer {
    client: Client,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn install_package(
        &self,
        name: &str,
        _package: &Package,
        platform_details: &PlatformDetails,
        config: &Config,
    ) -> Result<()> {
        let package_dir = config.packages_dir.join(name);
        let cache_dir = &config.cache_dir;

        fs::create_dir_all(&package_dir).await?;
        fs::create_dir_all(cache_dir).await?;

        // Download the file
        let cache_file_path = self.download_file(&platform_details.url, cache_dir).await?;

        let package_type = platform_details
            .package_type
            .as_deref()
            .unwrap_or("archive");

        match package_type {
            "archive" => {
                print_step("Extracting archive...");
                let extract_path = package_dir.clone();
                tokio::task::spawn_blocking(move || {
                    extract_archive_sync(&cache_file_path, &extract_path)
                })
                .await??;
            }
            "binary" => {
                print_step("Installing binary...");
                let executables = platform_details.get_executables();
                let executable = executables.get(0).ok_or_else(|| {
                    anyhow!("Binary package '{}' has no executables listed", name)
                })?;

                let dest_path = package_dir.join(&executable.path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                fs::copy(&cache_file_path, &dest_path).await?;

                let mut perms = fs::metadata(&dest_path).await?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest_path, perms).await?;
            }
            "build" => {
                print_step("Building from source...");
                self.build_from_source(name, platform_details, &cache_file_path, &package_dir)
                    .await?;
            }
            _ => {
                return Err(anyhow!("Unsupported package type: {}", package_type));
            }
        }

        print_success(&format!("Installation complete for '{}'", name));
        Ok(())
    }

    async fn build_from_source(
        &self,
        name: &str,
        platform_details: &PlatformDetails,
        cache_file_path: &Path,
        package_dir: &Path,
    ) -> Result<()> {
        // Create a temporary build directory
        let build_dir = package_dir.join("build_temp");
        fs::create_dir_all(&build_dir).await?;

        // Extract source code to build directory
        print_step("Extracting source code...");
        tokio::task::spawn_blocking({
            let cache_file_path = cache_file_path.to_path_buf();
            let build_dir = build_dir.clone();
            move || extract_archive_sync(&cache_file_path, &build_dir)
        })
        .await??;

        // Get build commands
        let build_commands = platform_details.get_build_commands();
        if build_commands.is_empty() {
            return Err(anyhow!(
                "No build commands specified for package '{}'",
                name
            ));
        }

        // Find the actual source directory (often extracted archives create a subdirectory)
        let source_dir = self.find_source_directory(&build_dir).await?;

        // Execute build commands
        print_step("Running build commands...");
        for (i, command) in build_commands.iter().enumerate() {
            print_info(&format!("Step {}/{}: {}", i + 1, build_commands.len(), command));

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&source_dir)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(anyhow!(
                    "Build command failed: {}\nStdout: {}\nStderr: {}",
                    command,
                    stdout,
                    stderr
                ));
            }
        }

        // Copy built executables to package directory
        print_step("Installing built executables...");
        for executable_info in platform_details.get_executables() {
            let source_exe = source_dir.join(&executable_info.path);
            let dest_exe = package_dir.join(&executable_info.path);

            if !source_exe.exists() {
                return Err(anyhow!(
                    "Built executable not found: {}",
                    source_exe.display()
                ));
            }

            if let Some(parent) = dest_exe.parent() {
                fs::create_dir_all(parent).await?;
            }

            fs::copy(&source_exe, &dest_exe).await?;

            // Make executable
            let mut perms = fs::metadata(&dest_exe).await?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_exe, perms).await?;
        }

        // Clean up build directory
        fs::remove_dir_all(&build_dir).await?;

        Ok(())
    }

    async fn find_source_directory(&self, build_dir: &Path) -> Result<PathBuf> {
        let mut entries = fs::read_dir(build_dir).await?;

        // Check if there's a single directory in the build dir (common for extracted archives)
        let mut dirs = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                dirs.push(entry.path());
            }
        }

        if dirs.len() == 1 {
            Ok(dirs.into_iter().next().unwrap())
        } else {
            // If multiple directories or no directories, use the build dir itself
            Ok(build_dir.to_path_buf())
        }
    }

    pub async fn download_file(&self, url: &str, cache_dir: &Path) -> Result<PathBuf> {
        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        let content_disposition = response
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok());

        let filename = if let Some(cd) = content_disposition {
            parse_content_disposition_filename(cd).unwrap_or_else(|| {
                // Fallback to URL-based filename
                url.split('/').last().unwrap_or("download").to_string()
            })
        } else {
            url.split('/').last().unwrap_or("download").to_string()
        };

        // Sanitize the filename for the current platform
        let safe_filename = sanitize_filename(&filename);
        let filepath = cache_dir.join(&safe_filename);

        // If file already exists in cache, skip download
        if filepath.exists() {
            print_info(&format!("Found {} in cache", safe_filename));
            return Ok(filepath);
        }

        print_info(&format!("Downloading {}", safe_filename));
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  [{bar:30}] {percent}% ({bytes}/{total_bytes})")?
                .progress_chars("█▉▊"),
        );

        let mut file = File::create(&filepath).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_and_clear();
        file.sync_all().await?;

        Ok(filepath)
    }
}
