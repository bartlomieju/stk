use crate::net::TcpStream;
use crate::runtime::io::{Interest, Registration};
use crate::runtime::Handle;
use std::task::Context;
use std::io;
use std::net::SocketAddr;

pub struct TcpListener {
    /// Mio listener
    mio: mio::net::TcpListener,

    addr: SocketAddr,

    /// Socket registered with the I/O driver
    registration: Registration,
}

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> io::Result<TcpListener> {
        let mio = mio::net::TcpListener::bind(addr.clone())?;
        TcpListener::new(mio, addr)
    }

    fn new(mut mio: mio::net::TcpListener, addr: SocketAddr) -> io::Result<TcpListener> {
        Handle::with_current(|handle| {
            let registration = handle
                .io()
                .register(&handle, &mut mio, Interest::READABLE)?;
            Ok(TcpListener { mio, registration, addr })
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        Ok(self.addr)
    }

    pub fn poll_accept(&self, cx: &mut Context<'_>) -> std::task::Poll<io::Result<(TcpStream, SocketAddr)>> {
        todo!()
    }

    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (mio, addr) = self.registration.async_read(|| self.mio.accept()).await?;

        let stream = TcpStream::new(mio, self.addr)?;
        Ok((stream, addr))
    }
}
