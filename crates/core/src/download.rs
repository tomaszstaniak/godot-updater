use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
}

impl DownloadProgress {
    pub fn percent(&self) -> Option<f64> {
        self.total_bytes
            .map(|total| self.bytes_downloaded as f64 / total as f64 * 100.0)
    }
}

/// Download a file from `url` to `dest_dir`, calling `on_progress` with updates.
/// Returns the path to the downloaded file.
pub async fn download_file<F>(
    url: &str,
    dest_dir: &Path,
    filename: &str,
    mut on_progress: F,
) -> Result<PathBuf, DownloadError>
where
    F: FnMut(DownloadProgress),
{
    std::fs::create_dir_all(dest_dir)?;
    let dest_path = dest_dir.join(filename);

    let client = reqwest::Client::builder()
        .user_agent("godot-updater/0.1")
        .build()?;

    let resp = client.get(url).send().await?.error_for_status()?;
    let total_bytes = resp.content_length();

    let mut stream = resp.bytes_stream();
    let mut file = tokio::fs::File::create(&dest_path).await.map_err(|e| {
        DownloadError::Io(e)
    })?;

    let mut bytes_downloaded: u64 = 0;

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await.map_err(DownloadError::Io)?;
        bytes_downloaded += chunk.len() as u64;

        on_progress(DownloadProgress {
            bytes_downloaded,
            total_bytes,
        });
    }

    file.flush().await.map_err(DownloadError::Io)?;

    Ok(dest_path)
}

/// Verify SHA-256 checksum of a file.
pub fn verify_checksum(file_path: &Path, expected_hash: &str) -> Result<bool, DownloadError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = format!("{:x}", hasher.finalize());
    Ok(hash == expected_hash.to_lowercase())
}
