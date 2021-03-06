/* SPDX-License-Identifier: GPL-2.0-or-later WITH Linux-syscall-note */
/*
 * Copyright (c) 2016 Intel Corporation
 * Copyright (c) 2020 Dorian Stoll
 * Copyright (c) 2021 Daniel Tang
 *
 * Linux driver for Intel Precise Touch & Stylus
 */

use nix::{ioctl_none, ioctl_read};

ioctl_read!(get_device_ready, 0x86, 0x01, u8);
ioctl_read!(get_device_info, 0x86, 0x02, crate::DeviceInfo);
ioctl_read!(get_doorbell, 0x86, 0x03, u32);
ioctl_none!(send_feedback, 0x86, 0x04);
// ioctl_none!(send_reset, 0x86, 0x05);
