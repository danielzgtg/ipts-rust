use crate::Report;
use byteorder::{BigEndian, ByteOrder};

const NONE_ID: u8 = 0;
const UP_ID: u8 = 1;
const UP_DOWN_ID: u8 = 2;
const DOWN_ID: u8 = 3;
const MOVE_ID: u8 = 4;

#[rustfmt::skip]
macro_rules! offsets {
    ($m: ident) => {
        $m!(
            0, 4, 13,
            1, 13, 22,
            2, 22, 31,
            3, 31, 40,
            4, 40, 49,
            5, 49, 58,
            6, 58, 67,
            7, 67, 76,
            8, 76, 85,
            9, 85, 94,
        );
    };
}

pub fn serialize_reports(reports: &[Report; 10], result: &mut [u8; 98]) {
    result[0] = b'I';
    result[1] = b'P';
    result[2] = b'T';
    result[3] = b'S';
    #[inline]
    fn serialize_report(report: Report, result: &mut [u8; 9]) {
        let (id, pos) = match report {
            Report::None => (NONE_ID, (0, 0)),
            Report::Up => (UP_ID, (0, 0)),
            Report::UpDown(x) => (UP_DOWN_ID, x),
            Report::Down(x) => (DOWN_ID, x),
            Report::Move(x) => (MOVE_ID, x),
        };
        result[0] = id;
        BigEndian::write_u32(&mut result[1..5], pos.0);
        BigEndian::write_u32(&mut result[5..9], pos.1);
    }
    macro_rules! write_data {
        ($($n: literal, $s: literal, $e: literal),+ $(,)?) => {{
            $(
                serialize_report(reports[$n], (&mut result[$s..$e]).try_into().unwrap());
            )+
        }};
    }
    offsets!(write_data);
    result[94] = b'I';
    result[95] = b'P';
    result[96] = b'T';
    result[97] = b'S';
}

pub fn deserialize_reports(data: &[u8; 98], result: &mut [Report; 10]) {
    if data[0] != b'I'
        || data[1] != b'P'
        || data[2] != b'T'
        || data[3] != b'S'
        || data[94] != b'I'
        || data[95] != b'P'
        || data[96] != b'T'
        || data[97] != b'S'
    {
        // TODO return an error
        result.fill(Report::None);
        return;
    }
    #[inline]
    fn deserialize_report(data: &[u8; 9], result: &mut Report) {
        let id = data[0];
        let pos = (
            BigEndian::read_u32(&data[1..5]),
            BigEndian::read_u32(&data[5..9]),
        );
        *result = match id {
            UP_ID => Report::Up,
            UP_DOWN_ID => Report::UpDown(pos),
            DOWN_ID => Report::Down(pos),
            MOVE_ID => Report::Move(pos),
            _ => Report::None,
        }
    }
    macro_rules! read_data {
        ($($n: literal, $s: literal, $e: literal),+ $(,)?) => {{
            $(
                deserialize_report((&data[$s..$e]).try_into().unwrap(), &mut result[$n]);
            )+
        }};
    }
    offsets!(read_data);
}
