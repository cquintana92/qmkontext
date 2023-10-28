use crate::{Event, EventSink, EventSource, Result, SendData};

pub struct Engine<Source, Sink>
where
    Source: EventSource + Send + Sync + 'static,
    Sink: EventSink,
{
    source: Source,
    sink: Sink,
}

impl<Source, Sink> Engine<Source, Sink>
where
    Source: EventSource + Send + Sync + 'static,
    Sink: EventSink,
{
    pub fn new(source: Source, sink: Sink) -> Self {
        Self { source, sink }
    }

    pub fn start(self) -> Result<()> {
        let channel = self.source.events();
        let source = self.source;
        std::thread::spawn(|| {
            source.start();
        });
        while let Some(evt) = channel.iter().next() {
            match evt {
                Event::Send {
                    command_id,
                    command_data,
                } => {
                    let payload = SendData {
                        command_id,
                        data: command_data,
                    };
                    self.sink.send(&payload)?;
                }
            }
        }

        Ok(())
    }
}
