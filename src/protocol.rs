use std::io::Write;

use byteorder::{WriteBytesExt, BigEndian};

use tokio::{net::TcpStream, io::AsyncReadExt};

mod prelude {
    pub use std::io::Cursor;
    pub use std::slice::Iter;
    pub use crate::types::*;
    pub use super::{ReadPacket, WritePacket, PacketField};
}

use prelude::*;

pub trait PacketField {
    type Repr;

    fn size(value: &Self::Repr) -> usize;

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>);
    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr>;
}

impl PacketField for Var<i32> {
    type Repr = i32;
    
    fn size(value: &Self::Repr) -> usize {
        let value = *value as u32;
        for i in 1..5 {
            if (value & (0xffffffffu32 << (7 * i))) == 0 {
                return i;
            }
        }
        5
    }

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
        use crate::types::consts::{VAR_CONTINUE_BIT, VAR_DATA_BITS};

        let mut value = value.clone() as u32;

        loop {
            if (value & (!VAR_DATA_BITS as u32)) == 0 {
                stream.write_u8(value as u8).unwrap();
                break;
            }

            stream.write_u8(((value & (VAR_DATA_BITS as u32)) | (VAR_CONTINUE_BIT as u32)) as u8).unwrap();

            value >>= 7;
        }
    }

    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
        use crate::types::consts::{VAR_CONTINUE_BIT, VAR_DATA_BITS};

        let mut output: i32 = 0;
        let mut offset = 0;

        loop {
            let byte = stream.next()?.clone() as i32;

            output |= (byte & VAR_DATA_BITS) << offset;

            if (byte & VAR_CONTINUE_BIT) == 0 { break; }

            offset += 7;

            // TODO: handle too big size
        }

        Some(output)
    }
}

impl PacketField for bool {
    type Repr = bool;

    fn size(_: &Self::Repr) -> usize { 1 }

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
        stream.write_u8(value as u8).unwrap();
    }

    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
        Some(*stream.next()? != 0)
    }
}

impl<T: PacketField> PacketField for Option<T> {
    type Repr = Option<T::Repr>;

    fn size(value: &Self::Repr) -> usize {
        match value {
            Some(value) => 1 + T::size(value),
            None => 1
        }
    }

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
        bool::encode(value.is_some(), stream);
        
        if let Some(value) = value {
            T::encode(value, stream);
        }
    }

    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
        let is_some = bool::decode(stream)?;

        Some(match is_some {
            true => T::decode(stream),
            false => None
        })
    }
}

impl<T: PacketField> PacketField for Vec<T> {
    type Repr = Vec<T::Repr>;

    fn size(value: &Self::Repr) -> usize {
        value.iter().fold(Var::<i32>::size(&(value.len() as i32)),
            |sum, item| sum + T::size(item)
        )
    }

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
        Var::<i32>::encode(value.len() as i32, stream);

        for item in value {
            T::encode(item, stream);
        }
    }

    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
        let length = Var::<i32>::decode(stream)? as usize;
        let mut output = Vec::with_capacity(length);

        for _ in 0..length {
            output.push(T::decode(stream)?);
        }

        Some(output)
    }
}

impl PacketField for String {
    type Repr = String;

    fn size(value: &Self::Repr) -> usize {
        let length = value.len();

        Var::<i32>::size(&(length as i32)) + length
    }

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
        let bytes = value.as_bytes();

        let length = bytes.len();

        Var::<i32>::encode(length as i32, stream);
        stream.write_all(bytes).unwrap();
    }

    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
        let length = Var::<i32>::decode(stream)? as usize;

        let bytes: Vec<u8> = stream.take(length).copied().collect();

        Some(String::from_utf8(bytes).ok()?)
    }
}

impl PacketField for Identifier {
    type Repr = Identifier;

    fn size(value: &Self::Repr) -> usize {
        String::size(&value.to_string())
    }

    fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
        String::encode(value.to_string(), stream)
    }

    fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
        // FIXME: Validate this
        let string = String::decode(stream)?;
        let mut parts = string.split(":");

        Some(Self::Repr {
            namespace: String::from(parts.next()?),
            value: String::from(parts.next()?)
        })
    }
}

pub trait ReadPacket: Sized {
    fn name(&self) -> &'static str;

    fn read(buffer: Vec<u8>) -> Option<Self>;
}

pub trait WritePacket {
    fn name(&self) -> &'static str;

    fn write(self) -> Vec<u8>;
}

macro_rules! impl_packetfield_for_numeric {
    ($type: ty, $repr: ty, 1, $write_fn: ident) => {
        impl PacketField for $type {
            type Repr = $repr;
        
            fn size(_: &Self::Repr) -> usize { 1 }
        
            fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
                stream.$write_fn(value).unwrap();
            }
        
            fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
                Some(stream.next().copied()? as $repr)
            }
        }
    };
    ($type: ty, $repr: ty, $size: literal, $write_fn: ident) => {
        impl PacketField for $type {
            type Repr = $repr;
        
            fn size(_: &Self::Repr) -> usize { $size }
        
            fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
                stream.$write_fn::<BigEndian>(value).unwrap();
            }
        
            fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
                Some(stream
                    .take($size)
                    .enumerate()
                    .fold(0, |sum, (i, byte)| sum + ((*byte as $repr) << (i * 8)))
                    as $repr
                )
            }
        }
    };
}

