use hyper::server::{conn::AddrIncoming, Builder, Server};
use listenfd::ListenFd;

pub fn make_server(port: u16) -> hyper::Result<Builder<AddrIncoming>> {
    let mut listenfd = ListenFd::from_env();
    if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(l)
    } else {
        Ok(Server::bind(&([0, 0, 0, 0], port).into()))
    }
}
