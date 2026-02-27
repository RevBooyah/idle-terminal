use std::collections::VecDeque;

use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::components::Component;
use crate::game::resources::format_si;
use crate::game::state::GameState;
use crate::theme;

pub struct Dashboard;

impl Dashboard {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_with_state(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        focused: bool,
        state: &GameState,
    ) -> Result<()> {
        let border_style = if focused {
            theme::border_focused()
        } else {
            theme::border_unfocused()
        };
        let border_type = if focused {
            BorderType::Double
        } else {
            BorderType::Rounded
        };

        let block = Block::default()
            .title(" DASHBOARD ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines = vec![
            Line::from(""),
            resource_line(
                "CPU",
                "Compute",
                state.resources.compute,
                state.production_per_tick.compute,
            ),
            Line::from(""),
            resource_line(
                "BW ",
                "Bandwidth",
                state.resources.bandwidth,
                state.production_per_tick.bandwidth,
            ),
            Line::from(""),
            resource_line(
                "SSD",
                "Storage",
                state.resources.storage,
                state.production_per_tick.storage,
            ),
            Line::from(""),
            resource_line(
                "REP",
                "Reputation",
                state.resources.reputation,
                state.production_per_tick.reputation,
            ),
            Line::from(""),
            resource_line(
                "BTC",
                "Crypto",
                state.resources.crypto,
                state.production_per_tick.crypto,
            ),
        ];

        // Sparkline for compute history
        if !state.compute_history.is_empty() {
            let width = (inner.width as usize).saturating_sub(4);
            let spark = sparkline_text(&state.compute_history, width);
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  ", theme::text_dim()),
                Span::styled(
                    spark,
                    ratatui::style::Style::default().fg(theme::FG_PRIMARY),
                ),
            ]));
        }

        // Prestige info
        if inner.height as usize > lines.len() + 2 {
            lines.push(Line::from(""));
            if state.can_prestige() {
                lines.push(Line::from(Span::styled(
                    "  * PRESTIGE AVAILABLE [p]",
                    ratatui::style::Style::default().fg(theme::ACCENT_MAGENTA),
                )));
            } else {
                let progress = (state.resources.compute / 1_000_000.0 * 100.0).min(100.0);
                lines.push(Line::from(vec![
                    Span::styled("  Prestige: ", theme::text_dim()),
                    Span::styled(format!("{:.1}% to 1M CPU", progress), theme::text_dim()),
                ]));
            }
        }

        // Achievements count
        if !state.achievements.is_empty() && (inner.height as usize) > lines.len() + 1 {
            lines.push(Line::from(vec![
                Span::styled("  Achievements: ", theme::text_dim()),
                Span::styled(format!("{}/10", state.achievements.len()), theme::text_value()),
            ]));
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, inner);
        Ok(())
    }
}

fn resource_line<'a>(symbol: &'a str, name: &'a str, amount: f64, per_tick: f64) -> Line<'a> {
    let per_sec = per_tick * 4.0;
    Line::from(vec![
        Span::styled(format!("  {symbol} "), theme::title()),
        Span::styled(format!("{:<10}", name), theme::text_dim()),
        Span::styled(format!("{:>8}", format_si(amount)), theme::text_value()),
        Span::styled("  +", theme::text_dim()),
        Span::styled(
            format!("{}/s", format_si(per_sec)),
            ratatui::style::Style::default().fg(theme::FG_PRIMARY),
        ),
    ])
}

/// Render a sparkline as text using Unicode block characters.
fn sparkline_text(data: &VecDeque<u64>, width: usize) -> String {
    if data.is_empty() {
        return String::new();
    }
    let bars = [' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
    let recent: Vec<u64> = data
        .iter()
        .rev()
        .take(width)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let max = recent.iter().copied().max().unwrap_or(1);
    let min = recent.iter().copied().min().unwrap_or(0);
    let range = (max - min).max(1) as f64;

    recent
        .iter()
        .map(|&v| {
            let idx = (((v - min) as f64 / range) * 8.0) as usize;
            bars[idx.min(8)]
        })
        .collect()
}

impl Component for Dashboard {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect, focused: bool) -> Result<()> {
        let border_style = if focused {
            theme::border_focused()
        } else {
            theme::border_unfocused()
        };
        let border_type = if focused {
            BorderType::Double
        } else {
            BorderType::Rounded
        };

        let block = Block::default()
            .title(" DASHBOARD ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let content = Paragraph::new("Loading...")
            .style(theme::text_dim())
            .block(block);

        frame.render_widget(content, area);
        Ok(())
    }
}
