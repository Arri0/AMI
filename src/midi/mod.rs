mod reader;
mod msg;

pub use reader::ReaderError;
pub use reader::MidiReader;
pub use msg::ControlChangeKind;
pub use msg::MessageKind;
pub use msg::Message;
use tokio::sync::broadcast;

pub type Sender = broadcast::Sender<msg::Message>;
pub type Receiver = broadcast::Receiver<msg::Message>;

pub fn create_channel(buffer: usize) -> (Sender, Receiver) {
    broadcast::channel(buffer)
}