use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, mpsc};

pub type TempoBpm = f32;

pub type BeatSender = broadcast::Sender<Message>;
pub type BeatReceiver = broadcast::Receiver<Message>;

pub type CtrSender = mpsc::Sender<CtrMessage>;
pub type CtrReceiver = mpsc::Receiver<CtrMessage>;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rhythm {
    pub num_beats: u8,
    pub num_divs: u8,
}

impl Rhythm {
    pub fn num_slots(&self) -> usize {
        self.num_beats as usize * self.num_divs as usize
    }
}

impl Default for Rhythm {
    fn default() -> Self {
        Self {
            num_beats: 4,
            num_divs: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatControllerConfig {
    pub tempo: TempoBpm,
    pub enabled: bool,
    pub rhythm: Rhythm,
}

impl Default for BeatControllerConfig {
    fn default() -> Self {
        Self {
            tempo: 100.0,
            enabled: false,
            rhythm: Default::default(),
        }
    }
}

pub struct BeatController {
    config: BeatControllerConfig,
    last_time: f32,
    sender: BeatSender,
    ctr_receiver: CtrReceiver,
    start: SystemTime,
    current_beat: u8,
    current_div: u8,
}

impl BeatController {
    pub fn new(sender: BeatSender, ctr_receiver: CtrReceiver) -> Self {
        let res = Self {
            config: Default::default(),
            last_time: 0.0,
            sender,
            ctr_receiver,
            start: SystemTime::now(),
            current_beat: 0,
            current_div: 0,
        };
        _ = res.sender.send(Message::SetRhythm(res.config.rhythm));
        res
    }

    pub fn reset(&mut self) {
        self.last_time = self.timestamp();
        self.current_beat = 0;
        self.current_div = 0;
    }

    pub fn set_enabled(&mut self, flag: bool) {
        if flag && !self.config.enabled {
            self.reset();
        }
        self.config.enabled = flag;
    }

    pub fn tick(&mut self) {
        self.receive_control_msgs();
        if self.config.enabled {
            let time = self.timestamp();
            let period = self.period();
            if time - self.last_time >= period {
                let msg = Message::BeatTick(self.current_beat, self.current_div);
                _ = self.sender.send(msg);
                self.advance_div();
                self.last_time += period;
            }
        }
    }

    fn receive_control_msgs(&mut self) {
        loop {
            match self.ctr_receiver.try_recv() {
                Ok(CtrMessage::SetEnabled(flag)) => self.set_enabled(flag),
                Ok(CtrMessage::SetRhythm(rhythm)) => self.set_rhythm(rhythm),
                Ok(CtrMessage::SetTempo(tempo)) => self.config.tempo = tempo,
                Ok(CtrMessage::Reset) => self.reset(),
                Err(_) => break,
            }
        }
    }

    pub fn set_tempo(&mut self, tempo: f32) {
        self.config.tempo = tempo;
    }

    pub fn set_rhythm(&mut self, rhythm: Rhythm) {
        self.config.rhythm = rhythm;
        _ = self.sender.send(Message::SetRhythm(rhythm));
    }

    pub fn period(&self) -> f32 {
        60.0 / (self.config.tempo * self.config.rhythm.num_divs as f32)
    }

    fn advance_div(&mut self) {
        self.current_div = (self.current_div + 1) % self.config.rhythm.num_divs;
        if self.current_div == 0 {
            self.advance_beat();
        }
    }

    fn advance_beat(&mut self) {
        self.current_beat = (self.current_beat + 1) % self.config.rhythm.num_beats;
    }

    fn timestamp(&self) -> f32 {
        self.start.elapsed().unwrap_or(Duration::ZERO).as_secs_f32()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
    BeatTick(u8, u8),
    SetRhythm(Rhythm),
}

pub fn create_channel(buffer: usize) -> (BeatSender, BeatReceiver) {
    broadcast::channel(buffer)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CtrMessage {
    SetEnabled(bool),
    SetRhythm(Rhythm),
    Reset,
    SetTempo(TempoBpm),
}

pub fn create_ctr_channel(buffer: usize) -> (CtrSender, CtrReceiver) {
    mpsc::channel(buffer)
}
