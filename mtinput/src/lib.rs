use uinput::Device;
use uinput::event::absolute::Absolute::Multi;
use uinput::event::absolute::Multi::{PositionX, PositionY, Slot, TrackingId};
use uinput::event::controller::Controller::Digi;
use uinput::event::controller::Digi::Touch;
use uinput::event::Event::{Absolute, Controller};
use utils::{Report, Counter};

pub struct MtInput {
    device: Device,
}

impl MtInput {
    pub fn new() -> MtInput {
        MtInput {
            device: uinput::default().unwrap()
                .name("IPTS Touch").unwrap()
                .event(Controller(Digi(Touch)))
                .unwrap()
                .event(Absolute(Multi(Slot))).unwrap()
                .min(0)
                .max(10)
                .event(Absolute(Multi(TrackingId))).unwrap()
                .event(Absolute(Multi(PositionX))).unwrap()
                .min(0)
                .max(2736)
                .event(Absolute(Multi(PositionY))).unwrap()
                .min(0)
                .max(1824)
                .create().unwrap()
        }
    }

    pub fn dispatch(&mut self, events: &[Report; 10], counter: &mut Counter) {
        for (i, e) in events.iter().enumerate() {
            if *e == Report::None { continue; }
            self.device.send(Slot, i as i32).unwrap();
            match e {
                Report::Up | Report::UpDown(_) => self.device.send(TrackingId, -1).unwrap(),
                _ => {},
            }
            match e {
                Report::Down(_) | Report::UpDown(_) => {
                    self.device.press(&Touch).unwrap();
                    self.device.send(TrackingId, counter.gen_id()).unwrap();
                },
                _ => {},
            }
            match e {
                Report::Down(x) | Report::UpDown(x) | Report::Move(x) => {
                    self.device.send(PositionX, x.0 as i32).unwrap();
                    self.device.send(PositionY, x.1 as i32).unwrap();
                },
                _ => {},
            }
        }
        self.device.synchronize().unwrap();
    }
}
