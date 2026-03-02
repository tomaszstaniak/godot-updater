use std::path::PathBuf;

use crate::versions::{Edition, GodotVersion};

/// Platform-appropriate config directory.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("godot-updater")
}

/// Default install directory per platform.
pub fn default_install_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        // Use a sensible default on Windows
        dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\"))
            .join("Godot")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("Applications")
            .join("Godot")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".local")
            .join("share")
            .join("godot")
    }
}

/// Export templates directory per platform.
pub fn templates_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Godot")
            .join("export_templates")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join("Library")
            .join("Application Support")
            .join("Godot")
            .join("export_templates")
    } else {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("godot")
            .join("export_templates")
    }
}

/// Resolve the expected GitHub asset filename for a given version.
pub fn asset_name(version: &GodotVersion) -> String {
    let tag = &version.tag;
    let mono = matches!(version.edition, Edition::Mono);

    if cfg!(target_os = "windows") {
        if mono {
            format!("Godot_v{}_mono_win64.zip", tag)
        } else {
            format!("Godot_v{}_win64.exe.zip", tag)
        }
    } else if cfg!(target_os = "macos") {
        if mono {
            format!("Godot_v{}_mono_macos.universal.zip", tag)
        } else {
            format!("Godot_v{}_macos.universal.zip", tag)
        }
    } else {
        if mono {
            format!("Godot_v{}_mono_linux.x86_64.zip", tag)
        } else {
            format!("Godot_v{}_linux.x86_64.zip", tag)
        }
    }
}

/// SHA-256 checksum asset name.
pub fn checksum_asset_name(_version: &GodotVersion) -> String {
    "SHA512-SUMS.txt".to_string()
}

/// Expected binary name after extraction.
pub fn binary_name(version: &GodotVersion) -> String {
    let tag = &version.tag;
    let mono = matches!(version.edition, Edition::Mono);

    if cfg!(target_os = "windows") {
        if mono {
            format!("Godot_v{}_mono_win64.exe", tag)
        } else {
            format!("Godot_v{}_win64.exe", tag)
        }
    } else if cfg!(target_os = "macos") {
        "Godot.app".to_string()
    } else {
        if mono {
            format!("Godot_v{}_mono_linux.x86_64", tag)
        } else {
            format!("Godot_v{}_linux.x86_64", tag)
        }
    }
}
