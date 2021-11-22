/* SPDX-License-Identifier: GPL-2.0-or-later WITH Linux-syscall-note */
/*
 * Copyright (c) 2016 Intel Corporation
 * Copyright (c) 2020 Dorian Stoll
 * Copyright (c) 2021 Daniel Tang
 *
 * Linux driver for Intel Precise Touch & Stylus
 */

use std::fs::File;
use std::io::Read;

use crate::{ioctl, DeviceInfo};

pub(super) const IPTS_BUFFERS: usize = 16;

pub struct IptsImpl {
    files: [File; IPTS_BUFFERS],
    current_doorbell: u32,
    device_info: DeviceInfo,
    backoff: u64,
}

impl IptsImpl {
    pub(super) fn new() -> Self {
        let files: [File; IPTS_BUFFERS] = (0..IPTS_BUFFERS)
            .map(|x| format!("/dev/ipts/{}", x))
            .map(|x| File::open(x).unwrap())
            .collect::<Vec<File>>()
            .try_into()
            .unwrap();
        let mut result = IptsImpl {
            files,
            current_doorbell: 0,
            device_info: DeviceInfo::default(),
            backoff: 0,
        };
        result.flush_sync();
        result.reload_device_info();
        result.current_doorbell = result.doorbell_sync();
        result
    }

    pub(super) fn current_file(&self) -> usize {
        self.current_doorbell as usize & (IPTS_BUFFERS - 1)
    }

    fn current_fd(&self) -> &File {
        &self.files[self.current_file()]
    }

    pub(super) fn ready(&self) -> bool {
        ioctl::get_device_ready(self.current_fd())
    }

    pub(super) fn wait_for_device_sync(&self) {
        loop {
            if self.ready() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    pub(super) fn send_feedback_impl(&mut self, f: usize) {
        ioctl::send_feedback(&self.files[f]);
        self.current_doorbell += 1;
    }

    pub(super) fn send_reset(self) {
        let f = self.files[self.current_file()].try_clone().unwrap();
        std::mem::drop(self);
        ioctl::send_reset(&f);
    }

    pub(super) fn flush_sync(&mut self) {
        for f in 0..IPTS_BUFFERS {
            self.wait_for_device_sync();
            self.send_feedback_impl(f)
        }
    }

    pub(super) fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn reload_device_info(&mut self) {
        self.device_info = ioctl::get_device_info(self.current_fd());
    }

    pub(super) fn doorbell_impl(&mut self) -> (u32, bool) {
        let new: u32 = ioctl::get_doorbell(self.current_fd());
        let flush_required = new < self.current_doorbell;
        if flush_required {
            self.current_doorbell = new;
        }
        (new, flush_required)
    }

    pub(super) fn doorbell_sync(&mut self) -> u32 {
        self.wait_for_device_sync();
        let (new, flush_required) = self.doorbell_impl();
        if flush_required {
            self.flush_sync();
        }
        new
    }

    pub(super) fn begin_wait_for_doorbell(&mut self, eager: bool) {
        self.backoff = if eager { 15 } else { 0 };
    }

    pub(super) fn keep_waiting_for_doorbell(
        &mut self,
        doorbell: u32,
    ) -> Option<std::time::Duration> {
        if doorbell > self.current_doorbell {
            None
        } else {
            self.backoff = self.backoff.saturating_sub(1);
            Some(std::time::Duration::from_millis(16 - self.backoff))
        }
    }

    pub(super) fn read(&self, buf: &mut [u8; 16384]) {
        self.current_fd().read(&mut buf[..]).unwrap();
    }

    // stop() is just Drop
}
