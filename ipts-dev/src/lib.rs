/* SPDX-License-Identifier: GPL-2.0-or-later WITH Linux-syscall-note */
/*
 * Copyright (c) 2016 Intel Corporation
 * Copyright (c) 2020 Dorian Stoll
 * Copyright (c) 2021 Daniel Tang
 *
 * Linux driver for Intel Precise Touch & Stylus
 */

use std::fs::File;
use std::convert::TryInto;
use std::io::Read;

const IPTS_BUFFERS: usize = 16;

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

pub struct Ipts {
    files: [File; IPTS_BUFFERS],
    current_doorbell: u32,
    device_info: DeviceInfo,
}

impl Ipts {
    pub fn new() -> Ipts {
        let files: [File; IPTS_BUFFERS] = (0..IPTS_BUFFERS)
            .map(|x| format!("/dev/ipts/{}", x))
            .map(|x| File::open(x).unwrap())
            .collect::<Vec<File>>()
            .try_into()
            .unwrap();
        let mut result = Ipts {
            files,
            current_doorbell: 0,
            device_info: DeviceInfo::default(),
        };
        result.flush();
        result.reload_device_info();
        result.current_doorbell = result.doorbell();
        result
    }

    fn current_file(&self) -> usize {
        self.current_doorbell as usize & (IPTS_BUFFERS - 1)
    }

    fn current_fd(&self) -> &File {
        &self.files[self.current_file()]
    }

    fn ready(&self) -> bool {
        ioctl::get_device_ready(self.current_fd())
    }

    fn wait_for_device(&self) {
        loop {
            if self.ready() { break; }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    pub fn send_feedback(&mut self) {
        self.inner_send_feedback(self.current_file());
    }

    fn inner_send_feedback(&mut self, f: usize) {
        self.wait_for_device();
        ioctl::send_feedback(&self.files[f]);
        self.current_doorbell += 1;
    }

    fn flush(&mut self) {
        for f in 0..IPTS_BUFFERS {
            self.inner_send_feedback(f)
        }
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    fn reload_device_info(&mut self) {
        self.device_info = ioctl::get_device_info(self.current_fd());
    }

    fn doorbell(&mut self) -> u32 {
        self.wait_for_device();
        let new: u32 = ioctl::get_doorbell(self.current_fd());
        if new < self.current_doorbell {
            self.flush();
            self.current_doorbell = new;
        }
        new
    }

    pub fn wait_for_doorbell(&mut self, eager: bool) {
        let mut backoff = if eager { 15 } else { 0 };
        while {
            self.doorbell() <= self.current_doorbell
        } {
            std::thread::sleep(std::time::Duration::from_millis(16 - backoff));
            backoff = backoff.saturating_sub(1);
        }
    }

    pub fn read(&self, buf: &mut [u8; 16384]) {
        self.current_fd().read(&mut buf[..]).unwrap();
    }

    // stop() is just Drop
}

pub struct HeaderAndBuffer<'a> {
    pub typ: u32,
    pub size: u32,
    pub buffer: u32,
    // uint8_t reserved[52];
    pub data: &'a [u8; 16320]
}

impl <'a> HeaderAndBuffer<'a> {
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

mod ioctl;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
