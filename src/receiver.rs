use bytes::{BytesMut, Buf};
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

use crate::decode::{DecodeResult, DecodeError, Decode};
use crate::packets::Packet;
use crate::types::VarI32;

pub struct Receiver {
    buffer: BytesMut,
    stream: TcpStream
}

impl Receiver {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::new()
        }
    }

    pub async fn receive(&mut self) -> DecodeResult<Packet> {
        // TODO: add timeout
        loop {
            let read_count = self.stream.read_buf(&mut self.buffer).await.unwrap();
            if read_count == 0 {
                return Err(DecodeError::UnexpectedEOF);
            }

            match VarI32::decode(&mut &self.buffer[..]) {
                Ok(size) => {
                    let size_len = size.len();
                    let size = i32::from(size) as usize;

                    if size > self.buffer.len() {
                        continue;
                    }

                    self.buffer.advance(size_len);

                    return Packet::decode(&mut &self.buffer.split_to(size)[..]);
                },
                Err(DecodeError::UnexpectedEOF) => continue
            }
        }
    }
}