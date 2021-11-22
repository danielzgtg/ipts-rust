use engine_avx512::process_heatmap;
use std::time::{Duration, Instant};

use ipts_dev::{HeaderAndBuffer, IptsAsync, IptsExt};
use quic_common::bytes::Bytes;
use quic_common::futures_util::future::{select, Either};
use quic_common::quinn::Connecting;
use quic_common::quinn::{Endpoint, NewConnection};
use quic_common::quinn_proto::VarInt;
use quic_common::tokio::pin;
use quic_common::{unexpect_all, IptsQuicConfig, ReportTransport, DATAGRAM_SIZE};
use utils::{get_heatmap, Pointers};

use crate::quic::new_client;

fn connect(endpoint: &mut Endpoint, config: &'static IptsQuicConfig) -> Connecting {
    let server_dns_name = <dyn AsRef<str>>::as_ref(&config.server_dns_name);
    let server_addr = &config.server_addr;
    println!("Connecting to {}", server_addr);
    endpoint
        .connect(server_addr, server_dns_name)
        .expect("Connect endpoint to server")
}

async fn handle(mut connection: Connecting, config: &'static IptsQuicConfig) {
    let mut ipts = IptsAsync::new();
    let mut ipts_buf = [0u8; 16384];
    let mut pointers = Pointers::new();
    let mut positions: [(u32, u32); 10] = [(0, 0); 10];

    match connection.handshake_data().await {
        Err(err) => {
            eprintln!("Error before handshake data {:?}", err);
            return;
        }
        Ok(x) => {
            debug_assert!(x.server_name.is_none());
            if match x.protocol {
                None => {
                    eprintln!("Missing ALPN");
                    return;
                }
                Some(x) => x,
            } != b"ipts"
            {
                eprintln!("Wrong ALPN");
                return;
            }
        }
    }

    let NewConnection {
        connection,
        uni_streams,
        bi_streams,
        datagrams,
        ..
    } = match connection.await {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Error after handshake data {:?}", err);
            return;
        }
    };
    if quic_common::webpki::EndEntityCert::from(
        &connection.peer_identity().unwrap().iter().next().unwrap().0,
    )
    .unwrap()
    .verify_is_valid_for_dns_name(config.server_dns_name.as_ref())
    .is_err()
    {
        eprintln!("Client certificate wrong SAN");
        return;
    }
    let mut join = Some(quic_common::tokio::task::spawn(unexpect_all(
        datagrams,
        uni_streams,
        bi_streams,
    )));

    let mut last_multitouch = Instant::now();
    let mut transport = ReportTransport::new();
    println!("Ready");
    loop {
        {
            let doorbell =
                ipts.wait_for_doorbell(Instant::now() - last_multitouch < Duration::from_secs(1));
            // Eagerness will be lowered after receiving a few idle messages without heatmaps
            pin!(doorbell);
            match select(join.take().unwrap(), doorbell).await {
                Either::Left(_) => {
                    eprintln!("Streams disturbed");
                    break;
                }
                Either::Right((_, j)) => join = Some(j),
            }
        }
        ipts.read(&mut ipts_buf);

        let parsed = HeaderAndBuffer::from(&ipts_buf);
        if parsed.typ == 3 && parsed.size == 3500 && parsed.data[0] == 0x0B {
            let data = get_heatmap((&parsed.data[..3500]).try_into().unwrap());
            let length = process_heatmap(data, &mut positions);
            pointers.update(positions, length);

            let mut out_buf = vec![0u8; DATAGRAM_SIZE];
            transport.offer(pointers.events());
            transport.serialize((&mut out_buf[..]).try_into().unwrap());

            if let Err(e) = connection.send_datagram(Bytes::from(out_buf)) {
                eprintln!("Datagram send failure {:?}", e);
                break;
            }

            last_multitouch = Instant::now();
        }

        ipts.send_feedback().await;
    }

    std::mem::drop(ipts);
    connection.close(VarInt::default(), &[]);
    if let Some(join) = join {
        join.abort();
        let _ = join.await;
    }
    println!("End {:?}", connection.stats());
}

pub async fn run(config: &'static IptsQuicConfig) {
    let mut endpoint = new_client();
    let join = quic_common::tokio::task::spawn(handle(connect(&mut endpoint, config), config));
    let ctrl_c = quic_common::tokio::signal::ctrl_c();
    pin!(ctrl_c);
    let join = match select(ctrl_c, join).await {
        Either::Left((_, join)) => Some(join),
        Either::Right(_) => None,
    };
    endpoint.close(VarInt::default(), &[]);
    if let Some(join) = join {
        // TODO why doesn't endpoint.close work before connection establishment?
        let _ = join.await;
    }
}
