use crate::net::TcpStream;
use crate::runtime::io::{Interest, Registration};
use crate::runtime::Handle;

use std::io;
use std::net::SocketAddr;

pub struct TcpListener {
    /// Mio listener
    mio: mio::net::TcpListener,

    /// Socket registered with the I/O driver
    registration: Registration,
}

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> io::Result<TcpListener> {
        let mio = mio::net::TcpListener::bind(addr)?;
        TcpListener::new(mio)
    }

    fn new(mut mio: mio::net::TcpListener) -> io::Result<TcpListener> {
        Handle::with_current(|handle| {
            let registration = handle
                .io()
                .register(&handle, &mut mio, Interest::READABLE)?;
            Ok(TcpListener { mio, registration })
        })
    }

    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (mio, addr) = self
            .registration
            .async_io(Interest::READABLE, || self.mio.accept())
            .await?;

        let stream = TcpStream::new(mio)?;
        Ok((stream, addr))
    }
}
