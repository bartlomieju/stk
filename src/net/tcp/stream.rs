use std::io;

pub struct TcpStream {
    mio: mio::net::TcpListener,
}

impl TcpStream {
    pub(crate) fn new(mio: mio::net::TcpStream) -> io::Result<TcpStream> {
        todo!();
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }

    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }
}
