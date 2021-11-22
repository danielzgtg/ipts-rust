use std::sync::Arc;

use quic_common::rustls::ciphersuite::{
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
};
use quic_common::rustls::{
    AllowAnyAuthenticatedClient, Certificate, NoServerSessionStorage, PrivateKey, ProtocolVersion,
    RootCertStore, ServerConfig,
};

pub(crate) fn new_rustls_server_config(certs: Vec<Certificate>, key: PrivateKey) -> ServerConfig {
    let mut x = ServerConfig::with_ciphersuites(
        AllowAnyAuthenticatedClient::new({
            let mut roots = RootCertStore::empty();
            assert_eq!(certs.iter().count(), 3);
            roots.add(certs.iter().last().unwrap()).unwrap();
            roots
        }),
        &[
            &TLS13_AES_128_GCM_SHA256,
            &TLS13_CHACHA20_POLY1305_SHA256,
            &TLS13_AES_256_GCM_SHA384,
        ],
    );
    x.set_persistence(Arc::new(NoServerSessionStorage {}));
    x.set_single_cert(certs, key)
        .expect("Inserting certificate and key");
    x.set_protocols(&[b"ipts".to_vec()]);
    x.versions = vec![ProtocolVersion::TLSv1_3];
    x
}
