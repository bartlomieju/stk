use crate::runtime::io::{Interest, Registration};
use crate::runtime::Handle;

use std::io::{self, Read, Write};

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

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.mio.set_nodelay(nodelay)
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let ready = self.registration.readiness(Interest::READABLE).await?;

            match self.mio.read(buf) {
                Ok(0) => return Ok(0),
                Ok(n) if n == buf.len() => return Ok(n),
                Ok(n) => {
                    // Partial read indicates the socket buffer has been drained
                    // Clear readiness, but return anyway
                    self.registration.clear_readiness(ready);
                    return Ok(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.registration.clear_readiness(ready);
                }
                x => return x,
            }
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.registration
            .async_io(Interest::WRITABLE, || self.mio.write(buf))
            .await
    }

    pub async fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            match self.write(buf).await {
                Ok(0) => {
                    return Err(io::ErrorKind::WriteZero.into());
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
