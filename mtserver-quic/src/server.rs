use quic_common::futures_util::stream::FuturesUnordered;
use quic_common::futures_util::StreamExt;
use quic_common::quinn::{Connecting, Incoming, NewConnection};
use quic_common::quinn_proto::VarInt;
use quic_common::tokio::select;
use quic_common::{unexpect_streams, IptsQuicConfig, DATAGRAM_SIZE};

use crate::mux::{InputConnection, InputMultiplexer};
use crate::quic::new_server;

async fn handle(
    mut connection: Connecting,
    mut input: InputConnection,
    config: &'static IptsQuicConfig,
) {
    let id = input.id();
    println!("[{}] Incoming {}", id, connection.remote_address());

    match connection.handshake_data().await {
        Err(err) => {
            eprintln!("[{}] Error before handshake data {:?}", id, err);
            return;
        }
        Ok(x) => {
            if match x.server_name {
                None => {
                    eprintln!("[{}] Missing SNI", id);
                    return;
                }
                Some(x) => x,
            } != <dyn AsRef<str>>::as_ref(&config.server_dns_name)
            {
                eprintln!("[{}] Wrong SNI", id);
                return;
            }
            if match x.protocol {
                None => {
                    eprintln!("[{}] Missing ALPN", id);
                    return;
                }
                Some(x) => x,
            } != b"ipts"
            {
                eprintln!("[{}] Wrong ALPN", id);
                return;
            }
        }
    }

    let NewConnection {
        connection,
        uni_streams,
        bi_streams,
        mut datagrams,
        ..
    } = match connection.await {
        Ok(x) => x,
        Err(err) => {
            eprintln!("[{}] Error after handshake data {:?}", id, err);
            return;
        }
    };
    if {
        let cert = connection.peer_identity().unwrap();
        let cert = &cert.iter().next().unwrap().0;
        let cert = quic_common::webpki::EndEntityCert::from(cert).unwrap();
        config
            .client_dns_names
            .iter()
            .all(|x| cert.verify_is_valid_for_dns_name(x.as_ref()).is_err())
    } {
        eprintln!("[{}] Client certificate wrong SAN", id);
        return;
    }
    let join = quic_common::tokio::task::spawn(unexpect_streams(uni_streams, bi_streams));

    println!("[{}] Ready", id);
    while let Some(data) = datagrams.next().await {
        let data = match data {
            Ok(x) => x,
            Err(e) => {
                eprintln!("[{}] Read error: {:?}", id, e);
                break;
            }
        };
        let data = match <&[u8; DATAGRAM_SIZE]>::try_from(&*data) {
            Ok(x) => x,
            Err(_) => {
                eprintln!("[{}] Wrong datagram size", id);
                break;
            }
        };
        if input.send(data).await.is_err() {
            eprintln!("[{}] Misbehaved", id);
            break;
        }
    }

    std::mem::drop(input);
    connection.close(VarInt::default(), &[]);
    join.abort();
    let _ = join.await;
    println!("[{}] End {:?}", id, connection.stats());
}

async fn listen(mut incoming: Incoming, config: &'static IptsQuicConfig) {
    let (mux, gc) = InputMultiplexer::new();
    let join = quic_common::tokio::task::spawn(gc);
    let mut nursery = FuturesUnordered::new();

    let mut nursery_active = false;
    println!("Server ready");
    loop {
        if let Some(incoming) = if !nursery_active {
            match incoming.next().await {
                x @ Some(_) => x,
                None => break,
            }
        } else {
            select! {
                biased;
                x = nursery.next() => {
                    nursery_active = x.is_some();
                    None
                },
                x @ Some(_) = incoming.next() => x,
                else => break,
            }
        } {
            nursery.push(handle(incoming, InputConnection::new(mux.clone()), config));
            nursery_active = true;
        }
    }

    join.abort();
    while let Some(_) = nursery.next().await {}
    let _ = join.await;
}

pub async fn run(config: &'static IptsQuicConfig) {
    let (shutdown, incoming) = new_server(config);
    let join = quic_common::tokio::task::spawn(listen(incoming, config));
    quic_common::tokio::signal::ctrl_c().await.unwrap();
    shutdown.close(VarInt::default(), &[]);
    let _ = join.await;
}
