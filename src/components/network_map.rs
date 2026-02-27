use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::components::Component;
use crate::game::buildings::all_building_defs;
use crate::game::network_info::LocalNetworkInfo;
use crate::game::state::GameState;
use crate::theme;

pub struct NetworkMap {
    net_info: LocalNetworkInfo,
    tick_counter: u64,
}

impl NetworkMap {
    pub fn new() -> Self {
        Self {
            net_info: LocalNetworkInfo::discover(),
            tick_counter: 0,
        }
    }

    pub fn draw_with_state(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        focused: bool,
        state: &GameState,
    ) -> Result<()> {
        self.tick_counter = state.total_ticks;

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
            .title(" NETWORK MAP ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines: Vec<Line> = Vec::new();
        // Header: hostname and gateway
        let hostname_display = format!("  {}@{}", whoami(), &self.net_info.hostname);
        lines.push(Line::from(Span::styled(hostname_display, theme::title())));

        // Gateway line
        if let Some(ref gw) = self.net_info.gateway {
            lines.push(Line::from(vec![
                Span::styled("  gw: ", theme::text_dim()),
                Span::styled(gw.as_str(), theme::text_value()),
            ]));
        }

        // Interface list
        for iface in &self.net_info.interfaces {
            lines.push(Line::from(vec![
                Span::styled("  if: ", theme::text_dim()),
                Span::styled(iface.as_str(), theme::text_value()),
            ]));
        }

        // DNS
        if !self.net_info.dns_servers.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  dns: ", theme::text_dim()),
                Span::styled(self.net_info.dns_servers.join(", "), theme::text_value()),
            ]));
        }

        lines.push(Line::from(""));

        // Network topology visualization
        // Show owned infrastructure as nodes connected by lines
        let defs = all_building_defs();
        let owned: Vec<_> = defs
            .iter()
            .filter(|d| {
                state
                    .buildings
                    .get(&d.kind)
                    .map(|b| b.count > 0)
                    .unwrap_or(false)
            })
            .collect();

        if owned.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No infrastructure deployed",
                theme::text_dim(),
            )));
        } else {
            // Animated traffic indicator
            let dots = ["·", "∘", "○", "●", "○", "∘"];
            let dot_idx = (self.tick_counter / 2) as usize % dots.len();
            let traffic_dot = dots[dot_idx];

            // Draw topology as a simple tree
            let host_short = if self.net_info.hostname.len() > 12 {
                &self.net_info.hostname[..12]
            } else {
                &self.net_info.hostname
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  [{host_short}]"),
                    theme::title(),
                ),
            ]));

            // Group by resource type for visual clarity
            let compute_nodes: Vec<_> = owned
                .iter()
                .filter(|d| matches!(d.resource_type, crate::game::buildings::ResourceType::Compute))
                .collect();
            let network_nodes: Vec<_> = owned
                .iter()
                .filter(|d| matches!(d.resource_type, crate::game::buildings::ResourceType::Bandwidth))
                .collect();
            let storage_nodes: Vec<_> = owned
                .iter()
                .filter(|d| matches!(d.resource_type, crate::game::buildings::ResourceType::Storage))
                .collect();

            let max_height = inner.height as usize;

            // Render each group with a branch
            let groups = [
                ("CPU", &compute_nodes),
                ("NET", &network_nodes),
                ("SSD", &storage_nodes),
            ];

            for (i, (label, nodes)) in groups.iter().enumerate() {
                if nodes.is_empty() || lines.len() >= max_height - 1 {
                    continue;
                }

                let connector = if i < groups.len() - 1 { "├" } else { "└" };
                let pipe = if i < groups.len() - 1 { "│" } else { " " };

                lines.push(Line::from(vec![
                    Span::styled(format!("   {connector}── "), theme::text_dim()),
                    Span::styled(format!("[{label}]"), theme::title()),
                    Span::styled(format!(" {traffic_dot}"), ratatui::style::Style::default().fg(theme::ACCENT_CYAN)),
                ]));

                for (j, node) in nodes.iter().enumerate() {
                    if lines.len() >= max_height - 1 {
                        break;
                    }
                    let count = state
                        .buildings
                        .get(&node.kind)
                        .map(|b| b.count)
                        .unwrap_or(0);
                    let sub_connector = if j < nodes.len() - 1 { "├" } else { "└" };

                    lines.push(Line::from(vec![
                        Span::styled(format!("   {pipe}   {sub_connector}─ "), theme::text_dim()),
                        Span::styled(node.name, theme::text_dim()),
                        Span::styled(format!(" x{count}"), theme::text_value()),
                    ]));
                }
            }

            // Traffic spike indicator
            if state.traffic_spike_remaining > 0 {
                if lines.len() < max_height {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!(
                            "  ⚡ TRAFFIC SPIKE x{:.1} ({}s)",
                            state.traffic_spike_multiplier,
                            state.traffic_spike_remaining / 4
                        ),
                        ratatui::style::Style::default().fg(theme::ACCENT_YELLOW),
                    )));
                }
            }
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, inner);
        Ok(())
    }
}

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into())
}

impl Component for NetworkMap {
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
            .title(" NETWORK MAP ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let content = Paragraph::new("  Scanning network...")
            .style(theme::text_dim())
            .block(block);

        frame.render_widget(content, area);
        Ok(())
    }
}
