use crate::runtime::io::{Interest, Ready, Registration};
use crate::runtime::Handle;

use std::fmt;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::task::{self, Poll};
use std::pin::Pin;
use std::os::unix::io::RawFd;
use std::os::unix::io::AsRawFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct TcpStream {
    mio: mio::net::TcpStream,

    addr: SocketAddr,

    /// Socket registered with the I/O driver
    registration: Registration,
}


impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TcpStream")
         .finish()
    }
}

impl TcpStream {
    pub(crate) fn new(mut mio: mio::net::TcpStream, addr: SocketAddr) -> io::Result<TcpStream> {
        Handle::with_current(|handle| {
            let registration =
                handle
                    .io()
                    .register(&handle, &mut mio, Interest::READABLE)?;
            Ok(TcpStream { mio, registration, addr })
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        Ok(self.addr)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.mio.set_nodelay(nodelay)
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        crate::future::poll_fn(|cx| self.poll_read_inner(cx, buf)).await
    }

    pub fn poll_peek(&mut self, cx: &mut task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<io::Result<usize>> {
        todo!()
    }

    fn poll_read_inner(&mut self, cx: &mut task::Context<'_>, buf: &mut [u8])
        -> Poll<io::Result<usize>>
    {
        loop {
            let ready = match self.registration.poll_read_ready(cx) {
                Poll::Ready(ready) => ready,
                Poll::Pending => return Poll::Pending,
            };

            if ready.is_read_closed() {
                // Nothing to read, just return.
                return Poll::Ready(Ok(0));
            }

            match self.mio.read(buf) {
                Ok(0) => return Poll::Ready(Ok(0)),
                Ok(n) if n == buf.len() => return Poll::Ready(Ok(n)),
                Ok(n) => {
                    // Partial read indicates the socket buffer has been drained
                    // Clear readiness, but return anyway
                    self.registration.clear_readiness(Ready::READABLE);
                    return Poll::Ready(Ok(n));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.registration.clear_readiness(Ready::READABLE);
                }
                x => return Poll::Ready(x),
            }
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_inner(buf)

        /*
        self.registration
            .async_write(|| {
                self.write_inner(buf)
            })
            .await
            */
    }

    fn write_inner(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = unsafe {
            libc::send(
                self.mio.as_raw_fd(),
                buf.as_ptr() as _,
                buf.len(),
                #[cfg(os = "linux")]
                libc::MSG_NOSIGNAL,
                #[cfg(not(os = "linux"))]
                0,
            )
        } as usize;
        Ok(n)
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

impl AsRawFd for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        todo!()
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // TODO: use the non-initializing method
        match self.poll_read_inner(cx, buf.initialize_unfilled()) {
            Poll::Ready(Ok(n)) => {
                buf.advance(n);
                Poll::Ready(Ok(()))
            },
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.write_inner(buf))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        // tcp stream is always flushed
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        // tcp stream is always done
        Poll::Ready(Ok(()))
    }
}
