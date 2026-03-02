use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Default,
    MagicWB,
}

impl ThemeName {
    pub fn label(&self) -> &str {
        match self {
            ThemeName::Default => "Default",
            ThemeName::MagicWB => "MagicWB",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ThemeName::Default => ThemeName::MagicWB,
            ThemeName::MagicWB => ThemeName::Default,
        }
    }
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::Default
    }
}

/// All semantic colors used throughout the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    // Chrome
    pub border: Color,
    pub title: Color,
    pub bg: Color,

    // Text
    pub text: Color,
    pub text_dim: Color,
    pub text_bold: Color,

    // Highlights / selection
    pub accent: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,

    // Filter pills
    pub filter_active_fg: Color,
    pub filter_active_bg: Color,
    pub filter_inactive: Color,
    pub edition_fg: Color,
    pub edition_bg: Color,

    // Channels
    pub ch_stable: Color,
    pub ch_dev: Color,
    pub ch_beta: Color,
    pub ch_rc: Color,
    pub ch_lts: Color,

    // Status
    pub installed: Color,
    pub available: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,

    // Progress bar
    pub progress_fg: Color,
    pub progress_bg: Color,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Default => Self::default_theme(),
            ThemeName::MagicWB => Self::magicwb(),
        }
    }

    fn default_theme() -> Self {
        Theme {
            border: Color::Reset,
            title: Color::Cyan,
            bg: Color::Reset,

            text: Color::Reset,
            text_dim: Color::DarkGray,
            text_bold: Color::White,

            accent: Color::Cyan,
            selection_bg: Color::DarkGray,
            selection_fg: Color::White,

            filter_active_fg: Color::Black,
            filter_active_bg: Color::Cyan,
            filter_inactive: Color::Gray,
            edition_fg: Color::Black,
            edition_bg: Color::Yellow,

            ch_stable: Color::Green,
            ch_dev: Color::Yellow,
            ch_beta: Color::Magenta,
            ch_rc: Color::Blue,
            ch_lts: Color::Cyan,

            installed: Color::Green,
            available: Color::DarkGray,
            error: Color::Red,
            warning: Color::Yellow,
            success: Color::Green,

            progress_fg: Color::Cyan,
            progress_bg: Color::DarkGray,
        }
    }

    /// MagicWB — the classic Amiga MagicWorkBench 8-color palette.
    ///
    /// Authentic palette from Wikipedia:
    ///   0: (149,149,149) medium gray  — workbench background
    ///   1: (  0,  0,  0) black        — text, dark elements
    ///   2: (255,255,255) white        — bright highlights
    ///   3: ( 59,103,162) steel blue   — window titles, primary accent
    ///   4: (123,123,123) dark gray    — shadows, recessed areas
    ///   5: (175,175,175) light gray   — raised surfaces, lighter UI
    ///   6: (170,144,124) warm brown   — secondary accent, earthy tone
    ///   7: (255,169,151) salmon       — attention, warm highlight
    fn magicwb() -> Self {
        let med_gray    = Color::Rgb(149, 149, 149); // 0
        let black       = Color::Rgb(  0,   0,   0); // 1
        let white       = Color::Rgb(255, 255, 255); // 2
        let steel_blue  = Color::Rgb( 59, 103, 162); // 3
        let dark_gray   = Color::Rgb(123, 123, 123); // 4
        let light_gray  = Color::Rgb(175, 175, 175); // 5
        let warm_brown  = Color::Rgb(170, 144, 124); // 6
        let salmon      = Color::Rgb(255, 169, 151); // 7

        Theme {
            border: dark_gray,
            title: steel_blue,
            bg: med_gray,

            text: black,
            text_dim: dark_gray,
            text_bold: white,

            accent: steel_blue,
            selection_bg: steel_blue,
            selection_fg: white,

            filter_active_fg: white,
            filter_active_bg: steel_blue,
            filter_inactive: dark_gray,
            edition_fg: black,
            edition_bg: warm_brown,

            ch_stable: steel_blue,
            ch_dev: salmon,
            ch_beta: warm_brown,
            ch_rc: warm_brown,
            ch_lts: dark_gray,

            installed: steel_blue,
            available: dark_gray,
            error: salmon,
            warning: salmon,
            success: steel_blue,

            progress_fg: steel_blue,
            progress_bg: light_gray,
        }
    }
}
