use stokio::net::{TcpListener, TcpStream};
use stokio::runtime::Runtime;

async fn process_socket(mut socket: TcpStream) {
    let mut req = [0; 4096];
    let res = b"HTTP/1.1 200 OK\r\nContent-length: 12\r\n\r\nHello world\n";

    loop {
        let n = match socket.read(&mut req).await {
            Ok(n) if n > 0 => n,
            _ => return,
        };

        socket.write_all(res).await.unwrap();
    }
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
