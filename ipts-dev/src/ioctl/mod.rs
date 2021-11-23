/* SPDX-License-Identifier: GPL-2.0-or-later WITH Linux-syscall-note */
/*
 * Copyright (c) 2016 Intel Corporation
 * Copyright (c) 2020 Dorian Stoll
 * Copyright (c) 2021 Daniel Tang
 *
 * Linux driver for Intel Precise Touch & Stylus
 */

use crate::DeviceInfo;
use std::fs::File;
use std::os::unix::io::AsRawFd;

mod raw;

pub fn get_device_ready(f: &File) -> bool {
    let mut result: u8 = 0;
    unsafe {
        raw::get_device_ready(f.as_raw_fd(), &mut result).unwrap();
    }
    result != 0
}

pub fn get_device_info(f: &File) -> DeviceInfo {
    let mut result: DeviceInfo = DeviceInfo::default();
    unsafe {
        raw::get_device_info(f.as_raw_fd(), &mut result).unwrap();
    }
    assert_eq!(result.vendor, 0x45E);
    assert_eq!(result.product, 0x99F);
    assert_eq!(result.version, 19088656);
    assert_eq!(result.buffer_size, 16384);
    assert_eq!(result.max_contacts, 10);
    result
}

pub fn get_doorbell(f: &File) -> u32 {
    let mut result: u32 = 0;
    unsafe {
        raw::get_doorbell(f.as_raw_fd(), &mut result).unwrap();
    }
    result
}

pub fn send_feedback(f: &File) {
    unsafe {
        raw::send_feedback(f.as_raw_fd()).unwrap();
    }
}

pub fn send_reset(f: &File) {
    unsafe {
        raw::send_reset(f.as_raw_fd()).unwrap();
    }
}
