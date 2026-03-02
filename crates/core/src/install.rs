use crate::versions::Edition;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Asset not found in ZIP")]
    AssetNotFound,
}

/// Manifest tracking installed versions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallManifest {
    pub installations: Vec<InstalledVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersion {
    pub tag: String,
    pub edition: Edition,
    pub path: PathBuf,
}

impl InstallManifest {
    fn manifest_path(install_dir: &Path) -> PathBuf {
        install_dir.join("installations.json")
    }

    pub fn load(install_dir: &Path) -> Self {
        let path = Self::manifest_path(install_dir);
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            InstallManifest::default()
        }
    }

    pub fn save(&self, install_dir: &Path) -> Result<(), InstallError> {
        let path = Self::manifest_path(install_dir);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn is_installed(&self, tag: &str, edition: Edition) -> bool {
        self.installations
            .iter()
            .any(|i| i.tag == tag && i.edition == edition)
    }

    pub fn add(&mut self, tag: String, edition: Edition, path: PathBuf) {
        if !self.is_installed(&tag, edition) {
            self.installations.push(InstalledVersion {
                tag,
                edition,
                path,
            });
        }
    }

    pub fn remove(&mut self, tag: &str, edition: Edition) {
        self.installations
            .retain(|i| !(i.tag == tag && i.edition == edition));
    }
}

/// Extract a ZIP file to the install directory.
pub fn extract_zip(zip_path: &Path, install_dir: &Path) -> Result<PathBuf, InstallError> {
    std::fs::create_dir_all(install_dir)?;

    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Determine the top-level directory or file in the archive
    let mut top_level = String::new();
    if archive.len() > 0 {
        let first = archive.by_index(0)?;
        let name = first.name().to_string();
        if let Some(idx) = name.find('/') {
            top_level = name[..idx].to_string();
        } else {
            top_level = name;
        }
    }

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        let out_path = install_dir.join(&name);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(install_dir.join(top_level))
}

/// Uninstall a version by removing its files and updating the manifest.
pub fn uninstall(
    install_dir: &Path,
    tag: &str,
    edition: Edition,
) -> Result<(), InstallError> {
    let mut manifest = InstallManifest::load(install_dir);

    if let Some(installed) = manifest
        .installations
        .iter()
        .find(|i| i.tag == tag && i.edition == edition)
    {
        let path = &installed.path;
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else if path.is_file() {
            std::fs::remove_file(path)?;
        }
    }

    manifest.remove(tag, edition);
    manifest.save(install_dir)?;

    Ok(())
}

/// Launch an installed Godot version. Spawns the process detached.
/// Returns the path that was launched.
pub fn launch(install_dir: &Path, tag: &str, edition: Edition) -> Result<PathBuf, InstallError> {
    let manifest = InstallManifest::load(install_dir);
    let installed = manifest
        .installations
        .iter()
        .find(|i| i.tag == tag && i.edition == edition)
        .ok_or_else(|| {
            InstallError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Version not installed",
            ))
        })?;

    let path = &installed.path;

    // Find the executable: if path is a directory, look for the binary inside it
    let exe_path = if path.is_dir() {
        find_executable_in_dir(path)?
    } else {
        path.clone()
    };

    std::process::Command::new(&exe_path)
        .current_dir(exe_path.parent().unwrap_or(install_dir))
        .spawn()?;

    Ok(exe_path)
}

/// Find a Godot executable inside a directory.
fn find_executable_in_dir(dir: &Path) -> Result<PathBuf, InstallError> {
    for entry in std::fs::read_dir(dir)?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_godot_exe = name.starts_with("Godot_v")
            && (name.ends_with(".exe")
                || name.ends_with(".x86_64")
                || name.ends_with(".universal")
                || name == "Godot.app");
        // Also match console-less variant
        let is_console = name.starts_with("Godot_v") && name.contains("console");
        if is_godot_exe && !is_console {
            return Ok(entry.path());
        }
    }
    // Fallback: any file starting with Godot_v
    for entry in std::fs::read_dir(dir)?.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("Godot_v") && entry.path().is_file() {
            return Ok(entry.path());
        }
    }
    Err(InstallError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("No Godot executable found in {}", dir.display()),
    )))
}

