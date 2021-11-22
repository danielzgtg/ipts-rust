pub struct HeaderAndBuffer<'a> {
    pub typ: u32,
    pub size: u32,
    pub buffer: u32,
    // uint8_t reserved[52];
    pub data: &'a [u8; 16320],
}

impl<'a> HeaderAndBuffer<'a> {
    pub fn from(buf: &[u8; 16384]) -> HeaderAndBuffer {
        HeaderAndBuffer {
            typ: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            size: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            buffer: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            data: (&buf[64..]).try_into().unwrap(),
        }
    }

    pub fn print(&self) {
        let size = self.size as usize;
        if size != 0 {
            print!("{:02X}", self.data[0]);
            for (i, x) in self.data.iter().enumerate().take(size).skip(1) {
                if i & 63 == 0 {
                    println!();
                } else {
                    print!(" ");
                }
                print!("{:02X}", x);
            }
        }
        println!();
        println!();
    }
}
