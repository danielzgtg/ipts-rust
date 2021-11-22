use crate::ipts::ext::IptsAccess;
use crate::ipts::inner::{IptsImpl, IPTS_BUFFERS};

pub struct IptsAsync(IptsImpl);

impl IptsAsync {
    pub fn new() -> Self {
        IptsAsync(IptsImpl::new())
    }

    async fn wait_for_device(&self) {
        loop {
            if self.0.ready() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }

    pub async fn send_feedback(&mut self) {
        // Async safe: This has to be just setting a flag somewhere
        self.wait_for_device().await;
        self.0.send_feedback_impl(self.0.current_file());
    }

    async fn flush(&mut self) {
        for f in 0..IPTS_BUFFERS {
            self.wait_for_device().await;
            self.0.send_feedback_impl(f)
        }
    }

    async fn doorbell(&mut self) -> u32 {
        self.wait_for_device().await;
        let (new, flush_required) = self.0.doorbell_impl();
        if flush_required {
            self.flush().await;
        }
        new
    }

    pub async fn wait_for_doorbell(&mut self, eager: bool) {
        self.0.begin_wait_for_doorbell(eager);
        while let Some(duration) = {
            let doorbell = self.doorbell().await;
            self.0.keep_waiting_for_doorbell(doorbell)
        } {
            tokio::time::sleep(duration).await;
        }
    }
}

impl IptsAccess for IptsAsync {
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
