use std::convert::Infallible;
use std::hash::Hasher;
use hyper::server::conn::Http;
use stokio::net::{TcpListener, TcpStream};
use stokio::runtime::Runtime;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use std::pin::Pin;
use hyper::http;
use hyper::service::Service;
use hyper::StatusCode;
use std::future::Future;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, server::stokio_tcp::AddrIncoming};

struct HelloWorld;

impl Service<Request<Body>> for HelloWorld {
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // create the body
        let body: Body = Body::from("Hello world\n".as_bytes().to_owned());
        // Create the HTTP response
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("Unable to create `http::Response`");

        // create a response in a future.
        let fut = async {
            Ok(resp)
        };

        // Return the response as an immediate future
        Box::pin(fut)
    }
}


async fn process_socket(mut socket: TcpStream) {
    let service = HelloWorld;
    let http = Http::new().with_executor(LocalExec)
        .serve_connection(socket, service);

    http.await.unwrap();
}

fn main() {
    let rt = Runtime::new().unwrap();

    rt.spawn(async {
        let addr = "[::1]:9000".parse().unwrap();
        let listener = TcpListener::bind(addr).unwrap();

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            socket.set_nodelay(true).unwrap();
            stokio::spawn(process_socket(socket));
        }
    });

    rt.run();
}

// Since the Server needs to spawn some background tasks, we needed
// to configure an Executor that can spawn !Send futures...
#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        // This will spawn into the currently running `LocalSet`.
        stokio::spawn(fut);
    }
}
