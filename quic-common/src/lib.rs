pub use bytes;
pub use futures_util;
pub use quinn;
pub use quinn_proto;
pub use rustls;
pub use rustls_pemfile;
pub use tokio;
pub use webpki;

pub use config::IptsQuicConfig;
pub use datagram::ReportTransport;
pub use quic_common::{load_certs, new_transport_config, unexpect_all, unexpect_streams};

mod config;
mod datagram;
mod quic_common;

pub const DATAGRAM_SIZE: usize = 168;
