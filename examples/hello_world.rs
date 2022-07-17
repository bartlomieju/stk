use stokio::net::{TcpListener, TcpStream};
use stokio::runtime::Runtime;

async fn process_socket(socket: TcpStream) {
    todo!();
}

fn main() {
    let rt = Runtime::new().unwrap();

    rt.spawn(async {
        let addr = "[::1]:9000".parse().unwrap();
        let listener = TcpListener::bind(addr).unwrap();

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            stokio::spawn(process_socket(socket));
        }
    });

    rt.run();
}
