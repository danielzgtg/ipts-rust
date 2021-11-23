use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use engine::Engine;
use ipts_dev::{HeaderAndBuffer, Ipts, IptsExt};
use utils::{get_heatmap, serialize_reports, Pointers};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Release);
    })
    .unwrap();

    let mut ipts = Ipts::new();
    let mut buf = [0u8; 16384];

    let mut pointers = Pointers::new();
    let mut positions: [(u32, u32); 10] = [(0, 0); 10];
    let mut engine = Engine::new(true);

    let mut stream = TcpStream::connect("daniel-desktop2.local:34254").unwrap();
    stream.set_nodelay(true).unwrap();
    stream.set_nonblocking(true).unwrap();
    let mut out_buf = [0u8; 98];

    let mut last_multitouch = Instant::now();
    while running.load(Ordering::Acquire) {
        ipts.wait_for_doorbell(Instant::now() - last_multitouch < Duration::from_secs(1));
        ipts.read(&mut buf);

        let parsed = HeaderAndBuffer::from(&buf);
        if parsed.typ == 3 && parsed.size == 3500 && parsed.data[0] == 0x0B {
            let data = get_heatmap((&parsed.data[..3500]).try_into().unwrap());
            let length = engine.run(data, &mut positions);
            pointers.update(positions, length);
            serialize_reports(pointers.events(), &mut out_buf);
            stream.write(&out_buf).unwrap();
            last_multitouch = Instant::now();
        }

        ipts.send_feedback();
    }
}
