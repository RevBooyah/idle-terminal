use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use rand::SeedableRng;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};

use crate::action::Action;
use crate::components::Component;
use crate::game::state::GameState;
use crate::game::tasks::{generate_random_task, ActiveTask, TaskKind, TASK_COOLDOWN_TICKS};
use crate::theme;

pub struct TaskTerminal {
    active_task: Option<ActiveTask>,
    cooldown_ticks: u32,
    rng: rand::rngs::StdRng,
    last_result: Option<TaskResult>,
    pending_reward: Option<crate::game::resources::Resources>,
}

enum TaskResult {
    Completed,
    Failed,
    Expired,
}

impl TaskTerminal {
    pub fn new() -> Self {
        Self {
            active_task: None,
            cooldown_ticks: TASK_COOLDOWN_TICKS / 2, // Shorter initial wait
            rng: rand::rngs::StdRng::from_entropy(),
            last_result: None,
            pending_reward: None,
        }
    }

    pub fn game_tick(&mut self, game_state: &mut GameState) {
        // Grant any pending reward from completed task
        if let Some(mut reward) = self.pending_reward.take() {
            reward.compute *= game_state.task_reward_multiplier;
            reward.bandwidth *= game_state.task_reward_multiplier;
            reward.storage *= game_state.task_reward_multiplier;
            game_state.resources.add(&reward);
            game_state.tasks_completed += 1;
        }

        if let Some(ref mut task) = self.active_task {
            task.tick();
            if task.is_expired() {
                self.last_result = Some(TaskResult::Expired);
                self.active_task = None;
                self.cooldown_ticks = TASK_COOLDOWN_TICKS;
            }
        } else {
            // Cooldown before spawning next task
            if self.cooldown_ticks > 0 {
                self.cooldown_ticks -= 1;
            } else {
                let def = generate_random_task(&mut self.rng);
                self.active_task = Some(ActiveTask::new(def));
                self.last_result = None;
            }
        }
    }

    pub fn handle_key_with_state(
        &mut self,
        key: KeyEvent,
        _state: &GameState,
    ) -> Result<Option<Action>> {
        let task = match self.active_task.as_mut() {
            Some(t) => t,
            None => return Ok(None),
        };

        match &task.definition.kind {
            TaskKind::TypeCommand { .. } => match key.code {
                KeyCode::Char(c) => {
                    task.input.push(c);
                    if task.check_completion() {
                        let reward = task.definition.reward.clone();
                        self.pending_reward = Some(reward);
                        self.last_result = Some(TaskResult::Completed);
                        self.active_task = None;
                        self.cooldown_ticks = TASK_COOLDOWN_TICKS;
                    }
                    Ok(Some(Action::None)) // Consumed the key
                }
                KeyCode::Backspace => {
                    task.input.pop();
                    Ok(Some(Action::None))
                }
                _ => Ok(None),
            },
            TaskKind::IncidentResponse { options, .. } => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if task.selected_option > 0 {
                        task.selected_option -= 1;
                    }
                    Ok(Some(Action::None))
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if task.selected_option < options.len() - 1 {
                        task.selected_option += 1;
                    }
                    Ok(Some(Action::None))
                }
                KeyCode::Enter => {
                    if task.check_completion() {
                        let reward = task.definition.reward.clone();
                        self.pending_reward = Some(reward);
                        self.last_result = Some(TaskResult::Completed);
                    } else {
                        self.last_result = Some(TaskResult::Failed);
                    }
                    self.active_task = None;
                    self.cooldown_ticks = TASK_COOLDOWN_TICKS;
                    Ok(Some(Action::None))
                }
                _ => Ok(None),
            },
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

