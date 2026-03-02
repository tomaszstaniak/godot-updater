mod app;
mod events;
pub mod theme;
mod ui;

use app::{App, DownloadStatus, View};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use godot_updater_core::config::Config;
use godot_updater_core::download::download_file;
use godot_updater_core::github::{self, find_download_url, ReleaseInfo};
use godot_updater_core::install::{self, InstallManifest};
use godot_updater_core::platform;
use godot_updater_core::versions::Edition;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: failed to load config ({}), using defaults", e);
        Config::default()
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, config).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

enum AsyncMsg {
    FetchComplete(Result<Vec<ReleaseInfo>, String>),
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(config);
    let (tx, mut rx) = mpsc::unbounded_channel::<AsyncMsg>();

    // Initial fetch
    app.loading = true;
    spawn_fetch(&app, tx.clone());

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Check for async messages (non-blocking)
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AsyncMsg::FetchComplete(Ok(releases)) => {
                    app.releases = releases;
                    app.update_filtered();
                    app.loading = false;
                }
                AsyncMsg::FetchComplete(Err(err)) => {
                    app.error_message = Some(err);
                    app.loading = false;
                }
            }
        }

        // Poll input events
        if let Some(event) = events::poll_event(Duration::from_millis(50)) {
            match event {
                events::AppEvent::Key(key) => {
                    if events::is_quit(&key) && app.view == View::Versions {
                        app.running = false;
                    } else {
                        let needs_refresh = app.loading;
                        app.handle_key(key);

                        // If user pressed R and triggered loading, spawn a new fetch
                        if app.loading && !needs_refresh {
                            spawn_fetch(&app, tx.clone());
                        }

                        // Trigger download if entering download view
                        if app.view == View::Download
                            && *app.download_status.lock().unwrap() == DownloadStatus::Downloading
                        {
                            if app.downloading_tag.is_some() {
                                spawn_download(&app);
                            }
                        }
                    }
                }
                events::AppEvent::Tick => {}
            }
        }

        if !app.running {
            break;
        }

        // Refresh manifest when download completes
        {
            let status = app.download_status.lock().unwrap().clone();
            if status == DownloadStatus::Complete && app.view == View::Download {
                app.manifest = InstallManifest::load(app.config.install_dir());
            }
        }
    }

    Ok(())
}

fn spawn_fetch(app: &App, tx: mpsc::UnboundedSender<AsyncMsg>) {
    let editions = match app.config.general.edition.as_str() {
        "standard" => vec![Edition::Standard],
        "mono" => vec![Edition::Mono],
        _ => vec![Edition::Standard, Edition::Mono],
    };
    let include_stable = app.config.channels.stable;
    let include_dev = app.config.channels.dev;
    let include_lts = app.config.channels.lts;

    tokio::spawn(async move {
        let result =
            github::fetch_all_versions(&editions, include_stable, include_dev, include_lts).await;
        let msg = match result {
            Ok(releases) => AsyncMsg::FetchComplete(Ok(releases)),
            Err(e) => AsyncMsg::FetchComplete(Err(e.to_string())),
        };
        let _ = tx.send(msg);
    });
}

fn spawn_download(app: &App) {
    let Some(tag) = app.downloading_tag.clone() else {
        return;
    };

    let edition = app.edition_filter.to_edition();

    let release = app
        .releases
        .iter()
        .find(|r| r.version.tag == tag && r.version.edition == edition);

    let Some(release) = release.cloned() else {
        *app.download_status.lock().unwrap() = DownloadStatus::Failed("Release not found".into());
        return;
    };

    let Some(url) = find_download_url(&release, &release.version) else {
        *app.download_status.lock().unwrap() =
            DownloadStatus::Failed("Download URL not found".into());
        return;
    };

    let asset_name = platform::asset_name(&release.version);
    let install_dir = app.config.install_dir().to_path_buf();
    let progress = Arc::clone(&app.download_progress);
    let status = Arc::clone(&app.download_status);
    let version = release.version.clone();

    tokio::spawn(async move {
        let temp_dir = install_dir.join(".downloads");

        let download_result = download_file(&url, &temp_dir, &asset_name, |p| {
            *progress.lock().unwrap() = Some(p);
        })
        .await;

        let zip_path = match download_result {
            Ok(path) => path,
            Err(e) => {
                *status.lock().unwrap() = DownloadStatus::Failed(e.to_string());
                return;
            }
        };

        *status.lock().unwrap() = DownloadStatus::Extracting;

        match install::extract_zip(&zip_path, &install_dir) {
            Ok(extracted_path) => {
                let mut manifest = InstallManifest::load(&install_dir);
                manifest.add(version.tag.clone(), version.edition, extracted_path);
                let _ = manifest.save(&install_dir);
                let _ = std::fs::remove_file(&zip_path);
                let _ = std::fs::remove_dir(&temp_dir);
                *status.lock().unwrap() = DownloadStatus::Complete;
            }
            Err(e) => {
                *status.lock().unwrap() = DownloadStatus::Failed(e.to_string());
            }
        }
    });
}
