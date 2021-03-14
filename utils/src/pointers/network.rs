use crate::Report;
use byteorder::{BigEndian, ByteOrder};
use std::convert::TryInto;

const NONE_ID: u8 = 0;
const UP_ID: u8 = 1;
const UP_DOWN_ID: u8 = 2;
const DOWN_ID: u8 = 3;
const MOVE_ID: u8 = 4;

#[inline]
fn serialize_report(report: Report, result: &mut[u8; 9]) {
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

pub fn serialize_reports(reports: &[Report; 10], result: &mut[u8; 98]) {
    result[0] = b'I';
    result[1] = b'P';
    result[2] = b'T';
    result[3] = b'S';
    serialize_report(reports[0], (&mut result[4..13]).try_into().unwrap());
    serialize_report(reports[1], (&mut result[13..22]).try_into().unwrap());
    serialize_report(reports[2], (&mut result[22..31]).try_into().unwrap());
    serialize_report(reports[3], (&mut result[31..40]).try_into().unwrap());
    serialize_report(reports[4], (&mut result[40..49]).try_into().unwrap());
    serialize_report(reports[5], (&mut result[49..58]).try_into().unwrap());
    serialize_report(reports[6], (&mut result[58..67]).try_into().unwrap());
    serialize_report(reports[7], (&mut result[67..76]).try_into().unwrap());
    serialize_report(reports[8], (&mut result[76..85]).try_into().unwrap());
    serialize_report(reports[9], (&mut result[85..94]).try_into().unwrap());
    result[94] = b'I';
    result[95] = b'P';
    result[96] = b'T';
    result[97] = b'S';
}

pub fn deserialize_reports(data: &[u8; 98], result: &mut [Report; 10]) {
    if data[0] != b'I' || data[1] != b'I' || data[2] != b'I' || data[3] != b'I' ||
        data[94] != b'I' || data[95] != b'I' || data[96] != b'I' || data[97] != b'I' {
        result.fill(Report::None);
    }
    deserialize_report((&data[4..13]).try_into().unwrap(), &mut result[0]);
    deserialize_report((&data[13..22]).try_into().unwrap(), &mut result[1]);
    deserialize_report((&data[22..31]).try_into().unwrap(), &mut result[2]);
    deserialize_report((&data[31..40]).try_into().unwrap(), &mut result[3]);
    deserialize_report((&data[40..49]).try_into().unwrap(), &mut result[4]);
    deserialize_report((&data[49..58]).try_into().unwrap(), &mut result[5]);
    deserialize_report((&data[58..67]).try_into().unwrap(), &mut result[6]);
    deserialize_report((&data[67..76]).try_into().unwrap(), &mut result[7]);
    deserialize_report((&data[76..85]).try_into().unwrap(), &mut result[8]);
    deserialize_report((&data[85..94]).try_into().unwrap(), &mut result[9]);
}
