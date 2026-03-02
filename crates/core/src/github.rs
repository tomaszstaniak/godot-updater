use crate::versions::{Edition, GodotVersion};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Rate limited — try again later")]
    RateLimited,
    #[error("No releases found")]
    NoReleases,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    #[allow(dead_code)]
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}

/// Cached release info for a single version.
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: GodotVersion,
    pub assets: Vec<Asset>,
    pub download_size: Option<u64>,
}

const STABLE_REPO: &str = "https://api.github.com/repos/godotengine/godot/releases";
const DEV_REPO: &str = "https://api.github.com/repos/godotengine/godot-builds/releases";
const USER_AGENT: &str = "godot-updater/0.1";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("failed to build HTTP client")
}

/// Fetch releases from a GitHub repo URL.
async fn fetch_releases(url: &str) -> Result<Vec<Release>, GithubError> {
    let resp = client()
        .get(url)
        .query(&[("per_page", "50")])
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(GithubError::RateLimited);
    }

    let releases: Vec<Release> = resp.error_for_status()?.json().await?;
    Ok(releases)
}

/// Convert GitHub releases to GodotVersion structs.
fn releases_to_versions(releases: Vec<Release>, editions: &[Edition]) -> Vec<ReleaseInfo> {
    let mut result = Vec::new();
    for release in releases {
        for &edition in editions {
            if let Some(version) = GodotVersion::parse(&release.tag_name, edition) {
                let asset_name = crate::platform::asset_name(&version);
                let download_size = release
                    .assets
                    .iter()
                    .find(|a| a.name == asset_name)
                    .map(|a| a.size);

                result.push(ReleaseInfo {
                    version,
                    assets: release.assets.clone(),
                    download_size,
                });
            }
        }
    }
    result
}

/// Fetch all available Godot versions from both repos.
pub async fn fetch_all_versions(
    editions: &[Edition],
    include_stable: bool,
    include_dev: bool,
    include_lts: bool,
) -> Result<Vec<ReleaseInfo>, GithubError> {
    let mut all = Vec::new();

    if include_stable || include_lts {
        let releases = fetch_releases(STABLE_REPO).await?;
        let versions = releases_to_versions(releases, editions);
        for info in versions {
            let dominated_by_channel = match info.version.channel {
                crate::versions::Channel::Stable => include_stable,
                crate::versions::Channel::LTS => include_lts,
                _ => false,
            };
            if dominated_by_channel {
                all.push(info);
            }
        }
    }

    if include_dev {
        let releases = fetch_releases(DEV_REPO).await?;
        let versions = releases_to_versions(releases, editions);
        for info in versions {
            match info.version.channel {
                crate::versions::Channel::Dev
                | crate::versions::Channel::Beta
                | crate::versions::Channel::RC => {
                    all.push(info);
                }
                // Stable releases sometimes also appear in godot-builds
                _ => {}
            }
        }
    }

    Ok(all)
}

/// Find the download URL for a specific version's asset.
pub fn find_download_url(release: &ReleaseInfo, version: &GodotVersion) -> Option<String> {
    let expected = crate::platform::asset_name(version);
    release
        .assets
        .iter()
        .find(|a| a.name == expected)
        .map(|a| a.browser_download_url.clone())
}
