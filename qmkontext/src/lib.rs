#[macro_use]
extern crate tracing;

mod engine;
mod error;
mod event_sink;
mod event_source;

#[derive(Clone, Debug)]
pub enum Event {
    Send { command_id: u8, command_data: u8 },
}

pub use chrono;
pub use hidapi;

pub type Result<T> = std::result::Result<T, Error>;

pub use engine::Engine;
pub use error::Error;
pub use event_sink::{CliSink, EventSink, HidEventSink, SendData};
pub use event_source::{EventSource, UserEventConfig, UserEventSource, UserEventSourceKind};
