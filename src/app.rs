use color_eyre::eyre::Result;
use crossterm::event::KeyCode;

use crate::action::Action;
use crate::components::dashboard::Dashboard;
use crate::components::header::Header;
use crate::components::log_stream::LogStream;
use crate::components::network_map::NetworkMap;
use crate::components::server_rack::ServerRack;
use crate::components::status_bar::StatusBar;
use crate::components::task_terminal::TaskTerminal;
use crate::components::Component;
use crate::event::{Event, EventHandler};
use crate::game::progression;
use crate::game::resources::format_si;
use crate::game::save;
use crate::game::state::GameState;
use crate::layout::{self, PaneId, FOCUSABLE_PANES};
use crate::tui;

const AUTO_SAVE_INTERVAL_TICKS: u64 = 240; // 60 seconds at 4Hz

pub struct App {
    should_quit: bool,
    focused_pane: PaneId,
    game_state: GameState,
    header: Header,
    dashboard: Dashboard,
    server_rack: ServerRack,
    network_map: NetworkMap,
    task_terminal: TaskTerminal,
    log_stream: LogStream,
    status_bar: StatusBar,
    ticks_since_save: u64,
    welcome_message: Option<String>,
    welcome_display_ticks: u32,
    show_prestige_confirm: bool,
    achievement_notification: Option<String>,
    achievement_display_ticks: u32,
}

