use std::net::UdpSocket;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use quic_common::quinn::{ClientConfig, Endpoint};
use quic_common::quinn_proto::EndpointConfig;
use quic_common::{load_certs, new_transport_config};

use crate::tls::new_rustls_client_config;

fn new_client_config() -> ClientConfig {
    let (certs, key) = load_certs("client.pem");
    let mut x = ClientConfig::new(Arc::new(new_rustls_client_config(certs, key)));
    x.transport = Arc::new(new_transport_config::<false>());
    x.version(1);
    x
}

pub fn new_client() -> Endpoint {
    let client_config = new_client_config();
    let mut x = EndpointConfig::default();
    x.supported_versions(vec![1]);
    let socket =
        UdpSocket::bind(&SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0)).expect("Bind socket");
    let (mut result, _) = Endpoint::new(x, None, socket).expect("Building endpoint");
    result.set_default_client_config(client_config);
    result
}
