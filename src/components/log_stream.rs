use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::components::Component;
use crate::game::events::EventSeverity;
use crate::game::state::GameState;
use crate::theme;

pub struct LogStream;

impl LogStream {
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
            .title(" LOG ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::FG_DIM));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if state.event_log.is_empty() {
            let msg = Paragraph::new(" [--:--:--] Awaiting events...")
                .style(theme::text_dim());
            frame.render_widget(msg, inner);
            return Ok(());
        }

        // Show as many recent events as fit in the single log line
        let mut spans: Vec<Span> = Vec::new();

        // Show the most recent events that fit
        let max_events = 3; // Typically fits in 1-line area
        let recent: Vec<_> = state
            .event_log
            .iter()
            .rev()
            .take(max_events)
            .collect();

        for (i, event) in recent.iter().rev().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" │ ", theme::text_dim()));
            }

            // Timestamp from tick (HH:MM:SS approximation)
            let secs = event.tick / 4;
            let h = (secs / 3600) % 24;
            let m = (secs / 60) % 60;
            let s = secs % 60;

            let severity_style = match event.kind.severity_color() {
                EventSeverity::Good => Style::default().fg(theme::FG_PRIMARY),
                EventSeverity::Warning => Style::default().fg(theme::ACCENT_YELLOW),
                EventSeverity::Error => Style::default().fg(theme::ACCENT_RED),
            };

            spans.push(Span::styled(
                format!(" [{:02}:{:02}:{:02}] ", h, m, s),
                theme::text_dim(),
            ));
            spans.push(Span::styled(event.kind.description(), severity_style));
        }

        let line = Line::from(spans);
        let content = Paragraph::new(vec![line]);
        frame.render_widget(content, inner);

        Ok(())
    }
}

impl Component for LogStream {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect, _focused: bool) -> Result<()> {
        let block = Block::default()
            .title(" LOG ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::FG_DIM));

        let content = Paragraph::new(" [--:--:--] Awaiting events...")
            .style(theme::text_dim())
            .block(block);

        frame.render_widget(content, area);
        Ok(())
    }
}
