use crate::runtime::io::{Interest, Registration};
use crate::runtime::Handle;

use std::io;

pub struct TcpStream {
    mio: mio::net::TcpStream,

    /// Socket registered with the I/O driver
    registration: Registration,
}

impl TcpStream {
    pub(crate) fn new(mut mio: mio::net::TcpStream) -> io::Result<TcpStream> {
        Handle::with_current(|handle| {
            let registration =
                handle
                    .io()
                    .register(&handle, &mut mio, Interest::READABLE | Interest::WRITABLE)?;
            Ok(TcpStream { mio, registration })
        })
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
