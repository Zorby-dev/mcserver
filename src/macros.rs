macro_rules! packets {
    ($($namespace:ident { $($bound:ident { $($id:literal => $packet:ident { $($field:ident : $type:ty),* }),* })* })*) => {
        $(
            pub mod $namespace {$(
                pub mod $bound {
                    use paste::paste;
                    use std::io::{Read, Write};
                    use crate::{protocol::{self, Decode, DecodeResult, Encode}, types::*};
                    paste!{
                        $(
                            #[derive(Debug)]
                            pub struct [<$packet Data>] {$(
                                pub $field: $type
                            ),*}
                        )*

                        #[derive(Debug)]
                        pub enum Packet {$(
                            $packet([<$packet Data>]),
                        )*}

                        impl Decode for Packet {
                            fn decode(reader: &mut impl Read) -> DecodeResult<Self> {
                                let id: i32 = VarI32::decode(reader)?.into();
                                
                                Ok(match id {
                                    $(
                                        $id => Self::$packet([<$packet Data>] {$(
                                            $field: $type::decode(reader)?
                                        ),*}),
                                    )*
                                    _ => unreachable!()
                                })
                            }
                        }

                        impl Encode for Packet {
                            fn encode(&self, mut writer: impl Write) -> usize {
                                match self {$(
                                    Self::$packet(_data) => {
                                        VarI32::from($id).encode(&mut writer) $(+ _data.$field.encode(&mut writer))*
                                    },
                                )*}
                            }
                        }

                        impl protocol::Packet for Packet {
                            fn name(&self) -> &'static str {
                                match &self {$(
                                    Self::$packet(_) => stringify!($packet),
                                )*}
                            }
                        }
                    }
                }
            )*}
        )*
    };
}

pub(crate) use packets;