use ratatui::style::{Color, Modifier, Style};

pub const BG: Color = Color::Black;
pub const FG_PRIMARY: Color = Color::Rgb(0, 255, 65); // Matrix green
pub const FG_DIM: Color = Color::Rgb(0, 100, 30);
pub const ACCENT_CYAN: Color = Color::Rgb(0, 255, 255);
pub const ACCENT_YELLOW: Color = Color::Rgb(255, 255, 0);
pub const ACCENT_RED: Color = Color::Rgb(255, 50, 50);
pub const ACCENT_MAGENTA: Color = Color::Rgb(200, 50, 255);

pub fn border_focused() -> Style {
    Style::default()
        .fg(ACCENT_CYAN)
        .add_modifier(Modifier::BOLD)
}

pub fn border_unfocused() -> Style {
    Style::default().fg(FG_DIM)
}

pub fn title() -> Style {
    Style::default()
        .fg(FG_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

pub fn text_primary() -> Style {
    Style::default().fg(FG_PRIMARY)
}

pub fn text_dim() -> Style {
    Style::default().fg(FG_DIM)
}

pub fn text_value() -> Style {
    Style::default().fg(Color::Rgb(0, 255, 200))
}
