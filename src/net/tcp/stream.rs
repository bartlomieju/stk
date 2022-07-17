use std::io;

pub struct TcpStream {
    mio: mio::net::TcpListener,
}

impl TcpStream {
    pub(crate) fn new(mio: mio::net::TcpStream) -> io::Result<TcpStream> {
        todo!();
    }
}