impl App {
    pub fn new() -> Self {
        // Try to load saved game
        let (game_state, welcome) = match save::load_game() {
            Ok(Some(result)) => {
                let msg = if result.offline_ticks > 0 {
                    let hours = result.offline_ticks / (4 * 3600);
                    let mins = (result.offline_ticks / (4 * 60)) % 60;
                    Some(format!(
                        "Welcome back! Away {}h {}m. Earned: +{} CPU, +{} BW, +{} SSD",
                        hours,
                        mins,
                        format_si(result.offline_earnings.compute),
                        format_si(result.offline_earnings.bandwidth),
                        format_si(result.offline_earnings.storage),
                    ))
                } else {
                    None
                };
                (result.state, msg)
            }
            Ok(None) => (GameState::new(), None),
            Err(e) => {
                tracing::warn!("Failed to load save: {e}");
                (GameState::new(), None)
            }
        };

        Self {
            should_quit: false,
            focused_pane: PaneId::Dashboard,
            game_state,
            header: Header::new(),
            dashboard: Dashboard::new(),
            server_rack: ServerRack::new(),
            network_map: NetworkMap::new(),
            task_terminal: TaskTerminal::new(),
            log_stream: LogStream::new(),
            status_bar: StatusBar::new(),
            ticks_since_save: 0,
            welcome_message: welcome,
            welcome_display_ticks: 40, // 10 seconds display
            show_prestige_confirm: false,
            achievement_notification: None,
            achievement_display_ticks: 0,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = tui::init()?;
        let mut events = EventHandler::new(33, 250);

        loop {
            let event = events.next().await?;

            match event {
                Event::Key(key) => {
                    // Dismiss welcome message on any key
                    if self.welcome_message.is_some() {
                        self.welcome_message = None;
                    }

                    // Dismiss achievement notification on any key
                    if self.achievement_notification.is_some() {
                        self.achievement_notification = None;
                    }

                    // Handle prestige confirmation mode
                    if self.show_prestige_confirm {
                        match key.code {
                            KeyCode::Char('y') => {
                                let rep_earned = self.game_state.prestige();
                                self.show_prestige_confirm = false;
                                self.achievement_notification = Some(format!(
                                    "PRESTIGE! +{:.0} Reputation (x{:.2} multiplier)",
                                    rep_earned,
                                    progression::reputation_multiplier(
                                        self.game_state.resources.reputation
                                    ),
                                ));
                                self.achievement_display_ticks = 40;
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                self.show_prestige_confirm = false;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Let focused component handle the key first
                    let component_action = match self.focused_pane {
                        PaneId::ServerRack => {
                            self.server_rack
                                .handle_key_with_state(key, &self.game_state)?
                        }
                        PaneId::TaskTerminal => {
                            self.task_terminal
                                .handle_key_with_state(key, &self.game_state)?
                        }
                        _ => None,
                    };

                    if let Some(action) = component_action {
                        self.dispatch_action(action);
                    } else {
                        let action = match key.code {
                            KeyCode::Char('q') => Action::Quit,
                            KeyCode::Tab => Action::NextPane,
                            KeyCode::BackTab => Action::PrevPane,
                            KeyCode::Char('1') => Action::FocusPane(PaneId::Dashboard),
                            KeyCode::Char('2') => Action::FocusPane(PaneId::ServerRack),
                            KeyCode::Char('3') => Action::FocusPane(PaneId::NetworkMap),
                            KeyCode::Char('4') => Action::FocusPane(PaneId::TaskTerminal),
                            KeyCode::Char('p') => Action::Prestige,
                            _ => Action::None,
                        };
                        self.dispatch_action(action);
                    }
                }
                Event::GameTick => {
                    self.game_state.tick();
                    self.task_terminal.game_tick(&mut self.game_state);

                    // Check achievements
                    let new_achievements = self.game_state.check_achievements();
                    if !new_achievements.is_empty() {
                        self.achievement_notification =
                            Some(format!("* {} unlocked!", new_achievements.join(", ")));
                        self.achievement_display_ticks = 32; // 8 seconds
                    }

                    // Tick down achievement notification
                    if self.achievement_notification.is_some() {
                        if self.achievement_display_ticks > 0 {
                            self.achievement_display_ticks -= 1;
                        } else {
                            self.achievement_notification = None;
                        }
                    }

                    // Tick down welcome message
                    if self.welcome_message.is_some() {
                        if self.welcome_display_ticks > 0 {
                            self.welcome_display_ticks -= 1;
                        } else {
                            self.welcome_message = None;
                        }
                    }

                    // Auto-save
                    self.ticks_since_save += 1;
                    if self.ticks_since_save >= AUTO_SAVE_INTERVAL_TICKS {
                        save::save_game(&self.game_state).ok();
                        self.ticks_since_save = 0;
                    }
                }
                Event::Render => {
                    self.status_bar.set_focused_pane(self.focused_pane);
                    let focused = self.focused_pane;
                    let game_state = &self.game_state;
                    let welcome = self.welcome_message.as_deref();
                    let show_prestige = self.show_prestige_confirm;
                    let achievement = self.achievement_notification.as_deref();
                    terminal.draw(|frame| {
                        let panes = layout::compute_layout(frame.area());

                        self.header
                            .draw_with_state(frame, panes.header, false, game_state)
                            .ok();
                        self.dashboard
                            .draw_with_state(
                                frame,
                                panes.dashboard,
                                focused == PaneId::Dashboard,
                                game_state,
                            )
                            .ok();
                        self.server_rack
                            .draw_with_state(
                                frame,
                                panes.server_rack,
                                focused == PaneId::ServerRack,
                                game_state,
                            )
                            .ok();
                        self.network_map
                            .draw_with_state(
                                frame,
                                panes.network_map,
                                focused == PaneId::NetworkMap,
                                game_state,
                            )
                            .ok();
                        self.task_terminal
                            .draw_with_state(
                                frame,
                                panes.task_terminal,
                                focused == PaneId::TaskTerminal,
                                game_state,
                            )
                            .ok();
                        self.log_stream
                            .draw_with_state(frame, panes.log_stream, false, game_state)
                            .ok();
                        self.status_bar
                            .draw(frame, panes.status_bar, false)
                            .ok();

                        // Achievement notification overlay
                        if let Some(msg) = achievement {
                            let popup_width =
                                (msg.len() as u16 + 4).min(frame.area().width.saturating_sub(4));
                            let popup_area = ratatui::layout::Rect {
                                x: (frame.area().width.saturating_sub(popup_width)) / 2,
                                y: 4,
                                width: popup_width,
                                height: 3,
                            };
                            let popup = ratatui::widgets::Paragraph::new(format!(" {msg}"))
                                .style(
                                    ratatui::style::Style::default()
                                        .fg(crate::theme::ACCENT_MAGENTA),
                                )
                                .block(
                                    ratatui::widgets::Block::default()
                                        .borders(ratatui::widgets::Borders::ALL)
                                        .border_type(ratatui::widgets::BorderType::Double)
                                        .border_style(
                                            ratatui::style::Style::default()
                                                .fg(crate::theme::ACCENT_MAGENTA),
                                        )
                                        .title(" ACHIEVEMENT "),
                                );
                            frame.render_widget(ratatui::widgets::Clear, popup_area);
                            frame.render_widget(popup, popup_area);
                        }

                        // Welcome back overlay
                        if let Some(msg) = welcome {
                            let popup_width =
                                (msg.len() as u16 + 4).min(frame.area().width.saturating_sub(4));
                            let popup_area = ratatui::layout::Rect {
                                x: (frame.area().width.saturating_sub(popup_width)) / 2,
                                y: frame.area().height / 2 - 1,
                                width: popup_width,
                                height: 3,
                            };
                            let popup = ratatui::widgets::Paragraph::new(format!(" {msg}"))
                                .style(
                                    ratatui::style::Style::default()
                                        .fg(crate::theme::FG_PRIMARY),
                                )
                                .block(
                                    ratatui::widgets::Block::default()
                                        .borders(ratatui::widgets::Borders::ALL)
                                        .border_type(ratatui::widgets::BorderType::Double)
                                        .border_style(
                                            ratatui::style::Style::default()
                                                .fg(crate::theme::ACCENT_CYAN),
                                        )
                                        .title(" WELCOME BACK "),
                                );
                            frame.render_widget(ratatui::widgets::Clear, popup_area);
                            frame.render_widget(popup, popup_area);
                        }

                        // Prestige confirmation overlay
                        if show_prestige {
                            let rep_preview =
                                progression::prestige_reputation(game_state.resources.compute);
                            let new_mult = progression::reputation_multiplier(
                                game_state.resources.reputation + rep_preview,
                            );

                            let lines = vec![
                                ratatui::text::Line::from(""),
                                ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                                    "  This will reset ALL resources and buildings.",
                                    ratatui::style::Style::default()
                                        .fg(crate::theme::ACCENT_YELLOW),
                                )]),
                                ratatui::text::Line::from(vec![
                                    ratatui::text::Span::styled(
                                        "  Reputation earned: +",
                                        crate::theme::text_dim(),
                                    ),
                                    ratatui::text::Span::styled(
                                        format!("{:.0}", rep_preview),
                                        ratatui::style::Style::default()
                                            .fg(crate::theme::ACCENT_MAGENTA),
                                    ),
                                ]),
                                ratatui::text::Line::from(vec![
                                    ratatui::text::Span::styled(
                                        "  New multiplier: x",
                                        crate::theme::text_dim(),
                                    ),
                                    ratatui::text::Span::styled(
                                        format!("{:.2}", new_mult),
                                        crate::theme::text_value(),
                                    ),
                                ]),
                                ratatui::text::Line::from(""),
                                ratatui::text::Line::from(vec![
                                    ratatui::text::Span::styled(
                                        "  [y] ",
                                        crate::theme::text_value(),
                                    ),
                                    ratatui::text::Span::styled(
                                        "Confirm  ",
                                        crate::theme::text_dim(),
                                    ),
                                    ratatui::text::Span::styled(
                                        "[n] ",
                                        crate::theme::text_value(),
                                    ),
                                    ratatui::text::Span::styled(
                                        "Cancel",
                                        crate::theme::text_dim(),
                                    ),
                                ]),
                            ];

                            let popup_width = 50u16.min(frame.area().width.saturating_sub(4));
                            let popup_height = 8u16;
                            let popup_area = ratatui::layout::Rect {
                                x: (frame.area().width.saturating_sub(popup_width)) / 2,
                                y: frame
                                    .area()
                                    .height
                                    .saturating_sub(popup_height)
                                    / 2,
                                width: popup_width,
                                height: popup_height,
                            };
                            let popup = ratatui::widgets::Paragraph::new(lines).block(
                                ratatui::widgets::Block::default()
                                    .borders(ratatui::widgets::Borders::ALL)
                                    .border_type(ratatui::widgets::BorderType::Double)
                                    .border_style(
                                        ratatui::style::Style::default()
                                            .fg(crate::theme::ACCENT_MAGENTA),
                                    )
                                    .title(" * PRESTIGE RESET * "),
                            );
                            frame.render_widget(ratatui::widgets::Clear, popup_area);
                            frame.render_widget(popup, popup_area);
                        }
                    })?;
                }
                Event::Resize(_, _) | Event::Mouse(_) => {}
            }

            if self.should_quit {
                break;
            }
        }

        // Save on quit
        save::save_game(&self.game_state).ok();
        tui::restore()?;
        Ok(())
    }

    fn dispatch_action(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::NextPane => {
                self.cycle_pane(1);
            }
            Action::PrevPane => {
                self.cycle_pane(-1);
            }
            Action::FocusPane(pane) => {
                self.focused_pane = pane;
            }
            Action::PurchaseBuilding(kind) => {
                self.game_state.purchase_building(kind);
            }
            Action::UpgradeBuilding(kind) => {
                self.game_state.upgrade_building(kind);
            }
            Action::PurchaseUpgrade(id) => {
                self.game_state.purchase_upgrade(id);
            }
            Action::Prestige => {
                if self.game_state.can_prestige() {
                    self.show_prestige_confirm = true;
                }
            }
            _ => {}
        }
    }

    fn cycle_pane(&mut self, direction: i32) {
        let idx = FOCUSABLE_PANES
            .iter()
            .position(|p| *p == self.focused_pane)
            .unwrap_or(0);
        let len = FOCUSABLE_PANES.len() as i32;
        let next = ((idx as i32 + direction).rem_euclid(len)) as usize;
        self.focused_pane = FOCUSABLE_PANES[next];
    }
}
