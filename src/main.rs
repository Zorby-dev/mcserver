use mcserver::{receiver::Receiver, packets::{HandshakingServerboundPacket, StatusServerboundPacket}, switch};
use tokio::net::{TcpListener, TcpStream};

enum ConnectionState {
    Handshaking,
    Status
}

async fn handle_connection(stream: TcpStream) {
    let mut connection_state = ConnectionState::Handshaking;
    let mut receiver = Receiver::new(stream);

    loop {
        match connection_state {
            ConnectionState::Handshaking => match receiver.receive().await.unwrap() {
                HandshakingServerboundPacket::Handshake(data) => {
                    match data.next_state.into() {
                        1 => connection_state = ConnectionState::Status,
                        _ => unreachable!()
                    }
                }
            },
            ConnectionState::Status => match receiver.receive().await.unwrap() {
                StatusServerboundPacket::StatusRequest(data) => {
                    println!("{:?}", data);
                }
            }
        }
    }
}

async fn accept_connections(listener: &mut TcpListener) {
    loop {
        let (stream, _client_address) = listener.accept().await.unwrap();

        tokio::spawn(async {
            handle_connection(stream).await;
        });
    }
}

#[tokio::main]
async fn main() {
    let address = "127.0.0.1:25565";

    let mut listener = switch!(
        TcpListener::bind(address).await;
        "listening on <green>'{}'</>", address;
        "failed to construct the listener";
    );

    accept_connections(&mut listener).await;
}