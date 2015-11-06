use types::{Result, OscType, OscPacket, OscBundle, OscMessage};
use errors::OscError;
use utils;

use byteorder::{ByteOrder, BigEndian};
use std::{iter, str};

pub fn encode_packet(packet: OscPacket) -> Result<Vec<u8>> {
    match packet {
        OscPacket::Message(msg) => encode_message(msg),
        OscPacket::Bundle(bundle) => encode_bundle(bundle),
    }
}

fn encode_message(msg: OscMessage) -> Result<Vec<u8>> {
    let mut msg_bytes: Vec<u8> = Vec::new();

    msg_bytes.extend(encode_string(msg.addr));
    let mut type_tags: Vec<char> = vec![','];
    let mut arg_bytes: Vec<u8> = Vec::new();

    if msg.args.is_some() {
        let args: Vec<OscType> = msg.args.unwrap();
        // Possible optimization: write this as iterator
        for arg in args {
            let (bytes, tag): (Option<Vec<u8>>, char) = try!(encode(arg));

            type_tags.push(tag);
            if bytes.is_some() {
                arg_bytes.extend(bytes.unwrap());
            }
        }
    }

    msg_bytes.extend(encode_string(type_tags.iter()
                                            .cloned()
                                            .collect()));
    if arg_bytes.len() > 0 {
        msg_bytes.extend(arg_bytes);
    }
    Ok(msg_bytes)
}

fn encode_bundle(bundle: OscBundle) -> Result<Vec<u8>> {
    Err(OscError::Unimplemented)
}

fn encode(arg: OscType) -> Result<(Option<Vec<u8>>, char)> {
    match arg {
        OscType::Int(x) => {
            let mut bytes = Vec::with_capacity(4);
            BigEndian::write_i32(&mut bytes, x);
            Ok((Some(bytes), 'i'))
        }
        OscType::Long(x) => {
            let mut bytes = Vec::with_capacity(8);
            BigEndian::write_i64(&mut bytes, x);
            Ok((Some(bytes), 'h'))
        }
        OscType::Float(x) => {
            let mut bytes = Vec::with_capacity(4);
            BigEndian::write_f32(&mut bytes, x);
            Ok((Some(bytes), 'f'))
        }
        OscType::Double(x) => {
            let mut bytes = Vec::with_capacity(8);
            BigEndian::write_f64(&mut bytes, x);
            Ok((Some(bytes), 'd'))
        }
        OscType::Char(x) => {
            let mut bytes = Vec::with_capacity(4);
            BigEndian::write_u32(&mut bytes, x as u32);
            Ok((Some(bytes), 'c'))
        }
        OscType::String(x) => {
            Ok((Some(encode_string(x)), 's'))
        }
        OscType::Blob(x) => {
            let mut bytes = x.clone();
            pad_bytes(&mut bytes);
            Ok((Some(bytes), 'b'))
        }
        OscType::Time(x, y) => {
            Ok((Some(encode_time_tag(x, y)), 't'))
        }
        OscType::Midi(x) => {
            Ok((Some(vec![x.port, x.status, x.data1, x.data2]), 'm'))
        }
        OscType::Bool(x) => {
            if x {
                Ok((None, 'T'))
            } else {
                Ok((None, 'F'))
            }
        }
        OscType::Nil => {
            Ok((None, 'N'))
        }
        OscType::Inf => {
            Ok((None, 'I'))
        }
        _ => Err(OscError::Unimplemented),
    }
}

fn encode_string(s: String) -> Vec<u8> {
    let mut bytes: Vec<u8> = s.as_bytes()
                              .iter()
                              .cloned()
                              .chain(iter::once(0u8))
                              .collect();
    pad_bytes(&mut bytes);
    bytes
}

fn pad_bytes(bytes: &mut Vec<u8>) {
    let padded_lengh = utils::pad(bytes.len() as u64);
    while bytes.len() < padded_lengh as usize {
        bytes.push(0u8)
    }
}

fn encode_time_tag(sec: u32, frac: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    BigEndian::write_u32(&mut bytes[..3], sec);
    BigEndian::write_u32(&mut bytes[4..], frac);
    bytes
}