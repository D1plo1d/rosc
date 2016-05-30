use types::{Result, OscType, OscPacket, OscBundle, OscMessage};
use errors::OscError;

use byteorder::{ByteOrder, BigEndian};

/// Takes a reference to an OSC packet and returns
/// a byte vector on success. If the packet was invalid
/// an `OscError` is returned.
///
/// # Example
///
/// ```
/// use rosc::{OscPacket,OscMessage,OscType};
/// use rosc::encoder;
///
/// let packet = OscPacket::Message(OscMessage{
///         addr: "/greet/me".to_string(),
///         args: Some(vec![OscType::String("hi!".to_string())])
///     }
/// );
/// assert!(encoder::encode(&packet).is_ok())
/// ```
pub fn encode(packet: &OscPacket) -> Result<Vec<u8>> {
    match *packet {
        OscPacket::Message(ref msg) => encode_message(msg),
        OscPacket::Bundle(ref bundle) => encode_bundle(bundle),
    }
}

fn encode_message(msg: &OscMessage) -> Result<Vec<u8>> {
    let mut msg_bytes: Vec<u8> = Vec::new();

    msg_bytes.extend(encode_string(msg.addr.clone()));
    let mut type_tags: Vec<char> = vec![','];
    let mut arg_bytes: Vec<u8> = Vec::new();

    if let Some(ref args) = msg.args {
        for arg in args {
            let (bytes, tag): (Option<Vec<u8>>, char) = try!(encode_arg(arg));

            type_tags.push(tag);
            if bytes.is_some() {
                arg_bytes.extend(bytes.unwrap());
            }
        }
    }

    msg_bytes.extend(encode_string(type_tags.into_iter()
        .collect::<String>()));
    if !arg_bytes.is_empty() {
        msg_bytes.extend(arg_bytes);
    }
    Ok(msg_bytes)
}

fn encode_bundle(bundle: &OscBundle) -> Result<Vec<u8>> {
    let mut bundle_bytes: Vec<u8> = Vec::new();
    bundle_bytes.extend(encode_string("#bundle".to_string()).into_iter());

    match try!(encode_arg(&bundle.timetag)) {
        (Some(x), _) => {
            bundle_bytes.extend(x.into_iter());
        }
        (None, _) => {
            return Err(OscError::BadBundle("Missing time tag!".to_string()));
        }
    }

    if bundle.content.is_empty() {
        // TODO: A bundle of length zero, should this really be supported?
        bundle_bytes.extend([0u8; 4].into_iter());
        return Ok(bundle_bytes);
    }

    for packet in &bundle.content {
        match *packet {
            OscPacket::Message(ref m) => {
                let msg = try!(encode_message(m));
                let mut msg_size = vec![0u8; 4];
                BigEndian::write_u32(&mut msg_size, msg.len() as u32);
                bundle_bytes.extend(msg_size.into_iter().chain(msg.into_iter()));
            }
            OscPacket::Bundle(ref b) => {
                let bdl = try!(encode_bundle(b));
                let mut bdl_size = vec![0u8; 4];
                BigEndian::write_u32(&mut bdl_size, bdl.len() as u32);
                bundle_bytes.extend(bdl_size.into_iter().chain(bdl.into_iter()));
            }
        }
    }

    Ok(bundle_bytes)
}

fn encode_arg(arg: &OscType) -> Result<(Option<Vec<u8>>, char)> {
    match *arg {
        OscType::Int(ref x) => {
            let mut bytes = vec![0u8; 4];
            BigEndian::write_i32(&mut bytes, *x);
            Ok((Some(bytes), 'i'))
        }
        OscType::Long(ref x) => {
            let mut bytes = vec![0u8; 8];
            BigEndian::write_i64(&mut bytes, *x);
            Ok((Some(bytes), 'h'))
        }
        OscType::Float(ref x) => {
            let mut bytes = vec![0u8; 4];
            BigEndian::write_f32(&mut bytes, *x);
            Ok((Some(bytes), 'f'))
        }
        OscType::Double(ref x) => {
            let mut bytes = vec![0u8; 8];
            BigEndian::write_f64(&mut bytes, *x);
            Ok((Some(bytes), 'd'))
        }
        OscType::Char(ref x) => {
            let mut bytes = vec![0u8; 4];
            BigEndian::write_u32(&mut bytes, *x as u32);
            Ok((Some(bytes), 'c'))
        }
        OscType::String(ref x) => Ok((Some(encode_string(x.clone())), 's')),
        OscType::Blob(ref x) => {
            let padded_blob_length: usize = pad(x.len() as u64) as usize;
            let mut bytes = vec![0u8; 4 + padded_blob_length];
            // write length
            BigEndian::write_i32(&mut bytes[..4], x.len() as i32);
            for (i, v) in x.iter().enumerate() {
                bytes[i + 4] = *v;
            }
            Ok((Some(bytes), 'b'))
        }
        OscType::Time(ref x, ref y) => Ok((Some(encode_time_tag(*x, *y)), 't')),
        OscType::Midi(ref x) => Ok((Some(vec![x.port, x.status, x.data1, x.data2]), 'm')),
        OscType::Color(ref x) => Ok((Some(vec![x.red, x.green, x.blue, x.alpha]), 'r')),
        OscType::Bool(ref x) => {
            if *x {
                Ok((None, 'T'))
            } else {
                Ok((None, 'F'))
            }
        }
        OscType::Nil => Ok((None, 'N')),
        OscType::Inf => Ok((None, 'I')),
    }
}

/// Null terminates the byte representation of string `s` and
/// adds null bytes until the length of the result is a
/// multiple of 4.
pub fn encode_string<S: Into<String>>(s: S) -> Vec<u8> {
    let mut bytes: Vec<u8> = s.into().as_bytes().into();
    bytes.push(0u8);
    pad_bytes(&mut bytes);
    bytes
}

fn pad_bytes(bytes: &mut Vec<u8>) {
    let padded_lengh = pad(bytes.len() as u64);
    while bytes.len() < padded_lengh as usize {
        bytes.push(0u8)
    }
}

/// Returns the position padded to 4 bytes.
///
/// # Example
///
/// ```
/// use rosc::encoder;
///
/// let pos: u64 = 10;
/// assert_eq!(12u64, encoder::pad(pos))
/// ```
pub fn pad(pos: u64) -> u64 {
    match pos % 4 {
        0 => pos,
        d => pos + (4 - d),
    }
}


fn encode_time_tag(sec: u32, frac: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 8];
    BigEndian::write_u32(&mut bytes[..4], sec);
    BigEndian::write_u32(&mut bytes[4..], frac);
    bytes
}

#[test]
fn test_pad() {
    assert_eq!(4, pad(4));
    assert_eq!(8, pad(5));
    assert_eq!(8, pad(6));
    assert_eq!(8, pad(7));
}
