use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::components::Component;
use crate::game::progression;
use crate::game::state::GameState;
use crate::theme;

pub struct Header;

impl Header {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_with_state(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        _focused: bool,
        state: &GameState,
    ) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::FG_DIM));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let now = chrono::Local::now();
        let clock = now.format("%H:%M:%S").to_string();

        let rep_mult = progression::reputation_multiplier(state.resources.reputation);

        let prestige_style = if state.prestige_count > 0 {
            Style::default().fg(theme::ACCENT_MAGENTA)
        } else {
            theme::text_value()
        };

        let line = Line::from(vec![
            Span::styled(" IDLE TERMINAL", theme::title()),
            Span::styled(" | ", theme::text_dim()),
            Span::styled("Tick:", theme::text_dim()),
            Span::styled(format!("{}", state.total_ticks), theme::text_value()),
            Span::styled(" | ", theme::text_dim()),
            Span::styled("P:", theme::text_dim()),
            Span::styled(format!("{}", state.prestige_count), prestige_style),
            Span::styled(format!(" (x{:.2})", rep_mult), theme::text_dim()),
            Span::styled(" | ", theme::text_dim()),
            Span::styled(clock, theme::text_value()),
        ]);

        let content = Paragraph::new(vec![line]);
        frame.render_widget(content, inner);

        Ok(())
    }
}

impl Component for Header {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect, _focused: bool) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::FG_DIM));

        let title = Paragraph::new(" IDLE TERMINAL")
            .style(theme::title())
            .block(block);

        frame.render_widget(title, area);
        Ok(())
    }
}
