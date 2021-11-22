use crate::ipts::ext::IptsAccess;
use crate::ipts::inner::IptsImpl;

pub struct Ipts(IptsImpl);

impl Ipts {
    pub fn new() -> Self {
        Ipts(IptsImpl::new())
    }

    pub fn send_feedback(&mut self) {
        self.0.wait_for_device_sync();
        self.0.send_feedback_impl(self.0.current_file());
    }

    pub fn wait_for_doorbell(&mut self, eager: bool) {
        self.0.begin_wait_for_doorbell(eager);
        while let Some(duration) = {
            let doorbell = self.0.doorbell_sync();
            self.0.keep_waiting_for_doorbell(doorbell)
        } {
            std::thread::sleep(duration);
        }
    }
}

impl IptsAccess for Ipts {
    fn ipts_impl(&self) -> &IptsImpl {
        &self.0
    }

    fn ipts_impl_mut(&mut self) -> &mut IptsImpl {
        &mut self.0
    }

    fn ipts_impl_destructure(self) -> IptsImpl {
        self.0
    }
}
