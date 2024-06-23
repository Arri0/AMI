mod reader;
mod msg;

pub use reader::ReaderError;
pub use reader::MidiReader;
pub use msg::ControlChangeKind;
pub use msg::MessageKind;
pub use msg::Message;

pub type Sender = tokio::sync::broadcast::Sender<msg::Message>;
pub type Receiver = tokio::sync::broadcast::Receiver<msg::Message>;

pub fn create_channel(buffer: usize) -> (Sender, Receiver) {
    tokio::sync::broadcast::channel(buffer)
}