use std::sync::Arc;

use quic_common::rustls::cipher_suite::{
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
};
use quic_common::rustls::kx_group::X25519;
use quic_common::rustls::server::{AllowAnyAuthenticatedClient, NoServerSessionStorage};
use quic_common::rustls::version::TLS13;
use quic_common::rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig};

pub(crate) fn new_rustls_server_config(certs: Vec<Certificate>, key: PrivateKey) -> ServerConfig {
    let mut result = ServerConfig::builder()
        .with_cipher_suites(&[
            TLS13_AES_128_GCM_SHA256,
            TLS13_CHACHA20_POLY1305_SHA256,
            TLS13_AES_256_GCM_SHA384,
        ])
        .with_kx_groups(&[&X25519])
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_client_cert_verifier(AllowAnyAuthenticatedClient::new({
            let mut roots = RootCertStore::empty();
            assert_eq!(certs.iter().count(), 3);
            roots.add(certs.iter().last().unwrap()).unwrap();
            roots
        }))
        .with_single_cert(certs, key)
        .expect("Inserting certificate and key");
    result.session_storage = Arc::new(NoServerSessionStorage {});
    result.alpn_protocols = vec![b"ipts".to_vec()];
    result
}
