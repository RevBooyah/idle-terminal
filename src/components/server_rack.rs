use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::action::Action;
use crate::components::Component;
use crate::game::buildings::{all_building_defs, BuildingKind};
use crate::game::resources::format_si;
use crate::game::state::GameState;
use crate::theme;

#[derive(Clone, Copy, PartialEq)]
enum View {
    Buildings,
    Upgrades,
}

pub struct ServerRack {
    selected_index: usize,
    scroll_offset: usize,
    view: View,
}

impl ServerRack {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            view: View::Buildings,
        }
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

        let title = match self.view {
            View::Buildings => " SERVER RACK ",
            View::Upgrades => " UPGRADES ",
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match self.view {
            View::Buildings => self.draw_buildings(frame, inner, focused, state),
            View::Upgrades => self.draw_upgrades(frame, inner, focused, state),
        }
    }

    fn draw_buildings(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        focused: bool,
        state: &GameState,
    ) -> Result<()> {
        let unlocked = state.unlocked_buildings();
        if unlocked.is_empty() {
            let msg = Paragraph::new("  No buildings available yet...").style(theme::text_dim());
            frame.render_widget(msg, area);
            return Ok(());
        }

        let defs = all_building_defs();
        let visible_height = area.height as usize;
        let lines_per_building = 3;
        let max_visible = visible_height.saturating_sub(2) / lines_per_building;

        let mut lines: Vec<Line> = Vec::new();

        for (i, kind) in unlocked.iter().enumerate() {
            if i < self.scroll_offset {
                continue;
            }
            if lines.len() / lines_per_building >= max_visible {
                break;
            }

            let def = match defs.iter().find(|d| d.kind == *kind) {
                Some(d) => d,
                None => continue,
            };
            let instance = match state.buildings.get(kind) {
                Some(inst) => inst,
                None => continue,
            };

            let is_selected = i == self.selected_index && focused;
            let can_afford = state.resources.can_afford(&def.cost_as_resources(instance.count));
            let next_cost = def.next_cost(instance.count);

            let cicd_count = state
                .buildings
                .get(&BuildingKind::CICDPipeline)
                .map(|b| b.count)
                .unwrap_or(0);
            let cicd_mult = 1.0 + (cicd_count as f64 * 0.10);
            let prod_per_sec =
                def.production_per_tick(instance.count, instance.level, state.global_multiplier * cicd_mult)
                    * 4.0;

            let marker = if is_selected { "▸ " } else { "  " };
            let name_style = if is_selected {
                theme::title()
            } else {
                theme::text_dim()
            };
            let count_str = if instance.count > 0 {
                format!("x{}", instance.count)
            } else {
                String::new()
            };
            let level_str = if instance.level > 0 {
                format!(" [Lv.{}]", instance.level)
            } else {
                String::new()
            };

            lines.push(Line::from(vec![
                Span::styled(marker, name_style),
                Span::styled(format!("{:<20}", def.name), name_style),
                Span::styled(count_str, theme::text_value()),
                Span::styled(level_str, theme::text_value()),
            ]));

            let cost_style = if can_afford {
                ratatui::style::Style::default().fg(theme::FG_PRIMARY)
            } else {
                ratatui::style::Style::default().fg(theme::ACCENT_RED)
            };

            let prod_str = if prod_per_sec > 0.0 {
                format!("+{}/s", format_si(prod_per_sec))
            } else if def.kind == BuildingKind::CICDPipeline && instance.count > 0 {
                format!("+{}% global", instance.count * 10)
            } else {
                String::from("--")
            };

            lines.push(Line::from(vec![
                Span::styled("    ", theme::text_dim()),
                Span::styled(format!("{:<14}", prod_str), ratatui::style::Style::default().fg(theme::FG_PRIMARY)),
                Span::styled("Cost: ", theme::text_dim()),
                Span::styled(format_si(next_cost), cost_style),
            ]));

            lines.push(Line::from(""));
        }

        if focused {
            lines.push(Line::from(vec![
                Span::styled(" [Enter]", theme::text_value()),
                Span::styled("Buy ", theme::text_dim()),
                Span::styled("[u]", theme::text_value()),
                Span::styled("Upgrade ", theme::text_dim()),
                Span::styled("[r]", theme::text_value()),
                Span::styled("Research", theme::text_dim()),
            ]));
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, area);
        Ok(())
    }

    fn draw_upgrades(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        focused: bool,
        state: &GameState,
    ) -> Result<()> {
        let available = state.available_upgrades();
        let purchased: Vec<_> = state.upgrades.iter().filter(|u| u.purchased).collect();

        let mut lines: Vec<Line> = Vec::new();

        // Available upgrades
        lines.push(Line::from(Span::styled(
            "  Available Research:",
            theme::title(),
        )));
        lines.push(Line::from(""));

        if available.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No upgrades available",
                theme::text_dim(),
            )));
        } else {
            let visible_height = area.height as usize;
            let max_visible = visible_height.saturating_sub(6) / 3;

            for (i, upgrade) in available.iter().enumerate() {
                if i >= max_visible {
                    break;
                }

                let is_selected = i == self.selected_index && focused;
                let can_afford = state.resources.can_afford(&upgrade.cost);

                let marker = if is_selected { "▸ " } else { "  " };
                let name_style = if is_selected {
                    theme::title()
                } else {
                    theme::text_dim()
                };
                let cost_style = if can_afford {
                    ratatui::style::Style::default().fg(theme::FG_PRIMARY)
                } else {
                    ratatui::style::Style::default().fg(theme::ACCENT_RED)
                };

                lines.push(Line::from(vec![
                    Span::styled(marker, name_style),
                    Span::styled(&upgrade.name, name_style),
                ]));

                // Cost line
                let cost_parts: Vec<String> = [
                    (upgrade.cost.compute, "CPU"),
                    (upgrade.cost.bandwidth, "BW"),
                    (upgrade.cost.storage, "SSD"),
                ]
                .iter()
                .filter(|(v, _)| *v > 0.0)
                .map(|(v, label)| format!("{} {}", format_si(*v), label))
                .collect();

                lines.push(Line::from(vec![
                    Span::styled("    ", theme::text_dim()),
                    Span::styled(&upgrade.description, theme::text_dim()),
                    Span::styled("  Cost: ", theme::text_dim()),
                    Span::styled(cost_parts.join(" + "), cost_style),
                ]));

                lines.push(Line::from(""));
            }
        }

        // Purchased count
        if !purchased.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} upgrades purchased", purchased.len()),
                    theme::text_dim(),
                ),
            ]));
        }

        if focused {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(" [Enter]", theme::text_value()),
                Span::styled("Buy ", theme::text_dim()),
                Span::styled("[r]", theme::text_value()),
                Span::styled("Buildings", theme::text_dim()),
            ]));
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, area);
        Ok(())
    }

    pub fn handle_key_with_state(
        &mut self,
        key: KeyEvent,
        state: &GameState,
    ) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Char('r') => {
                self.view = match self.view {
                    View::Buildings => View::Upgrades,
                    View::Upgrades => View::Buildings,
                };
                self.selected_index = 0;
                self.scroll_offset = 0;
                return Ok(Some(Action::None));
            }
            _ => {}
        }

        match self.view {
            View::Buildings => self.handle_building_keys(key, state),
            View::Upgrades => self.handle_upgrade_keys(key, state),
        }
    }

    fn handle_building_keys(
        &mut self,
        key: KeyEvent,
        state: &GameState,
    ) -> Result<Option<Action>> {
        let unlocked = state.unlocked_buildings();
        if unlocked.is_empty() {
            return Ok(None);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
                Ok(Some(Action::None))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < unlocked.len() - 1 {
                    self.selected_index += 1;
                }
                Ok(Some(Action::None))
            }
            KeyCode::Enter => {
                if self.selected_index < unlocked.len() {
                    Ok(Some(Action::PurchaseBuilding(unlocked[self.selected_index])))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('u') => {
                if self.selected_index < unlocked.len() {
                    Ok(Some(Action::UpgradeBuilding(
                        unlocked[self.selected_index],
                    )))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn handle_upgrade_keys(
        &mut self,
        key: KeyEvent,
        state: &GameState,
    ) -> Result<Option<Action>> {
        let available = state.available_upgrades();
        if available.is_empty() {
            return Ok(None);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                Ok(Some(Action::None))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < available.len() - 1 {
                    self.selected_index += 1;
                }
                Ok(Some(Action::None))
            }
            KeyCode::Enter => {
                if self.selected_index < available.len() {
                    Ok(Some(Action::PurchaseUpgrade(
                        available[self.selected_index].id,
                    )))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

impl Component for ServerRack {
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
            .title(" SERVER RACK ")
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
