use crate::config::Config;
use crate::package::Package;
use anyhow::Result;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use tar::Archive;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use xz2::read::XzDecoder;
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// This is now a private helper function, not a method on Installer.
/// This resolves the E0521 lifetime error.
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
        return Err(anyhow::anyhow!("Unsupported archive format: {}", filename));
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
        package: &Package,
        config: &Config,
    ) -> Result<()> {
        let package_dir = config.packages_dir.join(name);
        let cache_dir = &config.cache_dir;

        tokio::fs::create_dir_all(&package_dir).await?;
        tokio::fs::create_dir_all(cache_dir).await?;

        let url_filename = package.url.split('/').last().unwrap_or("download");
        let cache_file = cache_dir.join(format!("{}_{}", name, url_filename));

        if !cache_file.exists() {
            self.download_file(&package.url, &cache_file).await?;
        }

        let package_type = package.package_type.as_deref().unwrap_or("archive");

        match package_type {
            "archive" => {
                println!("📦 Extracting archive...");
                let archive_path = cache_file.clone();
                let extract_path = package_dir.clone();
                tokio::task::spawn_blocking(move || {
                    extract_archive_sync(&archive_path, &extract_path)
                })
                .await??;
            }
            "binary" => {
                println!("🚀 Installing binary...");

                // FIX for E0716: Store the Vec in a variable to extend its lifetime.
                let executables = package.get_executables();
                let executable = executables.get(0).ok_or_else(|| {
                    anyhow::anyhow!("Binary package '{}' has no executables listed", name)
                })?;

                let dest_path = package_dir.join(&executable.path);

                if let Some(parent) = dest_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                tokio::fs::copy(&cache_file, &dest_path).await?;

                #[cfg(unix)]
                {
                    let mut perms = tokio::fs::metadata(&dest_path).await?.permissions();
                    perms.set_mode(0o755);
                    tokio::fs::set_permissions(&dest_path, perms).await?;
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported package type: {}",
                    package_type
                ));
            }
        }

        println!("✅ Installation complete for '{}'", name);
        Ok(())
    }

    pub async fn download_file(&self, url: &str, filepath: &Path) -> Result<()> {
        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  📥 [{bar:30}] {percent}% ({bytes}/{total_bytes})")?
                .progress_chars("█░ "),
        );

        let mut file = File::create(filepath).await?;
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
        Ok(())
    }
}
