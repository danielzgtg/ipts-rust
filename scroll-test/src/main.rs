use mtinput::MtInput;
use utils::Pointers;

fn main() {
    let mut pointers = Pointers::new();
    let mut mt = MtInput::new();
    let mut positions: [(u32, u32); 10] = [(0, 0); 10];
    std::thread::sleep(std::time::Duration::from_secs(1));
    for y in 0..1000 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        positions[0] = (1000, 300 + y);
        pointers.update(positions, 1);
        mt.dispatch(pointers.events());
    }
}
