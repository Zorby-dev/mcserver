use std::error::Error;
use std::net::SocketAddr;

use paris::{log, info};
use tokio::net::TcpStream;
use tokio::{io::AsyncWriteExt, net::TcpListener};

use mcserver::protocol::{ReadPacket, WritePacket, receive_packet, packets};

enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play
}

fn announce_unknown_packet() {
    log!("  <bright-green>C</> -> <bright-cyan>S</> : <red>unknown packet");
}

fn announce_packet(packet: &impl ReadPacket) {
    log!("  <bright-green>C</> -> <bright-cyan>S</> : {}", packet.name());
}

async fn send_packet(stream: &mut TcpStream, packet: impl WritePacket) {
    log!("  <bright-cyan>S</> -> <bright-green>C</> : {}", packet.name());

    let buffer = packet.write();

    //println!("{:?}", buffer);

    stream.write_all(&buffer).await.unwrap();
}

async fn handle_connection(mut stream: TcpStream, client_address: SocketAddr) {
    let mut state = ConnectionState::Handshaking;

    loop {
        // FIXME: hack
        let mut hack = [0; 1];
        if let Ok(0) = stream.peek(&mut hack).await {
            info!("connection from <green>`{}`</> aborted", client_address);
            return
        }

        let buffer = receive_packet(&mut stream).await;

        match state {
            ConnectionState::Handshaking => {
                use packets::handshaking::serverbound::Packet;

                let packet = Packet::read(buffer);

                match packet {
                    Some(packet) => {
                        announce_packet(&packet);

                        match packet {
                            Packet::Handshake(request) => match request.next_state {
                                1 => state = ConnectionState::Status,
                                2 => state = ConnectionState::Login,
                                _ => unreachable!("invalid next state")
                            }
                        }
                    },
                    None => {
                        announce_unknown_packet();
                    }
                }
            },
            ConnectionState::Status => {
                use packets::status::{serverbound, clientbound};

                let packet = serverbound::Packet::read(buffer);

                match packet {
                    Some(packet) => {
                        announce_packet(&packet);

                        match packet {
                            serverbound::Packet::StatusRequest(_) => {
                                let response = clientbound::StatusResponse {
                                    response: format!("{{\"version\": {{\"name\": \"---------                                                   \",\"protocol\": -1}},\"players\": {{\"max\": -1,\"online\": 1,\"sample\": [{{\"name\": \"                                                                                                 \",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}}]}},\"description\": {{\"text\": \"{}\"}}}}", client_address)
                                };

                                send_packet(&mut stream, response).await;
                            }
                            serverbound::Packet::PingRequest(request) => {
                                let response = clientbound::PingResponse {
                                    payload: request.payload
                                };

                                send_packet(&mut stream, response).await;
                            }
                        }
                    }
                    None => {
                        announce_unknown_packet();
                    }
                }
            },
            ConnectionState::Login => {
                use packets::login::{serverbound, clientbound};

                let packet = serverbound::Packet::read(buffer);

                match packet {
                    Some(packet) => {
                        announce_packet(&packet);

                        match packet {
                            serverbound::Packet::LoginStart(request) => {
                                // TODO: Implement online mode

                                let response = clientbound::LoginSuccess {
                                    // FIXME: Generate actual UUID
                                    uuid: 0,
                                    username: request.name,
                                    properties: vec![]
                                };

                                send_packet(&mut stream, response).await;

                                state = ConnectionState::Play;
                            }
                        }
                    },
                    None => {
                        announce_unknown_packet();
                    }
                }
            },
            ConnectionState::Play => {

            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = String::from("127.0.0.1:25565");

    let listener = TcpListener::bind(&address).await?;
    info!("listening on <green>`{}`</>", &address);

    loop {
        let (stream, client_address) = listener.accept().await?;

        info!("esablished connection with <green>`{}`</>", client_address);

        tokio::spawn(async move {
            handle_connection(stream, client_address).await;
        });
    }
}
