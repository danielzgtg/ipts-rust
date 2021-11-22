use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use quic_common::quinn::{ClientConfig, Endpoint, EndpointBuilder};
use quic_common::quinn_proto::EndpointConfig;
use quic_common::{load_certs, new_transport_config};

use crate::tls::new_rustls_client_config;

fn new_client_config() -> ClientConfig {
    let (certs, key) = load_certs("../client.pem");
    let mut x = ClientConfig::default();
    x.transport = Arc::new(new_transport_config::<false>());
    x.crypto = Arc::new(new_rustls_client_config(certs, key));
    x
}

pub fn new_client() -> Endpoint {
    let client_config = new_client_config();
    let mut x = EndpointConfig::default();
    // x.supported_versions(vec![1], 1).unwrap(); // TODO
    EndpointBuilder::new(x, client_config)
        .bind(&SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0))
        .expect("Building endpoint")
        .0
}
