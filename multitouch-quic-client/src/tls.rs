use std::sync::Arc;

use quic_common::rustls::ciphersuite::{
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
};
use quic_common::rustls::{
    Certificate, ClientConfig, NoClientSessionStorage, PrivateKey, ProtocolVersion, RootCertStore,
};

pub(crate) fn new_rustls_client_config(certs: Vec<Certificate>, key: PrivateKey) -> ClientConfig {
    let mut x = ClientConfig::with_ciphersuites(&[
        &TLS13_AES_128_GCM_SHA256,
        &TLS13_CHACHA20_POLY1305_SHA256,
        &TLS13_AES_256_GCM_SHA384,
    ]);
    x.root_store = {
        let mut roots = RootCertStore::empty();
        assert_eq!(certs.iter().count(), 3);
        roots.add(certs.iter().last().unwrap()).unwrap();
        roots
    };
    x.set_protocols(&[b"ipts".to_vec()]);
    x.set_persistence(Arc::new(NoClientSessionStorage {}));
    x.enable_tickets = false;
    x.set_single_client_cert(certs, key)
        .expect("Inserting certificate and key");
    x.versions = vec![ProtocolVersion::TLSv1_3];
    x
}
