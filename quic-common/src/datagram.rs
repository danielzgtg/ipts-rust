use crate::DATAGRAM_SIZE;
use byteorder::{BigEndian, ByteOrder};
use std::cmp::Ordering;
use utils::{Report, SCREEN_X, SCREEN_Y};

/// Converts touch reports to and from unreliable unordered datagrams.
/// The datagrams' integrity must be protected or else an error will be returned for misbehaviour.
pub struct ReportTransport {
    /// The last seen datagram number
    last_no: u64,
    /// The reports entering or leaving the ReportTransport
    reports: [Report; 10],
    /// A ID for each touch, monotonically increasing on falling edge
    ids: [u64; 10],
    /// The positions of each touch, or (!0, !0) if currently lifted
    positions: [(u32, u32); 10],
}

#[rustfmt::skip]
macro_rules! id_offsets {
    ($m: ident, $f: ident, $src: ident) => {
        $m!(
            $f, $src,
            0, 8, 16,
            1, 16, 24,
            2, 24, 32,
            3, 32, 40,
            4, 40, 48,
            5, 48, 56,
            6, 56, 64,
            7, 64, 72,
            8, 72, 80,
            9, 80, 88,
        );
    };
}

#[rustfmt::skip]
macro_rules! position_offsets {
    ($m: ident, $f: ident, $src: ident) => {
        $m!(
            $f, $src,
            0, 88, 96,
            1, 96, 104,
            2, 104, 112,
            3, 112, 120,
            4, 120, 128,
            5, 128, 136,
            6, 136, 144,
            7, 144, 152,
            8, 152, 160,
            9, 160, 168,
        );
    };
}

impl ReportTransport {
    pub fn new() -> Self {
        ReportTransport {
            last_no: 0,
            reports: [Report::None; 10],
            ids: [0; 10],
            positions: [(0, 0); 10],
        }
    }

    pub fn serialize(&self, datagram: &mut [u8; DATAGRAM_SIZE]) {
        BigEndian::write_u64(&mut datagram[..8], self.last_no);

        fn write_id(id: u64, result: &mut [u8; 8]) {
            BigEndian::write_u64(result, id);
        }
        fn write_position((x, y): (u32, u32), result: &mut [u8; 8]) {
            let x_result: &mut [u8; 4] = (&mut result[0..4]).try_into().unwrap();
            BigEndian::write_u32(x_result, x);
            let y_result: &mut [u8; 4] = (&mut result[4..8]).try_into().unwrap();
            BigEndian::write_u32(y_result, y);
        }
        macro_rules! write_data {
            ($f: ident, $src: ident, $($n: literal, $s: literal, $e: literal),+ $(,)?) => {{
                $(
                    $f(self.$src[$n], (&mut datagram[$s..$e]).try_into().unwrap());
                )+
            }};
        }
        id_offsets!(write_data, write_id, ids);
        position_offsets!(write_data, write_position, positions);

        // Magic is in ALPN instead of each datagram
    }

    pub fn deserialize(&mut self, datagram: &[u8; DATAGRAM_SIZE]) -> Result<bool, ()> {
        {
            let cur_no = BigEndian::read_u64(&datagram[..8]);
            if cur_no <= self.last_no {
                return Ok(false);
            }
            self.last_no = cur_no;
        }

        let mut ids = [0u64; 10];
        let mut positions = [(0u32, 0u32); 10];
        {
            fn read_id(result: &[u8; 8]) -> u64 {
                BigEndian::read_u64(result)
            }
            fn read_position(result: &[u8; 8]) -> (u32, u32) {
                let x_result: &[u8; 4] = (&result[0..4]).try_into().unwrap();
                let x = BigEndian::read_u32(x_result);
                let y_result: &[u8; 4] = (&result[4..8]).try_into().unwrap();
                let y = BigEndian::read_u32(y_result);
                (x, y)
            }
            macro_rules! read_data {
                ($f: ident, $src: ident, $($n: literal, $s: literal, $e: literal),+ $(,)?) => {{
                    $(
                        $src[$n] = $f((&datagram[$s..$e]).try_into().unwrap());
                    )+
                }};
            }
            id_offsets!(read_data, read_id, ids);
            position_offsets!(read_data, read_position, positions);
        }
        let ids = ids;
        let positions = positions;
        let mut misbehaved = false;
        fn is_active((x, y): (u32, u32)) -> bool {
            x < SCREEN_X as u32 && y < SCREEN_Y as u32
        }
        let active: [bool; 10] = array_init::array_init(|i| {
            let (x, y) = positions[i];
            let result = is_active((x, y));
            if !result && !(x & y) != 0 {
                // Other values are reserved
                misbehaved = true;
            }
            return result;
        });
        if misbehaved {
            println!("E1");
            return Err(());
        }

        let mut reports = [Report::None; 10];
        for i in 0..10 {
            let active = active[i];
            let was_active = !matches!(self.reports[i], Report::None | Report::Up);
            let pos = positions[i];
            reports[i] = match ids[i].cmp(&self.ids[i]) {
                Ordering::Less => {
                    println!("E2");
                    return Err(());
                }
                Ordering::Equal => {
                    if was_active {
                        if active {
                            Report::Move(pos)
                        } else {
                            // ID must increase on falling edge
                            println!("E3");
                            return Err(());
                        }
                    } else {
                        if active {
                            Report::Down(pos)
                        } else {
                            Report::None
                        }
                    }
                }
                Ordering::Greater => {
                    if was_active {
                        if active {
                            Report::UpDown(pos)
                        } else {
                            Report::Up
                        }
                    } else {
                        // Missed a touch
                        if active {
                            Report::Down(pos)
                        } else {
                            Report::None
                        }
                    }
                }
            };
        }

        self.ids = ids;
        self.positions = positions;
        self.reports = reports;
        Ok(true)
    }

    pub fn offer(&mut self, reports: &[Report; 10]) {
        self.last_no += 1;
        self.positions.fill((!0, !0));
        for (i, &r) in reports.iter().enumerate() {
            let old = self.reports[i];
            match r {
                Report::None => {
                    // debug_assert! instead of assert! because there could only be non-remote
                    // misbehaviour here
                    debug_assert!(matches!(old, Report::None | Report::Up));
                }
                Report::Up => {
                    debug_assert!(matches!(
                        old,
                        Report::UpDown(_) | Report::Down(_) | Report::Move(_),
                    ));
                    // Increase ID on falling edge
                    self.ids[i] += 1;
                }
                Report::UpDown(pos) => {
                    debug_assert!(matches!(
                        old,
                        Report::UpDown(_) | Report::Down(_) | Report::Move(_),
                    ));
                    self.ids[i] += 1;
                    self.positions[i] = pos;
                }
                Report::Down(pos) => {
                    debug_assert!(matches!(old, Report::None | Report::Up));
                    self.positions[i] = pos;
                }
                Report::Move(pos) => {
                    debug_assert!(matches!(old, Report::Down(_) | Report::Move(_)));
                    self.positions[i] = pos;
                }
            }
            self.reports[i] = r;
        }
    }

    pub fn take(&self) -> &[Report; 10] {
        &self.reports
    }
}
