use bytes::{BytesMut, Buf, BufMut};
use paris::{log};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::{DecodeResult, DecodeError, Decode, Encode, Packet};
use crate::types::VarI32;

struct Receiver {
    buffer: BytesMut
}

impl Receiver {
    fn new() -> Self {
        Self {
            buffer: BytesMut::new()
        }
    }

    fn poll_receive<T: Decode>(&mut self) -> DecodeResult<T> {
        let size = VarI32::decode(&mut &self.buffer[..])?;
        let size_len = size.len();
        let size = i32::from(size) as usize;

        if size > self.buffer.len() {
            return Err(DecodeError::UnexpectedEOF)
        }

        self.buffer.advance(size_len);

        T::decode(&mut &self.buffer.split_to(size)[..])
    }
}

struct Sender {
    buffer: BytesMut
}

impl Sender {
    fn new() -> Self {
        Self {
            buffer: BytesMut::new()
        }
    }
}

pub struct Interface {
    stream: TcpStream,
    sender: Sender,
    receiver: Receiver
}

impl Interface {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            sender: Sender::new(),
            receiver: Receiver::new()
        }
    }

    pub async fn receive<T: Decode + Packet>(&mut self) -> DecodeResult<T> {
        loop {
            match self.receiver.poll_receive::<T>() {
                Ok(packet) => {
                    log!("  <blue>C</> -> <green>S</> : {}", packet.name());
                    return Ok(packet)
                },
                Err(DecodeError::UnexpectedEOF) => ()
            }

            let read_count = self.stream.read_buf(&mut self.receiver.buffer).await.unwrap();
            if read_count == 0 {
                return Err(DecodeError::UnexpectedEOF);
            }
        }
    }

    pub async fn send<T: Encode + Packet>(&mut self, packet: T) {
        let packet_len = packet.encode((&mut self.sender.buffer).writer());
        let packet_len_var = VarI32::from(packet_len as i32);
        
        self.sender.buffer.put_bytes(0, packet_len_var.len());
        self.sender.buffer.copy_within(0..packet_len, packet_len_var.len());
        packet_len_var.encode(&mut self.sender.buffer[..]);
        
        let out = self.sender.buffer.split();
        self.stream.write_all(&out).await.unwrap();
        log!("  <green>S</> -> <blue>C</> : {}", packet.name());
    }

    pub async fn disconnect(&mut self) {
        self.stream.shutdown().await.unwrap();
        log!("connection aborted or smth");
    }
}