use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    Frame,
};

use crate::components::Component;
use crate::layout::PaneId;
use crate::theme;

pub struct StatusBar {
    focused_pane: PaneId,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            focused_pane: PaneId::Dashboard,
        }
    }

    pub fn set_focused_pane(&mut self, pane: PaneId) {
        self.focused_pane = pane;
    }
}

impl Component for StatusBar {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect, _focused: bool) -> Result<()> {
        let pane_name = match self.focused_pane {
            PaneId::Dashboard => "DASHBOARD",
            PaneId::ServerRack => "SERVER RACK",
            PaneId::NetworkMap => "NETWORK MAP",
            PaneId::TaskTerminal => "TASK TERMINAL",
        };

        let line = Line::from(vec![
            Span::styled(" [Tab]", theme::text_value()),
            Span::styled("Pane ", theme::text_dim()),
            Span::styled("[1-4]", theme::text_value()),
            Span::styled("Jump ", theme::text_dim()),
            Span::styled("[p]", theme::text_value()),
            Span::styled("Prestige ", theme::text_dim()),
            Span::styled("[q]", theme::text_value()),
            Span::styled("Quit ", theme::text_dim()),
            Span::styled("| ", theme::text_dim()),
            Span::styled(pane_name, theme::title()),
        ]);

        frame.render_widget(line, area);
        Ok(())
    }
}
