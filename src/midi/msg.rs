// Resources:
// https://www.songstuff.com/recording/article/midi_message_format/
// https://www.midi.org/specifications-old/item/table-3-control-change-messages-data-bytes-2

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum MessageKind {
    NoteOff { note: u8, velocity: u8 },
    NoteOn { note: u8, velocity: u8 },
    PolyphonicAftertouch { note: u8, pressure: u8 },
    ControlChange { kind: ControlChangeKind, value: u8 },
    ProgramChange { program: u8 },
    ChannelAftertouch { pressure: u8 },
    PitchWheel { value: u16 },
}

impl MessageKind {
    pub fn as_number(&self) -> u8 {
        match *self {
            MessageKind::NoteOff { .. } => 0x80,
            MessageKind::NoteOn { .. } => 0x90,
            MessageKind::PolyphonicAftertouch { .. } => 0xA0,
            MessageKind::ControlChange { .. } => 0xB0,
            MessageKind::ProgramChange { .. } => 0xC0,
            MessageKind::ChannelAftertouch { .. } => 0xD0,
            MessageKind::PitchWheel { .. } => 0xE0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum ControlChangeKind {
    BankSelectMsb,
    ModulationWheelMsb,
    BreathControllerMsb,
    Undefined0Msb,
    FootControllerMsb,
    PortamentoTimeMsb,
    DataEntryMsb,
    ChannelVolumeMsb,
    BalanceMsb,
    Undefined1Msb,
    PanMsb,
    ExpressionControllerMsb,
    EffectControl1Msb,
    EffectControl2Msb,
    Undefined2Msb,
    Undefined3Msb,
    GeneralPurposeController1Msb,
    GeneralPurposeController2Msb,
    GeneralPurposeController3Msb,
    GeneralPurposeController4Msb,
    Undefined4Msb,
    Undefined5Msb,
    Undefined6Msb,
    Undefined7Msb,
    Undefined8Msb,
    Undefined9Msb,
    Undefined10Msb,
    Undefined11Msb,
    Undefined12Msb,
    Undefined13Msb,
    Undefined14Msb,
    Undefined15Msb,
    BankSelectLsb,
    ModulationWheelLsb,
    BreathControllerLsb,
    Undefined0Lsb,
    FootControllerLsb,
    PortamentoTimeLsb,
    DataEntryLsb,
    ChannelVolumeLsb,
    BalanceLsb,
    Undefined1Lsb,
    PanLsb,
    ExpressionControllerLsb,
    EffectControl1Lsb,
    EffectControl2Lsb,
    Undefined2Lsb,
    Undefined3Lsb,
    GeneralPurposeController1Lsb,
    GeneralPurposeController2Lsb,
    GeneralPurposeController3Lsb,
    GeneralPurposeController4Lsb,
    Undefined4Lsb,
    Undefined5Lsb,
    Undefined6Lsb,
    Undefined7Lsb,
    Undefined8Lsb,
    Undefined9Lsb,
    Undefined10Lsb,
    Undefined11Lsb,
    Undefined12Lsb,
    Undefined13Lsb,
    Undefined14Lsb,
    Undefined15Lsb,
    DamperPedal,
    Portamento,
    Sostenuto,
    SoftPedal,
    LegatoFootswitch,
    Hold2,
    SoundController1,  // default: Sound Variation
    SoundController2,  // default: Timbre/Harmonic Intens.
    SoundController3,  // default: Release Time
    SoundController4,  // default: Attack Time
    SoundController5,  // default: Brightness
    SoundController6,  // default: Decay Time
    SoundController7,  // default: Vibrato Rate
    SoundController8,  // default: Vibrato Depth
    SoundController9,  // default: Vibrato Delay
    SoundController10, // default: Vibrato Delay
    GeneralPurposeController5,
    GeneralPurposeController6,
    GeneralPurposeController7,
    GeneralPurposeController8,
    PortamentoControl,
    Undefined16,
    Undefined17,
    Undefined18,
    HighResolutionVelocityPrefix,
    Undefined19,
    Undefined20,
    Effects1Depth, // default: Reverb Send Level, formerly External Effects Depth
    Effects2Depth, // formerly Tremolo Depth
    Effects3Depth, // default: Chorus Send Level, formerly Chorus Depth
    Effects4Depth, // formerly Celeste [Detune] Depth
    Effects5Depth, // formerly Phaser Depth
    DataIncrement, // Data Entry +1
    DataDecrement, // Data Entry -1
    NonRegisteredParameterNumberLsb,
    NonRegisteredParameterNumberMsb,
    RegisteredParameterNumberLsb,
    RegisteredParameterNumberMsb,
    Undefined21,
    Undefined22,
    Undefined23,
    Undefined24,
    Undefined25,
    Undefined26,
    Undefined27,
    Undefined28,
    Undefined29,
    Undefined30,
    Undefined31,
    Undefined32,
    Undefined33,
    Undefined34,
    Undefined35,
    Undefined36,
    Undefined37,
    Undefined38,
    AllSoundsOff,
    ResetAllControllers,
    LocalControl,
    AllNotesOff,
    OmniModeOff,
    OmniModeOn,
    MonoModeOn,
    PolyModeOn,
}

impl ControlChangeKind {
    pub fn from_number(number: u8) -> Option<Self> {
        match number {
            0 => Some(Self::BankSelectMsb),
            1 => Some(Self::ModulationWheelMsb),
            2 => Some(Self::BreathControllerMsb),
            3 => Some(Self::Undefined0Msb),
            4 => Some(Self::FootControllerMsb),
            5 => Some(Self::PortamentoTimeMsb),
            6 => Some(Self::DataEntryMsb),
            7 => Some(Self::ChannelVolumeMsb),
            8 => Some(Self::BalanceMsb),
            9 => Some(Self::Undefined1Msb),
            10 => Some(Self::PanMsb),
            11 => Some(Self::ExpressionControllerMsb),
            12 => Some(Self::EffectControl1Msb),
            13 => Some(Self::EffectControl2Msb),
            14 => Some(Self::Undefined2Msb),
            15 => Some(Self::Undefined3Msb),
            16 => Some(Self::GeneralPurposeController1Msb),
            17 => Some(Self::GeneralPurposeController2Msb),
            18 => Some(Self::GeneralPurposeController3Msb),
            19 => Some(Self::GeneralPurposeController4Msb),
            20 => Some(Self::Undefined4Msb),
            21 => Some(Self::Undefined5Msb),
            22 => Some(Self::Undefined6Msb),
            23 => Some(Self::Undefined7Msb),
            24 => Some(Self::Undefined8Msb),
            25 => Some(Self::Undefined9Msb),
            26 => Some(Self::Undefined10Msb),
            27 => Some(Self::Undefined11Msb),
            28 => Some(Self::Undefined12Msb),
            29 => Some(Self::Undefined13Msb),
            30 => Some(Self::Undefined14Msb),
            31 => Some(Self::Undefined15Msb),
            32 => Some(Self::BankSelectLsb),
            33 => Some(Self::ModulationWheelLsb),
            34 => Some(Self::BreathControllerLsb),
            35 => Some(Self::Undefined0Lsb),
            36 => Some(Self::FootControllerLsb),
            37 => Some(Self::PortamentoTimeLsb),
            38 => Some(Self::DataEntryLsb),
            39 => Some(Self::ChannelVolumeLsb),
            40 => Some(Self::BalanceLsb),
            41 => Some(Self::Undefined1Lsb),
            42 => Some(Self::PanLsb),
            43 => Some(Self::ExpressionControllerLsb),
            44 => Some(Self::EffectControl1Lsb),
            45 => Some(Self::EffectControl2Lsb),
            46 => Some(Self::Undefined2Lsb),
            47 => Some(Self::Undefined3Lsb),
            48 => Some(Self::GeneralPurposeController1Lsb),
            49 => Some(Self::GeneralPurposeController2Lsb),
            50 => Some(Self::GeneralPurposeController3Lsb),
            51 => Some(Self::GeneralPurposeController4Lsb),
            52 => Some(Self::Undefined4Lsb),
            53 => Some(Self::Undefined5Lsb),
            54 => Some(Self::Undefined6Lsb),
            55 => Some(Self::Undefined7Lsb),
            56 => Some(Self::Undefined8Lsb),
            57 => Some(Self::Undefined9Lsb),
            58 => Some(Self::Undefined10Lsb),
            59 => Some(Self::Undefined11Lsb),
            60 => Some(Self::Undefined12Lsb),
            61 => Some(Self::Undefined13Lsb),
            62 => Some(Self::Undefined14Lsb),
            63 => Some(Self::Undefined15Lsb),
            64 => Some(Self::DamperPedal),
            65 => Some(Self::Portamento),
            66 => Some(Self::Sostenuto),
            67 => Some(Self::SoftPedal),
            68 => Some(Self::LegatoFootswitch),
            69 => Some(Self::Hold2),
            70 => Some(Self::SoundController1),
            71 => Some(Self::SoundController2),
            72 => Some(Self::SoundController3),
            73 => Some(Self::SoundController4),
            74 => Some(Self::SoundController5),
            75 => Some(Self::SoundController6),
            76 => Some(Self::SoundController7),
            77 => Some(Self::SoundController8),
            78 => Some(Self::SoundController9),
            79 => Some(Self::SoundController10),
            80 => Some(Self::GeneralPurposeController5),
            81 => Some(Self::GeneralPurposeController6),
            82 => Some(Self::GeneralPurposeController7),
            83 => Some(Self::GeneralPurposeController8),
            84 => Some(Self::PortamentoControl),
            85 => Some(Self::Undefined16),
            86 => Some(Self::Undefined17),
            87 => Some(Self::Undefined18),
            88 => Some(Self::HighResolutionVelocityPrefix),
            89 => Some(Self::Undefined19),
            90 => Some(Self::Undefined20),
            91 => Some(Self::Effects1Depth),
            92 => Some(Self::Effects2Depth),
            93 => Some(Self::Effects3Depth),
            94 => Some(Self::Effects4Depth),
            95 => Some(Self::Effects5Depth),
            96 => Some(Self::DataIncrement),
            97 => Some(Self::DataDecrement),
            98 => Some(Self::NonRegisteredParameterNumberLsb),
            99 => Some(Self::NonRegisteredParameterNumberMsb),
            100 => Some(Self::RegisteredParameterNumberLsb),
            101 => Some(Self::RegisteredParameterNumberMsb),
            102 => Some(Self::Undefined21),
            103 => Some(Self::Undefined22),
            104 => Some(Self::Undefined23),
            105 => Some(Self::Undefined24),
            106 => Some(Self::Undefined25),
            107 => Some(Self::Undefined26),
            108 => Some(Self::Undefined27),
            109 => Some(Self::Undefined28),
            110 => Some(Self::Undefined29),
            111 => Some(Self::Undefined30),
            112 => Some(Self::Undefined31),
            113 => Some(Self::Undefined32),
            114 => Some(Self::Undefined33),
            115 => Some(Self::Undefined34),
            116 => Some(Self::Undefined35),
            117 => Some(Self::Undefined36),
            118 => Some(Self::Undefined37),
            119 => Some(Self::Undefined38),
            120 => Some(Self::AllSoundsOff),
            121 => Some(Self::ResetAllControllers),
            122 => Some(Self::LocalControl),
            123 => Some(Self::AllNotesOff),
            124 => Some(Self::OmniModeOff),
            125 => Some(Self::OmniModeOn),
            126 => Some(Self::MonoModeOn),
            127 => Some(Self::PolyModeOn),
            _ => None,
        }
    }

    pub fn as_number(&self) -> u8 {
        match self {
            ControlChangeKind::BankSelectMsb => 0,
            ControlChangeKind::ModulationWheelMsb => 1,
            ControlChangeKind::BreathControllerMsb => 2,
            ControlChangeKind::Undefined0Msb => 3,
            ControlChangeKind::FootControllerMsb => 4,
            ControlChangeKind::PortamentoTimeMsb => 5,
            ControlChangeKind::DataEntryMsb => 6,
            ControlChangeKind::ChannelVolumeMsb => 7,
            ControlChangeKind::BalanceMsb => 8,
            ControlChangeKind::Undefined1Msb => 9,
            ControlChangeKind::PanMsb => 10,
            ControlChangeKind::ExpressionControllerMsb => 11,
            ControlChangeKind::EffectControl1Msb => 12,
            ControlChangeKind::EffectControl2Msb => 13,
            ControlChangeKind::Undefined2Msb => 14,
            ControlChangeKind::Undefined3Msb => 15,
            ControlChangeKind::GeneralPurposeController1Msb => 16,
            ControlChangeKind::GeneralPurposeController2Msb => 17,
            ControlChangeKind::GeneralPurposeController3Msb => 18,
            ControlChangeKind::GeneralPurposeController4Msb => 19,
            ControlChangeKind::Undefined4Msb => 20,
            ControlChangeKind::Undefined5Msb => 21,
            ControlChangeKind::Undefined6Msb => 22,
            ControlChangeKind::Undefined7Msb => 23,
            ControlChangeKind::Undefined8Msb => 24,
            ControlChangeKind::Undefined9Msb => 25,
            ControlChangeKind::Undefined10Msb => 26,
            ControlChangeKind::Undefined11Msb => 27,
            ControlChangeKind::Undefined12Msb => 28,
            ControlChangeKind::Undefined13Msb => 29,
            ControlChangeKind::Undefined14Msb => 30,
            ControlChangeKind::Undefined15Msb => 31,
            ControlChangeKind::BankSelectLsb => 32,
            ControlChangeKind::ModulationWheelLsb => 33,
            ControlChangeKind::BreathControllerLsb => 34,
            ControlChangeKind::Undefined0Lsb => 35,
            ControlChangeKind::FootControllerLsb => 36,
            ControlChangeKind::PortamentoTimeLsb => 37,
            ControlChangeKind::DataEntryLsb => 38,
            ControlChangeKind::ChannelVolumeLsb => 39,
            ControlChangeKind::BalanceLsb => 40,
            ControlChangeKind::Undefined1Lsb => 41,
            ControlChangeKind::PanLsb => 42,
            ControlChangeKind::ExpressionControllerLsb => 43,
            ControlChangeKind::EffectControl1Lsb => 44,
            ControlChangeKind::EffectControl2Lsb => 45,
            ControlChangeKind::Undefined2Lsb => 46,
            ControlChangeKind::Undefined3Lsb => 47,
            ControlChangeKind::GeneralPurposeController1Lsb => 48,
            ControlChangeKind::GeneralPurposeController2Lsb => 49,
            ControlChangeKind::GeneralPurposeController3Lsb => 50,
            ControlChangeKind::GeneralPurposeController4Lsb => 51,
            ControlChangeKind::Undefined4Lsb => 52,
            ControlChangeKind::Undefined5Lsb => 53,
            ControlChangeKind::Undefined6Lsb => 54,
            ControlChangeKind::Undefined7Lsb => 55,
            ControlChangeKind::Undefined8Lsb => 56,
            ControlChangeKind::Undefined9Lsb => 57,
            ControlChangeKind::Undefined10Lsb => 58,
            ControlChangeKind::Undefined11Lsb => 59,
            ControlChangeKind::Undefined12Lsb => 60,
            ControlChangeKind::Undefined13Lsb => 61,
            ControlChangeKind::Undefined14Lsb => 62,
            ControlChangeKind::Undefined15Lsb => 63,
            ControlChangeKind::DamperPedal => 64,
            ControlChangeKind::Portamento => 65,
            ControlChangeKind::Sostenuto => 66,
            ControlChangeKind::SoftPedal => 67,
            ControlChangeKind::LegatoFootswitch => 68,
            ControlChangeKind::Hold2 => 69,
            ControlChangeKind::SoundController1 => 70,
            ControlChangeKind::SoundController2 => 71,
            ControlChangeKind::SoundController3 => 72,
            ControlChangeKind::SoundController4 => 73,
            ControlChangeKind::SoundController5 => 74,
            ControlChangeKind::SoundController6 => 75,
            ControlChangeKind::SoundController7 => 76,
            ControlChangeKind::SoundController8 => 77,
            ControlChangeKind::SoundController9 => 78,
            ControlChangeKind::SoundController10 => 79,
            ControlChangeKind::GeneralPurposeController5 => 80,
            ControlChangeKind::GeneralPurposeController6 => 81,
            ControlChangeKind::GeneralPurposeController7 => 82,
            ControlChangeKind::GeneralPurposeController8 => 83,
            ControlChangeKind::PortamentoControl => 84,
            ControlChangeKind::Undefined16 => 85,
            ControlChangeKind::Undefined17 => 86,
            ControlChangeKind::Undefined18 => 87,
            ControlChangeKind::HighResolutionVelocityPrefix => 88,
            ControlChangeKind::Undefined19 => 89,
            ControlChangeKind::Undefined20 => 90,
            ControlChangeKind::Effects1Depth => 91,
            ControlChangeKind::Effects2Depth => 92,
            ControlChangeKind::Effects3Depth => 93,
            ControlChangeKind::Effects4Depth => 94,
            ControlChangeKind::Effects5Depth => 95,
            ControlChangeKind::DataIncrement => 96,
            ControlChangeKind::DataDecrement => 97,
            ControlChangeKind::NonRegisteredParameterNumberLsb => 98,
            ControlChangeKind::NonRegisteredParameterNumberMsb => 99,
            ControlChangeKind::RegisteredParameterNumberLsb => 100,
            ControlChangeKind::RegisteredParameterNumberMsb => 101,
            ControlChangeKind::Undefined21 => 102,
            ControlChangeKind::Undefined22 => 103,
            ControlChangeKind::Undefined23 => 104,
            ControlChangeKind::Undefined24 => 105,
            ControlChangeKind::Undefined25 => 106,
            ControlChangeKind::Undefined26 => 107,
            ControlChangeKind::Undefined27 => 108,
            ControlChangeKind::Undefined28 => 109,
            ControlChangeKind::Undefined29 => 110,
            ControlChangeKind::Undefined30 => 111,
            ControlChangeKind::Undefined31 => 112,
            ControlChangeKind::Undefined32 => 113,
            ControlChangeKind::Undefined33 => 114,
            ControlChangeKind::Undefined34 => 115,
            ControlChangeKind::Undefined35 => 116,
            ControlChangeKind::Undefined36 => 117,
            ControlChangeKind::Undefined37 => 118,
            ControlChangeKind::Undefined38 => 119,
            ControlChangeKind::AllSoundsOff => 120,
            ControlChangeKind::ResetAllControllers => 121,
            ControlChangeKind::LocalControl => 122,
            ControlChangeKind::AllNotesOff => 123,
            ControlChangeKind::OmniModeOff => 124,
            ControlChangeKind::OmniModeOn => 125,
            ControlChangeKind::MonoModeOn => 126,
            ControlChangeKind::PolyModeOn => 127,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct Message {
    pub kind: MessageKind,
    pub channel: u8,
}

impl Message {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            None
        } else {
            decode_non_empty_message(bytes)
        }
    }

    pub fn get_pitch_wheel_signed(value: u16) -> i16 {
        (value as i16) - 8192
    }

    pub fn get_pitch_wheel_freq_coef(value: u16, delta_semitones: Option<f32>) -> f32 {
        let ds = delta_semitones.unwrap_or(2.0);
        let wheel = (Self::get_pitch_wheel_signed(value) as f32) / 8192.0;
        2f32.powf(wheel * ds / 12.0)
    }

    pub fn get_note_frequency(note: u8) -> f32 {
        440.0 * 2f32.powf(((note as i8) - 69) as f32 / 12.0)
    }
}

fn decode_non_empty_message(bytes: &[u8]) -> Option<Message> {
    let cmd = bytes[0] & 0xF0;
    let channel = bytes[0] & 0x0F;
    let kind = match cmd {
        0x80 => parse_note_off(bytes)?,
        0x90 => parse_note_on(bytes)?,
        0xA0 => parse_polyphonic_aftertouch(bytes)?,
        0xB0 => parse_control_change(bytes)?,
        0xC0 => parse_program_change(bytes)?,
        0xD0 => parse_channel_aftertouch(bytes)?,
        0xE0 => parse_pitch_wheel(bytes)?,
        _ => None?,
    };
    Some(Message { kind, channel })
}

fn parse_note_on(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 3 {
        None
    } else {
        let velocity = bytes[2];
        if velocity == 0 {
            Some(MessageKind::NoteOff {
                note: bytes[1],
                velocity,
            })
        } else {
            Some(MessageKind::NoteOn {
                note: bytes[1],
                velocity,
            })
        }
    }
}

fn parse_note_off(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 3 {
        None
    } else {
        Some(MessageKind::NoteOff {
            note: bytes[1],
            velocity: bytes[2],
        })
    }
}

fn parse_polyphonic_aftertouch(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 3 {
        None
    } else {
        Some(MessageKind::PolyphonicAftertouch {
            note: bytes[1],
            pressure: bytes[2],
        })
    }
}

fn parse_control_change(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 3 {
        None
    } else {
        Some(MessageKind::ControlChange {
            kind: ControlChangeKind::from_number(bytes[1])?,
            value: bytes[2],
        })
    }
}

fn parse_program_change(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 2 {
        None
    } else {
        Some(MessageKind::ProgramChange { program: bytes[1] })
    }
}

fn parse_channel_aftertouch(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 2 {
        None
    } else {
        Some(MessageKind::ChannelAftertouch { pressure: bytes[1] })
    }
}

fn parse_pitch_wheel(bytes: &[u8]) -> Option<MessageKind> {
    if bytes.len() < 3 {
        None
    } else {
        let value = ((bytes[1] as u16) & 0x7F) | (((bytes[2] as u16) & 0x7F) << 7);
        Some(MessageKind::PitchWheel { value })
    }
}
