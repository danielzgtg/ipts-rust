use std::convert::TryInto;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use utils::get_heatmap;

fn main() {
    let data = fs::read(r"../heatmap.bin").unwrap();
    let data: &[u8; 3500] = &data.try_into().unwrap();
    let data = get_heatmap(&data);

    let path = Path::new(r"heatmap.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 64, 44);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Best);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&data[..])
        .unwrap();
}