macro_rules! packetfield_struct {
    ($name: ident { $($member: ident : $member_type: ty),* }) => {
        use crate::protocol::prelude::*;

        #[derive(Debug)]
        pub struct $name {
            $(
                pub $member: <$member_type as PacketField>::Repr
            ),*
        }

        impl PacketField for $name {
            type Repr = $name;

            fn size(value: &Self::Repr) -> usize {
                0 $(+ <$member_type as PacketField>::size(&value.$member))*
            }
        
            fn encode(value: Self::Repr, stream: &mut Cursor<&mut Vec<u8>>) {
                $(
                    <$member_type as PacketField>::encode(value.$member, stream);
                )*
            }
        
            fn decode(stream: &mut Iter<u8>) -> Option<Self::Repr> {
                Some(Self::Repr {$(
                    $member: <$member_type as PacketField>::decode(stream)?
                ),*})
            }
        }
    }
}

macro_rules! packets {
    ($($id: literal => $name: ident { $($member: ident : $member_type: ty),* })*) => {
        #[allow(unused_imports)]
        use crate::protocol::prelude::*;

        $(
            #[derive(Debug)]
            pub struct $name {
                $(
                    pub $member: <$member_type as PacketField>::Repr
                ),*
            }

            impl WritePacket for $name {
                fn name(&self) -> &'static str {
                    stringify!($name)
                }

                fn write(self) -> Vec<u8> {
                    let size = Var::<i32>::size(&$id) $(+ <$member_type as PacketField>::size(&self.$member))*;
                    let mut buffer = vec![0; size];
                    let mut stream = Cursor::new(&mut buffer);

                    Var::<i32>::encode(size as i32, &mut stream);
                    Var::<i32>::encode($id, &mut stream);

                    $(
                        <$member_type as PacketField>::encode(self.$member, &mut stream);
                    )*

                    buffer
                }
            }
        )*

        #[derive(Debug)]
        pub enum Packet {
            $(
                $name ($name)
            ),*
        }

        impl ReadPacket for Packet {
            fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$name(packet) => packet.name(),
                    )*
                }
            }

            fn read(buffer: Vec<u8>) -> Option<Self> {
                let mut iterator = buffer.iter();

                let packet_id = Var::<i32>::decode(&mut iterator)?;

                match packet_id {
                    $(
                        $id => Some(
                            Self::$name($name {
                                $(
                                    $member: <$member_type as PacketField>::decode(&mut iterator)?
                                ),*
                            })
                        ),
                    )*
                    _ => None
                }
            }
        }
    };
}

impl_packetfield_for_numeric!(u8,   u8,   1,  write_u8);
impl_packetfield_for_numeric!(i8,   i8,   1,  write_i8);
impl_packetfield_for_numeric!(u16,  u16,  2,  write_u16);
impl_packetfield_for_numeric!(i32,  i32,  4,  write_i32);
impl_packetfield_for_numeric!(i64,  i64,  8,  write_i64);
impl_packetfield_for_numeric!(UUID, u128, 16, write_u128);

pub mod packets {
    pub mod handshaking {
        pub mod serverbound {packets!(
            0x00 => Handshake { protocol_version: Var<i32>, server_address: String, server_port: u16, next_state: Var<i32> }
        );}
    }

    pub mod status {
        pub mod serverbound {packets!(
            0x00 => StatusRequest { }
            0x01 => PingRequest { payload: i64 }
        );}

        pub mod clientbound {packets!(
            0x00 => StatusResponse { response: String }
            0x01 => PingResponse { payload: i64 }
        );}
    }

    pub mod login {
        pub mod serverbound {packets!(
            0x00 => LoginStart { name: String, uuid: Option<UUID> }
        );}

        pub mod clientbound {
            packetfield_struct!(LoginSuccessProperty {
                name: String, value: String, signature: Option<String>
            });
            packets!(
                0x02 => LoginSuccess { uuid: UUID, username: String, properties: Vec<LoginSuccessProperty> }
            );
        }
    }

    pub mod play {
        pub mod clientbound {
            // TODO: Implement
            /*packetfield_struct!(LoginDeathLocation {
                dimension_name: Identifier, location: Position
            })*/
            packets!(
                0x28 => Login {
                    entity_id: i32, is_hardcore: bool, gamemode: u8, previous_gamemode: i8, dimensions: Vec<Identifier>, dimension_type: Identifier, dimension_name: Identifier, hashed_seed: i64, max_players: Var<i32>, view_distance: Var<i32>, simulation_distance: Var<i32>, reduced_debug_info: bool, enable_respawn_screen: bool, is_debug: bool, is_flat: bool, death_location: Option<bool>
                }
            );
        }
    }
}

async fn get_packet_length(stream: &mut TcpStream) -> usize {
    use crate::types::consts::{VAR_CONTINUE_BIT, VAR_DATA_BITS};

    let mut output: usize = 0;
    let mut offset = 0;

    loop {
        let byte = stream.read_u8().await.unwrap().clone() as i32;

        output |= ((byte & VAR_DATA_BITS) as usize) << offset;

        if (byte & VAR_CONTINUE_BIT) == 0 { break; }

        offset += 7;

        // TODO: handle too big size
    }

    output
}

pub async fn receive_packet(stream: &mut TcpStream) -> Vec<u8> {
    let packet_length = get_packet_length(stream).await;
    let mut buffer = vec![0; packet_length];

    stream.read_exact(&mut buffer).await.unwrap();

    buffer
}