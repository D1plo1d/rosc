use errors;
use std::result;

// see OSC Type Tag String: http://opensoundcontrol.org/spec-1_0
// padding: zero bytes (n*4)

#[derive(Debug)]
pub enum OscType {
    Int(i32),
    Float(f32),
    String(String),
    Blob(Vec<u8>),
    // use struct for time tag to avoid destructuring
    Time(u32, u32),
    Long(i64),
    Double(f64),
    Char(char),
    Color(OscColor),
    Midi(OscMidiMessage),
    Bool(bool),
    Nil,
    Inf,
    Array(Vec<OscType>),
}

#[derive(Debug)]
pub struct OscMidiMessage {
    pub port: u8,
    pub status: u8,
    pub data1: u8, // maybe use an enum for data?
    pub data2: u8,
}

/// An *osc packet* can contain an *osc message* or a bundle of nested messages
/// which is called *osc bundle*.
#[derive(Debug)]
pub enum OscPacket {
    Message(OscMessage),
    Bundle(OscBundle),
}

#[derive(Debug)]
pub struct OscMessage {
    pub addr: String,
    pub args: Option<Vec<OscType>>,
}

#[derive(Debug)]
pub struct OscBundle {
    pub timetag: OscType,
    pub content: Vec<OscPacket>,
}

#[derive(Debug)]
pub struct OscColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

pub type Result<T> = result::Result<T, errors::OscError>;
