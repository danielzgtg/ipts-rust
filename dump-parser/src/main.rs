use std::fs;

fn main() {
    let txt = fs::read_to_string(r"../dbg.txt").unwrap();
    let mut result = Vec::with_capacity(3500);
    for line in txt
        .lines()
        .skip_while(|x| !x.starts_with('=') || !x.contains("Size: 3500"))
        .skip(1)
        .take_while(|x| !x.is_empty())
    {
        for hex in line.trim_end().split(' ').map(|x| x.as_bytes()) {
            result.push(parse_hex(hex));
        }
    }
    assert_eq!(result.len(), 3500);
    fs::write(r"../heatmap.bin", result).unwrap();
}

fn parse_hex(hex: &[u8]) -> u8 {
    assert_eq!(hex.len(), 2);
    parse_nibble(hex[0]) << 4 | parse_nibble(hex[1])
}

fn parse_nibble(nibble: u8) -> u8 {
    match nibble {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'A' => 10,
        b'B' => 11,
        b'C' => 12,
        b'D' => 13,
        b'E' => 14,
        b'F' => 15,
        _ => panic!(),
    }
}
