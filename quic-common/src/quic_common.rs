use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use crate::DATAGRAM_SIZE;
use futures_util::StreamExt;
use quinn::{Datagrams, IncomingBiStreams, IncomingUniStreams, TransportConfig};
use quinn_proto::{IdleTimeout, VarInt};
use rustls::{Certificate, PrivateKey};
use tokio::select;

pub fn load_certs(path: &'static str) -> (Vec<Certificate>, PrivateKey) {
    let f = File::open(path).expect("Failed to open pem");
    let mut f = BufReader::new(f);
    let mut certs: Vec<Certificate> = Vec::with_capacity(3);
    let mut key: Option<PrivateKey> = None;
    while let Some(x) = rustls_pemfile::read_one(&mut f).expect("Failed reading pem") {
        match x {
            rustls_pemfile::Item::X509Certificate(x) => {
                certs.push(Certificate(x));
            }
            rustls_pemfile::Item::RSAKey(_) => println!("PKCS1 not expected from pem"),
            rustls_pemfile::Item::PKCS8Key(x) => {
                assert!(key.is_none(), "Too many keys in pem");
                key = Some(PrivateKey(x))
            }
        }
    }
    assert_eq!(certs.len(), 3, "Wrong number of certificates");
    (certs, key.expect("Missing private key"))
}

pub fn new_transport_config<const SERVER: bool>() -> TransportConfig {
    let mut x = TransportConfig::default();
    x.max_concurrent_bidi_streams(VarInt::from_u32(0));
    x.max_concurrent_uni_streams(VarInt::from_u32(0));
    x.max_idle_timeout(Some(IdleTimeout::try_from(Duration::from_secs(5)).unwrap()));
    x.stream_receive_window(VarInt::from_u32(0));
    x.receive_window(VarInt::from_u32(0));
    x.send_window(0);
    x.max_tlps(2);
    x.packet_threshold(10);
    x.time_threshold(2.0);
    x.initial_rtt(Duration::from_millis(10));
    x.persistent_congestion_threshold(3);
    x.keep_alive_interval(Some(Duration::from_secs(1)));
    x.crypto_buffer_size(4096);
    x.allow_spin(false);
    if SERVER {
        const DATAGRAM_OVERHEAD: usize = 9; // TODO ask quinn upstream to expose this
        x.datagram_receive_buffer_size(Some(DATAGRAM_SIZE + DATAGRAM_OVERHEAD));
        x.datagram_send_buffer_size(0);
    } else {
        x.datagram_receive_buffer_size(None);
        x.datagram_send_buffer_size(DATAGRAM_SIZE);
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_client() {
        let _ = new_transport_config::<false>();
    }

    #[test]
    fn config_server() {
        let _ = new_transport_config::<true>();
    }
}

pub async fn unexpect_streams(mut uni: IncomingUniStreams, mut bi: IncomingBiStreams) {
    if select! {
        biased;
        v = uni.next() => v.map_or(false, |x| x.is_ok()),
        v = bi.next() => v.map_or(false, |x| x.is_ok()),
    } {
        // Assert unreachable because of new_transport_config
        panic!("Stream unexpected");
    }
}

pub async fn unexpect_all(
    mut datagrams: Datagrams,
    mut uni: IncomingUniStreams,
    mut bi: IncomingBiStreams,
) {
    if select! {
        biased;
        v = datagrams.next() => v.map_or(false, |x| x.is_ok()),
        v = uni.next() => v.map_or(false, |x| x.is_ok()),
        v = bi.next() => v.map_or(false, |x| x.is_ok()),
    } {
        // Assert unreachable because of new_transport_config
        panic!("No incoming data expected");
    }
}
