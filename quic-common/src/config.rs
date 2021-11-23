use std::net::SocketAddr;

use serde::{self, Deserialize};
use webpki::{DNSName, DNSNameRef};

#[derive(Deserialize)]
#[serde(crate = "self::serde")]
struct SerializedConfig {
    config_version: u16,
    server_addr: SocketAddr,
    server_dns_name: String,
    client_dns_names: Vec<String>,
}

pub struct IptsQuicConfig {
    pub server_addr: SocketAddr,
    pub server_dns_name: DNSName,
    pub client_dns_names: Vec<DNSName>,
}

impl IptsQuicConfig {
    pub fn load() -> Self {
        let db: SerializedConfig = toml::from_str(
            &std::fs::read_to_string("ipts-quic.toml").expect("Read ipts-quic.toml"),
        )
        .expect("Parse ipts-quic.toml");
        assert_eq!(db.config_version, 1);
        IptsQuicConfig {
            server_addr: db.server_addr,
            server_dns_name: DNSNameRef::try_from_ascii_str(&db.server_dns_name)
                .expect("Parse server_dns_name")
                .to_owned(),
            client_dns_names: db
                .client_dns_names
                .into_iter()
                .map(|x| {
                    DNSNameRef::try_from_ascii_str(&x)
                        .expect("Parse client_dns_names")
                        .to_owned()
                })
                .collect(),
        }
    }
}
