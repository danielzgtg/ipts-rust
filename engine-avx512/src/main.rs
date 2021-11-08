use engine_avx512::process_heatmap;
use std::fs;
use std::time::Instant;
use utils::get_heatmap;

fn main() {
    let data = fs::read(r"../heatmap.bin").unwrap();
    let data: &[u8; 3500] = &data.try_into().unwrap();
    let data = get_heatmap(&data);
    let mut results = [(0u32, 0u32); 10];
    let mut count = 0;
    process_heatmap(data, &mut results);
    let start = Instant::now();
    for _ in 0..1000000 {
        count += process_heatmap(data, &mut results);
    }
    let duration = Instant::now() - start;
    println!("{:?} {} {}ms", results, count, duration.as_millis());
}
