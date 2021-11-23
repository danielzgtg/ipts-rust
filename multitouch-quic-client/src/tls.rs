use std::sync::Arc;

use quic_common::rustls::cipher_suite::{
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
};
use quic_common::rustls::client::NoClientSessionStorage;
use quic_common::rustls::kx_group::X25519;
use quic_common::rustls::version::TLS13;
use quic_common::rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore};

pub(crate) fn new_rustls_client_config(certs: Vec<Certificate>, key: PrivateKey) -> ClientConfig {
    let mut result = ClientConfig::builder()
        .with_cipher_suites(&[
            TLS13_AES_128_GCM_SHA256,
            TLS13_CHACHA20_POLY1305_SHA256,
            TLS13_AES_256_GCM_SHA384,
        ])
        .with_kx_groups(&[&X25519])
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_root_certificates({
            let mut roots = RootCertStore::empty();
            assert_eq!(certs.iter().count(), 3);
            roots.add(certs.iter().last().unwrap()).unwrap();
            roots
        })
        .with_single_cert(certs, key)
        .expect("Inserting certificate and key");
    result.alpn_protocols = vec![b"ipts".to_vec()];
    result.session_storage = Arc::new(NoClientSessionStorage {});
    result.enable_tickets = false;
    result
}
