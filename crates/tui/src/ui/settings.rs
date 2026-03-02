use crate::app::{App, SettingKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
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
            Constraint::Length(3), // Title
            Constraint::Min(5),   // Settings list
            Constraint::Length(3), // Footer
        ])
        .split(frame.area());

    let title = Paragraph::new(" Settings")
        .style(
            Style::default()
                .fg(t.accent)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        )
        .block(themed_block(app, Some(" Settings ")));
    frame.render_widget(title, chunks[0]);

    // Settings fields
    let fields = app.settings_fields();
    let mut lines: Vec<Line> = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let selected = i == app.settings_cursor;
        let cursor = if selected { "▸ " } else { "  " };

        let label_style = if selected {
            Style::default()
                .fg(t.accent)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text).bg(t.bg)
        };

        let value_display = if app.settings_editing && selected {
            format!("{}▎", app.settings_edit_buf)
        } else {
            match &field.kind {
                SettingKind::Bool => {
                    if field.value == "true" {
                        "[x]".to_string()
                    } else {
                        "[ ]".to_string()
                    }
                }
                SettingKind::Toggle(_) => format!("< {} >", field.value),
                SettingKind::FreeText => field.value.clone(),
            }
        };

        let value_style = match &field.kind {
            SettingKind::Bool => {
                if field.value == "true" {
                    Style::default().fg(t.success).bg(t.bg)
                } else {
                    Style::default().fg(t.text_dim).bg(t.bg)
                }
            }
            SettingKind::Toggle(_) => Style::default().fg(t.accent).bg(t.bg),
            SettingKind::FreeText => Style::default().fg(t.warning).bg(t.bg),
        };

        lines.push(Line::from(vec![
            Span::styled(cursor, Style::default().fg(t.text).bg(t.bg)),
            Span::styled(format!("{:<20}", field.label), label_style),
            Span::styled(value_display, value_style),
        ]));
        lines.push(Line::styled("", Style::default().bg(t.bg)));
    }

    let settings = Paragraph::new(lines)
        .style(Style::default().bg(t.bg))
        .block(themed_block(app, None));
    frame.render_widget(settings, chunks[1]);

    // Footer
    let footer_text = if app.settings_editing {
        " Enter: Save  Esc: Cancel"
    } else {
        " Enter/Space: Toggle/Edit  Esc/F2: Back"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(t.text_dim).bg(t.bg))
        .block(themed_block(app, None));
    frame.render_widget(footer, chunks[2]);
}
