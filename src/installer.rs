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

        // Create package directory
        tokio::fs::create_dir_all(&package_dir).await?;
        tokio::fs::create_dir_all(cache_dir).await?;

        // Determine filename from URL
        let url_path = package.url.split('/').last().unwrap_or("archive");
        let cache_file = cache_dir.join(format!("{}_{}", name, url_path));

        // Download if not cached
        if !cache_file.exists() {
            self.download_file(&package.url, &cache_file).await?;
        }

        // Extract archive
        self.extract_archive(&cache_file, &package_dir).await?;

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

    pub async fn extract_archive(&self, archive_path: &Path, extract_to: &Path) -> Result<()> {
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
            let mut archive = zip::ZipArchive::new(file)?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = extract_to.join(file.name());

                if file.name().ends_with('/') {
                    tokio::fs::create_dir_all(&outpath).await?;
                } else {
                    if let Some(p) = outpath.parent() {
                        tokio::fs::create_dir_all(p).await?;
                    }
                    let mut outfile = tokio::fs::File::create(&outpath).await?;
                    let mut buffer = Vec::new();
                    std::io::copy(&mut file, &mut buffer)?;
                    outfile.write_all(&buffer).await?;
                }
            }
        } else {
            return Err(anyhow::anyhow!("Unsupported archive format: {}", filename));
        }

        Ok(())
    }
}
