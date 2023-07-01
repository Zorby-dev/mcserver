use std::io::{Read, Write};

mod r#impl;

#[derive(Debug)]
pub enum DecodeError {
    UnexpectedEOF
}

pub type DecodeResult<T> = Result<T, DecodeError>;

pub trait Decode {
    fn decode(reader: &mut impl Read) -> DecodeResult<Self> where Self: Sized;
}

pub trait Encode {
    fn encode(&self, writer: impl Write) -> usize;
}

pub trait Packet {
    fn name(&self) -> &'static str;
}