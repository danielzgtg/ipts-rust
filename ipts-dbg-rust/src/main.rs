use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ipts_dev::{HeaderAndBuffer, Ipts, IptsExt};

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Release);
    })
    .unwrap();

    let mut ipts = Ipts::new();
    let mut buf = [0u8; 16384];
    let info = ipts.device_info();
    println!("Vendor:       {:04X}", info.vendor);
    println!("Product:      {:04X}", info.product);
    println!("Version:      {}", info.version);
    println!("Buffer Size:  {}", info.buffer_size);
    println!("Max Contacts: {}", info.max_contacts);
    println!();

    while running.load(Ordering::Acquire) {
        ipts.wait_for_doorbell(false);
        ipts.read(&mut buf);

        let parsed = HeaderAndBuffer::from(&buf);
        println!(
            "====== Buffer: {} == Type: {} == Size: {} ======",
            parsed.buffer, parsed.typ, parsed.size
        );
        parsed.print();

        ipts.send_feedback()
    }
}
