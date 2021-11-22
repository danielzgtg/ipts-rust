/* SPDX-License-Identifier: GPL-2.0-or-later WITH Linux-syscall-note */
/*
 * Copyright (c) 2016 Intel Corporation
 * Copyright (c) 2020 Dorian Stoll
 * Copyright (c) 2021 Daniel Tang
 *
 * Linux driver for Intel Precise Touch & Stylus
 */

#[derive(Default)]
#[repr(C)]
pub struct DeviceInfo {
    pub vendor: u16,
    pub product: u16,
    pub version: u32,
    pub buffer_size: u32,
    pub max_contacts: u8,
    reserved: [u8; 19],
}
