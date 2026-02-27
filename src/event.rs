use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Render,
    GameTick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(render_rate_ms: u64, game_tick_ms: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            let mut reader = event::EventStream::new();
            let mut render_interval = tokio::time::interval(Duration::from_millis(render_rate_ms));
            let mut game_interval = tokio::time::interval(Duration::from_millis(game_tick_ms));

            loop {
                tokio::select! {
                    maybe_event = reader.next() => {
                        if let Some(Ok(evt)) = maybe_event {
                            let mapped = match evt {
                                CrosstermEvent::Key(key) => Some(Event::Key(key)),
                                CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                                CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
                                _ => None,
                            };
                            if let Some(e) = mapped {
                                if tx.send(e).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    _ = render_interval.tick() => {
                        if tx.send(Event::Render).is_err() {
                            break;
                        }
                    }
                    _ = game_interval.tick() => {
                        if tx.send(Event::GameTick).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self { rx, _task: task }
    }

    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| color_eyre::eyre::eyre!("Event channel closed"))
    }
}
