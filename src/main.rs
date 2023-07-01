use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use mcserver::{interface::Interface, packets::{handshaking, status::{self, clientbound::{PingResponseData, StatusResponseData}}, login::{self, clientbound::LoginSuccessData}}, switch, protocol::{DecodeError, DecodeResult}, types::{UUID, VarI32}, world::World};
use paris::info;
use tokio::net::{TcpListener, TcpStream};

enum ConnectionState {
    Handshaking,
    Status,
    Login
}

enum ConnectionOutcome {
    Disconnected,
    Connect(Interface)
}

async fn handle_connection(stream: TcpStream, client_address: SocketAddr) -> ConnectionOutcome {
    info!("established connection with <green>'{:?}'</>", client_address);

    let mut connection_state = ConnectionState::Handshaking;
    let mut interface = Interface::new(stream);

    enum PacketOutcome {
        Continue,
        Connect
    }

    async fn handle_packet(connection_state: &mut ConnectionState, interface: &mut Interface, client_address: &SocketAddr) -> DecodeResult<PacketOutcome> {
        match connection_state {
            ConnectionState::Handshaking => match interface.receive().await? {
                handshaking::serverbound::Packet::Handshake(data) => {
                    match data.next_state.into() {
                        1 => *connection_state = ConnectionState::Status,
                        2 => *connection_state = ConnectionState::Login,
                        _ => unreachable!()
                    }
                }
            },
            ConnectionState::Status => match interface.receive().await? {
                status::serverbound::Packet::StatusRequest(_) => {
                    if !client_address.is_ipv4() {
                        unimplemented!()
                    }

                    let address_string = client_address.to_string();
                    let address_length = address_string.len();
                    let digits_num = address_string.chars()
                        .filter(|character| character.is_numeric())
                        .count();
                    let separators_num = address_length - digits_num;
                    let address_width = digits_num * 6 + separators_num * 2 + address_length - 1;

                    let free_space = (251 - address_width) / 2;

                    let mut block_amount = free_space / 9;

                    while block_amount > 0 {
                        let space_for_spaces = free_space - block_amount * 8 - block_amount;

                        if space_for_spaces % 5 == 0 {
                            break;
                        }

                        block_amount -= 1;
                    }

                    let blocks = "█".repeat(block_amount);
                    let spaces = " ".repeat((free_space - block_amount * 8 - block_amount) / 5);

                    let response = status::clientbound::Packet::StatusResponse(StatusResponseData {
                        response: format!(
                            "{{\"version\":{{\"name\":\"§k███████████████████████████>>\",\"protocol\":-1}},\"players\":{{\"max\":0,\"online\":0}},\"description\":[{{\"text\":\"{}\",\"color\":\"red\",\"obfuscated\":true}},{{\"text\":\"{}{}{}\",\"color\":\"white\",\"bold\":true,\"obfuscated\":false}},{{\"text\":\"{}\n████████████████████████████\",\"color\":\"red\",\"bold\":false,\"obfuscated\":true}}],\"favicon\":\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAAXNSR0IArs4c6QAAAjVJREFUeJztW8tuAjEMZBdDF2hVVT30//8RFbSE0Gs9kTKy9jCHeG5hsxvL8jh+MX1e3l67f7B5/r/cVVvcer1d3dr2fv9yNLcupfp19WsEnn9/lP5+OJ+9j/v9akCkAtQCqDH9fF2cD7ivfc4hkPPR98uz7xPeT8fu96Pn43nDW0AqQC2AGtbc08CR6D2P9zDe+/gcwb6PwOd19j7jbP45+ojhLSAVoBZAjemyHFwcsByAgxtjd/QhyNG5rpu+F/U5ZpkLOKQC1AKoYcgpBIvVWdyAqI+7W897wmESh+C93siDPqJkLuCQClALoIbNx7P7oay/fgPhIILl62w/u9eRw/j9683HFSyOGd4CUgFqAdSwucC9TDhIOfvsc5YhWj9ARH3W8BaQClALoIZFOcdyA6wnNO+TexnB6gE70oZgPmt4C0gFqAVQoyFstB+PiHJ8a66AiM4XDG8BqQC1AGoYq7GxXh1yDPv5iKaGR3IFWq9gPoHkBsNbQCpALYAazXwA8wlNLH3wc4SlxGaE0IdgLlGB87TuT+KQ7AsAUgFqAdSYvj9Or94G7KdH5wC3IporoI9g8w7DW0AqQC2AGvao3gVMO3AJQHnauwMOYm6AvbvoHCJDdMZpeAtIBagFUMOQ81tngnAdjRu21gCjPml4C0gFqAVQwxjHsCa3wDwBA5v7o71IwmnWi0TOZz0AkApQC6BGMx/QxuJ9ztMaHIkb6Plr/x5vD4Ql+e/y8BaQClALoMYf4BMn6crJkqUAAAAASUVORK5CYII=\"}}",
                            &blocks, &spaces, client_address, &spaces, &blocks
                        )
                    });

                    interface.send(response).await;
                },
                status::serverbound::Packet::PingRequest(data) => {
                    let response = status::clientbound::Packet::PingResponse(PingResponseData {
                        payload: data.payload
                    });

                    interface.send(response).await;
                }
            },
            ConnectionState::Login => match interface.receive().await? {
                login::serverbound::Packet::LoginStart(data) => {
                    let response = login::clientbound::Packet::LoginSuccess(LoginSuccessData {
                        uuid: data.uuid.unwrap_or(UUID(0)),
                        username: data.name,
                        number_of_properties: VarI32::from(0)
                    });

                    interface.send(response).await;

                    return Ok(PacketOutcome::Connect)
                }
            }
        }
        Ok(PacketOutcome::Continue)
    }

    loop {
        match handle_packet(&mut connection_state, &mut interface, &client_address).await {
            Ok(PacketOutcome::Continue) => (),
            Ok(PacketOutcome::Connect) => {
                return ConnectionOutcome::Connect(interface);
            }
            Err(DecodeError::UnexpectedEOF) => {
                info!("connection with <green>'{}'</> aborted", &client_address);
                return ConnectionOutcome::Disconnected
            }
        }
    }
}

async fn accept_connections(listener: &mut TcpListener, world: &Arc<Mutex<World>>) {
    loop {
        let world = world.clone();
        let (stream, client_address) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            match handle_connection(stream, client_address).await {
                ConnectionOutcome::Disconnected => (),
                ConnectionOutcome::Connect(interface) => {
                    let mut lock = world.lock().await;
                    (*lock).connect_client(interface).await;
                }
            };
        });
    }
}

#[tokio::main]
async fn main() {
    let address = "192.168.0.10:25565";
    //let address = "localhost:25565";

    let mut listener = switch!(
        TcpListener::bind(address).await;
        "listening on <green>'{}'</>", address;
        "failed to construct the listener";
    );

    let world = Arc::new(Mutex::new(World::new()));

    accept_connections(&mut listener, &world).await;
}