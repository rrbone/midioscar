use std::convert::From;
use std::fmt::{self, Display};

use portmidi::MidiMessage;
use rosc::{OscType, OscMessage};

#[derive(Debug)]
pub enum Status {
    NoteOff(u8),
    NoteOn(u8),
    KeyPressure(u8),
    ControllerChange(u8),
    ProgramChange(u8),
    ChannelPressure(u8),
    PitchBend(u8),
    SystemExclusive,
    SongPosition,
    SongSelect,
    TuneRequest,
    EndOfSysex,
    TimingTick,
    StartSong,
    ContinueSong,
    StopSong,
}

#[derive(Debug)]
pub struct Message {
    status: Status,
    data1: u8,
    data2: u8,
}

impl From<MidiMessage> for Message {
    fn from(msg: MidiMessage) -> Message {
        let status = Status::from(msg.status);

        Message {
            status: status,
            data1: msg.data1,
            data2: msg.data2,
        }
    }
}

impl Message {
    pub fn to_osc(&self, device_name: &str) -> OscMessage {
        let path = format!("/midi/{}/{}", device_name.replace(" ", "_"), self.status.channel());

        let args = vec![
            OscType::String(self.status.to_string()),
            OscType::Int(self.data1 as i32),
            OscType::Int(self.data2 as i32),
        ];

        OscMessage {
            addr: path,
            args: Some(args),
        }
    }
}

impl From<u8> for Status {
    fn from(status: u8) -> Status {
        use self::Status::*;

        match status & 0xf0 {
            0x80 => return NoteOff(status & 0xf),
            0x90 => return NoteOn(status & 0xf),
            0xa0 => return KeyPressure(status & 0xf),
            0xb0 => return ControllerChange(status & 0xf),
            0xc0 => return ProgramChange(status & 0xf),
            0xd0 => return ChannelPressure(status & 0xf),
            0xe0 => return PitchBend(status & 0xf),
            _ => {},
        }

        match status {
            0xf0 => SystemExclusive,
            0xf2 => SongPosition,
            0xf3 => SongSelect,
            0xf6 => TuneRequest,
            0xf7 => EndOfSysex,
            0xf8 => TimingTick,
            0xfa => StartSong,
            0xfb => ContinueSong,
            0xfc => StopSong,
            _ => panic!("Invalid status encountered: {:?}", status)
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Status::*;

        let name = match *self {
            NoteOff(_) => "note_off",
            NoteOn(_) => "note_on",
            KeyPressure(_) => "key_pressure",
            ControllerChange(_) => "controller_change",
            ProgramChange(_) => "program_change",
            ChannelPressure(_) => "channel_pressure",
            PitchBend(_) => "pitch_bend",
            SystemExclusive => "system_exclusive",
            SongPosition => "song_position",
            SongSelect => "song_select",
            TuneRequest => "tune_request",
            EndOfSysex => "end_of_sysex",
            TimingTick => "timing_tick",
            StartSong => "start_song",
            ContinueSong => "continue_song",
            StopSong => "stop_song",
        };

        write!(f, "{}", name)
    }
}

impl Status {
    pub fn channel(&self) -> String {
        use self::Status::*;

        let c = match *self {
            NoteOff(c) => c,
            NoteOn(c) => c,
            KeyPressure(c) => c,
            ControllerChange(c) => c,
            ProgramChange(c) => c,
            ChannelPressure(c) => c,
            PitchBend(c) => c,
            _ => return String::from(""),
        };
        c.to_string()
    }
}
