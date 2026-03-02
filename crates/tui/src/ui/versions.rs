use crate::app::{App, ChannelFilter};
use crate::theme::ThemeName;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header + filters
            Constraint::Min(5),   // Table
            Constraint::Length(3), // Footer/help
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);
}

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

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let filters = [
        ChannelFilter::All,
        ChannelFilter::Stable,
        ChannelFilter::Preview,
        ChannelFilter::LTS,
    ];

    let mut spans: Vec<Span> = vec![Span::styled(
        " Channel: ",
        Style::default().fg(t.text).bg(t.bg),
    )];
    for f in &filters {
        let style = if *f == app.channel_filter {
            Style::default()
                .fg(t.filter_active_fg)
                .bg(t.filter_active_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.filter_inactive).bg(t.bg)
        };
        spans.push(Span::styled(format!(" {} ", f.label()), style));
        spans.push(Span::styled(" ", Style::default().bg(t.bg)));
    }

    spans.push(Span::styled(
        "  Edition: ",
        Style::default().fg(t.text).bg(t.bg),
    ));
    spans.push(Span::styled(
        format!(" {} ", app.edition_filter.label()),
        Style::default()
            .fg(t.edition_fg)
            .bg(t.edition_bg)
            .add_modifier(Modifier::BOLD),
    ));

    let mut block = themed_block(app, Some(" Godot Updater "));

    // Easter egg: show slogan for MagicWB if there's enough room
    if app.theme_name == ThemeName::MagicWB && area.width >= 60 {
        block = block.title_top(
            Line::from(Span::styled(
                " Keeping the Amiga spirit alive ",
                Style::default()
                    .fg(t.text_dim)
                    .bg(t.bg)
                    .add_modifier(Modifier::ITALIC),
            ))
            .alignment(Alignment::Right),
        );
    }

    let header = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(t.bg))
        .block(block);

    frame.render_widget(header, area);
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;

    if app.loading {
        let loading = Paragraph::new(" Loading releases...")
            .style(Style::default().fg(t.warning).bg(t.bg))
            .block(themed_block(app, None));
        frame.render_widget(loading, area);
        return;
    }

    if let Some(ref err) = app.error_message {
        let error = Paragraph::new(format!(" Error: {}", err))
            .style(Style::default().fg(t.error).bg(t.bg))
            .block(themed_block(app, None));
        frame.render_widget(error, area);
        return;
    }

    if app.filtered_indices.is_empty() {
        let empty = Paragraph::new(" No versions found. Press [R] to refresh.")
            .style(Style::default().fg(t.text_dim).bg(t.bg))
            .block(themed_block(app, None));
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(" Version").style(
            Style::default()
                .fg(t.text_bold)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Channel").style(
            Style::default()
                .fg(t.text_bold)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Status").style(
            Style::default()
                .fg(t.text_bold)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Size").style(
            Style::default()
                .fg(t.text_bold)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1);

    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(display_idx, &release_idx)| {
            let release = &app.releases[release_idx];
            let v = &release.version;

            let installed = app.manifest.is_installed(&v.tag, v.edition);
            let status_symbol = if installed {
                "● Installed"
            } else {
                "○ Available"
            };
            let status_color = if installed { t.installed } else { t.available };

            let size = release
                .download_size
                .map(|s| format_size(s))
                .unwrap_or_else(|| "—".to_string());

            let channel_color = match v.channel {
                godot_updater_core::versions::Channel::Stable => t.ch_stable,
                godot_updater_core::versions::Channel::Dev => t.ch_dev,
                godot_updater_core::versions::Channel::Beta => t.ch_beta,
                godot_updater_core::versions::Channel::RC => t.ch_rc,
                godot_updater_core::versions::Channel::LTS => t.ch_lts,
            };

            let cursor = if display_idx == app.selected {
                "▸"
            } else {
                " "
            };

            Row::new(vec![
                Cell::from(format!("{} {}", cursor, v.tag))
                    .style(Style::default().fg(t.text).bg(t.bg)),
                Cell::from(v.channel.to_string())
                    .style(Style::default().fg(channel_color).bg(t.bg)),
                Cell::from(status_symbol)
                    .style(Style::default().fg(status_color).bg(t.bg)),
                Cell::from(size).style(Style::default().fg(t.text_dim).bg(t.bg)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(25),
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(themed_block(app, None))
        .row_highlight_style(
            Style::default()
                .bg(t.selection_bg)
                .fg(t.selection_fg)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;

    // Show launch message if present, otherwise show keybinding help
    if let Some((ref msg, is_error)) = app.launch_message {
        let color = if is_error { t.error } else { t.success };
        let footer = Paragraph::new(format!(" {}", msg))
            .style(Style::default().fg(color).bg(t.bg))
            .block(themed_block(app, None));
        frame.render_widget(footer, area);
        return;
    }

    let help = Line::from(vec![
        Span::styled(" Enter", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Install  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("L", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Launch  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("D", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Delete  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("R", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Refresh  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("Tab", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Channel  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("E", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Edition  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("F2", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Settings  ", Style::default().fg(t.text).bg(t.bg)),
        Span::styled("Q", Style::default().fg(t.accent).bg(t.bg)),
        Span::styled(" Quit", Style::default().fg(t.text).bg(t.bg)),
    ]);

    let footer = Paragraph::new(help)
        .style(Style::default().bg(t.bg))
        .block(themed_block(app, None));
    frame.render_widget(footer, area);
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.0} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.0} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}
