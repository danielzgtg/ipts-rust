use std::convert::TryInto;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ipts_dev::{HeaderAndBuffer, Ipts};
use mtinput::MtInput;
use utils::Pointers;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::Release);
    }).unwrap();

    let mut ipts = Ipts::new();
    let mut buf = [0u8; 16384];

    let mut pointers = Pointers::new();
    let mut mt = MtInput::new();
    let mut positions: [(u32, u32); 10] = [(0, 0); 10];

    while running.load(Ordering::Acquire) {
        ipts.wait_for_doorbell();
        ipts.read_and_send_feedback(&mut buf);

        let parsed = HeaderAndBuffer::from(&buf);
        if parsed.typ == 3 && parsed.size == 6 && parsed.data[0] == 0x40 {
            if parsed.data[1] == 0 {
                pointers.update(positions, 0);
            } else {
                let x_raw = u16::from_le_bytes(parsed.data[2..4].try_into().unwrap());
                let y_raw = u16::from_le_bytes(parsed.data[4..6].try_into().unwrap());
                let x = (x_raw as u32 * 2736) >> 15;
                let y = (y_raw as u32 * 1824) >> 15;
                positions[0] = (x, y);
                pointers.update(positions, 1);
            }
            mt.dispatch(&mut pointers);
        }
    }
}
