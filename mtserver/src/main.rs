use std::net::TcpListener;

use mtinput::MtInput;
use utils::{Counter, deserialize_reports, Report};
use std::io::Read;

fn main() {
    // Warning: There is no encryption. Only use this on a trusted network.
    let mut mt = MtInput::new();
    let mut buf = [0u8; 98];
    let mut events = [Report::None; 10];
    let mut counter = Counter::default();
    let mut stream = TcpListener::bind("0.0.0.0:34254").unwrap().accept().unwrap().0;

    loop {
        stream.read(&mut buf).unwrap();
        deserialize_reports(&buf, &mut events);
        mt.dispatch(&events, &mut counter);
    }
}
