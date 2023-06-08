use mcserver::{receiver::Receiver, packets::Packet, switch};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(stream: TcpStream) {
    let mut receiver = Receiver::new(stream);

    loop {
        match receiver.receive().await.unwrap() {
            Packet::Handshake(data) => {
                println!("{:?}", data);
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