/// Scan install_dir for existing Godot executables and register them in the manifest.
/// Matches filenames like:
///   Godot_v4.6.1-stable_win64.exe
///   Godot_v4.6-stable_win64_console.exe
///   Godot_v4.5.1-stable_mono_win64.exe
///   Godot_v4.7-dev3_linux.x86_64
///   Godot_v4.6-stable_macos.universal.zip (directories on macOS)
/// Console executables are skipped — only the main binary is registered.
pub fn scan_existing_installs(install_dir: &Path) -> Vec<(String, Edition, PathBuf)> {
    let mut found = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let entries = match std::fs::read_dir(install_dir) {
        Ok(e) => e,
        Err(_) => return found,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip console executables
        if name.contains("_console") {
            continue;
        }

        if !name.starts_with("Godot_v") {
            continue;
        }

        if let Some((tag, edition)) = extract_tag_from_filename(&name) {
            let key = (tag.clone(), edition);
            if seen.insert(key) {
                found.push((tag, edition, entry.path()));
            }
        }

        // Also check directories (mono installs extract to a folder)
        if entry.path().is_dir() {
            let dir_name = &name;
            if let Some((tag, edition)) = extract_tag_from_filename(dir_name) {
                let key = (tag.clone(), edition);
                if seen.insert(key) {
                    found.push((tag, edition, entry.path()));
                }
            }
        }
    }

    found
}

/// Extract tag and edition from a Godot filename.
/// "Godot_v4.6.1-stable_win64.exe" -> Some(("4.6.1-stable", Standard))
/// "Godot_v4.5.1-stable_mono_win64.exe" -> Some(("4.5.1-stable", Mono))
fn extract_tag_from_filename(name: &str) -> Option<(String, Edition)> {
    let rest = name.strip_prefix("Godot_v")?;

    // The tag is a version like "4.6.1-stable" or "4.7-dev3".
    // It ends at the first '_' that follows the channel suffix (stable/dev/beta/rc + optional number).
    // Pattern: {major}.{minor}[.{patch}]-{channel}[N]_{platform_stuff}

    // Find the channel separator '-'
    let dash_pos = rest.find('-')?;
    let after_dash = &rest[dash_pos + 1..];

    // The channel part is letters optionally followed by digits, ending at '_'
    let channel_end = after_dash
        .find('_')
        .unwrap_or(after_dash.len());

    let tag = &rest[..dash_pos + 1 + channel_end];

    // Verify tag looks valid (has a digit before the dash)
    if !tag[..dash_pos].chars().next()?.is_ascii_digit() {
        return None;
    }

    let edition = if rest[dash_pos + 1 + channel_end..].contains("mono") {
        Edition::Mono
    } else {
        Edition::Standard
    };

    Some((tag.to_string(), edition))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_exe() {
        let (tag, ed) = extract_tag_from_filename("Godot_v4.5.1-stable_win64.exe").unwrap();
        assert_eq!(tag, "4.5.1-stable");
        assert_eq!(ed, Edition::Standard);
    }

    #[test]
    fn parse_console_tag() {
        let (tag, ed) = extract_tag_from_filename("Godot_v4.5.1-stable_win64_console.exe").unwrap();
        assert_eq!(tag, "4.5.1-stable");
        assert_eq!(ed, Edition::Standard);
    }

    #[test]
    fn parse_no_patch() {
        let (tag, ed) = extract_tag_from_filename("Godot_v4.6-stable_win64.exe").unwrap();
        assert_eq!(tag, "4.6-stable");
        assert_eq!(ed, Edition::Standard);
    }

    #[test]
    fn parse_mono() {
        let (tag, ed) = extract_tag_from_filename("Godot_v4.5.1-stable_mono_win64.exe").unwrap();
        assert_eq!(tag, "4.5.1-stable");
        assert_eq!(ed, Edition::Mono);
    }

    #[test]
    fn parse_dev() {
        let (tag, ed) = extract_tag_from_filename("Godot_v4.7-dev3_win64.exe").unwrap();
        assert_eq!(tag, "4.7-dev3");
        assert_eq!(ed, Edition::Standard);
    }

    #[test]
    fn console_skipped_in_scan() {
        // Console executables should be filtered at the scan level, not parse level
        assert!(extract_tag_from_filename("Godot_v4.6-stable_win64_console.exe").is_some());
    }
}
