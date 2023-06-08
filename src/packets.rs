use crate::types::VarI32;

#[derive(Debug)]
pub struct HandshakeData {
    pub protocol_version: VarI32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarI32
}

pub enum HandshakingServerboundPacket {
    Handshake(HandshakeData)
}

#[derive(Debug)]
pub struct StatusRequestData { }

pub enum StatusServerboundPacket {
    StatusRequest(StatusRequestData)
}

#[derive(Debug)]
pub struct StatusResponseData {
    response: String
}

pub enum StatusClientboundPacket {
    StatusResponse(StatusResponseData)
}