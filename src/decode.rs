use std::io::Read;

use byteorder::{ReadBytesExt, BigEndian};

use crate::{types::VarI32, packets::{HandshakingServerboundPacket, HandshakeData, StatusServerboundPacket, StatusRequestData}};

pub trait Decode {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized;
}

#[derive(Debug)]
pub enum DecodeError {
    UnexpectedEOF
}

pub type DecodeResult<T> = Result<T, DecodeError>;

impl Decode for String {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        let len = i32::from(VarI32::decode(reader)?) as usize;
        let mut buffer = vec![0; len];
        reader.read_exact(&mut buffer)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        Ok(String::from_utf8(buffer).unwrap())
    }
}

impl Decode for u16 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        reader.read_u16::<BigEndian>()
            .map_err(|_| DecodeError::UnexpectedEOF)
    }
}

impl Decode for HandshakingServerboundPacket {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> {
        let id: i32 = VarI32::decode(reader)?.into();

        dbg!(id);

        Ok(match id {
            0x00 => Self::Handshake(HandshakeData {
                protocol_version: VarI32::decode(reader)?,
                server_address: String::decode(reader)?,
                server_port: u16::decode(reader)?,
                next_state: VarI32::decode(reader)?
            }),
            _ => unreachable!()
        })
    }
}

impl Decode for StatusServerboundPacket {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> {
        let id: i32 = VarI32::decode(reader)?.into();

        dbg!(id);

        Ok(match id {
            0x00 => Self::StatusRequest(StatusRequestData { }),
            _ => unreachable!()
        })
    }
}

impl Decode for VarI32 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> {
        let mut output: [u8; 5] = [0; 5];
        
        for i in 0..5 {
            let byte = reader.read_u8()
                .map_err(|_| DecodeError::UnexpectedEOF)?;
            output[i] = byte;

            if byte & 0x80 == 0 {
                break;
            }
        }

        Ok(Self (output))
    }
}