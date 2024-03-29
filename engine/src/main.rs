use std::fs;
use std::time::Instant;

use engine::Engine;
use utils::get_heatmap;

fn main() {
    let data = fs::read(r"../heatmap.bin").unwrap();
    let data: &[u8; 3500] = &data.try_into().unwrap();
    let data = get_heatmap(&data);
    let mut positions: [(u32, u32); 10] = [(0, 0); 10];

    let mut engine = Engine::new(true);
    std::thread::sleep(std::time::Duration::from_secs(1));
    engine.run(data, &mut positions);
    let start = Instant::now();
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    engine.run(data, &mut positions);
    println!("Elapsed: {:?}", Instant::now() - start);
    println!("{:?}", positions);
}
