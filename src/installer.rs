use crate::config::Config;
use crate::package::{Package, PlatformDetails};
use anyhow::{Result, anyhow};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tar::Archive;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use xz2::read::XzDecoder;
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    } else if filename.ends_with(".zip") {
        let mut archive = ZipArchive::new(file)?;
        archive.extract(extract_to)?;
    } else {
        return Err(anyhow!("Unsupported archive format: {}", filename));
    }

    Ok(())
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

        // Download the file, which now returns the actual path of the cached file
        let cache_file_path = self.download_file(&platform_details.url, cache_dir).await?;

        let package_type = platform_details
            .package_type
            .as_deref()
            .unwrap_or("archive");

        match package_type {
            "archive" => {
                println!("📦 Extracting archive...");
                let extract_path = package_dir.clone();
                tokio::task::spawn_blocking(move || {
                    extract_archive_sync(&cache_file_path, &extract_path)
                })
                .await??;
            }
            "binary" => {
                println!("🚀 Installing binary...");
                let executables = platform_details.get_executables();
                let executable = executables.get(0).ok_or_else(|| {
                    anyhow!("Binary package '{}' has no executables listed", name)
                })?;

                // For a binary, the "path" is just the filename. We place it in the package dir.
                let dest_path = package_dir.join(&executable.path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                fs::copy(&cache_file_path, &dest_path).await?;

                #[cfg(unix)]
                {
                    let mut perms = fs::metadata(&dest_path).await?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&dest_path, perms).await?;
                }
            }
            _ => {
                return Err(anyhow!("Unsupported package type: {}", package_type));
            }
        }

        println!("✅ Installation complete for '{}'", name);
        Ok(())
    }

    /// Downloads a file and saves it to the cache.
    /// Returns the final path of the downloaded file.
    pub async fn download_file(&self, url: &str, cache_dir: &Path) -> Result<PathBuf> {
        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        // --- NEW: Get filename from server response ---
        let content_disposition = response
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok());

        let filename = if let Some(cd) = content_disposition {
            cd.split("filename=")
                .last()
                .map(|f| f.trim_matches('"'))
                .unwrap_or("download")
        } else {
            url.split('/').last().unwrap_or("download")
        };

        let filepath = cache_dir.join(filename);

        // If file already exists in cache, skip download
        if filepath.exists() {
            println!("Found {} in cache", filename);
            return Ok(filepath);
        }

        println!("Downloading {}", filename);
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  📥 [{bar:30}] {percent}% ({bytes}/{total_bytes})")?
                .progress_chars("█░ "),
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

