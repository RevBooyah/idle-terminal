use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneId {
    Dashboard,
    ServerRack,
    NetworkMap,
    TaskTerminal,
}

pub const FOCUSABLE_PANES: &[PaneId] = &[
    PaneId::Dashboard,
    PaneId::ServerRack,
    PaneId::NetworkMap,
    PaneId::TaskTerminal,
];

pub struct PaneLayout {
    pub header: Rect,
    pub dashboard: Rect,
    pub server_rack: Rect,
    pub network_map: Rect,
    pub task_terminal: Rect,
    pub log_stream: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(area: Rect) -> PaneLayout {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // header
            Constraint::Percentage(42), // top row panes
            Constraint::Percentage(42), // bottom row panes
            Constraint::Length(3),      // log stream
            Constraint::Length(1),      // status bar
        ])
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[2]);

    PaneLayout {
        header: outer[0],
        dashboard: top_row[0],
        server_rack: top_row[1],
        network_map: bottom_row[0],
        task_terminal: bottom_row[1],
        log_stream: outer[3],
        status_bar: outer[4],
    }
}
