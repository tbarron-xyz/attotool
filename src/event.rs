use crossterm::event::{
    Event as CEvent, EventStream, KeyCode, MouseEvent, MouseEventKind,
};
use futures::StreamExt;

pub enum Event {
    Input(KeyCode),
    ScrollUp,
    ScrollDown,
    Tick,
}

pub struct Events {
    stream: EventStream,
}

impl Events {
    pub fn new() -> Self {
        Self {
            stream: EventStream::new(),
        }
    }

    pub async fn next(&mut self) -> Result<Event, Box<dyn std::error::Error>> {
        match self.stream.next().await {
            Some(Ok(CEvent::Key(key))) => Ok(Event::Input(key.code)),
            Some(Ok(CEvent::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                ..
            }))) => Ok(Event::ScrollUp),
            Some(Ok(CEvent::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                ..
            }))) => Ok(Event::ScrollDown),
            Some(Ok(_)) => Ok(Event::Tick), // Handle other events as Tick or ignore
            Some(Err(e)) => Err(e.into()),
            None => Err("Event stream ended".into()),
        }
    }
}
