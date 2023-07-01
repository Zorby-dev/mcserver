use std::io::{Read, Write};

use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};

use crate::types::{VarI32, UUID, Nbt, Identifier, Pos};

use super::{Decode, DecodeResult, DecodeError, Encode};

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

impl Decode for i64 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_i64::<BigEndian>().unwrap())
    }
}

impl Decode for VarI32 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> {
        let mut output: [u8; VarI32::MAX_LEN] = [0; VarI32::MAX_LEN];
        
        for i in 0..output.len() {
            let byte = reader.read_u8()
                .map_err(|_| DecodeError::UnexpectedEOF)?;
            output[i] = byte;

            if byte & 0x80 == 0 {
                return Ok(Self (output, i as u8 + 1));
            }
        }

        unreachable!()
    }
}

impl Encode for VarI32 {
    fn encode(&self, mut writer: impl Write) -> usize {
        for byte in self.bytes() {
            writer.write_u8(*byte).unwrap();

            if byte & 0x80 == 0 {
                break;
            }
        }

        self.len()
    }
}

impl Encode for String {
    fn encode(&self, mut writer: impl Write) -> usize {
        let size = VarI32::from(self.len() as i32).encode(&mut writer)
            + self.len();
        
        writer.write_all(self.as_bytes()).unwrap();

        size
    }
}

impl Encode for i64 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_i64::<BigEndian>(*self).unwrap();
        8
    }
}

impl Encode for u16 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_u16::<BigEndian>(*self).unwrap();
        2
    }
}

impl Decode for bool {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_u8().unwrap() != 0)
    }
}

impl Encode for bool {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_u8(*self as u8).unwrap();
        1
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(match bool::decode(reader)? {
            true => Some(T::decode(reader)?),
            false => None
        })
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, mut writer: impl Write) -> usize {
        match self {
            Some(inner) => {
                true.encode(&mut writer) + inner.encode(writer)
            },
            None => {
                false.encode(writer)
            }
        }
    }
}

impl Decode for u128 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_u128::<BigEndian>().unwrap())
    }
}

impl Encode for u128 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_u128::<BigEndian>(*self).unwrap();
        16
    }
}

impl Decode for i32 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_i32::<BigEndian>().unwrap())
    }
}

impl Encode for i32 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_i32::<BigEndian>(*self).unwrap();
        4
    }
}

impl Decode for f32 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_f32::<BigEndian>().unwrap())
    }
}

impl Encode for f32 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_f32::<BigEndian>(*self).unwrap();
        4
    }
}

impl Decode for i8 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_i8().unwrap())
    }
}

impl Encode for i8 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_i8(*self).unwrap();
        1
    }
}

impl Decode for u8 {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(reader.read_u8().unwrap())
    }
}

impl Encode for u8 {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_u8(*self).unwrap();
        1
    }
}

impl Decode for UUID {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(Self(u128::decode(reader)?))
    }
}

impl Encode for UUID {
    fn encode(&self, writer: impl Write) -> usize {
        self.0.encode(writer)
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, mut writer: impl Write) -> usize {
        let mut size = VarI32::from(self.len() as i32).encode(&mut writer);
        
        for item in self {
            size += item.encode(&mut writer);
        }

        size
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        let len = i32::from(VarI32::decode(reader)?) as usize;
        let mut out = Vec::with_capacity(len);
        
        for _ in 0..len {
            out.push(T::decode(reader)?)
        }

        Ok(out)
    }
}

impl Encode for Nbt {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_all(&self.0).unwrap();
        self.0.len()
    }
}

impl Decode for Nbt {
    fn decode(_reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        unimplemented!()
    }
}

impl Encode for Identifier {
    fn encode(&self, writer: impl Write) -> usize {
        String::from(self).encode(writer)
    }
}

impl Decode for Identifier {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        Ok(Self::from(String::decode(reader)?.as_str()))
    }
}

impl Encode for Pos {
    fn encode(&self, mut writer: impl Write) -> usize {
        writer.write_i64::<BigEndian>(
            ((self.x as i64 & 0x3FFFFFF) << 38) |
            ((self.z as i64 & 0x3FFFFFF) << 12) |
            ( self.y as i64 & 0xFFF)
        ).unwrap();
        8
    }
}

impl Decode for Pos {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized {
        let data = i64::decode(reader)?;

        Ok(Self {
            x: (data >> 38) as i32,
            y: (data << 52 >> 52) as i16,
            z: (data << 26 >> 38) as i32
        })
    }
}