        let block = Block::default()
            .title(" TASK TERMINAL ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match &self.active_task {
            None => {
                let mut lines = vec![];

                // Show last result
                if let Some(ref result) = self.last_result {
                    lines.push(Line::from(""));
                    match result {
                        TaskResult::Completed => {
                            lines.push(Line::from(Span::styled(
                                "  ✓ Task completed! Reward granted.",
                                ratatui::style::Style::default().fg(theme::FG_PRIMARY),
                            )));
                        }
                        TaskResult::Failed => {
                            lines.push(Line::from(Span::styled(
                                "  ✗ Wrong answer.",
                                ratatui::style::Style::default().fg(theme::ACCENT_RED),
                            )));
                        }
                        TaskResult::Expired => {
                            lines.push(Line::from(Span::styled(
                                "  ✗ Task expired!",
                                ratatui::style::Style::default().fg(theme::ACCENT_YELLOW),
                            )));
                        }
                    }
                    lines.push(Line::from(""));
                }

                if self.cooldown_ticks > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("  Next task in: ", theme::text_dim()),
                        Span::styled(
                            format!("{}s", self.cooldown_ticks / 4),
                            theme::text_value(),
                        ),
                    ]));
                } else {
                    lines.push(Line::from(Span::styled(
                        "  Awaiting task...",
                        theme::text_dim(),
                    )));
                }

                let content = Paragraph::new(lines);
                frame.render_widget(content, inner);
            }
            Some(task) => {
                let mut lines = vec![];

                // Task name and timer
                lines.push(Line::from(vec![
                    Span::styled("  TASK: ", theme::text_dim()),
                    Span::styled(&task.definition.name, theme::title()),
                    Span::styled(
                        format!("  [{:.0}s]", task.remaining_ticks as f64 / 4.0),
                        if task.time_fraction() < 0.25 {
                            ratatui::style::Style::default().fg(theme::ACCENT_RED)
                        } else {
                            theme::text_value()
                        },
                    ),
                ]));
                lines.push(Line::from(""));

                match &task.definition.kind {
                    TaskKind::TypeCommand { command } => {
                        lines.push(Line::from(vec![
                            Span::styled("  $ ", theme::title()),
                            Span::styled(command.as_str(), theme::text_value()),
                        ]));
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("  > ", theme::title()),
                            Span::styled(&task.input, ratatui::style::Style::default().fg(theme::FG_PRIMARY)),
                            Span::styled("_", if (state.total_ticks / 2) % 2 == 0 {
                                ratatui::style::Style::default().fg(theme::FG_PRIMARY)
                            } else {
                                ratatui::style::Style::default().fg(theme::BG)
                            }),
                        ]));

                        // Show character match feedback
                        if !task.input.is_empty() {
                            let matches = task
                                .input
                                .chars()
                                .zip(command.chars())
                                .all(|(a, b)| a == b)
                                && task.input.len() <= command.len();

                            if !matches {
                                lines.push(Line::from(""));
                                lines.push(Line::from(Span::styled(
                                    "  ✗ Mismatch! Backspace to fix.",
                                    ratatui::style::Style::default().fg(theme::ACCENT_RED),
                                )));
                            }
                        }
                    }
                    TaskKind::IncidentResponse {
                        question, options, ..
                    } => {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", question),
                            theme::text_value(),
                        )));
                        lines.push(Line::from(""));

                        for (i, option) in options.iter().enumerate() {
                            let marker = if i == task.selected_option && focused {
                                "  ▸ "
                            } else {
                                "    "
                            };
                            let style = if i == task.selected_option && focused {
                                theme::title()
                            } else {
                                theme::text_dim()
                            };
                            lines.push(Line::from(Span::styled(
                                format!("{}{}", marker, option),
                                style,
                            )));
                        }

                        if focused {
                            lines.push(Line::from(""));
                            lines.push(Line::from(vec![
                                Span::styled("  [↑/↓]", theme::text_value()),
                                Span::styled(" Select  ", theme::text_dim()),
                                Span::styled("[Enter]", theme::text_value()),
                                Span::styled(" Submit", theme::text_dim()),
                            ]));
                        }
                    }
                }

                let content = Paragraph::new(lines);
                frame.render_widget(content, inner);

                // Timer bar at bottom of inner area
                if inner.height > 2 {
                    let timer_area = Rect {
                        x: inner.x + 1,
                        y: inner.y + inner.height - 1,
                        width: inner.width.saturating_sub(2),
                        height: 1,
                    };
                    let ratio = task.time_fraction();
                    let gauge_color = if ratio > 0.5 {
                        theme::FG_PRIMARY
                    } else if ratio > 0.25 {
                        theme::ACCENT_YELLOW
                    } else {
                        theme::ACCENT_RED
                    };
                    let gauge = Gauge::default()
                        .ratio(ratio)
                        .gauge_style(ratatui::style::Style::default().fg(gauge_color));
                    frame.render_widget(gauge, timer_area);
                }
            }
        }

        Ok(())
    }
}

impl Component for TaskTerminal {
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
            .title(" TASK TERMINAL ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style);

        let content = Paragraph::new("  Awaiting task...")
            .style(theme::text_dim())
            .block(block);

        frame.render_widget(content, area);
        Ok(())
    }
}
