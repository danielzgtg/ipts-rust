use std::path::Path;

use uinput::event::absolute::Absolute::Multi;
use uinput::event::absolute::Multi::{PositionX, PositionY, Slot, TrackingId};
use uinput::event::controller::Controller::Digi;
use uinput::event::controller::Digi::Touch;
use uinput::event::Event::{Absolute, Controller};
use uinput::Device;

use utils::{Report, SCREEN_X, SCREEN_Y};

fn check_warn_libinput() {
    if !Path::new("/usr/lib/xorg/modules/input/libinput_drv.so").exists() {
        eprintln!("You might need to install xserver-xorg-input-libinput");
    }
}

pub struct MtInput {
    device: Device,
    next_id: i32,
}

impl MtInput {
    pub fn new() -> MtInput {
        check_warn_libinput();
        let device = uinput::default()
            .unwrap()
            .name("IPTS Touch")
            .unwrap()
            .event(Controller(Digi(Touch)))
            .unwrap()
            .event(Absolute(Multi(Slot)))
            .unwrap()
            .min(0)
            .max(10)
            .event(Absolute(Multi(TrackingId)))
            .unwrap()
            .event(Absolute(Multi(PositionX)))
            .unwrap()
            .min(0)
            .max(SCREEN_X as i32)
            .event(Absolute(Multi(PositionY)))
            .unwrap()
            .min(0)
            .max(SCREEN_Y as i32)
            .create()
            .unwrap();
        MtInput { device, next_id: 0 }
    }

    pub fn dispatch(&mut self, events: &[Report; 10]) {
        for (i, e) in events.iter().enumerate() {
            if *e == Report::None {
                continue;
            }
            self.device.send(Slot, i as i32).unwrap();
            match e {
                Report::Up | Report::UpDown(_) => self.device.send(TrackingId, -1).unwrap(),
                _ => {}
            }
            match e {
                Report::Down(_) | Report::UpDown(_) => {
                    self.device.press(&Touch).unwrap();
                    self.device
                        .send(TrackingId, {
                            self.next_id += 1;
                            self.next_id
                        })
                        .unwrap();
                }
                _ => {}
            }
            match e {
                Report::Down(x) | Report::UpDown(x) | Report::Move(x) => {
                    self.device.send(PositionX, x.0 as i32).unwrap();
                    self.device.send(PositionY, x.1 as i32).unwrap();
                }
                _ => {}
            }
        }
        self.device.synchronize().unwrap();
    }
}
