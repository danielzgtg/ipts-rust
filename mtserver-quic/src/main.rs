use mtserver_quic::run;
use quic_common::IptsQuicConfig;
use std::process::exit;

fn main() -> ! {
    let rt = quic_common::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(run(Box::leak(Box::new(IptsQuicConfig::load()))));
    rt.shutdown_background();
    exit(0)
}
