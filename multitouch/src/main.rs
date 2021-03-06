use std::convert::TryInto;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ipts_dev::{HeaderAndBuffer, Ipts};
use mtinput::MtInput;
use utils::{Pointers, get_heatmap};
use engine::Engine;

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
    let mut engine = Engine::new();

    while running.load(Ordering::Acquire) {
        ipts.wait_for_doorbell();
        ipts.read(&mut buf);

        let parsed = HeaderAndBuffer::from(&buf);
        if parsed.typ == 3 && parsed.size == 3500 && parsed.data[0] == 0x0B {
            let data = get_heatmap((&parsed.data[..3500]).try_into().unwrap());
            let length = engine.run(data, &mut positions);
            pointers.update(positions, length);
            mt.dispatch(&mut pointers);
        }

        ipts.send_feedback();
    }
}