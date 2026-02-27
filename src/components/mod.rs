pub mod dashboard;
pub mod header;
pub mod log_stream;
pub mod network_map;
pub mod server_rack;
pub mod status_bar;
pub mod task_terminal;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::action::Action;

pub trait Component {
    fn handle_key_event(&mut self, _key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, _action: &Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, focused: bool) -> Result<()>;
}
