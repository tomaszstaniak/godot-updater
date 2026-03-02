use crate::app::{App, DownloadStatus};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

fn themed_block<'a>(app: &App, title: Option<&'a str>) -> Block<'a> {
    let t = &app.theme;
    let mut b = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border).bg(t.bg))
        .style(Style::default().bg(t.bg));
    if let Some(title) = title {
        b = b
            .title(title)
            .title_style(Style::default().fg(t.title).bg(t.bg).add_modifier(Modifier::BOLD));
    }
    b
}

pub fn draw(frame: &mut Frame, app: &App) {
    let t = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Progress
            Constraint::Min(3),    // Status
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    let tag = app.downloading_tag.as_deref().unwrap_or("unknown");

    let title = Paragraph::new(format!(" Downloading: {}", tag))
        .style(
            Style::default()
                .fg(t.accent)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        )
        .block(themed_block(app, Some(" Download ")));
    frame.render_widget(title, chunks[0]);

    // Progress bar
    let progress = app.download_progress.lock().unwrap();
    let (percent, label) = if let Some(ref p) = *progress {
        let pct = p.percent().unwrap_or(0.0);
        let downloaded_mb = p.bytes_downloaded as f64 / 1_000_000.0;
        let total_mb = p
            .total_bytes
            .map(|t| format!("{:.1}", t as f64 / 1_000_000.0))
            .unwrap_or_else(|| "?".to_string());
        (
            pct.min(100.0),
            format!("{:.1} / {} MB ({:.0}%)", downloaded_mb, total_mb, pct),
        )
    } else {
        (0.0, "Starting...".to_string())
    };
    drop(progress);

    let gauge = Gauge::default()
        .block(themed_block(app, None))
        .gauge_style(Style::default().fg(t.progress_fg).bg(t.progress_bg))
        .percent(percent as u16)
        .label(label);
    frame.render_widget(gauge, chunks[1]);

    // Status
    let status = app.download_status.lock().unwrap().clone();
    let (status_text, status_color) = match &status {
        DownloadStatus::Idle => ("Idle".to_string(), t.text_dim),
        DownloadStatus::Downloading => ("Downloading...".to_string(), t.warning),
        DownloadStatus::Extracting => ("Extracting...".to_string(), t.accent),
        DownloadStatus::Complete => ("Installation complete!".to_string(), t.success),
        DownloadStatus::Failed(err) => (format!("Failed: {}", err), t.error),
    };

    let status_paragraph = Paragraph::new(format!(" {}", status_text))
        .style(Style::default().fg(status_color).bg(t.bg))
        .block(themed_block(app, Some(" Status ")));
    frame.render_widget(status_paragraph, chunks[2]);

    // Footer
    let footer_text = match &status {
        DownloadStatus::Complete | DownloadStatus::Failed(_) => " Press any key to return",
        _ => " Press Esc to cancel",
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(t.text_dim).bg(t.bg))
        .block(themed_block(app, None));
    frame.render_widget(footer, chunks[3]);
}
