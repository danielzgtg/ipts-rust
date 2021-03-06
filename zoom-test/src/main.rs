use mtinput::MtInput;
use utils::Pointers;

fn main() {
    let mut pointers = Pointers::new();
    let mut mt = MtInput::new();
    let mut positions: [(u32, u32); 10] = [(0, 0); 10];
    std::thread::sleep(std::time::Duration::from_secs(1));
    for y in 0..500 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        positions[0] = (1000, 600 - y);
        positions[1] = (1000, 700 + y);
        pointers.update(positions, 2);
        mt.dispatch(&mut pointers);
    }
}
