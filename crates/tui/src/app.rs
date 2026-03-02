use crossterm::event::{KeyCode, KeyEvent};
use godot_updater_core::config::Config;
use godot_updater_core::download::DownloadProgress;
use godot_updater_core::github::ReleaseInfo;
use godot_updater_core::install::InstallManifest;
use godot_updater_core::versions::{Channel, Edition};
use std::sync::{Arc, Mutex};

use crate::theme::{Theme, ThemeName};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Versions,
    Download,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelFilter {
    All,
    Stable,
    Preview,
    LTS,
}

impl ChannelFilter {
    pub fn label(&self) -> &str {
        match self {
            ChannelFilter::All => "All",
            ChannelFilter::Stable => "Stable",
            ChannelFilter::Preview => "Preview",
            ChannelFilter::LTS => "LTS",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ChannelFilter::All => ChannelFilter::Stable,
            ChannelFilter::Stable => ChannelFilter::Preview,
            ChannelFilter::Preview => ChannelFilter::LTS,
            ChannelFilter::LTS => ChannelFilter::All,
        }
    }

    pub fn matches(&self, channel: Channel) -> bool {
        match self {
            ChannelFilter::All => true,
            ChannelFilter::Stable => matches!(channel, Channel::Stable),
            ChannelFilter::Preview => {
                matches!(channel, Channel::Dev | Channel::Beta | Channel::RC)
            }
            ChannelFilter::LTS => matches!(channel, Channel::LTS),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditionFilter {
    Standard,
    Mono,
}

impl EditionFilter {
    pub fn label(&self) -> &str {
        match self {
            EditionFilter::Standard => "Standard",
            EditionFilter::Mono => "Mono",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            EditionFilter::Standard => EditionFilter::Mono,
            EditionFilter::Mono => EditionFilter::Standard,
        }
    }

    pub fn to_edition(&self) -> Edition {
        match self {
            EditionFilter::Standard => Edition::Standard,
            EditionFilter::Mono => Edition::Mono,
        }
    }
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub config: Config,
    pub manifest: InstallManifest,
    pub theme: Theme,
    pub theme_name: ThemeName,

    // Version list state
    pub releases: Vec<ReleaseInfo>,
    pub filtered_indices: Vec<usize>,
    pub selected: usize,
    pub channel_filter: ChannelFilter,
    pub edition_filter: EditionFilter,

    // Loading state
    pub loading: bool,
    pub error_message: Option<String>,

    // Download state
    pub download_progress: Arc<Mutex<Option<DownloadProgress>>>,
    pub download_status: Arc<Mutex<DownloadStatus>>,
    pub downloading_tag: Option<String>,

    // Launch state
    pub launch_message: Option<(String, bool)>, // (message, is_error)

    // Settings state
    pub settings_cursor: usize,
    pub settings_editing: bool,
    pub settings_edit_buf: String,
}

/// Number of settings fields.
pub const SETTINGS_COUNT: usize = 6;

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Idle,
    Downloading,
    Extracting,
    Complete,
    Failed(String),
}

/// Describes how a settings field should be edited.
#[derive(Debug, Clone)]
pub enum SettingKind {
    FreeText,
    Toggle(Vec<String>), // cycle through these values
    Bool,
}

#[derive(Debug, Clone)]
pub struct SettingField {
    pub label: &'static str,
    pub value: String,
    pub kind: SettingKind,
}

impl App {
    pub fn new(config: Config) -> Self {
        let manifest = InstallManifest::load(config.install_dir());
        let edition_filter = match config.general.edition.as_str() {
            "mono" => EditionFilter::Mono,
            _ => EditionFilter::Standard,
        };
        let theme_name = match config.general.theme.as_str() {
            "MagicWB" => ThemeName::MagicWB,
            _ => ThemeName::Default,
        };
        let theme = Theme::from_name(theme_name);

        let mut app = App {
            running: true,
            view: View::Versions,
            config,
            manifest,
            theme,
            theme_name,
            releases: Vec::new(),
            filtered_indices: Vec::new(),
            selected: 0,
            channel_filter: ChannelFilter::All,
            edition_filter,
            loading: false,
            error_message: None,
            download_progress: Arc::new(Mutex::new(None)),
            download_status: Arc::new(Mutex::new(DownloadStatus::Idle)),
            downloading_tag: None,
            launch_message: None,
            settings_cursor: 0,
            settings_editing: false,
            settings_edit_buf: String::new(),
        };
        app.rescan_install_dir();
        app
    }

    pub fn update_filtered(&mut self) {
        self.filtered_indices = self
            .releases
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                self.channel_filter.matches(r.version.channel)
                    && r.version.edition == self.edition_filter.to_edition()
            })
            .map(|(i, _)| i)
            .collect();

        if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len().saturating_sub(1);
        }
    }

    /// Scan the install directory for existing Godot binaries and rebuild the manifest.
    pub fn rescan_install_dir(&mut self) {
        let install_dir = self.config.install_dir().to_path_buf();
        let mut manifest = InstallManifest::load(&install_dir);

        let found = godot_updater_core::install::scan_existing_installs(&install_dir);
        for (tag, edition, path) in &found {
            manifest.add(tag.clone(), *edition, path.clone());
        }

        // Remove manifest entries whose files no longer exist on disk
        manifest.installations.retain(|i| i.path.exists());

        let _ = manifest.save(&install_dir);
        self.manifest = manifest;
    }

    pub fn selected_release(&self) -> Option<&ReleaseInfo> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&i| self.releases.get(i))
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.view {
            View::Versions => self.handle_versions_key(key),
            View::Download => self.handle_download_key(key),
            View::Settings => self.handle_settings_key(key),
        }
    }

    fn handle_versions_key(&mut self, key: KeyEvent) {
        // Clear launch message on any key except L
        if key.code != KeyCode::Char('l') {
            self.launch_message = None;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.filtered_indices.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::End => {
                self.selected = self.filtered_indices.len().saturating_sub(1);
            }
            KeyCode::Tab => {
                self.channel_filter = self.channel_filter.next();
                self.update_filtered();
            }
            KeyCode::Char('e') => {
                self.edition_filter = self.edition_filter.toggle();
                self.update_filtered();
            }
            KeyCode::F(2) => {
                self.view = View::Settings;
                self.settings_cursor = 0;
                self.settings_editing = false;
            }
            KeyCode::Char('d') => {
                if let Some(release) = self.selected_release() {
                    let tag = release.version.tag.clone();
                    let edition = release.version.edition;
                    let install_dir = self.config.install_dir().to_path_buf();
                    if self.manifest.is_installed(&tag, edition) {
                        let _ =
                            godot_updater_core::install::uninstall(&install_dir, &tag, edition);
                        self.manifest = InstallManifest::load(&install_dir);
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(release) = self.selected_release() {
                    if !self
                        .manifest
                        .is_installed(&release.version.tag, release.version.edition)
                    {
                        self.downloading_tag = Some(release.version.tag.clone());
                        self.view = View::Download;
                        *self.download_status.lock().unwrap() = DownloadStatus::Downloading;
                        *self.download_progress.lock().unwrap() = None;
                    }
                }
            }
            KeyCode::Char('l') => {
                self.launch_message = None;
                if let Some(release) = self.selected_release() {
                    let tag = release.version.tag.clone();
                    let edition = release.version.edition;
                    let install_dir = self.config.install_dir().to_path_buf();
                    if self.manifest.is_installed(&tag, edition) {
                        match godot_updater_core::install::launch(&install_dir, &tag, edition) {
                            Ok(path) => {
                                self.launch_message = Some((
                                    format!("Launched {}", path.display()),
                                    false,
                                ));
                            }
                            Err(e) => {
                                self.launch_message =
                                    Some((format!("Launch failed: {}", e), true));
                            }
                        }
                    } else {
                        self.launch_message = Some((
                            "Version not installed — press Enter to install first".into(),
                            true,
                        ));
                    }
                }
            }
            KeyCode::Char('r') => {
                self.loading = true;
                self.error_message = None;
            }
            _ => {}
        }
    }

    fn handle_download_key(&mut self, key: KeyEvent) {
        let status = self.download_status.lock().unwrap().clone();
        match status {
            DownloadStatus::Complete | DownloadStatus::Failed(_) => {
                self.view = View::Versions;
                *self.download_status.lock().unwrap() = DownloadStatus::Idle;
                self.manifest = InstallManifest::load(self.config.install_dir());
            }
            _ => {
                if key.code == KeyCode::Esc {
                    self.view = View::Versions;
                    *self.download_status.lock().unwrap() = DownloadStatus::Idle;
                }
            }
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        if self.settings_editing {
            match key.code {
                KeyCode::Enter => {
                    self.apply_settings_edit();
                    self.settings_editing = false;
                }
                KeyCode::Esc => {
                    self.settings_editing = false;
                }
                KeyCode::Char(c) => {
                    self.settings_edit_buf.push(c);
                }
                KeyCode::Backspace => {
                    self.settings_edit_buf.pop();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::F(2) => {
                let _ = self.config.save();
                self.rescan_install_dir();
                self.view = View::Versions;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_cursor = self.settings_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings_cursor + 1 < SETTINGS_COUNT {
                    self.settings_cursor += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_setting();
            }
            _ => {}
        }
    }

    /// Activate the current setting — toggle for bools/cycles, edit for free text.
    fn activate_setting(&mut self) {
        let fields = self.settings_fields();
        let field = &fields[self.settings_cursor];
        match &field.kind {
            SettingKind::Bool => self.toggle_bool_setting(),
            SettingKind::Toggle(options) => self.cycle_setting(options),
            SettingKind::FreeText => {
                self.settings_edit_buf = field.value.clone();
                self.settings_editing = true;
            }
        }
    }

    fn toggle_bool_setting(&mut self) {
        match self.settings_cursor {
            2 => self.config.channels.stable = !self.config.channels.stable,
            3 => self.config.channels.dev = !self.config.channels.dev,
            4 => self.config.channels.lts = !self.config.channels.lts,
            _ => {}
        }
    }

    fn cycle_setting(&mut self, options: &[String]) {
        match self.settings_cursor {
            1 => {
                // Edition: standard → mono → both → ...
                let current = &self.config.general.edition;
                let idx = options.iter().position(|o| o == current).unwrap_or(0);
                let next = (idx + 1) % options.len();
                self.config.general.edition = options[next].clone();
            }
            5 => {
                // Theme: cycle through
                self.theme_name = self.theme_name.next();
                self.theme = Theme::from_name(self.theme_name);
                self.config.general.theme = self.theme_name.label().to_string();
            }
            _ => {}
        }
    }

    fn apply_settings_edit(&mut self) {
        match self.settings_cursor {
            0 => {
                self.config.general.install_dir = self.settings_edit_buf.clone().into();
                self.rescan_install_dir();
            }
            _ => {}
        }
    }

    pub fn settings_fields(&self) -> Vec<SettingField> {
        vec![
            SettingField {
                label: "Install Directory",
                value: self.config.general.install_dir.to_string_lossy().to_string(),
                kind: SettingKind::FreeText,
            },
            SettingField {
                label: "Edition",
                value: self.config.general.edition.clone(),
                kind: SettingKind::Toggle(vec![
                    "standard".into(),
                    "mono".into(),
                    "both".into(),
                ]),
            },
            SettingField {
                label: "Stable Channel",
                value: format!("{}", self.config.channels.stable),
                kind: SettingKind::Bool,
            },
            SettingField {
                label: "Preview Channel",
                value: format!("{}", self.config.channels.dev),
                kind: SettingKind::Bool,
            },
            SettingField {
                label: "LTS Channel",
                value: format!("{}", self.config.channels.lts),
                kind: SettingKind::Bool,
            },
            SettingField {
                label: "Theme",
                value: self.theme_name.label().to_string(),
                kind: SettingKind::Toggle(vec![
                    "Default".into(),
                    "MagicWB".into(),
                ]),
            },
        ]
    }
}
