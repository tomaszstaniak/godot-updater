pub mod download;
pub mod settings;
pub mod versions;

use crate::app::App;
use ratatui::style::Style;
use ratatui::widgets::{Block, Widget};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    // Fill entire frame with theme background color
    let bg_block = Block::default().style(Style::default().bg(app.theme.bg));
    bg_block.render(frame.area(), frame.buffer_mut());

    match app.view {
        crate::app::View::Versions => versions::draw(frame, app),
        crate::app::View::Download => download::draw(frame, app),
        crate::app::View::Settings => settings::draw(frame, app),
    }
}
