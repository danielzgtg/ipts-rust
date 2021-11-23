use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use quic_common::quinn::{Endpoint, Incoming, ServerConfig};
use quic_common::quinn_proto::EndpointConfig;
use quic_common::{load_certs, new_transport_config, IptsQuicConfig};

use crate::tls::new_rustls_server_config;

fn new_server_config() -> ServerConfig {
    let (certs, key) = load_certs("server.pem");
    let mut x = ServerConfig::with_crypto(Arc::new(new_rustls_server_config(certs, key)));
    x.transport = Arc::new(new_transport_config::<true>());
    x.use_retry(true);
    // Handshake should take max 500 ms, but we get INVALID_TOKEN on localhost with it set to that,
    // so this is currently raised to 2 seconds. TODO fix
    x.retry_token_lifetime(Duration::from_secs(2));
    x.concurrent_connections(20);
    x.migration(false);
    x
}

pub fn new_server(config: &IptsQuicConfig) -> (Endpoint, Incoming) {
    let server_config = new_server_config();
    let mut x = EndpointConfig::default();
    x.supported_versions(vec![1]);
    let socket = UdpSocket::bind(&{
        let mut addr = config.server_addr.clone();
        addr.set_ip(IpAddr::from([0, 0, 0, 0]));
        addr
    })
    .expect("Bind socket");
    Endpoint::new(x, Some(server_config), socket).expect("Building endpoint")
}
